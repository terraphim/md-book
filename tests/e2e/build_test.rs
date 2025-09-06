use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::process::Command;
use tokio;

#[tokio::test]
async fn test_full_build_pipeline() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("input");
    let output_path = temp_dir.path().join("output");
    
    // Create input directory structure
    fs::create_dir_all(&input_path)?;
    fs::create_dir_all(input_path.join("docs"))?;
    fs::create_dir_all(input_path.join("guides"))?;
    
    // Create test markdown files
    fs::write(
        input_path.join("index.md"),
        r#"# Welcome to Test Documentation

This is the home page of our test documentation site.

## Features
- Full-text search functionality
- Responsive design
- Syntax highlighting

Search for "test" or "documentation" to verify search works.
"#,
    )?;
    
    fs::write(
        input_path.join("docs").join("getting-started.md"),
        r#"# Getting Started

This guide will help you get started with the software.

## Installation

1. Download the binary
2. Extract the archive
3. Run the installer

Use the search feature to find installation instructions quickly.

## Configuration

Configuration options are available in the config file.
"#,
    )?;
    
    fs::write(
        input_path.join("guides").join("advanced.md"),
        r#"# Advanced Usage

This guide covers advanced topics and features.

## Environment Variables

- `DEBUG`: Enable debug mode
- `LOG_LEVEL`: Set logging verbosity
- `PORT`: Configure server port

## API Reference

The API provides programmatic access to all features.

### Endpoints

- `GET /api/docs`: List all documentation
- `POST /api/search`: Perform search query
- `PUT /api/config`: Update configuration
"#,
    )?;
    
    // Create book.toml configuration
    fs::write(
        input_path.join("book.toml"),
        r#"[book]
title = "Test Documentation"
authors = ["Test Author"]
description = "Test documentation for CI/CD pipeline"
language = "en"

[output.html]
mathjax-support = false
allow_html = true

[markdown]
format = "gfm"
frontmatter = false
"#,
    )?;
    
    // Run the build command
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()?;
    
    // Check that build succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Build failed!\nStdout: {}\nStderr: {}",
            stdout, stderr
        );
    }
    
    // Verify HTML files were generated
    assert!(output_path.join("index.html").exists(), "index.html should be generated");
    assert!(output_path.join("docs").join("getting-started.html").exists(), "getting-started.html should be generated");
    assert!(output_path.join("guides").join("advanced.html").exists(), "advanced.html should be generated");
    
    // Verify CSS and JS files were copied
    assert!(output_path.join("css").join("styles.css").exists(), "styles.css should be copied");
    assert!(output_path.join("css").join("search.css").exists(), "search.css should be copied");
    assert!(output_path.join("js").join("pagefind-search.js").exists(), "pagefind-search.js should be copied");
    assert!(output_path.join("components").join("search-modal.js").exists(), "search-modal.js should be copied");
    
    // Verify Pagefind search index was created
    assert!(output_path.join("pagefind").exists(), "pagefind directory should be created");
    
    // Check that HTML contains expected content
    let index_content = fs::read_to_string(output_path.join("index.html"))?;
    assert!(index_content.contains("Welcome to Test Documentation"), "Index should contain title");
    assert!(index_content.contains("search-modal"), "Index should include search modal");
    assert!(index_content.contains("pagefind-search.js"), "Index should include search script");
    
    let getting_started_content = fs::read_to_string(output_path.join("docs").join("getting-started.html"))?;
    assert!(getting_started_content.contains("Getting Started"), "Getting started should contain title");
    assert!(getting_started_content.contains("Installation"), "Getting started should contain installation section");
    
    // Verify search functionality files exist
    if output_path.join("pagefind").join("pagefind.js").exists() {
        let pagefind_js = fs::read_to_string(output_path.join("pagefind").join("pagefind.js"))?;
        assert!(!pagefind_js.is_empty(), "pagefind.js should not be empty");
    }
    
    println!("✅ Full build pipeline test passed");
    Ok(())
}

#[tokio::test] 
async fn test_build_with_custom_templates() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("input");
    let output_path = temp_dir.path().join("output");
    
    // Create input and templates directory
    fs::create_dir_all(&input_path)?;
    fs::create_dir_all(input_path.join("templates"))?;
    
    // Create custom header template
    fs::write(
        input_path.join("templates").join("header.html.tera"),
        r#"<header class="custom-header">
    <h1>{{ config.book.title }}</h1>
    <div class="custom-search">
        <input id="header-search-input" placeholder="Custom search..." />
    </div>
</header>"#,
    )?;
    
    // Create test content
    fs::write(
        input_path.join("test.md"),
        "# Test Page\n\nThis tests custom templates.\n",
    )?;
    
    // Run build with custom templates
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()?;
    
    assert!(output.status.success(), "Build should succeed with custom templates");
    
    // Verify custom template was used
    let html_content = fs::read_to_string(output_path.join("test.html"))?;
    assert!(html_content.contains("custom-header"), "Should use custom header template");
    assert!(html_content.contains("Custom search"), "Should include custom search text");
    
    println!("✅ Custom templates test passed");
    Ok(())
}

#[tokio::test]
async fn test_build_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let nonexistent_input = temp_dir.path().join("nonexistent");
    let output_path = temp_dir.path().join("output");
    
    // Try to build with nonexistent input directory
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            nonexistent_input.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()?;
    
    // Should fail gracefully
    assert!(!output.status.success(), "Should fail with nonexistent input directory");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("Error"), "Should contain error message");
    
    println!("✅ Error handling test passed");
    Ok(())
}

#[tokio::test]
async fn test_build_with_different_formats() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("input");
    let output_path = temp_dir.path().join("output");
    
    fs::create_dir_all(&input_path)?;
    
    // Create book.toml with GFM format
    fs::write(
        input_path.join("book.toml"),
        r#"[book]
title = "Format Test"

[markdown]
format = "gfm"
"#,
    )?;
    
    // Create markdown with GFM features
    fs::write(
        input_path.join("gfm-test.md"),
        r#"# GFM Features Test

## Task Lists
- [x] Completed task
- [ ] Incomplete task

## Tables
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |

## Strikethrough
~~This text is strikethrough~~

## Code blocks with syntax highlighting
```rust
fn main() {
    println!("Hello, world!");
}
```
"#,
    )?;
    
    // Run build
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()?;
    
    assert!(output.status.success(), "Build should succeed with GFM format");
    
    // Verify GFM features were processed
    let html_content = fs::read_to_string(output_path.join("gfm-test.html"))?;
    assert!(html_content.contains("<table>"), "Should process tables");
    assert!(html_content.contains("checked"), "Should process task lists");
    assert!(html_content.contains("<del>"), "Should process strikethrough");
    assert!(html_content.contains("rust"), "Should include language class");
    
    println!("✅ Different formats test passed");
    Ok(())
}

#[tokio::test]
async fn test_watch_mode_preparation() -> Result<()> {
    // This test verifies the setup for watch mode testing
    // Actual watch mode testing would require more complex async handling
    
    let temp_dir = TempDir::new()?;
    let input_path = temp_dir.path().join("input");
    
    fs::create_dir_all(&input_path)?;
    fs::write(input_path.join("test.md"), "# Test\n\nContent for watch test.\n")?;
    
    // Verify watch mode can be started (but don't actually run it in CI)
    let help_output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()?;
    
    let help_text = String::from_utf8_lossy(&help_output.stdout);
    assert!(help_text.contains("--watch"), "Should support watch flag");
    assert!(help_text.contains("--serve"), "Should support serve flag");
    
    println!("✅ Watch mode preparation test passed");
    Ok(())
}