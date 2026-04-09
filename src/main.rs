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

fn main() {
    let markdown = fs::read_to_string("input.md")
        .expect("Could not read file");

    let html = convert(&markdown);
    println!("{}", html);
}