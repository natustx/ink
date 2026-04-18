use crate::input::{self, Action};
use crate::layout;
use crate::parser::frontmatter;
use crate::render::{self, FooterStatus};
use crate::search::SearchState;
use crate::stats;
use crate::theme;
use crate::toc::TocState;
use crate::watch::{format_err, FileWatcher, HeadingAnchor, ScrollAnchor};
use crate::Args;
use anyhow::Result;
use comrak::{parse_document, Arena};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

struct Tab {
    filename: String,
    source: String,
    watch_path: Option<String>,
    styled_lines: Vec<crate::layout::StyledLine>,
    ratatui_lines: Vec<Line<'static>>,
    scroll_offset: u16,
    toc: TocState,
    word_count: usize,
    reading_time: usize,
    watcher: Option<FileWatcher>,
    reload_pending: bool,
    reload_not_before: Option<Instant>,
    last_reload: Option<Instant>,
    last_reload_failed: Option<(Instant, String)>,
}

struct TabPreservedState {
    scroll_offset: u16,
    toc_visible: bool,
    toc_width: u16,
    watcher: Option<FileWatcher>,
    reload_pending: bool,
    reload_not_before: Option<Instant>,
    last_reload: Option<Instant>,
    last_reload_failed: Option<(Instant, String)>,
}

struct NavEntry {
    filename: String,
    scroll_offset: u16,
}

const RELOAD_RETRY_DELAY: Duration = Duration::from_millis(125);

/// Available themes for the theme picker.
const THEME_LIST: &[&str] = &[
    "dark",
    "light",
    "dracula",
    "catppuccin",
    "nord",
    "tokyo-night",
    "gruvbox",
    "solarized",
];

