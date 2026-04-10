---
title: md-to-html Test Page
description: Testing all supported Markdown elements
author: Andrew
date: 2026-04-09
---

# Heading 1

## Heading 2

### Heading 3

#### Heading 4

##### Heading 5

###### Heading 6

---

## Inline Formatting

This is **bold text** and this is **also bold**.

This is _italic text_ and this is _also italic_.

This is ~~strikethrough~~ text.

This is `inline code` inside a sentence.

Combined: **bold and _nested italic_** together.

---

## Links and Images

[Visit Rust Lang](https://www.rust-lang.org)

![Rust Logo](https://www.rust-lang.org/logos/rust-logo-512x512.png)

---

## Blockquote

> This is a blockquote.
> It can contain **bold** and _italic_ text too.

---

## Unordered List

- First item
- Second item with **bold**
- Third item with `code`

## Ordered List

1. First step
2. Second step
3. Third step with _emphasis_

---

## Code Block

```rust
fn main() {
    let x: u32 = 42;
    println!("The answer is {}", x);
}
```

```
plain code block
no language specified
```

## Paragraph

This is a regular paragraph with a [link](https://example.com), some **bold**, some _italic_, and some `code` all mixed together.

This is a second paragraph to make sure blank lines between paragraphs work correctly.
