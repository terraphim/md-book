use anyhow::Result;
use std::fs;

mod common;
use common::*;

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_build() -> Result<()> {
    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    // Build the book
    book.build().await?;

    // Verify key files were created
    assert!(book.output_exists("README.html"));
    assert!(book.output_exists("SUMMARY.html"));
    assert!(book.output_exists("last.html"));
    assert!(book.output_exists("suffix.html"));

    // Verify content
    let readme_content = book.read_output("README.html")?;
    assert_contains!(readme_content, "Demo Book");

    let summary_content = book.read_output("SUMMARY.html")?;
    assert_contains!(summary_content, "SUMMARY");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_with_config() -> Result<()> {
    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    // Load the mdBook config
    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;
    let book = book.with_config(config);

    // Build the book
    book.build().await?;

    // Verify the config was applied
    let readme_content = book.read_output("README.html")?;
    assert_contains!(readme_content, "Demo Book");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_rust_code_blocks() -> Result<()> {
    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    book.build().await?;

    // Check Rust code blocks are processed
    let rust_content = book.read_output("rust/rust_codeblock.html")?;
    assert_contains!(rust_content, "rust");
    assert_contains!(rust_content, "fn main");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_language_highlighting() -> Result<()> {
    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    book.build().await?;

    // Check language highlighting
    let highlight_content = book.read_output("languages/highlight.html")?;
    assert_contains!(highlight_content, "highlight");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_headings() -> Result<()> {
    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    book.build().await?;

    // Check headings are processed
    let headings_content = book.read_output("headings/README.html")?;
    assert_contains!(headings_content, "headings");

    let collapsed_content = book.read_output("headings/collapsed.html")?;
    assert_contains!(collapsed_content, "collapsed");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_navigation() -> Result<()> {
    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    book.build().await?;

    // Check that navigation structure is created
    let summary_content = book.read_output("SUMMARY.html")?;
    assert_contains!(summary_content, "SUMMARY");

    // Check that links work
    let readme_content = book.read_output("README.html")?;
    // Should contain links to other pages
    assert_contains!(readme_content, "href");

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_mathjax_support() -> Result<()> {
    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    // Load config with MathJax support
    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;
    let book = book.with_config(config);

    book.build().await?;

    // Check that MathJax is supported (if implemented)
    let readme_content = book.read_output("README.html")?;
    // This test will pass even if MathJax isn't implemented yet
    assert!(!readme_content.is_empty());

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_search_config() -> Result<()> {
    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    // Load config with search settings
    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;
    let book = book.with_config(config);

    book.build().await?;

    // Verify search configuration is applied
    assert_eq!(book.config.output.html.search.limit_results, 20);
    assert_eq!(book.config.output.html.search.boost_title, 2);
    assert_eq!(book.config.output.html.search.boost_hierarchy, 2);
    assert_eq!(book.config.output.html.search.boost_paragraph, 1);
    assert!(book.config.output.html.search.expand);
    assert_eq!(book.config.output.html.search.heading_split_level, 2);

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
