use std::fs;
use std::sync::LazyLock;
use regex::Regex;

static CODE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`([^`\n]+)`").unwrap());
static IMG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\(([^)\n]+)\)").unwrap());
static LINK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\]\n]+)\]\(([^)\n]+)\)").unwrap());
static BOLD_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*|__(.+?)__").unwrap());
static ITALIC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*(.+?)\*|_(.+?)_").unwrap());
static STRIKE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"~~(.+?)~~").unwrap());
static OL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+\. (.+)").unwrap());

struct FrontMatter {
    title: Option<String>,
    description: Option<String>,
    author: Option<String>,
    date: Option<String>,
}

/// Strips and parses a YAML front matter block from the top of the document.
/// Returns parsed fields and the remaining markdown content.
fn parse_front_matter(input: &str) -> (FrontMatter, &str) {
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

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn process_inline(text: &str) -> String {
    let mut result = text.to_string();

    // Inline code first (protects content from further processing)
    result = CODE_RE.replace_all(&result, |caps: &regex::Captures| {
        format!("<code>{}</code>", escape_html(&caps[1]))
    }).to_string();

    // Images before links (image syntax contains link syntax)
    result = IMG_RE.replace_all(&result, "<img src=\"$2\" alt=\"$1\">").to_string();

    // Links
    result = LINK_RE.replace_all(&result, "<a href=\"$2\">$1</a>").to_string();

    // Bold before italic (handles *** correctly)
    result = BOLD_RE.replace_all(&result, |caps: &regex::Captures| {
        let inner = caps.get(1).or(caps.get(2)).map_or("", |m| m.as_str());
        format!("<strong>{}</strong>", inner)
    }).to_string();

    // Italic
    result = ITALIC_RE.replace_all(&result, |caps: &regex::Captures| {
        let inner = caps.get(1).or(caps.get(2)).map_or("", |m| m.as_str());
        format!("<em>{}</em>", inner)
    }).to_string();

    // Strikethrough
    result = STRIKE_RE.replace_all(&result, "<del>$1</del>").to_string();

    result
}

fn close_blocks(html: &mut String, in_ul: &mut bool, in_ol: &mut bool, in_blockquote: &mut bool) {
    if *in_ul { html.push_str("</ul>\n"); *in_ul = false; }
    if *in_ol { html.push_str("</ol>\n"); *in_ol = false; }
    if *in_blockquote { html.push_str("</blockquote>\n"); *in_blockquote = false; }
}

fn convert(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_ul = false;
    let mut in_ol = false;
    let mut in_blockquote = false;

    for line in markdown.lines() {
        // Code block fence
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
                let lang = line[3..].trim();
                if lang.is_empty() {
                    html.push_str("<pre><code>");
                } else {
                    html.push_str(&format!("<pre><code class=\"language-{}\">", escape_html(lang)));
                }
                in_code_block = true;
            }
            continue;
        }

        // Inside code block — output verbatim with HTML escaping
        if in_code_block {
            html.push_str(&format!("{}\n", escape_html(line)));
            continue;
        }

        // Empty line closes open blocks
        if line.trim().is_empty() {
            close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
            continue;
        }

        let trimmed = line.trim();

        // Horizontal rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
            html.push_str("<hr>\n");
        // Headings h1–h6
        } else if line.starts_with("# ") {
            close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
            html.push_str(&format!("<h1>{}</h1>\n", process_inline(&line[2..])));
        } else if line.starts_with("## ") {
            close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
            html.push_str(&format!("<h2>{}</h2>\n", process_inline(&line[3..])));
        } else if line.starts_with("### ") {
            close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
            html.push_str(&format!("<h3>{}</h3>\n", process_inline(&line[4..])));
        } else if line.starts_with("#### ") {
            close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
            html.push_str(&format!("<h4>{}</h4>\n", process_inline(&line[5..])));
        } else if line.starts_with("##### ") {
            close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
            html.push_str(&format!("<h5>{}</h5>\n", process_inline(&line[6..])));
        } else if line.starts_with("###### ") {
            close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
            html.push_str(&format!("<h6>{}</h6>\n", process_inline(&line[7..])));
        // Blockquote
        } else if line.starts_with("> ") {
            if in_ul { html.push_str("</ul>\n"); in_ul = false; }
            if in_ol { html.push_str("</ol>\n"); in_ol = false; }
            if !in_blockquote {
                html.push_str("<blockquote>\n");
                in_blockquote = true;
            }
            html.push_str(&format!("  <p>{}</p>\n", process_inline(&line[2..])));
        // Unordered list
        } else if line.starts_with("- ") || line.starts_with("* ") {
            if in_ol { html.push_str("</ol>\n"); in_ol = false; }
            if in_blockquote { html.push_str("</blockquote>\n"); in_blockquote = false; }
            if !in_ul {
                html.push_str("<ul>\n");
                in_ul = true;
            }
            html.push_str(&format!("  <li>{}</li>\n", process_inline(&line[2..])));
        // Ordered list
        } else if let Some(caps) = OL_RE.captures(line) {
            if in_ul { html.push_str("</ul>\n"); in_ul = false; }
            if in_blockquote { html.push_str("</blockquote>\n"); in_blockquote = false; }
            if !in_ol {
                html.push_str("<ol>\n");
                in_ol = true;
            }
            html.push_str(&format!("  <li>{}</li>\n", process_inline(&caps[1])));
        // Paragraph
        } else {
            close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
            html.push_str(&format!("<p>{}</p>\n", process_inline(line)));
        }
    }

    // Close any remaining open blocks
    close_blocks(&mut html, &mut in_ul, &mut in_ol, &mut in_blockquote);
    if in_code_block {
        html.push_str("</code></pre>\n");
    }

    html
}

fn wrap_html(body: &str, fm: &FrontMatter) -> String {
    let title = fm.title.as_deref().unwrap_or("Untitled");
    let title = escape_html(title);

    let meta_description = fm.description.as_deref().map(|d| {
        format!("  <meta name=\"description\" content=\"{}\">\n", escape_html(d))
    }).unwrap_or_default();

    let meta_author = fm.author.as_deref().map(|a| {
        format!("  <meta name=\"author\" content=\"{}\">\n", escape_html(a))
    }).unwrap_or_default();

    let date_banner = match (&fm.author, &fm.date) {
        (Some(author), Some(date)) => format!(
            "  <p class=\"meta\">By {} &mdash; {}</p>\n",
            escape_html(author), escape_html(date)
        ),
        (Some(author), None) => format!("  <p class=\"meta\">By {}</p>\n", escape_html(author)),
        (None, Some(date)) => format!("  <p class=\"meta\">{}</p>\n", escape_html(date)),
        (None, None) => String::new(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title}</title>
{meta_description}{meta_author}  <style>
    *, *::before, *::after {{ box-sizing: border-box; }}

    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      font-size: 1rem;
      line-height: 1.7;
      color: #1a1a1a;
      background: #f9f9f9;
      margin: 0;
      padding: 2rem 1rem;
    }}

    main {{
      max-width: 740px;
      margin: 0 auto;
      background: #fff;
      padding: 2.5rem 3rem;
      border-radius: 8px;
      box-shadow: 0 2px 12px rgba(0,0,0,0.07);
    }}

    h1, h2, h3, h4, h5, h6 {{
      line-height: 1.3;
      margin-top: 2rem;
      margin-bottom: 0.5rem;
      font-weight: 600;
    }}
    h1 {{ font-size: 2rem; border-bottom: 2px solid #e0e0e0; padding-bottom: 0.3rem; }}
    h2 {{ font-size: 1.5rem; border-bottom: 1px solid #e0e0e0; padding-bottom: 0.2rem; }}
    h3 {{ font-size: 1.25rem; }}

    p {{ margin: 0.75rem 0; }}

    .meta {{
      color: #666;
      font-size: 0.9rem;
      margin-top: -0.5rem;
      margin-bottom: 1.5rem;
    }}

    a {{ color: #0070f3; text-decoration: none; }}
    a:hover {{ text-decoration: underline; }}

    img {{ max-width: 100%; height: auto; border-radius: 4px; }}

    strong {{ font-weight: 600; }}
    em {{ font-style: italic; }}
    del {{ opacity: 0.6; }}

    code {{
      font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
      font-size: 0.875em;
      background: #f0f0f0;
      padding: 0.15em 0.4em;
      border-radius: 4px;
    }}

    pre {{
      background: #1e1e1e;
      color: #d4d4d4;
      padding: 1.25rem 1.5rem;
      border-radius: 6px;
      overflow-x: auto;
      margin: 1.25rem 0;
    }}
    pre code {{
      background: none;
      padding: 0;
      font-size: 0.9rem;
      color: inherit;
    }}

    blockquote {{
      border-left: 4px solid #0070f3;
      margin: 1.25rem 0;
      padding: 0.5rem 1.25rem;
      background: #f0f7ff;
      border-radius: 0 4px 4px 0;
      color: #444;
    }}
    blockquote p {{ margin: 0.3rem 0; }}

    ul, ol {{
      padding-left: 1.75rem;
      margin: 0.75rem 0;
    }}
    li {{ margin: 0.3rem 0; }}

    hr {{
      border: none;
      border-top: 1px solid #e0e0e0;
      margin: 2rem 0;
    }}
  </style>
</head>
<body>
  <main>
{date_banner}{body}  </main>
</body>
</html>
"#
    )
}

fn main() {
    let raw = fs::read_to_string("input.md")
        .expect("Could not read file");

    let (fm, markdown) = parse_front_matter(&raw);
    let body = convert(markdown);
    let html = wrap_html(&body, &fm);

    fs::create_dir_all("dist").expect("Could not create dist directory");
    fs::write("dist/index.html", &html).expect("Could not write dist/index.html");

    println!("Written to dist/index.html");
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Headings ---
    #[test]
    fn test_h1() {
        assert_eq!(convert("# Hello"), "<h1>Hello</h1>\n");
    }

    #[test]
    fn test_h2() {
        assert_eq!(convert("## Hello"), "<h2>Hello</h2>\n");
    }

    #[test]
    fn test_h3() {
        assert_eq!(convert("### Hello"), "<h3>Hello</h3>\n");
    }

    #[test]
    fn test_h4() {
        assert_eq!(convert("#### Hello"), "<h4>Hello</h4>\n");
    }

    #[test]
    fn test_h5() {
        assert_eq!(convert("##### Hello"), "<h5>Hello</h5>\n");
    }

    #[test]
    fn test_h6() {
        assert_eq!(convert("###### Hello"), "<h6>Hello</h6>\n");
    }

    // --- Inline formatting ---
    #[test]
    fn test_bold_asterisk() {
        assert_eq!(convert("**bold**"), "<p><strong>bold</strong></p>\n");
    }

    #[test]
    fn test_bold_underscore() {
        assert_eq!(convert("__bold__"), "<p><strong>bold</strong></p>\n");
    }

    #[test]
    fn test_italic_asterisk() {
        assert_eq!(convert("*italic*"), "<p><em>italic</em></p>\n");
    }

    #[test]
    fn test_italic_underscore() {
        assert_eq!(convert("_italic_"), "<p><em>italic</em></p>\n");
    }

    #[test]
    fn test_strikethrough() {
        assert_eq!(convert("~~text~~"), "<p><del>text</del></p>\n");
    }

    #[test]
    fn test_inline_code() {
        assert_eq!(convert("`code`"), "<p><code>code</code></p>\n");
    }

    // --- Links & images ---
    #[test]
    fn test_link() {
        assert_eq!(
            convert("[Rust](https://rust-lang.org)"),
            "<p><a href=\"https://rust-lang.org\">Rust</a></p>\n"
        );
    }

    #[test]
    fn test_image() {
        assert_eq!(
            convert("![alt](img.png)"),
            "<p><img src=\"img.png\" alt=\"alt\"></p>\n"
        );
    }

    // --- Block elements ---
    #[test]
    fn test_paragraph() {
        assert_eq!(convert("hello world"), "<p>hello world</p>\n");
    }

    #[test]
    fn test_horizontal_rule_dashes() {
        assert_eq!(convert("---"), "<hr>\n");
    }

    #[test]
    fn test_horizontal_rule_asterisks() {
        assert_eq!(convert("***"), "<hr>\n");
    }

    #[test]
    fn test_horizontal_rule_underscores() {
        assert_eq!(convert("___"), "<hr>\n");
    }

    #[test]
    fn test_unordered_list_dash() {
        let input = "- one\n- two";
        let expected = "<ul>\n  <li>one</li>\n  <li>two</li>\n</ul>\n";
        assert_eq!(convert(input), expected);
    }

    #[test]
    fn test_unordered_list_asterisk() {
        let input = "* one\n* two";
        let expected = "<ul>\n  <li>one</li>\n  <li>two</li>\n</ul>\n";
        assert_eq!(convert(input), expected);
    }

    #[test]
    fn test_ordered_list() {
        let input = "1. first\n2. second";
        let expected = "<ol>\n  <li>first</li>\n  <li>second</li>\n</ol>\n";
        assert_eq!(convert(input), expected);
    }

    #[test]
    fn test_blockquote() {
        let input = "> hello";
        let expected = "<blockquote>\n  <p>hello</p>\n</blockquote>\n";
        assert_eq!(convert(input), expected);
    }

    #[test]
    fn test_code_block_no_lang() {
        let input = "```\nfn main() {}\n```";
        let expected = "<pre><code>fn main() {}\n</code></pre>\n";
        assert_eq!(convert(input), expected);
    }

    #[test]
    fn test_code_block_with_lang() {
        let input = "```rust\nlet x = 1;\n```";
        assert!(convert(input).contains("class=\"language-rust\""));
    }

    // --- Security (HTML escaping) ---
    #[test]
    fn test_code_block_escapes_html() {
        let input = "```\n<script>alert(1)</script>\n```";
        let output = convert(input);
        assert!(!output.contains("<script>"));
        assert!(output.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_inline_code_escapes_html() {
        assert_eq!(convert("`<b>`"), "<p><code>&lt;b&gt;</code></p>\n");
    }

    // --- State / edge cases ---
    #[test]
    fn test_empty_input() {
        assert_eq!(convert(""), "");
    }

    #[test]
    fn test_blank_line_closes_list() {
        let input = "- item\n\n# After";
        let output = convert(input);
        assert!(output.contains("</ul>"));
        assert!(output.contains("<h1>After</h1>"));
    }

    #[test]
    fn test_blank_line_closes_blockquote() {
        let input = "> quote\n\n# After";
        let output = convert(input);
        assert!(output.contains("</blockquote>"));
        assert!(output.contains("<h1>After</h1>"));
    }

    #[test]
    fn test_inline_formatting_in_list_item() {
        let input = "- **bold** item";
        assert!(convert(input).contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_inline_formatting_in_heading() {
        let input = "# Hello `world`";
        assert!(convert(input).contains("<code>world</code>"));
    }

    #[test]
    fn test_image_before_link_match() {
        // Image syntax must not be consumed by the link regex
        let input = "![alt](img.png)";
        let output = convert(input);
        assert!(output.contains("<img"));
        assert!(!output.contains("<a href"));
    }

    // --- Front matter ---
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