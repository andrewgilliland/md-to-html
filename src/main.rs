use std::fs;

fn convert(markdown: &str) -> String {
    let mut html = String::new();

    for line in markdown.lines() {
        if line.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", &line[2..]));
        } else if line.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", &line[3..]));
        } else if line.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", &line[4..]));
        } else {
            html.push_str(&format!("<p>{}</p>\n", line));
        }
    }

    html
}

fn main() {
    let markdown = fs::read_to_string("input.md")
        .expect("Could not read file");

    let html = convert(&markdown);
    println!("{}", html);
}