pub fn run(source: String, args: Args) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_inner(&mut terminal, source, args);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_inner(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    source: String,
    mut args: Args,
) -> Result<()> {
    let size = terminal.size()?;

    let mut tabs: Vec<Tab> = Vec::new();
    let mut active_tab: usize = 0;

    let first_tab = build_tab(
        source,
        args.inputs.first().map(|s| s.as_str()).unwrap_or("stdin"),
        args.inputs.first().map(|s| s.as_str()),
        &args,
        size.width,
    );
    tabs.push(first_tab);

    for input in args.inputs.iter().skip(1) {
        if let Ok(src) = std::fs::read_to_string(input) {
            tabs.push(build_tab(src, input, Some(input), &args, size.width));
        }
    }

    let mut search = SearchState::new();
    let mut nav_history: Vec<NavEntry> = Vec::new();
    let mut nav_forward: Vec<NavEntry> = Vec::new();
    let mut theme_picker_open = false;
    let mut theme_picker_index: usize = 0;

    // Find current theme index
    for (i, t) in THEME_LIST.iter().enumerate() {
        if *t == args.theme {
            theme_picker_index = i;
            break;
        }
    }

    loop {
        let terminal_size = terminal.size()?;
        let viewport_height = terminal_size.height.saturating_sub(3);
        let now = Instant::now();

        if args.watch {
            for index in 0..tabs.len() {
                let active_search = if index == active_tab {
                    Some(&mut search)
                } else {
                    None
                };

                if let Some(tab) = tabs.get_mut(index) {
                    let changed = tab
                        .watcher
                        .as_mut()
                        .map(|watcher| watcher.poll())
                        .unwrap_or(false);
                    if changed {
                        schedule_reload(tab, now);
                    }
                    if tab.reload_pending && ready_to_reload(tab, now) {
                        reload_tab(
                            tab,
                            active_search,
                            &args,
                            terminal_size.width,
                            viewport_height,
                        );
                    }
                }
            }
        }

        let tab = &tabs[active_tab];
        let total_lines = tab.ratatui_lines.len();
        let max_scroll = max_scroll_for_tab(tab, viewport_height);
        let footer_status = status_for_footer(tab, Instant::now());

        terminal.draw(|frame| {
            let size = frame.area();
            let tab = &tabs[active_tab];

            // Fill entire frame with theme background color
            let t = theme::resolve_theme(&args.theme);
            if let Some(ref bg_hex) = t.colors.bg {
                let bg_color = theme::hex_to_color(bg_hex);
                let bg_block =
                    ratatui::widgets::Block::default().style(Style::default().bg(bg_color));
                frame.render_widget(bg_block, size);
            }

            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(2),
                ])
                .split(size);

            let top_bar_area = vertical[0];
            let main_area = vertical[1];
            let bottom_area = vertical[2];

            // Split bottom into separator line + bar
            let bottom_split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1)])
                .split(bottom_area);
            let separator_area = bottom_split[0];
            let bottom_bar_area = bottom_split[1];

            // Separator: ▔ chars in bar-bg on content-bg = half-row visual gap
            let bar_bg = theme::hex_to_color(&t.colors.status_bar_bg);
            let content_bg = t.colors.bg.as_ref().map(|b| theme::hex_to_color(b));
            let sep_style = if let Some(cbg) = content_bg {
                Style::default().fg(bar_bg).bg(cbg)
            } else {
                Style::default().fg(bar_bg)
            };
            let sep_line = Line::from(Span::styled(
                "▁".repeat(separator_area.width as usize),
                sep_style,
            ));
            frame.render_widget(Paragraph::new(vec![sep_line]), separator_area);

            // Top bar: filename + progress
            let tab_info = if tabs.len() > 1 {
                Some((active_tab, tabs.len()))
            } else {
                None
            };
            render::render_top_bar(
                frame,
                top_bar_area,
                &tab.filename,
                tab.scroll_offset as usize,
                total_lines,
                viewport_height as usize,
                &args.theme,
                tab_info,
            );

            let (toc_area, doc_area) = if tab.toc.visible && main_area.width > 40 {
                let horizontal = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(tab.toc.width), Constraint::Min(1)])
                    .split(main_area);
                (Some(horizontal[0]), horizontal[1])
            } else {
                (None, main_area)
            };

            if let Some(toc_area) = toc_area {
                render::render_toc(
                    frame,
                    toc_area,
                    &tab.toc.headings,
                    tab.toc.selected,
                    &args.theme,
                );
            }

            // Render document (with search highlights)
            render::render_document_with_search(
                frame,
                doc_area,
                &tab.ratatui_lines,
                tab.scroll_offset,
                total_lines,
                &search,
                &args.theme,
            );

            // Theme picker overlay
            if theme_picker_open {
                render::render_theme_picker(
                    frame,
                    main_area,
                    THEME_LIST,
                    theme_picker_index,
                    &args.theme,
                );
            }

            // Bottom bar: search input OR keybindings + stats
            if search.active {
                render::render_search_bar(frame, bottom_bar_area, &search, &args.theme);
            } else {
                render::render_bottom_bar(
                    frame,
                    bottom_bar_area,
                    &args.theme,
                    &tab.filename,
                    tab.word_count,
                    tab.reading_time,
                    tabs.len() > 1,
                    tab_info,
                    &footer_status,
                );
            }
        })?;

        if let Some(action) = input::poll_action(Duration::from_millis(50), search.active) {
            if theme_picker_open {
                match action {
                    Action::Quit | Action::CloseSearch => {
                        theme_picker_open = false;
                    }
                    Action::ScrollDown(_) => {
                        theme_picker_index = (theme_picker_index + 1) % THEME_LIST.len();
                        // Live preview: rebuild all tabs with new theme
                        args.theme = THEME_LIST[theme_picker_index].to_string();
                        rebuild_all_tabs(&mut tabs, &args, terminal.size()?.width);
                    }
                    Action::ScrollUp(_) => {
                        theme_picker_index = if theme_picker_index == 0 {
                            THEME_LIST.len() - 1
                        } else {
                            theme_picker_index - 1
                        };
                        args.theme = THEME_LIST[theme_picker_index].to_string();
                        rebuild_all_tabs(&mut tabs, &args, terminal.size()?.width);
                    }
                    Action::SearchConfirm => {
                        // Confirm theme selection
                        theme_picker_open = false;
                    }
                    _ => {}
                }
                continue;
            }

            match action {
                Action::Quit => break,

                // Search
                Action::Search => {
                    search.activate();
                }
                Action::CloseSearch => {
                    search.deactivate();
                }
                Action::SearchConfirm => {
                    // Keep matches visible but exit search input mode
                    search.active = false;
                }
                Action::SearchInput(c) => {
                    search.push_char(c);
                    search.update_matches(&tabs[active_tab].styled_lines);
                    // Auto-jump to first match
                    if let Some(line) = search.current_line() {
                        tabs[active_tab].scroll_offset = (line as u16).min(max_scroll);
                    }
                }
                Action::SearchBackspace => {
                    search.pop_char();
                    search.update_matches(&tabs[active_tab].styled_lines);
                    if let Some(line) = search.current_line() {
                        tabs[active_tab].scroll_offset = (line as u16).min(max_scroll);
                    }
                }
                Action::SearchNext => {
                    search.next_match();
                    if let Some(line) = search.current_line() {
                        tabs[active_tab].scroll_offset = (line as u16).min(max_scroll);
                    }
                }
                Action::SearchPrev => {
                    search.prev_match();
                    if let Some(line) = search.current_line() {
                        tabs[active_tab].scroll_offset = (line as u16).min(max_scroll);
                    }
                }

                // Scrolling
                Action::ScrollDown(n) => {
                    tabs[active_tab].scroll_offset = tabs[active_tab]
                        .scroll_offset
                        .saturating_add(n)
                        .min(max_scroll);
                }
                Action::ScrollUp(n) => {
                    tabs[active_tab].scroll_offset =
                        tabs[active_tab].scroll_offset.saturating_sub(n);
                }
                Action::PageDown => {
                    let jump = viewport_height.saturating_sub(2); // keep 2 lines overlap
                    tabs[active_tab].scroll_offset = tabs[active_tab]
                        .scroll_offset
                        .saturating_add(jump)
                        .min(max_scroll);
                }
                Action::PageUp => {
                    let jump = viewport_height.saturating_sub(2);
                    tabs[active_tab].scroll_offset =
                        tabs[active_tab].scroll_offset.saturating_sub(jump);
                }
                Action::Home => tabs[active_tab].scroll_offset = 0,
                Action::End => tabs[active_tab].scroll_offset = max_scroll,

                // TOC
                Action::ToggleToc => tabs[active_tab].toc.toggle(),

                // Theme picker
                Action::ThemePicker => {
                    theme_picker_open = true;
                }

                // Heading jump: n = next heading, N = previous heading
                Action::NextHeading => {
                    let current = tabs[active_tab].scroll_offset as usize;
                    if let Some(next) = tabs[active_tab]
                        .toc
                        .headings
                        .iter()
                        .find(|h| h.line_index > current + 1)
                    {
                        tabs[active_tab].scroll_offset =
                            (next.line_index.saturating_sub(1) as u16).min(max_scroll);
                    }
                }
                Action::PrevHeading => {
                    let current = tabs[active_tab].scroll_offset as usize;
                    if let Some(prev) = tabs[active_tab]
                        .toc
                        .headings
                        .iter()
                        .rev()
                        .find(|h| h.line_index + 1 < current)
                    {
                        tabs[active_tab].scroll_offset =
                            (prev.line_index.saturating_sub(1) as u16).min(max_scroll);
                    }
                }

                // Links are handled via OSC 8 hyperlinks (Cmd+Click / Ctrl+Click)

                // Tabs
                Action::NextTab if tabs.len() > 1 => {
                    active_tab = (active_tab + 1) % tabs.len();
                    sync_search_with_tab(&mut search, &tabs[active_tab]);
                }
                Action::PrevTab if tabs.len() > 1 => {
                    active_tab = if active_tab == 0 {
                        tabs.len() - 1
                    } else {
                        active_tab - 1
                    };
                    sync_search_with_tab(&mut search, &tabs[active_tab]);
                }

                // Follow relative link
                Action::FollowLink => {
                    let offset = tabs[active_tab].scroll_offset as usize;
                    let total_lines = tabs[active_tab].ratatui_lines.len();
                    let mut found_link = None;
                    for i in offset..=(offset + 5).min(total_lines.saturating_sub(1)) {
                        if let Some(line) = tabs[active_tab].styled_lines.get(i) {
                            for span in &line.spans {
                                if let Some(ref url) = span.style.link_url {
                                    if url.ends_with(".md") || url.ends_with(".markdown") {
                                        found_link = Some(url.clone());
                                        break;
                                    }
                                }
                            }
                            if found_link.is_some() {
                                break;
                            }
                        }
                    }
                    if let Some(link_path) = found_link {
                        let base = Path::new(&tabs[active_tab].filename)
                            .parent()
                            .unwrap_or(Path::new("."));
                        let target = base.join(&link_path);
                        if let Ok(src) = std::fs::read_to_string(&target) {
                            nav_history.push(NavEntry {
                                filename: tabs[active_tab].filename.clone(),
                                scroll_offset: tabs[active_tab].scroll_offset,
                            });
                            nav_forward.clear();
                            tabs[active_tab] = build_tab(
                                src,
                                target.to_str().unwrap_or(&link_path),
                                target.to_str(),
                                &args,
                                terminal.size()?.width,
                            );
                            sync_search_with_tab(&mut search, &tabs[active_tab]);
                        }
                    }
                }
                // Navigation history
                Action::NavBack => {
                    if let Some(entry) = nav_history.pop() {
                        nav_forward.push(NavEntry {
                            filename: tabs[active_tab].filename.clone(),
                            scroll_offset: tabs[active_tab].scroll_offset,
                        });
                        if let Ok(src) = std::fs::read_to_string(&entry.filename) {
                            let mut new_tab = build_tab(
                                src,
                                &entry.filename,
                                Some(&entry.filename),
                                &args,
                                terminal.size()?.width,
                            );
                            new_tab.scroll_offset = entry.scroll_offset;
                            new_tab.toc.update_selection(new_tab.scroll_offset as usize);
                            tabs[active_tab] = new_tab;
                            sync_search_with_tab(&mut search, &tabs[active_tab]);
                        }
                    }
                }
                Action::NavForward => {
                    if let Some(entry) = nav_forward.pop() {
                        nav_history.push(NavEntry {
                            filename: tabs[active_tab].filename.clone(),
                            scroll_offset: tabs[active_tab].scroll_offset,
                        });
                        if let Ok(src) = std::fs::read_to_string(&entry.filename) {
                            let mut new_tab = build_tab(
                                src,
                                &entry.filename,
                                Some(&entry.filename),
                                &args,
                                terminal.size()?.width,
                            );
                            new_tab.scroll_offset = entry.scroll_offset;
                            new_tab.toc.update_selection(new_tab.scroll_offset as usize);
                            tabs[active_tab] = new_tab;
                            sync_search_with_tab(&mut search, &tabs[active_tab]);
                        }
                    }
                }
                Action::Resize(_, _) => {
                    // Rebuild on resize
                    rebuild_all_tabs(&mut tabs, &args, terminal.size()?.width);
                    sync_search_with_tab(&mut search, &tabs[active_tab]);
                }
                _ => {}
            }

            let current_offset = tabs[active_tab].scroll_offset as usize;
            tabs[active_tab].toc.update_selection(current_offset);
        }
    }

    Ok(())
}

