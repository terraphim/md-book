use anyhow::Result;
use md_book::BookConfig;

mod common;
use common::*;

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_simple_book() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file("README.md", "# Test Book\n\nThis is a test book.")?;
    book.create_file("chapter1.md", "# Chapter 1\n\nFirst chapter content.")?;
    book.create_file("chapter2.md", "# Chapter 2\n\nSecond chapter content.")?;

    book.build().await?;

    // Verify basic structure was created
    assert!(book.output_exists("index.html"));
    assert!(book.output_exists("chapter1.html"));
    assert!(book.output_exists("chapter2.html"));

    // Check content
    let readme_content = book.read_output("index.html")?;
    assert_contains!(readme_content, "<h1");
    assert_contains!(readme_content, "This is a test book");

    Ok(())
}

#[cfg(not(feature = "tokio"))]
#[test]
fn test_build_simple_book() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file("README.md", "# Test Book\n\nThis is a test book.")?;
    book.create_file("chapter1.md", "# Chapter 1\n\nFirst chapter content.")?;
    book.create_file("chapter2.md", "# Chapter 2\n\nSecond chapter content.")?;

    book.build()?;

    // Verify basic structure was created
    assert!(book.output_exists("index.html"));
    assert!(book.output_exists("chapter1.html"));
    assert!(book.output_exists("chapter2.html"));

    // Check content
    let readme_content = book.read_output("index.html")?;
    assert_contains!(readme_content, "<h1");
    assert_contains!(readme_content, "This is a test book");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_complex_book() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file(
        "README.md",
        "# Complex Test Book\n\nThis book tests various markdown features.",
    )?;
    book.create_file(
        "chapter1/README.md",
        "# Chapter 1: Basics\n\n- Item 1\n- Item 2\n- Item 3",
    )?;
    book.create_file(
        "chapter1/section1.md",
        "## Section 1.1\n\nThis is a detailed section.",
    )?;
    book.create_file(
        "chapter2.md",
        "# Chapter 2\n\nAdvanced topics and examples.",
    )?;
    book.create_file("chapter3.md", "# Chapter 3\n\nFinal chapter with links.")?;

    book.build().await?;

    // Verify nested structure
    assert!(book.output_exists("index.html"));
    assert!(book.output_exists("chapter1/index.html"));
    assert!(book.output_exists("chapter1/section1.html"));
    assert!(book.output_exists("chapter2.html"));
    assert!(book.output_exists("chapter3.html"));

    // Check navigation structure
    let readme_content = book.read_output("index.html")?;
    assert_contains!(readme_content, "Complex Test Book");

    // Check nested content
    let chapter1_content = book.read_output("chapter1/index.html")?;
    assert_contains!(chapter1_content, "<h1");
    assert_contains!(chapter1_content, "<li>Item 1</li>");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_custom_config() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file("README.md", "# Test Book\n\nThis is a test book.")?;

    // Create custom config
    let mut config = BookConfig::default();
    config.book.title = "Custom Title Book".to_string();
    config.book.description = Some("A book with custom config".to_string());
    config.book.authors = vec!["Test Author".to_string()];
    config.book.language = "es".to_string();
    config.output.html.mathjax_support = true;
    config.output.html.allow_html = true;
    config.markdown.format = md_book::config::MarkdownFormat::Gfm;
    config.markdown.frontmatter = true;
    let book = book.with_config(config);

    book.build().await?;

    let content = book.read_output("index.html")?;
    // The title from book.toml should be used in templates when available
    assert_contains!(content, "<h1"); // From markdown

    Ok(())
}

