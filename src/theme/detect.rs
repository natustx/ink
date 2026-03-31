/// Detect if the terminal has a dark background.
pub fn is_dark_background() -> bool {
    // Check COLORFGBG environment variable (set by some terminals)
    if let Ok(val) = std::env::var("COLORFGBG") {
        if let Some(bg) = val.split(';').next_back() {
            if let Ok(bg_num) = bg.parse::<u8>() {
                // Low values = dark background
                return bg_num < 8;
            }
        }
    }

    // Check for common dark terminal indicators
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        // Most modern terminals default to dark
        match term_program.as_str() {
            "iTerm.app" | "WezTerm" | "Ghostty" => return true,
            _ => {}
        }
    }

    // Default to dark (most developers use dark themes)
    true
}
