use crate::convert::escape_html;
use crate::frontmatter::FrontMatter;

pub fn wrap_html(body: &str, fm: &FrontMatter) -> String {
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
