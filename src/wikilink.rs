/// Pre-process markdown source to convert wikilinks to standard markdown links.
///
/// Converts:
/// - `[[target]]` → `[target](target.md)`
/// - `[[target|display]]` → `[display](target.md)`
/// - `[[target.pdf]]` → `[target.pdf](target.pdf)` (keeps existing extensions)
///
/// Skips wikilinks inside fenced code blocks and inline code.
pub fn process_wikilinks(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut in_code_block = false;

    for line in source.split('\n') {
        let trimmed = line.trim_start();

        // Track fenced code blocks (``` or ~~~)
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_code_block {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Process line, skipping inline code spans
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            // Skip inline code: `...`
            if bytes[i] == b'`' {
                result.push('`');
                i += 1;
                while i < len && bytes[i] != b'`' {
                    // Safe: we're scanning for ASCII backtick
                    let ch = line[i..].chars().next().unwrap_or('?');
                    result.push(ch);
                    i += ch.len_utf8();
                }
                if i < len {
                    result.push('`');
                    i += 1;
                }
                continue;
            }

            // Check for wikilink: [[...]]
            if i + 1 < len && bytes[i] == b'[' && bytes[i + 1] == b'[' {
                if let Some(close) = find_close_brackets(bytes, i + 2) {
                    let inner = &line[i + 2..close];
                    if !inner.is_empty() && !inner.contains('\n') {
                        let (display, target_path) = if let Some(pipe) = inner.find('|') {
                            let target = &inner[..pipe];
                            let display = &inner[pipe + 1..];
                            (display.to_string(), normalize_target(target))
                        } else {
                            (inner.to_string(), normalize_target(inner))
                        };
                        result.push_str(&format!("[{display}]({target_path})"));
                        i = close + 2; // skip past ]]
                        continue;
                    }
                }
            }

            // Regular character (UTF-8 safe)
            let ch = line[i..].chars().next().unwrap_or('?');
            result.push(ch);
            i += ch.len_utf8();
        }

        result.push('\n');
    }

    // Remove trailing newline if source didn't end with one
    if !source.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Find the position of `]]` starting from `start`.
fn find_close_brackets(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b']' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Normalize a wikilink target to a file path.
/// Adds `.md` extension if the target doesn't already have one.
fn normalize_target(target: &str) -> String {
    let target = target.trim();
    if target.contains('.') {
        target.to_string()
    } else {
        format!("{target}.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_wikilink() {
        assert_eq!(
            process_wikilinks("See [[my page]] for details"),
            "See [my page](my page.md) for details"
        );
    }

    #[test]
    fn wikilink_with_display() {
        assert_eq!(
            process_wikilinks("See [[target|click here]]"),
            "See [click here](target.md)"
        );
    }

    #[test]
    fn wikilink_with_extension() {
        assert_eq!(
            process_wikilinks("See [[doc.pdf]]"),
            "See [doc.pdf](doc.pdf)"
        );
    }

    #[test]
    fn skip_code_block() {
        let input = "```\n[[not a link]]\n```";
        assert_eq!(process_wikilinks(input), input);
    }

    #[test]
    fn skip_inline_code() {
        let input = "Use `[[not a link]]` syntax";
        assert_eq!(process_wikilinks(input), input);
    }

    #[test]
    fn no_wikilinks() {
        let input = "Just normal [markdown](link.md) text";
        assert_eq!(process_wikilinks(input), input);
    }
}
