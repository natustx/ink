/// Strip YAML or TOML frontmatter from markdown source.
/// Returns (frontmatter, remaining_content).
pub fn strip_frontmatter(source: &str) -> (Option<String>, String) {
    let trimmed = source.trim_start();

    // YAML frontmatter: starts with ---
    if let Some(after) = trimmed.strip_prefix("---") {
        if let Some(end) = after.find("\n---") {
            let fm = after[..end].trim().to_string();
            let rest = after[end + 4..].to_string();
            return (Some(fm), rest);
        }
    }

    // TOML frontmatter: starts with +++
    if let Some(after) = trimmed.strip_prefix("+++") {
        if let Some(end) = after.find("\n+++") {
            let fm = after[..end].trim().to_string();
            let rest = after[end + 4..].to_string();
            return (Some(fm), rest);
        }
    }

    (None, source.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_frontmatter() {
        let source = "---\ntitle: Hello\nauthor: World\n---\n# Content";
        let (fm, rest) = strip_frontmatter(source);
        assert_eq!(fm, Some("title: Hello\nauthor: World".to_string()));
        assert!(rest.contains("# Content"));
    }

    #[test]
    fn test_no_frontmatter() {
        let source = "# Just a heading\nSome content.";
        let (fm, rest) = strip_frontmatter(source);
        assert!(fm.is_none());
        assert_eq!(rest, source);
    }

    #[test]
    fn test_toml_frontmatter() {
        let source = "+++\ntitle = \"Hello\"\n+++\n# Content";
        let (fm, rest) = strip_frontmatter(source);
        assert!(fm.is_some());
        assert!(rest.contains("# Content"));
    }
}
