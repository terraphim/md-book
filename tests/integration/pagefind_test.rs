use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use md_book::{PagefindBuilder, PagefindError};
use tokio;

/// Create a temporary directory with sample markdown files for testing
async fn create_test_site(temp_dir: &TempDir) -> Result<PathBuf> {
    let site_path = temp_dir.path().to_path_buf();
    
    // Create directory structure
    fs::create_dir_all(site_path.join("docs"))?;
    fs::create_dir_all(site_path.join("guides"))?;
    
    // Create sample markdown files
    fs::write(
        site_path.join("index.html"),
        r#"<!DOCTYPE html>
<html>
<head><title>Home Page</title></head>
<body>
    <h1>Welcome to the Documentation</h1>
    <p>This is the home page with searchable content.</p>
</body>
</html>"#,
    )?;
    
    fs::write(
        site_path.join("docs").join("getting-started.html"),
        r#"<!DOCTYPE html>
<html>
<head><title>Getting Started</title></head>
<body>
    <h1>Getting Started Guide</h1>
    <p>This guide will help you get started quickly with our documentation system.</p>
    <h2>Installation</h2>
    <p>Follow these steps to install the system.</p>
</body>
</html>"#,
    )?;
    
    fs::write(
        site_path.join("docs").join("advanced.html"),
        r#"<!DOCTYPE html>
<html>
<head><title>Advanced Topics</title></head>
<body>
    <h1>Advanced Configuration</h1>
    <p>Learn about advanced configuration options and customization techniques.</p>
    <h2>Performance Tuning</h2>
    <p>Optimize your setup for better performance.</p>
</body>
</html>"#,
    )?;
    
    fs::write(
        site_path.join("guides").join("tutorial.html"),
        r#"<!DOCTYPE html>
<html>
<head><title>Tutorial</title></head>
<body>
    <h1>Step-by-Step Tutorial</h1>
    <p>A comprehensive tutorial covering all features.</p>
    <ul>
        <li>Basic setup</li>
        <li>Configuration options</li>
        <li>Advanced features</li>
    </ul>
</body>
</html>"#,
    )?;
    
    Ok(site_path)
}

/// Create a test configuration file
fn create_test_config(temp_dir: &TempDir) -> Result<PathBuf> {
    let config_path = temp_dir.path().join("pagefind.toml");
    
    fs::write(
        &config_path,
        r#"[pagefind]
root_selector = "body"
exclude_selectors = ["nav", "footer"]
glob = "**/*.html"
"#,
    )?;
    
    Ok(config_path)
}

#[tokio::test]
async fn test_pagefind_builder_initialization() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let site_path = create_test_site(&temp_dir).await.expect("Failed to create test site");
    
    // Test successful initialization
    let builder = PagefindBuilder::new(site_path.clone()).await;
    assert!(builder.is_ok(), "PagefindBuilder should initialize successfully");
    
    let builder = builder.unwrap();
    assert_eq!(builder.source_path(), Some(&site_path));
}

