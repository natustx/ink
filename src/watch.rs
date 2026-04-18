use ratatui::text::Line;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingAnchor {
    pub level: u8,
    pub text: String,
    pub line_index: usize,
}

#[derive(Debug, Clone)]
pub struct ScrollAnchor {
    pub heading_text: Option<String>,
    pub heading_level: Option<u8>,
    pub heading_occurrence: usize,
    pub lines_below_heading: u16,
    pub first_visible_hash: Option<u64>,
    pub original_scroll: u16,
}

#[derive(Debug, Clone)]
pub struct FileWatcher {
    path: PathBuf,
    last_mtime: Option<SystemTime>,
    last_size: Option<u64>,
    was_stat_ok: bool,
    initialized: bool,
}

impl FileWatcher {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            last_mtime: None,
            last_size: None,
            was_stat_ok: false,
            initialized: false,
        }
    }

    pub fn poll(&mut self) -> bool {
        let snapshot = fs::metadata(&self.path)
            .ok()
            .map(|metadata| (metadata.modified().ok(), Some(metadata.len())));
        let stat_ok = snapshot.is_some();

        let changed = if !self.initialized {
            false
        } else if self.was_stat_ok != stat_ok {
            true
        } else if let Some((mtime, size)) = snapshot {
            self.last_mtime != mtime || self.last_size != size
        } else {
            false
        };

        self.initialized = true;
        self.was_stat_ok = stat_ok;
        if let Some((mtime, size)) = snapshot {
            self.last_mtime = mtime;
            self.last_size = size;
        } else {
            self.last_mtime = None;
            self.last_size = None;
        }

        changed
    }
}

