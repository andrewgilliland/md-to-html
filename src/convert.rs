use std::sync::LazyLock;
use regex::Regex;

static CODE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`([^`\n]+)`").unwrap());
static IMG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\(([^)\n]+)\)").unwrap());
static LINK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\]\n]+)\]\(([^)\n]+)\)").unwrap());
static BOLD_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*|__(.+?)__").unwrap());
static ITALIC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*(.+?)\*|_(.+?)_").unwrap());
static STRIKE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"~~(.+?)~~").unwrap());
static OL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+\. (.+)").unwrap());

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn process_inline(text: &str) -> String {
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

pub fn convert(markdown: &str) -> String {
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
        let input = "![alt](img.png)";
        let output = convert(input);
        assert!(output.contains("<img"));
        assert!(!output.contains("<a href"));
    }
}