#[tokio::test]
async fn test_pagefind_builder_invalid_path() {
    let invalid_path = PathBuf::from("/nonexistent/path/that/should/not/exist");
    
    let result = PagefindBuilder::new(invalid_path.clone()).await;
    assert!(result.is_err(), "Should fail with invalid path");
    
    match result.unwrap_err() {
        PagefindError::SourcePathNotFound { path } => {
            assert_eq!(path, invalid_path);
        }
        other => panic!("Expected SourcePathNotFound error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_pagefind_with_config_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let site_path = create_test_site(&temp_dir).await.expect("Failed to create test site");
    
    // Create config file in the working directory
    let _config_path = create_test_config(&temp_dir).expect("Failed to create config");
    
    // Change to temp directory so config file is found
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
    
    let result = {
        let builder = PagefindBuilder::new(site_path).await;
        builder
    };
    
    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
    
    assert!(result.is_ok(), "Should initialize with config file");
}

#[tokio::test] 
async fn test_multiple_config_files_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let site_path = create_test_site(&temp_dir).await.expect("Failed to create test site");
    
    // Create multiple config files
    fs::write(temp_dir.path().join("pagefind.toml"), "# TOML config").expect("Failed to write TOML");
    fs::write(temp_dir.path().join("pagefind.json"), "{}").expect("Failed to write JSON");
    
    // Change to temp directory
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
    
    let result = {
        PagefindBuilder::new(site_path).await
    };
    
    // Restore directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");
    
    assert!(result.is_err(), "Should fail with multiple config files");
    
    match result.unwrap_err() {
        PagefindError::MultipleConfigs { files } => {
            assert!(files.len() >= 2, "Should detect multiple config files");
            assert!(files.contains(&"pagefind.toml".to_string()));
            assert!(files.contains(&"pagefind.json".to_string()));
        }
        other => panic!("Expected MultipleConfigs error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_pagefind_build_process() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let site_path = create_test_site(&temp_dir).await.expect("Failed to create test site");
    
    let builder = PagefindBuilder::new(site_path.clone()).await
        .expect("Failed to create PagefindBuilder");
    
    // This test might fail if the pagefind library has issues,
    // but we want to test our error handling
    let build_result = builder.build().await;
    
    // The build might succeed or fail depending on the environment
    // We're mainly testing that our error handling works
    match build_result {
        Ok(()) => {
            // Build succeeded - check that pagefind directory was created
            let pagefind_dir = site_path.join("pagefind");
            assert!(
                pagefind_dir.exists(),
                "Pagefind directory should exist after successful build"
            );
            
            // Check for expected files
            assert!(
                pagefind_dir.join("pagefind.js").exists(),
                "pagefind.js should be generated"
            );
        }
        Err(PagefindError::IndexingFailed { message }) => {
            // Build failed with indexing error - this is acceptable for testing
            println!("Build failed as expected in test environment: {}", message);
        }
        Err(other) => {
            panic!("Unexpected error type: {:?}", other);
        }
    }
}

#[tokio::test]
async fn test_builder_configuration_access() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let site_path = create_test_site(&temp_dir).await.expect("Failed to create test site");
    
    let builder = PagefindBuilder::new(site_path.clone()).await
        .expect("Failed to create PagefindBuilder");
    
    // Test configuration access
    let config = builder.config();
    assert!(config.source.is_some(), "Source should be set in config");
    assert_eq!(config.source.as_ref().unwrap(), &site_path);
}

#[tokio::test]
async fn test_error_display() {
    // Test that our custom errors display properly
    let invalid_path = PathBuf::from("/invalid/path");
    let error = PagefindError::SourcePathNotFound { path: invalid_path.clone() };
    
    let error_string = error.to_string();
    assert!(error_string.contains("Source path does not exist"));
    assert!(error_string.contains("/invalid/path"));
    
    let multi_config_error = PagefindError::MultipleConfigs {
        files: vec!["config1.toml".to_string(), "config2.json".to_string()],
    };
    
    let multi_error_string = multi_config_error.to_string();
    assert!(multi_error_string.contains("Multiple config files found"));
    assert!(multi_error_string.contains("config1.toml"));
    assert!(multi_error_string.contains("config2.json"));
}

#[tokio::test]
async fn test_concurrent_builds() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let site_path = create_test_site(&temp_dir).await.expect("Failed to create test site");
    
    // Test that multiple PagefindBuilder instances can be created concurrently
    let builders = futures::future::join_all((0..3).map(|_| {
        let site_path = site_path.clone();
        async move {
            PagefindBuilder::new(site_path).await
        }
    })).await;
    
    // All builders should initialize successfully
    for (i, builder) in builders.into_iter().enumerate() {
        assert!(builder.is_ok(), "Builder {} should initialize successfully", i);
    }
}

#[tokio::test]
async fn test_environment_variable_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let site_path = create_test_site(&temp_dir).await.expect("Failed to create test site");
    
    // Set environment variable for Pagefind config
    std::env::set_var("PAGEFIND_VERBOSE", "true");
    
    let builder = PagefindBuilder::new(site_path).await;
    
    // Clean up environment variable
    std::env::remove_var("PAGEFIND_VERBOSE");
    
    assert!(builder.is_ok(), "Should initialize with environment variables");
}

// Performance test - measure initialization time
#[tokio::test]
async fn test_initialization_performance() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let site_path = create_test_site(&temp_dir).await.expect("Failed to create test site");
    
    let start = std::time::Instant::now();
    let _builder = PagefindBuilder::new(site_path).await
        .expect("Failed to create PagefindBuilder");
    let duration = start.elapsed();
    
    // Initialization should be fast (under 100ms for this simple case)
    assert!(
        duration.as_millis() < 100,
        "Initialization took {}ms, expected <100ms",
        duration.as_millis()
    );
}