pub struct FrontMatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
}

/// Strips and parses a YAML front matter block from the top of the document.
/// Returns parsed fields and the remaining markdown content.
pub fn parse_front_matter(input: &str) -> (FrontMatter, &str) {
    let mut fm = FrontMatter {
        title: None,
        description: None,
        author: None,
        date: None,
    };

    let Some(rest) = input.strip_prefix("---\n") else {
        return (fm, input);
    };

    // Handle closing fence at start of rest (empty block) or after content
    let (end, skip) = if rest.starts_with("---\n") {
        (0, 4)
    } else if let Some(pos) = rest.find("\n---\n") {
        (pos + 1, 4)
    } else {
        return (fm, input);
    };

    let block = &rest[..end];
    let after = &rest[end + skip..];

    for line in block.lines() {
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim().trim_matches('"').trim_matches('\'').to_string();
            match key {
                "title" => fm.title = Some(val),
                "description" => fm.description = Some(val),
                "author" => fm.author = Some(val),
                "date" => fm.date = Some(val),
                _ => {}
            }
        }
    }

    (fm, after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_front_matter_all_fields() {
        let input = "---\ntitle: My Title\ndescription: A description\nauthor: Alice\ndate: 2026-01-01\n---\n# Hello";
        let (fm, rest) = parse_front_matter(input);
        assert_eq!(fm.title.as_deref(), Some("My Title"));
        assert_eq!(fm.description.as_deref(), Some("A description"));
        assert_eq!(fm.author.as_deref(), Some("Alice"));
        assert_eq!(fm.date.as_deref(), Some("2026-01-01"));
        assert_eq!(rest, "# Hello");
    }

    #[test]
    fn test_front_matter_strips_from_markdown() {
        let input = "---\ntitle: Test\n---\n# Heading";
        let (_, rest) = parse_front_matter(input);
        assert_eq!(rest, "# Heading");
        assert!(!rest.contains("---"));
        assert!(!rest.contains("title"));
    }

    #[test]
    fn test_front_matter_no_block_returns_input_unchanged() {
        let input = "# Just a heading\nNo front matter here.";
        let (fm, rest) = parse_front_matter(input);
        assert!(fm.title.is_none());
        assert!(fm.description.is_none());
        assert!(fm.author.is_none());
        assert!(fm.date.is_none());
        assert_eq!(rest, input);
    }

    #[test]
    fn test_front_matter_unclosed_returns_input_unchanged() {
        let input = "---\ntitle: Oops\n# No closing fence";
        let (fm, rest) = parse_front_matter(input);
        assert!(fm.title.is_none());
        assert_eq!(rest, input);
    }

    #[test]
    fn test_front_matter_quoted_values() {
        let input = "---\ntitle: \"Quoted Title\"\nauthor: 'Single Quoted'\n---\n";
        let (fm, _) = parse_front_matter(input);
        assert_eq!(fm.title.as_deref(), Some("Quoted Title"));
        assert_eq!(fm.author.as_deref(), Some("Single Quoted"));
    }

    #[test]
    fn test_front_matter_unknown_keys_ignored() {
        let input = "---\ntitle: Known\nunknown_key: ignored\n---\n";
        let (fm, _) = parse_front_matter(input);
        assert_eq!(fm.title.as_deref(), Some("Known"));
    }

    #[test]
    fn test_front_matter_partial_fields() {
        let input = "---\ntitle: Only Title\n---\n";
        let (fm, _) = parse_front_matter(input);
        assert_eq!(fm.title.as_deref(), Some("Only Title"));
        assert!(fm.description.is_none());
        assert!(fm.author.is_none());
        assert!(fm.date.is_none());
    }

    #[test]
    fn test_front_matter_empty_block() {
        let input = "---\n---\n# Content";
        let (fm, rest) = parse_front_matter(input);
        assert!(fm.title.is_none());
        assert_eq!(rest, "# Content");
    }
}