fn rebuild_all_tabs(tabs: &mut [Tab], args: &Args, term_width: u16) {
    for tab in tabs.iter_mut() {
        let source = tab.source.clone();
        rebuild_tab_preserving_state(tab, source, args, term_width);
    }
}

fn build_tab(
    source: String,
    filename: &str,
    watch_path: Option<&str>,
    args: &Args,
    term_width: u16,
) -> Tab {
    let (_, content) = if args.frontmatter {
        (None, source.clone())
    } else {
        frontmatter::strip_frontmatter(&source)
    };

    // Pre-process wikilinks before parsing
    let content = crate::wikilink::process_wikilinks(&content);

    let arena = Arena::new();
    let options = crate::parser::options();
    let root = parse_document(&arena, &content, &options);

    let max_content_width = args.width.unwrap_or(term_width.saturating_sub(4)).min(120);
    let center_margin = if term_width > max_content_width + 4 {
        ((term_width - max_content_width) / 2) as usize
    } else {
        2
    };

    // Resolve base directory for relative image paths
    let base_dir = Path::new(filename).parent();

    let styled_lines = layout::layout_document(
        root,
        &theme::resolve_theme(&args.theme),
        max_content_width,
        args.spacing,
        center_margin,
        base_dir,
        args.no_images,
    );
    let ratatui_lines = render::styled_lines_to_ratatui(&styled_lines, &args.theme);

    let (word_count, reading_time) = stats::document_stats(&content);

    // Extract headings from the already-parsed AST (avoid parsing twice)
    let headings = crate::parser::extract_headings_from_ast(root);
    let toc_entries: Vec<crate::toc::TocEntry> = headings
        .iter()
        .filter_map(|h| {
            for (i, line) in styled_lines.iter().enumerate() {
                for span in &line.spans {
                    if span.style.bold && span.text.contains(&h.text) && !h.text.is_empty() {
                        return Some(crate::toc::TocEntry {
                            level: h.level,
                            text: h.text.clone(),
                            line_index: i,
                        });
                    }
                }
            }
            None
        })
        .collect();

    let mut toc = TocState::empty();
    toc.headings = toc_entries;
    toc.visible = args.toc;

    let resolved_watch_path = watch_path.and_then(|path| {
        if args.watch && Path::new(path).is_file() {
            Some(path.to_string())
        } else {
            None
        }
    });
    let watcher = resolved_watch_path.as_ref().map(FileWatcher::new);

    Tab {
        filename: filename.to_string(),
        source,
        watch_path: resolved_watch_path,
        styled_lines,
        ratatui_lines,
        scroll_offset: 0,
        toc,
        word_count,
        reading_time,
        watcher,
        reload_pending: false,
        reload_not_before: None,
        last_reload: None,
        last_reload_failed: None,
    }
}

