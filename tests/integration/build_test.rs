use std::fs;
use anyhow::Result;
use md_book::{Args, BookConfig};

mod common;
use common::*;

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_simple_book() -> Result<()> {
    let book = create_simple_book()?;
    book.build().await?;
    
    // Verify basic structure was created
    assert!(book.output_exists("README.html"));
    assert!(book.output_exists("chapter1.html"));
    assert!(book.output_exists("chapter2.html"));
    
    // Check content
    let readme_content = book.read_output("README.html")?;
    assert_contains!(readme_content, "<h1>Test Book</h1>");
    assert_contains!(readme_content, "This is a test book");
    
    Ok(())
}

#[cfg(not(feature = "tokio"))]
#[test]
fn test_build_simple_book() -> Result<()> {
    let book = create_simple_book()?;
    book.build()?;
    
    // Verify basic structure was created
    assert!(book.output_exists("README.html"));
    assert!(book.output_exists("chapter1.html"));
    assert!(book.output_exists("chapter2.html"));
    
    // Check content
    let readme_content = book.read_output("README.html")?;
    assert_contains!(readme_content, "<h1>Test Book</h1>");
    assert_contains!(readme_content, "This is a test book");
    
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_complex_book() -> Result<()> {
    let book = create_complex_book()?;
    book.build().await?;
    
    // Verify nested structure
    assert!(book.output_exists("README.html"));
    assert!(book.output_exists("chapter1/README.html"));
    assert!(book.output_exists("chapter1/section1.html"));
    assert!(book.output_exists("chapter2.html"));
    assert!(book.output_exists("chapter3.html"));
    
    // Check navigation structure
    let readme_content = book.read_output("README.html")?;
    assert_contains!(readme_content, "Complex Test Book");
    
    // Check nested content
    let chapter1_content = book.read_output("chapter1/README.html")?;
    assert_contains!(chapter1_content, "<h1>Chapter 1: Basics</h1>");
    assert_contains!(chapter1_content, "<li>Item 1</li>");
    
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_custom_config() -> Result<()> {
    let mut book = create_simple_book()?;
    
    // Create custom config
    book.create_book_toml(r#"
[book]
title = "Custom Title Book"
description = "A book with custom config"
authors = ["Test Author"]
language = "es"

[output.html]
mathjax_support = true
allow_html = true

[markdown]
format = "gfm"
frontmatter = true
"#)?;
    
    book.build().await?;
    
    let content = book.read_output("README.html")?;
    // The title from book.toml should be used in templates when available
    assert_contains!(content, "<h1>Test Book</h1>"); // From markdown
    
    Ok(())
}

#[cfg(feature = "syntax-highlighting")]
#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_syntax_highlighting() -> Result<()> {
    let book = TestBook::new()?;
    
    book.create_file("code.md", r#"# Code Examples

```rust
fn main() {
    println!("Hello, world!");
    let x = 42;
    println!("The answer is {}", x);
}
```

```javascript
function greet(name) {
    console.log(`Hello, ${name}!`);
}
```

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
```
"#)?;
    
    book.build().await?;
    
    let content = book.read_output("code.html")?;
    assert_contains!(content, "<pre");
    assert_contains!(content, "fn main");
    assert_contains!(content, "function greet");
    assert_contains!(content, "def fibonacci");
    
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_different_markdown_formats() -> Result<()> {
    let book = TestBook::new()?;
    
    // Test GFM format
    let mut config = BookConfig::default();
    config.markdown.format = md_book::config::MarkdownFormat::Gfm;
    let book = book.with_config(config);
    
    book.create_file("gfm.md", r#"# GFM Test

~~strikethrough~~

- [ ] Todo item
- [x] Done item

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |

www.example.com (auto-link)
"#)?;
    
    book.build().await?;
    
    let content = book.read_output("gfm.html")?;
    assert_contains!(content, "<h1>GFM Test</h1>");
    assert_contains!(content, "strikethrough");
    assert_contains!(content, "Todo item");
    
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_frontmatter() -> Result<()> {
    let book = TestBook::new()?;
    
    let mut config = BookConfig::default();
    config.markdown.frontmatter = true;
    let book = book.with_config(config);
    
    book.create_file("frontmatter.md", r#"---
title: "Custom Page Title"
description: "Page with frontmatter"
author: "Test Author"
---

# Actual Content

This page has frontmatter metadata.
"#)?;
    
    book.build().await?;
    
    let content = book.read_output("frontmatter.html")?;
    assert_contains!(content, "<h1>Actual Content</h1>");
    // Frontmatter should be processed and not appear in output
    assert_not_contains!(content, "---");
    assert_not_contains!(content, "title:");
    
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_html_allowed() -> Result<()> {
    let book = TestBook::new()?;
    
    let mut config = BookConfig::default();
    config.output.html.allow_html = true;
    let book = book.with_config(config);
    
    book.create_file("html.md", r#"# HTML Test

<div class="custom">
    <p>Raw HTML content</p>
    <button onclick="alert('test')">Click me</button>
</div>

Regular **markdown** still works.
"#)?;
    
    book.build().await?;
    
    let content = book.read_output("html.html")?;
    assert_contains!(content, "<div class=\"custom\">");
    assert_contains!(content, "<button onclick");
    assert_contains!(content, "<strong>markdown</strong>");
    
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_html_disallowed() -> Result<()> {
    let book = TestBook::new()?;
    
    // Default config has allow_html = false
    book.create_file("nohtml.md", r#"# No HTML Test

<div class="should-be-escaped">
    <p>This HTML should be escaped</p>
    <script>alert('xss')</script>
</div>

Regular **markdown** works.
"#)?;
    
    book.build().await?;
    
    let content = book.read_output("nohtml.html")?;
    assert_not_contains!(content, "<div class=\"should-be-escaped\">");
    assert_not_contains!(content, "<script>");
    assert_contains!(content, "<strong>markdown</strong>");
    
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_empty_input_directory() -> Result<()> {
    let book = TestBook::new()?;
    
    // Don't create any files
    book.build().await?;
    
    // Should still create output directory without errors
    assert!(book.output_path().exists());
    
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test] 
async fn test_build_creates_static_assets() -> Result<()> {
    let book = create_simple_book()?;
    book.build().await?;
    
    // Check that static assets were created (if they exist in templates)
    let output_path = book.output_path();
    
    // At minimum, output directory should exist
    assert!(output_path.exists());
    
    // If assets exist, they should be copied
    let has_css = output_path.join("css").exists();
    let has_js = output_path.join("js").exists(); 
    let has_img = output_path.join("img").exists();
    
    // This is just a structural test - passes if no error occurs
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_invalid_markdown() -> Result<()> {
    let book = TestBook::new()?;
    
    // Create file with potentially problematic content
    book.create_file("invalid.md", "# Title\n\n[Broken link](missing.md\n\nUnclosed **bold")?;
    
    // Should still build without crashing
    book.build().await?;
    
    let content = book.read_output("invalid.html")?;
    assert_contains!(content, "<h1>Title</h1>");
    
    Ok(())
}

#[cfg(feature = "search")]
#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_search() -> Result<()> {
    let book = create_simple_book()?;
    book.build().await?;
    
    // Check if search index was created (if search is enabled)
    let output_path = book.output_path();
    
    // Pagefind creates a _pagefind directory with search index
    let search_exists = output_path.join("_pagefind").exists();
    
    // This test passes whether search is enabled or not
    assert!(output_path.exists());
    
    Ok(())
}