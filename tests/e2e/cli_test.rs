use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn get_binary_path() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove 'deps' directory
    path.push("md-book");
    path.to_string_lossy().to_string()
}

#[test]
fn test_cli_help() {
    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--input"));
    assert!(stdout.contains("--output"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("md-book"));
}

#[test]
fn test_cli_missing_required_args() {
    let output = Command::new(get_binary_path())
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("required") || stderr.contains("Usage:"));
}

#[test]
fn test_cli_basic_build() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("src");
    let output_dir = temp_dir.path().join("book");

    // Create input directory and file
    fs::create_dir_all(&input_dir)?;
    fs::write(input_dir.join("README.md"), "# Test Book\n\nHello, world!")?;

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.to_str().unwrap())
        .arg("--output")
        .arg(output_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    // Print output for debugging
    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success(), "Command failed");
    assert!(output_dir.exists(), "Output directory not created");
    assert!(
        output_dir.join("README.html").exists(),
        "HTML file not created"
    );

    Ok(())
}

#[test]
fn test_cli_with_config_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("src");
    let output_dir = temp_dir.path().join("book");
    let config_file = temp_dir.path().join("custom.toml");

    // Create input
    fs::create_dir_all(&input_dir)?;
    fs::write(
        input_dir.join("test.md"),
        "# Config Test\n\nWith custom config.",
    )?;

    // Create config file
    fs::write(
        &config_file,
        r#"
[book]
title = "CLI Test Book"
description = "Testing CLI with config"
authors = ["CLI Tester"]
language = "en"

[output.html]
mathjax_support = false
allow_html = false
"#,
    )?;

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.to_str().unwrap())
        .arg("--output")
        .arg(output_dir.to_str().unwrap())
        .arg("--config")
        .arg(config_file.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success());
    assert!(output_dir.exists());
    assert!(output_dir.join("test.html").exists());

    Ok(())
}

#[test]
fn test_cli_invalid_input_directory() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("book");

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg("nonexistent_directory")
        .arg("--output")
        .arg(output_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    // Should handle gracefully - either succeed with empty directory or fail with clear error
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(!stderr.is_empty(), "Should provide error message");
    }
}

#[test]
fn test_cli_complex_directory_structure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("src");
    let output_dir = temp_dir.path().join("book");

    // Create complex input structure
    fs::create_dir_all(&input_dir)?;
    fs::create_dir_all(input_dir.join("chapter1"))?;
    fs::create_dir_all(input_dir.join("chapter2"))?;

    fs::write(input_dir.join("README.md"), "# Main Book\n\nRoot content.")?;
    fs::write(
        input_dir.join("chapter1").join("intro.md"),
        "# Chapter 1\n\nIntroduction.",
    )?;
    fs::write(
        input_dir.join("chapter1").join("section1.md"),
        "## Section 1.1\n\nDetails.",
    )?;
    fs::write(
        input_dir.join("chapter2").join("advanced.md"),
        "# Chapter 2\n\nAdvanced topics.",
    )?;

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.to_str().unwrap())
        .arg("--output")
        .arg(output_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success());
    assert!(output_dir.exists());
    assert!(output_dir.join("README.html").exists());
    assert!(output_dir.join("chapter1").join("intro.html").exists());
    assert!(output_dir.join("chapter1").join("section1.html").exists());
    assert!(output_dir.join("chapter2").join("advanced.html").exists());

    Ok(())
}

#[cfg(feature = "server")]
#[test]
fn test_cli_server_options() {
    // Test that server options are accepted (don't actually start server in test)
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("src");
    let output_dir = temp_dir.path().join("book");

    fs::create_dir_all(&input_dir).unwrap();
    fs::write(input_dir.join("test.md"), "# Server Test").unwrap();

    // Test that the CLI accepts server arguments without error
    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.to_str().unwrap())
        .arg("--output")
        .arg(output_dir.to_str().unwrap())
        .arg("--port")
        .arg("8080")
        .arg("--help") // Use --help to avoid actually starting server
        .output()
        .expect("Failed to execute command");

    // Should succeed and show help with server options
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--port") || stdout.contains("--serve"));
}

#[cfg(feature = "watcher")]
#[test]
fn test_cli_watch_option() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("src");
    let output_dir = temp_dir.path().join("book");

    fs::create_dir_all(&input_dir).unwrap();
    fs::write(input_dir.join("test.md"), "# Watch Test").unwrap();

    // Test that the CLI accepts watch argument
    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.to_str().unwrap())
        .arg("--output")
        .arg(output_dir.to_str().unwrap())
        .arg("--help") // Use --help to check argument exists
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--watch"));
}

#[test]
fn test_cli_with_book_toml_in_working_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("src");
    let output_dir = temp_dir.path().join("book");
    let book_toml = temp_dir.path().join("book.toml");

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    // Create structure
    fs::create_dir_all(&input_dir)?;
    fs::write(input_dir.join("test.md"), "# Book TOML Test")?;
    fs::write(
        &book_toml,
        r#"
[book]
title = "Auto-detected Config"
authors = ["Auto Tester"]
"#,
    )?;

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg("src")
        .arg("--output")
        .arg("book")
        .output()
        .expect("Failed to execute command");

    // Restore directory
    std::env::set_current_dir(original_dir)?;

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success());
    assert!(output_dir.exists());

    Ok(())
}

#[test]
fn test_cli_output_permissions() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("src");
    let output_dir = temp_dir.path().join("book");

    fs::create_dir_all(&input_dir)?;
    fs::write(
        input_dir.join("permissions.md"),
        "# Permissions Test\n\nTesting file permissions.",
    )?;

    let output = Command::new(get_binary_path())
        .arg("--input")
        .arg(input_dir.to_str().unwrap())
        .arg("--output")
        .arg(output_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Check that output files have reasonable permissions
    let html_file = output_dir.join("permissions.html");
    assert!(html_file.exists());

    let metadata = fs::metadata(&html_file)?;
    assert!(metadata.is_file());
    assert!(metadata.len() > 0); // File should have content

    Ok(())
}