fn reload_tab(
    tab: &mut Tab,
    active_search: Option<&mut SearchState>,
    args: &Args,
    term_width: u16,
    viewport_height: u16,
) {
    let Some(watch_path) = tab.watch_path.clone() else {
        tab.reload_pending = false;
        tab.reload_not_before = None;
        return;
    };

    let now = Instant::now();
    let anchor = ScrollAnchor::capture(&watch_headings(tab), &tab.ratatui_lines, tab.scroll_offset);

    let source = match std::fs::read_to_string(&watch_path) {
        Ok(source) => source,
        Err(error) => {
            tab.last_reload_failed = Some((now, format_err(&error)));
            schedule_reload(tab, now);
            return;
        }
    };

    rebuild_tab_preserving_state(tab, source, args, term_width);

    let max_scroll = max_scroll_for_tab(tab, viewport_height);
    tab.scroll_offset = anchor.resolve(&watch_headings(tab), &tab.ratatui_lines, max_scroll);

    if let Some(search) = active_search {
        if !search.query.is_empty() {
            search.update_matches(&tab.styled_lines);
            if search.active {
                if let Some(line) = search.current_line() {
                    tab.scroll_offset = (line as u16).min(max_scroll);
                }
            }
        }
    }

    tab.toc.update_selection(tab.scroll_offset as usize);
    tab.reload_pending = false;
    tab.reload_not_before = None;
    tab.last_reload = Some(now);
    tab.last_reload_failed = None;
}

