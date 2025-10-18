use anyhow::Result;
use std::fs;

mod common;
use common::*;

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_compatibility_full_build() -> Result<()> {
    // Test building the entire mdBook test book
    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    // Load the original mdBook config
    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;
    let book = book.with_config(config);

    // Build the book
    book.build().await?;

    // Verify all expected files are present
    let expected_files = [
        "README.html",
        "SUMMARY.html",
        "last.html",
        "suffix.html",
        "rust/README.html",
        "rust/rust_codeblock.html",
        "languages/README.html",
        "languages/highlight.html",
        "headings/README.html",
        "headings/collapsed.html",
    ];

    for file in &expected_files {
        assert!(book.output_exists(file), "Expected file {} not found", file);
    }

    // Verify the main page has the correct title
    let readme_content = book.read_output("README.html")?;
    assert_contains!(readme_content, "Demo Book");

    // Verify SUMMARY page exists and has content
    let summary_content = book.read_output("SUMMARY.html")?;
    assert_contains!(summary_content, "SUMMARY");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_compatibility_config_parsing() -> Result<()> {
    // Test that we can parse the mdBook config correctly
    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;

    // Verify book metadata
    assert_eq!(config.book.title, "mdBook test book");
    assert_eq!(
        config.book.description,
        Some("A demo book to test and validate changes".to_string())
    );
    assert_eq!(config.book.authors, vec!["YJDoc2"]);
    assert_eq!(config.book.language, "en");

    // Verify Rust edition
    assert_eq!(config.rust.edition, "2018");

    // Verify HTML output settings
    assert!(config.output.html.mathjax_support);

    // Verify playground settings
    assert!(config.output.html.playground.editable);
    assert!(config.output.html.playground.line_numbers);

    // Verify search settings
    assert_eq!(config.output.html.search.limit_results, 20);
    assert!(config.output.html.search.use_boolean_and);
    assert_eq!(config.output.html.search.boost_title, 2);
    assert_eq!(config.output.html.search.boost_hierarchy, 2);
    assert_eq!(config.output.html.search.boost_paragraph, 1);
    assert!(config.output.html.search.expand);
    assert_eq!(config.output.html.search.heading_split_level, 2);

    // Note: redirects are not currently supported in our config

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_compatibility_markdown_parsing() -> Result<()> {
    let book = TestBook::new()?;
    copy_mdbook_test_book(&book)?;

    // Test that we can parse various markdown features from the test book
    book.build().await?;

    // Test code blocks
    let rust_content = book.read_output("rust/rust_codeblock.html")?;
    assert_contains!(rust_content, "rust");
    assert_contains!(rust_content, "fn main");

    // Test language highlighting
    let highlight_content = book.read_output("languages/highlight.html")?;
    assert_contains!(highlight_content, "highlight");

    // Test headings
    let headings_content = book.read_output("headings/README.html")?;
    assert_contains!(headings_content, "headings");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_compatibility_navigation() -> Result<()> {
    let book = TestBook::new()?;
    copy_mdbook_test_book(&book)?;

    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;
    let book = book.with_config(config);

    book.build().await?;

    // Test that navigation links work
    let readme_content = book.read_output("README.html")?;
    assert_contains!(readme_content, "href");

    // Test that SUMMARY is processed
    let summary_content = book.read_output("SUMMARY.html")?;
    assert_contains!(summary_content, "SUMMARY");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_compatibility_syntax_highlighting() -> Result<()> {
    let book = TestBook::new()?;
    copy_mdbook_test_book(&book)?;

    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;
    let book = book.with_config(config);

    book.build().await?;

    // Test that code blocks are syntax highlighted
    let rust_content = book.read_output("rust/rust_codeblock.html")?;
    // Should contain syntax highlighting markup
    assert_contains!(rust_content, "rust");

    let highlight_content = book.read_output("languages/highlight.html")?;
    // Should contain language highlighting
    assert_contains!(highlight_content, "highlight");

    Ok(())
}

fn copy_mdbook_test_book(book: &TestBook) -> Result<()> {
    // Copy all markdown files from the mdBook test book
    let src_dir = "test_book_mdbook/src";

    if !std::path::Path::new(src_dir).exists() {
        anyhow::bail!("mdBook test book not found at {}", src_dir);
    }

    // Copy all .md files recursively
    for entry in walkdir::WalkDir::new(src_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    let relative_path = entry.path().strip_prefix(src_dir)?;
                    let content = fs::read_to_string(entry.path())?;
                    book.create_file(relative_path, &content)?;
                }
            }
        }
    }

    Ok(())
}