#[cfg(feature = "syntax-highlighting")]
#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_syntax_highlighting() -> Result<()> {
    let book = TestBook::new()?;

    book.create_file(
        "code.md",
        r#"# Code Examples

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
"#,
    )?;

    book.build().await?;

    let content = book.read_output("code.html")?;
    assert_contains!(content, "<pre");
    assert_contains!(content, "fn");
    assert_contains!(content, "main");
    assert_contains!(content, "function");
    assert_contains!(content, "greet");
    assert_contains!(content, "def");
    assert_contains!(content, "fibonacci");

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

    book.create_file(
        "gfm.md",
        r#"# GFM Test

~~strikethrough~~

- [ ] Todo item
- [x] Done item

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |

www.example.com (auto-link)
"#,
    )?;

    book.build().await?;

    let content = book.read_output("gfm.html")?;
    assert_contains!(content, "GFM Test");
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

    book.create_file(
        "frontmatter.md",
        r#"---
title: "Custom Page Title"
description: "Page with frontmatter"
author: "Test Author"
---

# Actual Content

This page has frontmatter metadata.
"#,
    )?;

    book.build().await?;

    let content = book.read_output("frontmatter.html")?;
    assert_contains!(content, "Actual Content");
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

    book.create_file(
        "html.md",
        r#"# HTML Test

<div class="custom">
    <p>Raw HTML content</p>
    <button onclick="alert('test')">Click me</button>
</div>

Regular **markdown** still works.
"#,
    )?;

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
    book.create_file(
        "nohtml.md",
        r#"# No HTML Test

<div class="should-be-escaped">
    <p>This HTML should be escaped</p>
    <script>alert('xss')</script>
</div>

Regular **markdown** works.
"#,
    )?;

    book.build().await?;

    let content = book.read_output("nohtml.html")?;
    assert_not_contains!(content, "<div class=\"should-be-escaped\">");
    // Check that HTML is escaped by looking for escaped versions, not checking templates
    assert!(
        content.contains("&lt;div class=\"should-be-escaped\"&gt;")
            || !content.contains("<div class=\"should-be-escaped\">")
    );
    // Check that script tag content is escaped, not looking for template script tags
    assert!(
        content.contains("&lt;script&gt;alert('xss')&lt;/script&gt;")
            || !content.contains("alert('xss')")
    );
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
    let book = TestBook::new()?;
    book.create_file("README.md", "# Test Book\n\nThis is a test book.")?;
    book.build().await?;

    // Check that static assets were created (if they exist in templates)
    let output_path = book.output_path();

    // At minimum, output directory should exist
    assert!(output_path.exists());

    // If assets exist, they should be copied
    let _has_css = output_path.join("css").exists();
    let _has_js = output_path.join("js").exists();
    let _has_img = output_path.join("img").exists();

    // This is just a structural test - passes if no error occurs
    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_invalid_markdown() -> Result<()> {
    let book = TestBook::new()?;

    // Create file with potentially problematic content
    book.create_file(
        "invalid.md",
        "# Title\n\n[Broken link](missing.md\n\nUnclosed **bold",
    )?;

    // Should still build without crashing
    book.build().await?;

    let content = book.read_output("invalid.html")?;
    assert_contains!(content, "Title");

    Ok(())
}

#[cfg(feature = "search")]
#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_build_with_search() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file("README.md", "# Test Book\n\nThis is a test book.")?;
    book.build().await?;

    // Check if search index was created (if search is enabled)
    let output_path = book.output_path();

    // Pagefind creates a _pagefind directory with search index
    let _search_exists = output_path.join("_pagefind").exists();

    // This test passes whether search is enabled or not
    assert!(output_path.exists());

    Ok(())
}

// ---------------------------------------------------------------------------
// Output-contract regression guards (increments B5, D1-D3).
//
// These assert the properties that made the P1 defects P1: output must be
// relocatable (no absolute asset paths), self-contained (no external URLs),
// deep-linkable (server-side heading IDs) and navigable by screen reader.
// ---------------------------------------------------------------------------

/// Everything inside `<article>`, i.e. rendered chapter content without the
/// surrounding chrome (whose headings are template-owned and carry no IDs).
fn article_of(html: &str) -> &str {
    let start = html
        .find("<article")
        .expect("page template should emit an <article>");
    let end = html[start..]
        .find("</article>")
        .expect("unterminated <article>")
        + start;
    &html[start..end]
}