fn rebuild_tab_preserving_state(tab: &mut Tab, source: String, args: &Args, term_width: u16) {
    let preserved = snapshot_tab_state(tab);
    let filename = tab.filename.clone();
    let watch_path = tab.watch_path.clone();

    *tab = build_tab(source, &filename, watch_path.as_deref(), args, term_width);
    restore_tab_state(tab, preserved);
}

fn snapshot_tab_state(tab: &Tab) -> TabPreservedState {
    TabPreservedState {
        scroll_offset: tab.scroll_offset,
        toc_visible: tab.toc.visible,
        toc_width: tab.toc.width,
        watcher: tab.watcher.clone(),
        reload_pending: tab.reload_pending,
        reload_not_before: tab.reload_not_before,
        last_reload: tab.last_reload,
        last_reload_failed: tab.last_reload_failed.clone(),
    }
}

fn restore_tab_state(tab: &mut Tab, preserved: TabPreservedState) {
    tab.scroll_offset = preserved.scroll_offset;
    tab.toc.visible = preserved.toc_visible;
    tab.toc.width = preserved.toc_width;
    tab.toc.update_selection(tab.scroll_offset as usize);
    tab.watcher = preserved.watcher;
    tab.reload_pending = preserved.reload_pending;
    tab.reload_not_before = preserved.reload_not_before;
    tab.last_reload = preserved.last_reload;
    tab.last_reload_failed = preserved.last_reload_failed;
}

fn watch_headings(tab: &Tab) -> Vec<HeadingAnchor> {
    tab.toc
        .headings
        .iter()
        .map(|heading| HeadingAnchor {
            level: heading.level,
            text: heading.text.clone(),
            line_index: heading.line_index,
        })
        .collect()
}

fn max_scroll_for_tab(tab: &Tab, viewport_height: u16) -> u16 {
    (tab.ratatui_lines.len() as u16).saturating_sub(viewport_height)
}

fn schedule_reload(tab: &mut Tab, now: Instant) {
    tab.reload_pending = true;
    tab.reload_not_before = Some(now + RELOAD_RETRY_DELAY);
}

fn ready_to_reload(tab: &Tab, now: Instant) -> bool {
    tab.reload_not_before.is_none_or(|deadline| now >= deadline)
}

