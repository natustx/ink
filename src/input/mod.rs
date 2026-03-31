use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    ScrollUp(u16),
    ScrollDown(u16),
    PageUp,
    PageDown,
    Home,
    End,
    ToggleToc,
    Search,
    SearchNext,
    SearchPrev,
    CloseSearch,
    SearchInput(char),
    SearchBackspace,
    SearchConfirm,
    Resize(u16, u16),
    NextHeading,
    PrevHeading,
    NextTab,
    PrevTab,
    FollowLink,
    NavBack,
    NavForward,
    ThemePicker,
    None,
}

pub fn poll_action(timeout: std::time::Duration, search_active: bool) -> Option<Action> {
    if event::poll(timeout).ok()? {
        let event = event::read().ok()?;
        Some(map_event(event, search_active))
    } else {
        None
    }
}

fn map_event(event: Event, search_active: bool) -> Action {
    match event {
        Event::Key(key) => {
            if search_active {
                map_search_key(key)
            } else {
                map_key(key)
            }
        }
        Event::Mouse(mouse) => map_mouse(mouse),
        Event::Resize(w, h) => Action::Resize(w, h),
        _ => Action::None,
    }
}

fn map_search_key(key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Action::CloseSearch;
    }
    match key.code {
        KeyCode::Esc => Action::CloseSearch,
        KeyCode::Enter => Action::SearchConfirm,
        KeyCode::Backspace => Action::SearchBackspace,
        KeyCode::Char(c) => Action::SearchInput(c),
        KeyCode::Down => Action::SearchNext,
        KeyCode::Up => Action::SearchPrev,
        _ => Action::None,
    }
}

fn map_key(key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Action::Quit,
            KeyCode::Char('d') => Action::PageDown,
            KeyCode::Char('u') => Action::PageUp,
            KeyCode::Char('f') => Action::PageDown,
            KeyCode::Char('b') => Action::PageUp,
            _ => Action::None,
        };
    }

    if key.modifiers.contains(KeyModifiers::ALT) {
        return match key.code {
            KeyCode::Down => Action::ScrollDown(10),
            KeyCode::Up => Action::ScrollUp(10),
            KeyCode::Left => Action::NavBack,
            KeyCode::Right => Action::NavForward,
            _ => Action::None,
        };
    }

    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Down | KeyCode::Char('j') => Action::ScrollDown(1),
        KeyCode::Up | KeyCode::Char('k') => Action::ScrollUp(1),
        KeyCode::PageDown | KeyCode::Char(' ') => Action::PageDown,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::Home => Action::Home,
        KeyCode::End | KeyCode::Char('G') => Action::End,
        KeyCode::Char('t') => Action::ToggleToc,
        KeyCode::Char('/') => Action::Search,
        KeyCode::Char('n') => Action::NextHeading,
        KeyCode::Char('N') => Action::PrevHeading,
        KeyCode::Esc => Action::CloseSearch,
        KeyCode::Char('T') => Action::ThemePicker,
        KeyCode::Tab => Action::NextTab,
        KeyCode::BackTab => Action::PrevTab,
        KeyCode::Enter => Action::FollowLink,
        KeyCode::Char('[') => Action::NavBack,
        KeyCode::Char(']') => Action::NavForward,
        _ => Action::None,
    }
}

fn map_mouse(mouse: MouseEvent) -> Action {
    match mouse.kind {
        MouseEventKind::ScrollUp => Action::ScrollUp(3),
        MouseEventKind::ScrollDown => Action::ScrollDown(3),
        _ => Action::None,
    }
}