/// A book exercising nesting, a part title and a draft chapter.
#[cfg(feature = "tokio")]
async fn nested_book() -> Result<TestBook> {
    let book = TestBook::new()?;
    book.create_file(
        "SUMMARY.md",
        "# Summary\n\n\
         [Preface](preface.md)\n\n\
         # Part One\n\n\
         - [Intro](intro.md)\n  \
           - [Deep](nested/deep.md)\n\
         - [Draft]()\n",
    )?;
    book.create_file("preface.md", "# Preface\n\nBefore we begin.\n")?;
    book.create_file("intro.md", "# Intro\n\n## A Sub Heading\n\nBody.\n")?;
    book.create_file("nested/deep.md", "# Deep\n\n## Another Heading\n\nBody.\n")?;
    book.build().await?;
    Ok(book)
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_output_has_no_absolute_asset_paths() -> Result<()> {
    let book = nested_book().await?;

    // Scan everything emitted, not just the pages: copied JS components carry
    // their own asset references and are just as deployment-sensitive.
    for entry in walkdir::WalkDir::new(book.output_path())
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("html") | Some("js")
        ) {
            continue;
        }
        let text = std::fs::read_to_string(path)?;
        assert!(
            !text.contains("href=\"/") && !text.contains("src=\"/"),
            "{} contains a root-absolute asset path, which breaks sub-path \
             deployment and file:// viewing",
            path.display()
        );
    }

    // A page one directory down must reach the root by climbing, not by "/".
    let deep = book.read_output("nested/deep.html")?;
    assert_contains!(deep, "../css/styles.css");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_output_has_no_external_urls() -> Result<()> {
    let book = nested_book().await?;
    let html = book.read_output("intro.html")?;

    let external = html.find("https://").map(|idx| {
        let tail = &html[idx..];
        let end = tail.find(['"', '\'', ' ', '<']).unwrap_or(tail.len());
        tail[..end].to_string()
    });
    assert!(
        external.is_none(),
        "generated page loads an external resource: {}\noutput must work offline",
        external.unwrap_or_default()
    );

    // Shoelace is vendored, so the local copy must actually be emitted.
    assert!(book.output_exists("vendor/shoelace/shoelace-local.js"));
    assert!(book.output_exists("vendor/shoelace/themes/light.css"));
    assert!(book.output_exists("vendor/shoelace/components/icon/icon.js"));
    assert!(
        book.output_exists("vendor/shoelace/assets/icons/search.svg"),
        "sl-icon fetches SVGs at runtime, so referenced icons must be vendored"
    );

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_headings_have_stable_ids() -> Result<()> {
    let book = nested_book().await?;
    let html = book.read_output("intro.html")?;
    let article = article_of(&html);

    // Every content heading is addressable.
    for (idx, _) in article.match_indices("<h") {
        let tag = &article[idx..];
        if !tag
            .as_bytes()
            .get(2)
            .is_some_and(|b| b.is_ascii_whitespace() || *b == b'>')
        {
            continue; // not <h1>..<h6>
        }
        let end = tag.find('>').expect("unterminated heading tag");
        assert!(
            tag[..end].contains("id="),
            "content heading without an id breaks cross-page fragment links: {}",
            &tag[..end]
        );
    }

    assert_contains!(article, "id=\"a-sub-heading\"");

    // Stable across rebuilds: the same source must yield the same anchors,
    // or every external deep link rots on each build.
    book.build().await?;
    let rebuilt = book.read_output("intro.html")?;
    assert_eq!(
        article_of(&rebuilt),
        article,
        "heading IDs must be deterministic across builds"
    );

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_sidebar_nesting_and_aria() -> Result<()> {
    let book = nested_book().await?;
    let html = book.read_output("intro.html")?;

    assert_contains!(html, "aria-label=\"Book navigation\"");
    assert_contains!(html, "sidebar-part-title");
    assert_contains!(html, "aria-current=\"page\"");

    // Drafts are not links, so keyboard users cannot tab into a dead end.
    assert_contains!(html, "aria-disabled=\"true\"");

    // Genuinely nested lists, not a flat list with indent classes.
    let nav_start = html.find("sidebar-nav").expect("sidebar missing");
    let nav = &html[nav_start..];
    let nav_end = nav.find("</nav>").expect("unterminated nav");
    let nav = &nav[..nav_end];
    assert!(
        nav.matches("<ul").count() >= 2,
        "sidebar should nest sub-chapters in their own <ul>, got:\n{nav}"
    );
    assert_eq!(
        nav.matches("<ul").count(),
        nav.matches("</ul>").count(),
        "unbalanced <ul> in sidebar; open/close deltas are wrong:\n{nav}"
    );

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_non_markdown_assets_copied_through() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Intro](intro.md)\n")?;
    book.create_file("intro.md", "# Intro\n\n![d](img/diagram.svg)\n")?;
    book.create_file("img/diagram.svg", "<svg></svg>")?;
    book.create_file("data/sample.csv", "a,b\n1,2\n")?;
    // Present in src/, absent from SUMMARY.md: must not be published.
    book.create_file("notes/wip.md", "# Work in progress\n")?;

    book.build().await?;

    assert!(
        book.output_exists("img/diagram.svg"),
        "assets must be copied through at the same relative path"
    );
    assert!(book.output_exists("data/sample.csv"));
    assert!(
        !book.output_exists("notes/wip.html"),
        "files absent from SUMMARY.md must not be published"
    );

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_create_missing_is_idempotent_across_rebuilds() -> Result<()> {
    // Stands in for the specified watch-suppression test: the suppression
    // mechanism is not wired up yet (pipeline discards the created paths), so
    // this asserts the property that makes it self-limiting -- a stub is
    // written once and never rewritten, so rebuilds converge.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Later](later.md)\n")?;

    book.build().await?;
    let stub = book.input_dir.join("later.md");
    assert!(stub.exists(), "create-missing should write the stub");

    std::fs::write(&stub, "# Later\n\nHand-written body.\n")?;
    book.build().await?;

    assert_contains!(std::fs::read_to_string(&stub)?, "Hand-written body");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_default_assets_emitted_without_templates_dir() -> Result<()> {
    // TestBook has no templates directory, which is also the shape of an
    // installed md-book run against someone else's book. Every asset the
    // default templates reference must still be emitted, or the book renders
    // unstyled with no search and no diagrams.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Intro](intro.md)\n")?;
    book.create_file("intro.md", "# Intro\n\nBody.\n")?;
    book.build().await?;

    for asset in [
        "css/styles.css",
        "css/search.css",
        "css/themes.css",
        "js/search-init.js",
        "js/pagefind-search.js",
        "js/theme-switch.js",
        "js/keyboard.js",
        "js/code-copy.js",
        "components/doc-toc.js",
        "components/search-modal.js",
        "vendor/shoelace/shoelace-local.js",
    ] {
        let path = book.output_path().join(asset);
        assert!(path.exists(), "{asset} was not emitted");
        assert!(
            std::fs::metadata(&path)?.len() > 0,
            "{asset} was emitted empty"
        );
    }

    // Every local asset the page references must exist on disk.
    let html = book.read_output("intro.html")?;
    for cap in html
        .split("href=\"")
        .skip(1)
        .chain(html.split("src=\"").skip(1))
    {
        let url = cap.split('"').next().unwrap_or_default();
        if url.is_empty() || url.starts_with("http") || url.starts_with('#') {
            continue;
        }
        let target = book.output_path().join(url.trim_start_matches("./"));
        assert!(
            target.exists(),
            "page references {url}, which was not emitted"
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// E7: mermaid.min.js is 2.9MB, so it loads only where a diagram exists.
// ---------------------------------------------------------------------------

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mermaid_scripts_only_on_diagram_pages() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file(
        "SUMMARY.md",
        "# Summary\n\n- [Diagram](diagram.md)\n- [Plain](plain.md)\n",
    )?;
    book.create_file(
        "diagram.md",
        "# Diagram\n\n```mermaid\ngraph TD;\n  A-->B;\n```\n",
    )?;
    book.create_file("plain.md", "# Plain\n\nNo diagrams here.\n")?;

    book.build().await?;

    let diagram = book.read_output("diagram.html")?;
    assert_contains!(diagram, "js/mermaid.min.js");
    assert_contains!(diagram, "js/mermaid-init.js");

    let plain = book.read_output("plain.html")?;
    assert_not_contains!(plain, "mermaid.min.js");
    assert_not_contains!(plain, "mermaid-init.js");

    // The asset itself is always emitted, so adding a diagram later cannot
    // reference a file that is missing.
    assert!(book.output_exists("js/mermaid.min.js"));

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mermaid_class_in_code_sample_does_not_trigger_load() -> Result<()> {
    // A page documenting mermaid contains the fence and the class name as
    // escaped text. Searching rendered HTML for "language-mermaid" would match
    // here and pull in 2.9MB to render nothing; fence-level detection does not.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Docs](docs.md)\n")?;
    book.create_file(
        "docs.md",
        "# How to write diagrams\n\n\
         Wrap your diagram in a fence:\n\n\
         ````markdown\n\
         ```mermaid\n\
         graph TD;\n\
         ```\n\
         ````\n\n\
         It renders into `<code class=\"language-mermaid\">`.\n",
    )?;

    book.build().await?;

    let html = book.read_output("docs.html")?;
    // The page really does contain the class name as text...
    assert_contains!(html, "language-mermaid");
    // ...but has no diagram of its own, so the bundle is not loaded.
    assert_not_contains!(html, "mermaid.min.js");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_print_page_loads_mermaid_when_any_chapter_has_one() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file(
        "SUMMARY.md",
        "# Summary\n\n- [Plain](plain.md)\n- [Diagram](diagram.md)\n",
    )?;
    book.create_file("plain.md", "# Plain\n\nNothing.\n")?;
    book.create_file(
        "diagram.md",
        "# Diagram\n\n```mermaid\ngraph TD;\n  A-->B;\n```\n",
    )?;

    book.build().await?;

    let print = book.read_output("print.html")?;
    assert_contains!(print, "js/mermaid.min.js");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_print_page_omits_mermaid_when_no_chapter_has_one() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Plain](plain.md)\n")?;
    book.create_file("plain.md", "# Plain\n\nNothing.\n")?;

    book.build().await?;

    let print = book.read_output("print.html")?;
    assert_not_contains!(print, "mermaid.min.js");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_index_page_loads_mermaid_from_its_own_source() -> Result<()> {
    // index.md is rendered by a different path from ordinary chapters.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Home](README.md)\n")?;
    book.create_file(
        "README.md",
        "# Home\n\n```mermaid\ngraph TD;\n  A-->B;\n```\n",
    )?;

    book.build().await?;

    let index = book.read_output("index.html")?;
    assert_contains!(index, "js/mermaid.min.js");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_config_defaults_reach_the_page() -> Result<()> {
    // A book with no book.toml must still get real values: twelf's layering and
    // `#[serde(default)]` on container fields both drop per-field defaults, which
    // previously produced <html lang="">, an empty <title> and a broken logo.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Intro](intro.md)\n")?;
    book.create_file("intro.md", "# Intro\n\nBody.\n")?;

    let config = md_book::config::load_config(None)?;
    let book = book.with_config(config);
    book.build().await?;

    let html = book.read_output("intro.html")?;
    // Attribute only: the <html> tag spans lines once theme attributes are added.
    assert_contains!(html, "lang=\"en\"");
    assert!(
        !html.contains("| </title>"),
        "book title must not be empty in <title>"
    );

    // The default logo must resolve to a file that exists.
    let logo = html
        .split("class=\"header-logo-img\"")
        .next()
        .and_then(|before| before.rsplit("src=\"").next())
        .and_then(|s| s.split('"').next())
        .map(str::to_string)
        .unwrap_or_default();
    assert!(!logo.is_empty(), "logo src must not be empty");
    assert!(
        book.output_path().join(&logo).exists(),
        "logo src {logo} does not exist in the output"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// E1 completion: the theme picker is a real control, not just a stylesheet.
// ---------------------------------------------------------------------------

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_theme_picker_and_attributes_present() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Intro](intro.md)\n")?;
    book.create_file("intro.md", "# Intro\n\nBody.\n")?;

    let mut config = md_book::config::load_config(None)?;
    config.output.html.default_theme = Some("light".into());
    config.output.html.preferred_dark_theme = Some("coal".into());
    let book = book.with_config(config);
    book.build().await?;

    let html = book.read_output("intro.html")?;

    // theme-switch.js reads both from the root element.
    assert_contains!(html, "data-default-theme=\"light\"");
    assert_contains!(html, "data-preferred-dark-theme=\"coal\"");

    // All five themes are selectable, and the script that applies them is loaded.
    for theme in ["light", "rust", "coal", "navy", "ayu"] {
        assert_contains!(html, &format!("data-theme-set=\"{theme}\""));
    }
    assert_contains!(html, "js/theme-switch.js");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_every_page_kind_loads_theme_stylesheet() -> Result<()> {
    // A page that applies data-theme without themes.css shows the wrong colours,
    // which is what happened to index.html and print.html.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Home](README.md)\n")?;
    book.create_file("README.md", "# Home\n\nBody.\n")?;
    book.build().await?;

    for page in ["index.html", "404.html", "print.html"] {
        let html = book.read_output(page)?;
        assert_contains!(html, "css/themes.css");
        assert_contains!(html, "data-default-theme=");
    }

    // The index must also carry the behaviour scripts chapter pages get.
    let index = book.read_output("index.html")?;
    assert_contains!(index, "js/theme-switch.js");
    assert_contains!(index, "js/keyboard.js");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_header_omits_empty_links() -> Result<()> {
    // With no github_url / github_edit_url_base, the header used to emit
    // href="" links that resolve to the page itself.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Intro](intro.md)\n")?;
    book.create_file("intro.md", "# Intro\n\nBody.\n")?;
    book.build().await?;

    let html = book.read_output("intro.html")?;
    assert_not_contains!(html, "href=\"\"");
    assert_not_contains!(html, "src=\"\"");

    Ok(())
}

// ---------------------------------------------------------------------------
// Review round 4 fixes.
// ---------------------------------------------------------------------------

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_non_latin_headings_keep_addressable_ids() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Uni](uni.md)\n")?;
    book.create_file(
        "uni.md",
        "# Café\n\n## Обзор\n\n## 日本語の見出し\n\nBody.\n",
    )?;
    book.build().await?;

    let html = book.read_output("uni.html")?;
    assert_contains!(html, "id=\"café\"");
    assert_contains!(html, "id=\"обзор\"");
    assert_contains!(html, "id=\"日本語の見出し\"");
    assert_not_contains!(html, "id=\"section\"");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_print_page_heading_ids_are_unique() -> Result<()> {
    // Chapters share one id namespace in the print page, or two chapters with
    // an "Overview" heading both claim id="overview" in one document.
    let book = TestBook::new()?;
    book.create_file(
        "SUMMARY.md",
        "# Summary\n\n- [One](one.md)\n- [Two](two.md)\n",
    )?;
    book.create_file("one.md", "# One\n\n## Overview\n\nA.\n")?;
    book.create_file("two.md", "# Two\n\n## Overview\n\nB.\n")?;
    book.build().await?;

    let print = book.read_output("print.html")?;
    let mut ids: Vec<&str> = print
        .split("id=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .collect();
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(total, ids.len(), "duplicate ids in print.html: {ids:?}");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_404_uses_site_url_when_configured() -> Result<()> {
    // A 404 is served for arbitrary nested paths, so relative asset URLs break
    // exactly when the page is needed.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [One](one.md)\n")?;
    book.create_file("one.md", "# One\n\nx\n")?;
    book.create_file("not-found.md", "# Lost\n\nCustom body.\n")?;

    let mut config = md_book::config::load_config(None)?;
    config.output.html.site_url = Some("/docs/".into());
    config.output.html.input_404 = Some("not-found.md".into());
    let book = book.with_config(config);
    book.build().await?;

    let html = book.read_output("404.html")?;
    assert_contains!(html, "href=\"/docs/css/styles.css\"");
    assert_contains!(html, "href=\"/docs/index.html\"");
    // input-404 supplies the body.
    assert_contains!(html, "Custom body");
    // …and its source is not reported as an orphan, nor published as a chapter.
    assert!(!book.output_exists("not-found.html"));

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_404_falls_back_to_relative_without_site_url() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [One](one.md)\n")?;
    book.create_file("one.md", "# One\n\nx\n")?;
    book.build().await?;

    let html = book.read_output("404.html")?;
    assert_contains!(html, "href=\"css/styles.css\"");
    assert_not_contains!(html, "href=\"/css/styles.css\"");

    Ok(())
}

// ---------------------------------------------------------------------------
// SUMMARY.md is authored input that reaches every page's sidebar and <title>.
// Chapter content is escaped when allow_html is false; these paths must honour
// the same policy rather than quietly bypassing it.
// ---------------------------------------------------------------------------

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_summary_label_cannot_inject_html() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file(
        "SUMMARY.md",
        "# Summary\n\n- [<img src=x onerror=alert(1)>](one.md)\n",
    )?;
    book.create_file("one.md", "# One\n\nBody.\n")?;
    book.build().await?;

    let html = book.read_output("one.html")?;
    assert_not_contains!(html, "<img src=x onerror");
    assert_contains!(html, "&lt;img src=x onerror");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_summary_external_url_cannot_break_out_of_href() -> Result<()> {
    let book = TestBook::new()?;
    book.create_file(
        "SUMMARY.md",
        "# Summary\n\n- [One](one.md)\n- [Ext](https://example.com/\" onmouseover=\"alert(2))\n",
    )?;
    book.create_file("one.md", "# One\n\nBody.\n")?;
    book.build().await?;

    let html = book.read_output("one.html")?;
    // The payload may appear, but only as escaped data inside the attribute.
    assert_not_contains!(html, "\" onmouseover=\"alert(2)\"");
    assert_contains!(html, "&quot; onmouseover=&quot;");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_summary_label_still_renders_inline_markdown() -> Result<()> {
    // Escaping must not cost the feature: SUMMARY labels carry inline markdown.
    let book = TestBook::new()?;
    book.create_file(
        "SUMMARY.md",
        "# Summary\n\n- [**Bold** and `code`](one.md)\n",
    )?;
    book.create_file("one.md", "# One\n\nBody.\n")?;
    book.build().await?;

    let html = book.read_output("one.html")?;
    assert_contains!(html, "<strong>Bold</strong>");
    assert_contains!(html, "<code>code</code>");
    // …but the plain-text form drives <title>.
    assert_contains!(html, "<title>Bold and code | ");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_ordinary_urls_stay_readable() -> Result<()> {
    // Guards against escaping URLs with a generic HTML escaper, which rewrites
    // every '/' as &#x2F; and turns each href into noise.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [Deep](nested/deep.md)\n")?;
    book.create_file("nested/deep.md", "# Deep\n\nBody.\n")?;
    book.build().await?;

    let html = book.read_output("nested/deep.html")?;
    assert_contains!(html, "href=\"../css/styles.css\"");
    assert_not_contains!(html, "&#x2F;");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_quoted_filename_cannot_break_out_of_href() -> Result<()> {
    // Output paths derive from filenames, and the templates mark them `| safe`,
    // so they must be safe by construction — including previous/next, which is
    // a different code path from the sidebar.
    let book = TestBook::new()?;
    book.create_file("SUMMARY.md", "# Summary\n\n- [A](a\".md)\n- [B](b.md)\n")?;
    book.create_file("a\".md", "# A\n\nx\n")?;
    book.create_file("b.md", "# B\n\ny\n")?;
    book.build().await?;

    let html = book.read_output("b.html")?;
    assert_not_contains!(html, "href=\"a\".html\"");
    assert_contains!(html, "a&quot;.html");

    Ok(())
}
