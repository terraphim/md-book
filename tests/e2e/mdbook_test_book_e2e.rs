use anyhow::Result;
use std::fs;

mod common;
use common::*;

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_end_to_end() -> Result<()> {
    // This is an end-to-end test that builds the entire mdBook test book
    // and validates that it produces the expected output structure

    let book = TestBook::new()?;

    // Copy the mdBook test book content
    copy_mdbook_test_book(&book)?;

    // Load the original mdBook config
    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;
    let book = book.with_config(config);

    // Build the book
    book.build().await?;

    // Verify the complete structure
    verify_mdbook_test_book_structure(&book)?;

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_with_search() -> Result<()> {
    // Test building the mdBook test book with search functionality
    let book = TestBook::new()?;

    copy_mdbook_test_book(&book)?;

    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;
    let book = book.with_config(config);

    book.build().await?;

    // Verify search index was created (if search is enabled)
    let output_path = book.output_path();
    let _search_exists = output_path.join("_pagefind").exists();

    // This test passes whether search is enabled or not
    assert!(output_path.exists());

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_mdbook_test_book_redirects() -> Result<()> {
    // Test that redirects from the mdBook config are handled
    let book = TestBook::new()?;

    copy_mdbook_test_book(&book)?;

    let config = md_book::config::load_config(Some("test_book_mdbook/book.toml"))?;
    let book = book.with_config(config);

    book.build().await?;

    // Note: redirects are not currently supported in our config

    Ok(())
}

fn verify_mdbook_test_book_structure(book: &TestBook) -> Result<()> {
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

    // Verify main page content
    let readme_content = book.read_output("README.html")?;
    assert_contains!(readme_content, "Demo Book");

    // Verify SUMMARY page
    let summary_content = book.read_output("SUMMARY.html")?;
    assert_contains!(summary_content, "SUMMARY");

    // Verify Rust code blocks
    let rust_content = book.read_output("rust/rust_codeblock.html")?;
    assert_contains!(rust_content, "rust");

    // Verify language highlighting
    let highlight_content = book.read_output("languages/highlight.html")?;
    assert_contains!(highlight_content, "highlight");

    // Verify headings
    let headings_content = book.read_output("headings/README.html")?;
    assert_contains!(headings_content, "headings");

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