impl ScrollAnchor {
    pub fn capture(headings: &[HeadingAnchor], lines: &[Line<'_>], scroll_offset: u16) -> Self {
        let original_scroll = scroll_offset;
        let line_index = scroll_offset as usize;
        let first_visible_hash = lines.get(line_index).map(hash_line);

        let nearest_heading = headings
            .iter()
            .enumerate()
            .rev()
            .find(|(_, heading)| heading.line_index <= line_index);

        if let Some((idx, heading)) = nearest_heading {
            let occurrence = headings[..=idx]
                .iter()
                .filter(|candidate| {
                    candidate.level == heading.level
                        && normalize_heading(&candidate.text) == normalize_heading(&heading.text)
                })
                .count()
                .saturating_sub(1);

            Self {
                heading_text: Some(heading.text.clone()),
                heading_level: Some(heading.level),
                heading_occurrence: occurrence,
                lines_below_heading: scroll_offset.saturating_sub(heading.line_index as u16),
                first_visible_hash,
                original_scroll,
            }
        } else {
            Self {
                heading_text: None,
                heading_level: None,
                heading_occurrence: 0,
                lines_below_heading: 0,
                first_visible_hash,
                original_scroll,
            }
        }
    }

    pub fn resolve(&self, headings: &[HeadingAnchor], lines: &[Line<'_>], max_scroll: u16) -> u16 {
        if let (Some(text), Some(level)) = (&self.heading_text, self.heading_level) {
            let normalized = normalize_heading(text);
            let matches: Vec<&HeadingAnchor> = headings
                .iter()
                .filter(|heading| {
                    heading.level == level && normalize_heading(&heading.text) == normalized
                })
                .collect();

            if let Some(heading) = matches
                .get(self.heading_occurrence)
                .copied()
                .or_else(|| matches.first().copied())
            {
                let resolved = heading
                    .line_index
                    .saturating_add(self.lines_below_heading as usize);
                return (resolved as u16).min(max_scroll);
            }
        }

        if let Some(first_visible_hash) = self.first_visible_hash {
            if let Some((index, _)) = lines
                .iter()
                .enumerate()
                .find(|(_, line)| hash_line(line) == first_visible_hash)
            {
                return (index as u16).min(max_scroll);
            }
        }

        self.original_scroll.min(max_scroll)
    }
}

pub fn format_err(error: &io::Error) -> String {
    let message = match error.kind() {
        io::ErrorKind::NotFound => "file not found",
        io::ErrorKind::PermissionDenied => "permission denied",
        io::ErrorKind::InvalidData => "invalid utf-8",
        io::ErrorKind::TimedOut => "read timed out",
        _ => {
            if error.raw_os_error() == Some(21) {
                "is a directory"
            } else {
                "read failed"
            }
        }
    };

    truncate_detail(message)
}

fn hash_line(line: &Line<'_>) -> u64 {
    let mut hasher = DefaultHasher::new();
    for span in &line.spans {
        span.content.hash(&mut hasher);
    }
    hasher.finish()
}

fn normalize_heading(text: &str) -> String {
    text.trim().to_lowercase()
}

fn truncate_detail(message: &str) -> String {
    let char_count = message.chars().count();
    if char_count <= 24 {
        return message.to_string();
    }

    let truncated: String = message.chars().take(23).collect();
    format!("{truncated}…")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Span;
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    fn temp_path(name: &str) -> PathBuf {
        let unique = format!(
            "ink-watch-test-{}-{}-{}",
            name,
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        std::env::temp_dir().join(unique)
    }

    fn line(text: &str) -> Line<'static> {
        Line::from(vec![Span::raw(text.to_string())])
    }

    fn heading(level: u8, text: &str, line_index: usize) -> HeadingAnchor {
        HeadingAnchor {
            level,
            text: text.to_string(),
            line_index,
        }
    }

    fn sleep_for_mtime_tick() {
        thread::sleep(Duration::from_millis(1100));
    }

    #[test]
    fn watcher_first_poll_returns_false() {
        let path = temp_path("first-poll.md");
        fs::write(&path, "one").unwrap();

        let mut watcher = FileWatcher::new(&path);
        assert!(!watcher.poll());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn watcher_detects_size_change() {
        let path = temp_path("size-change.md");
        fs::write(&path, "one").unwrap();

        let mut watcher = FileWatcher::new(&path);
        assert!(!watcher.poll());

        fs::write(&path, "one two three").unwrap();
        assert!(watcher.poll());
        assert!(!watcher.poll());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn watcher_detects_mtime_only_change() {
        let path = temp_path("mtime-change.md");
        fs::write(&path, "same-size-a").unwrap();

        let mut watcher = FileWatcher::new(&path);
        assert!(!watcher.poll());

        sleep_for_mtime_tick();
        fs::write(&path, "same-size-b").unwrap();
        assert!(watcher.poll());
        assert!(!watcher.poll());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn watcher_returns_false_when_nothing_changed() {
        let path = temp_path("no-change.md");
        fs::write(&path, "steady").unwrap();

        let mut watcher = FileWatcher::new(&path);
        assert!(!watcher.poll());
        assert!(!watcher.poll());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn watcher_detects_atomic_replace() {
        let path = temp_path("atomic.md");
        let replacement = path.with_extension("new");
        fs::write(&path, "before").unwrap();

        let mut watcher = FileWatcher::new(&path);
        assert!(!watcher.poll());

        fs::write(&replacement, "after replacement").unwrap();
        fs::rename(&replacement, &path).unwrap();
        assert!(watcher.poll());
        assert!(!watcher.poll());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn watcher_detects_stat_success_to_failure_transition() {
        let path = temp_path("delete.md");
        fs::write(&path, "gone soon").unwrap();

        let mut watcher = FileWatcher::new(&path);
        assert!(!watcher.poll());

        fs::remove_file(&path).unwrap();
        assert!(watcher.poll());
        assert!(!watcher.poll());
    }

    #[test]
    fn watcher_detects_stat_failure_to_success_transition() {
        let path = temp_path("recreate.md");
        fs::write(&path, "exists").unwrap();

        let mut watcher = FileWatcher::new(&path);
        assert!(!watcher.poll());

        fs::remove_file(&path).unwrap();
        assert!(watcher.poll());
        assert!(!watcher.poll());

        fs::write(&path, "back again").unwrap();
        assert!(watcher.poll());
        assert!(!watcher.poll());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn scroll_anchor_restores_exact_position_when_headings_unchanged() {
        let headings = vec![heading(1, "Intro", 0), heading(2, "Section", 5)];
        let lines = vec![
            line("# Intro"),
            line("a"),
            line("b"),
            line("c"),
            line("d"),
            line("## Section"),
            line("line 1"),
            line("line 2"),
        ];

        let anchor = ScrollAnchor::capture(&headings, &lines, 7);
        assert_eq!(anchor.resolve(&headings, &lines, 7), 7);
    }

    #[test]
    fn scroll_anchor_uses_heading_relative_offset_when_headings_move() {
        let old_headings = vec![heading(1, "Intro", 0), heading(2, "Section", 5)];
        let old_lines = vec![
            line("# Intro"),
            line("a"),
            line("b"),
            line("c"),
            line("d"),
            line("## Section"),
            line("line 1"),
            line("line 2"),
        ];
        let new_headings = vec![heading(1, "Intro", 0), heading(2, "Section", 8)];
        let new_lines = vec![
            line("# Intro"),
            line("a"),
            line("b"),
            line("c"),
            line("d"),
            line("e"),
            line("f"),
            line("g"),
            line("## Section"),
            line("line 1"),
            line("line 2"),
        ];

        let anchor = ScrollAnchor::capture(&old_headings, &old_lines, 7);
        assert_eq!(anchor.resolve(&new_headings, &new_lines, 10), 10);
    }

    #[test]
    fn scroll_anchor_falls_back_to_line_hash_when_heading_changes() {
        let old_headings = vec![heading(2, "Section", 3)];
        let old_lines = vec![
            line("a"),
            line("b"),
            line("c"),
            line("## Section"),
            line("target"),
        ];
        let new_headings = vec![heading(2, "Renamed", 3)];
        let new_lines = vec![
            line("a"),
            line("b"),
            line("c"),
            line("## Renamed"),
            line("target"),
        ];

        let anchor = ScrollAnchor::capture(&old_headings, &old_lines, 4);
        assert_eq!(anchor.resolve(&new_headings, &new_lines, 4), 4);
    }

    #[test]
    fn scroll_anchor_falls_back_to_same_line_number_when_nothing_matches() {
        let old_headings = vec![heading(2, "Section", 3)];
        let old_lines = vec![
            line("a"),
            line("b"),
            line("c"),
            line("## Section"),
            line("target"),
        ];
        let new_headings = vec![heading(2, "Different", 0)];
        let new_lines = vec![line("x"), line("y"), line("z")];

        let anchor = ScrollAnchor::capture(&old_headings, &old_lines, 4);
        assert_eq!(anchor.resolve(&new_headings, &new_lines, 10), 4);
    }

    #[test]
    fn scroll_anchor_clamps_when_new_doc_is_shorter() {
        let headings = vec![heading(2, "Section", 3)];
        let lines = vec![
            line("a"),
            line("b"),
            line("c"),
            line("## Section"),
            line("target"),
        ];

        let anchor = ScrollAnchor::capture(&headings, &lines, 4);
        assert_eq!(anchor.resolve(&[], &[line("short")], 0), 0);
    }

    #[test]
    fn scroll_anchor_disambiguates_duplicate_headings_by_occurrence() {
        let old_headings = vec![
            heading(2, "Repeated", 1),
            heading(2, "Repeated", 5),
            heading(2, "Repeated", 9),
        ];
        let old_lines = vec![
            line("preamble"),
            line("## Repeated"),
            line("a"),
            line("b"),
            line("c"),
            line("## Repeated"),
            line("d"),
            line("e"),
            line("f"),
            line("## Repeated"),
            line("g"),
        ];
        let new_headings = vec![
            heading(2, "Repeated", 2),
            heading(2, "Repeated", 7),
            heading(2, "Repeated", 12),
        ];
        let new_lines = vec![
            line("x"),
            line("y"),
            line("## Repeated"),
            line("a"),
            line("b"),
            line("c"),
            line("d"),
            line("## Repeated"),
            line("e"),
            line("f"),
            line("g"),
            line("h"),
            line("## Repeated"),
            line("i"),
        ];

        let anchor = ScrollAnchor::capture(&old_headings, &old_lines, 6);
        assert_eq!(anchor.resolve(&new_headings, &new_lines, 13), 8);
    }

    #[test]
    fn format_err_maps_known_kinds_and_caps_length() {
        let cases = vec![
            (
                io::Error::new(io::ErrorKind::NotFound, "missing"),
                "file not found",
            ),
            (
                io::Error::new(io::ErrorKind::PermissionDenied, "nope"),
                "permission denied",
            ),
            (
                io::Error::new(io::ErrorKind::InvalidData, "utf8"),
                "invalid utf-8",
            ),
            (
                io::Error::new(io::ErrorKind::TimedOut, "slow"),
                "read timed out",
            ),
            (
                io::Error::new(io::ErrorKind::Other, "mystery"),
                "read failed",
            ),
        ];

        for (error, expected) in cases {
            let actual = format_err(&error);
            assert_eq!(actual, expected);
            assert!(actual.chars().count() <= 24);
        }
    }
}