fn status_for_footer(tab: &Tab, now: Instant) -> FooterStatus {
    if let Some((_, detail)) = &tab.last_reload_failed {
        return FooterStatus::Failed(detail.clone());
    }

    let ok = tab
        .last_reload
        .filter(|t| now.duration_since(*t) < Duration::from_secs(2));

    match ok {
        Some(_) => FooterStatus::Reloaded,
        None if tab.watcher.is_some() => FooterStatus::Watching,
        None => FooterStatus::None,
    }
}

fn sync_search_with_tab(search: &mut SearchState, tab: &Tab) {
    search.update_matches(&tab.styled_lines);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Spacing;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    fn test_args() -> Args {
        Args {
            inputs: vec![],
            theme: "dark".to_string(),
            width: None,
            slides: false,
            plain: false,
            watch: true,
            toc: false,
            no_images: true,
            frontmatter: false,
            spacing: Spacing::Normal,
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let unique = format!(
            "ink-app-test-{}-{}-{}",
            name,
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        std::env::temp_dir().join(unique)
    }

    #[test]
    fn build_tab_does_not_watch_stdin_fallback_name() {
        let args = test_args();
        let tab = build_tab("# From stdin\n".to_string(), "stdin", None, &args, 80);

        assert!(tab.watcher.is_none());
        assert!(tab.watch_path.is_none());
    }

    #[test]
    fn reload_updates_search_matches_without_overriding_scroll_when_search_inactive() {
        let args = test_args();
        let path = temp_path("search-scroll.md");
        let original =
            "# Title\n\nmatch here\n\nline 4\n\n## Section\n\nline 8\nline 9\nline 10\nline 11\n";
        fs::write(&path, original).unwrap();

        let mut tab = build_tab(
            original.to_string(),
            path.to_str().unwrap(),
            path.to_str(),
            &args,
            80,
        );
        tab.scroll_offset = 6;
        tab.reload_pending = true;

        let mut search = SearchState::new();
        search.query = "match".to_string();
        search.update_matches(&tab.styled_lines);
        search.active = false;

        let updated = "# Title\n\nmatch here\n\nline 4\n\n## Section\n\nline 8\nline 9\nline 10\nline 11\n\nmatch again\n";
        fs::write(&path, updated).unwrap();

        reload_tab(&mut tab, Some(&mut search), &args, 80, 10);

        assert_eq!(search.match_count(), 2);
        assert_eq!(tab.scroll_offset, 6);
        assert_ne!(search.current_line(), Some(tab.scroll_offset as usize));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn failed_reload_stays_pending_until_success() {
        let args = test_args();
        let path = temp_path("retry.md");
        fs::write(&path, "# Original\n").unwrap();

        let mut tab = build_tab(
            "# Original\n".to_string(),
            path.to_str().unwrap(),
            path.to_str(),
            &args,
            80,
        );
        tab.reload_pending = true;
        tab.reload_not_before = None;

        fs::remove_file(&path).unwrap();
        reload_tab(&mut tab, None, &args, 80, 10);
        assert!(tab.reload_pending);
        assert!(tab.last_reload_failed.is_some());
        assert!(tab.reload_not_before.is_some());

        fs::write(&path, "# Recovered\n").unwrap();
        tab.reload_not_before = None;
        reload_tab(&mut tab, None, &args, 80, 10);
        assert!(!tab.reload_pending);
        assert!(tab.last_reload_failed.is_none());
        assert_eq!(tab.source, "# Recovered\n");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn failed_status_is_sticky_until_success() {
        let args = test_args();
        let mut tab = build_tab("# Title\n".to_string(), "doc.md", None, &args, 80);
        tab.last_reload_failed = Some((
            Instant::now() - Duration::from_secs(10),
            "file not found".to_string(),
        ));

        assert_eq!(
            status_for_footer(&tab, Instant::now()),
            FooterStatus::Failed("file not found".to_string())
        );
    }

    #[test]
    fn pending_reload_waits_for_deadline() {
        let args = test_args();
        let mut tab = build_tab("# Title\n".to_string(), "doc.md", None, &args, 80);
        let now = Instant::now();
        tab.reload_pending = true;
        tab.reload_not_before = Some(now + RELOAD_RETRY_DELAY);

        assert!(!ready_to_reload(&tab, now));
        assert!(ready_to_reload(
            &tab,
            now + RELOAD_RETRY_DELAY + Duration::from_millis(1)
        ));
    }
}
