use std::fs;

mod convert;
mod frontmatter;
mod template;

fn main() {
    let raw = fs::read_to_string("input.md")
        .expect("Could not read file");

    let (fm, markdown) = frontmatter::parse_front_matter(&raw);
    let body = convert::convert(markdown);
    let html = template::wrap_html(&body, &fm);

    fs::create_dir_all("dist").expect("Could not create dist directory");
    fs::write("dist/index.html", &html).expect("Could not write dist/index.html");

    println!("Written to dist/index.html");
}

