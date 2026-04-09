# md-to-html

> Converts Markdown to HTML. No dependencies. No tests. No regrets. Blazingly fast™.

_(Narrator: it has dependencies. It has tests. There are regrets.)_

---

## What is this?

A hand-crafted, artisanal, small-batch Markdown-to-HTML converter written in Rust — because clearly `pandoc`, `marked`, and every other mature battle-tested tool were not an option.

It reads a file called `input.md`, converts it to HTML, and writes it to `dist/index.html`. That's it. That's the whole product.

## Features

- ✅ Headings (h1 through h6, we're not animals)
- ✅ **Bold**, _italic_, ~~strikethrough~~, and `inline code`
- ✅ Links and images (unverified, we trust you)
- ✅ Unordered lists
- ✅ Ordered lists (numbers optional, vibes required)
- ✅ Blockquotes (for your inner philosopher)
- ✅ Fenced code blocks with syntax highlighting classes (the highlighting itself is your problem)
- ✅ Horizontal rules (---) for dramatic effect
- ✅ HTML escaping in code blocks (we take security semi-seriously)
- ✅ 32 unit tests (so technically the description on GitHub is a lie)
- ✅ A GitHub Actions CI pipeline (for a ~150 line Rust file, as one does)

## Installation

```bash
git clone https://github.com/you/md-to-html
cd md-to-html
cargo build --release
```

Requires Rust. If you don't have Rust, visit [rustup.rs](https://rustup.rs) and add another toolchain to your machine that you'll use twice.

## Usage

Put your Markdown in `input.md`, then run:

```bash
cargo convert
```

Or, if you're too good for aliases:

```bash
cargo run
```

Your HTML will appear in `dist/index.html`, fully structured with an embedded CSS stylesheet, because we're not savages.

## Running Tests

```bash
cargo test
```

32 tests. All green. We checked.

## Why Rust?

Great question. Rust gives us:

- ⚡ Blazing fast™ performance for a task that takes 2ms in any language
- 🦀 The ability to put "Rust" on a resume
- 😤 A compiler that argues with you more than any coworker ever has

## Why not just use `pulldown-cmark`?

We could have used `pulldown-cmark` — a full CommonMark-compliant Markdown parser that handles every edge case, written by experts, battle-tested by millions.

We did not do that.

## Limitations

- No nested lists (your indentation addiction ends here)
- No tables (use a spreadsheet like a normal person)
- No footnotes (nobody reads those anyway)
- Only processes `input.md` (hardcoded, no regrets)
- Not CommonMark compliant (CommonMark has 652 spec examples, we have vibes)

## License

Do whatever you want. We're just happy you're here.
