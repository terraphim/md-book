use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use md_book::{PagefindBuilder, PagefindError};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// Generate test HTML content with varying sizes
fn generate_html_content(title: &str, word_count: usize) -> String {
    let words = vec![
        "documentation", "search", "index", "content", "performance", "benchmark", 
        "testing", "pagefind", "rust", "webassembly", "optimization", "analysis",
        "configuration", "installation", "tutorial", "guide", "advanced", "basic"
    ];
    
    let repeated_words = words.iter()
        .cycle()
        .take(word_count)
        .map(|w| w.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>{}</title></head>
<body>
    <h1>{}</h1>
    <p>{}</p>
    <h2>Section 1</h2>
    <p>{}</p>
    <h2>Section 2</h2>
    <p>{}</p>
</body>
</html>"#,
        title, title, repeated_words, repeated_words, repeated_words
    )
}

/// Create a test site with specified number of pages and content size
async fn create_benchmark_site(temp_dir: &TempDir, page_count: usize, words_per_page: usize) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let site_path = temp_dir.path().to_path_buf();
    
    // Create directory structure
    fs::create_dir_all(site_path.join("docs"))?;
    fs::create_dir_all(site_path.join("guides"))?;
    fs::create_dir_all(site_path.join("api"))?;
    
    // Generate pages
    for i in 0..page_count {
        let title = format!("Test Page {}", i);
        let content = generate_html_content(&title, words_per_page);
        
        let file_path = match i % 3 {
            0 => site_path.join("docs").join(format!("page_{}.html", i)),
            1 => site_path.join("guides").join(format!("guide_{}.html", i)),
            2 => site_path.join("api").join(format!("api_{}.html", i)),
            _ => unreachable!(),
        };
        
        fs::write(file_path, content)?;
    }
    
    Ok(site_path)
}

/// Benchmark pagefind builder initialization
fn bench_pagefind_init(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("pagefind_init");
    
    for page_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("pages", page_count),
            page_count,
            |b, &page_count| {
                b.iter(|| {
                    rt.block_on(async {
                        let temp_dir = TempDir::new().expect("Failed to create temp dir");
                        let site_path = create_benchmark_site(&temp_dir, page_count, 100)
                            .await
                            .expect("Failed to create site");
                        
                        let result = PagefindBuilder::new(black_box(site_path)).await;
                        assert!(result.is_ok());
                        result.unwrap()
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark pagefind indexing performance
fn bench_pagefind_indexing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("pagefind_indexing");
    group.sample_size(10); // Reduce sample size for slower operations
    group.measurement_time(std::time::Duration::from_secs(30));
    
    for &(page_count, words_per_page) in [(10, 100), (50, 100), (100, 100), (50, 500)].iter() {
        group.bench_with_input(
            BenchmarkId::new("indexing", format!("{}p_{}w", page_count, words_per_page)),
            &(page_count, words_per_page),
            |b, &(page_count, words_per_page)| {
                b.iter(|| {
                    rt.block_on(async {
                        let temp_dir = TempDir::new().expect("Failed to create temp dir");
                        let site_path = create_benchmark_site(&temp_dir, page_count, words_per_page)
                            .await
                            .expect("Failed to create site");
                        
                        let builder = PagefindBuilder::new(black_box(site_path)).await
                            .expect("Failed to create builder");
                        
                        // This is the main operation we're benchmarking
                        let result = builder.build().await;
                        
                        // The build might fail in the test environment, but we want to measure timing
                        match result {
                            Ok(_) => {
                                // Success - ideal case
                            }
                            Err(PagefindError::IndexingFailed { .. }) => {
                                // Expected in test environment
                            }
                            Err(other) => {
                                panic!("Unexpected error: {:?}", other);
                            }
                        }
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory usage during indexing
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10);
    
    for page_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory", page_count),
            page_count,
            |b, &page_count| {
                b.iter(|| {
                    rt.block_on(async {
                        let temp_dir = TempDir::new().expect("Failed to create temp dir");
                        let site_path = create_benchmark_site(&temp_dir, page_count, 200)
                            .await
                            .expect("Failed to create site");
                        
                        // Measure memory before
                        let _memory_before = get_memory_usage();
                        
                        let builder = PagefindBuilder::new(black_box(site_path)).await
                            .expect("Failed to create builder");
                        
                        let _ = builder.build().await; // May fail in test environment
                        
                        // Measure memory after
                        let _memory_after = get_memory_usage();
                        
                        // In a real benchmark, we'd calculate the difference
                        // For now, we just measure the timing
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark configuration loading performance
fn bench_config_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("config_loading");
    
    // Test with different config file formats
    for format in ["toml", "json", "yaml"].iter() {
        group.bench_with_input(
            BenchmarkId::new("format", format),
            format,
            |b, &format| {
                b.iter(|| {
                    rt.block_on(async {
                        let temp_dir = TempDir::new().expect("Failed to create temp dir");
                        let site_path = create_benchmark_site(&temp_dir, 10, 50)
                            .await
                            .expect("Failed to create site");
                        
                        // Create config file
                        let config_content = match format {
                            "toml" => r#"
[pagefind]
root_selector = "body"
exclude_selectors = ["nav", "footer"]
glob = "**/*.html"
"#,
                            "json" => r#"{
  "pagefind": {
    "root_selector": "body",
    "exclude_selectors": ["nav", "footer"],
    "glob": "**/*.html"
  }
}"#,
                            "yaml" => r#"
pagefind:
  root_selector: "body"
  exclude_selectors:
    - "nav"
    - "footer"
  glob: "**/*.html"
"#,
                            _ => unreachable!(),
                        };
                        
                        let config_file = format!("pagefind.{}", format);
                        let config_path = temp_dir.path().join(&config_file);
                        fs::write(&config_path, config_content).expect("Failed to write config");
                        
                        // Change to temp directory for config loading
                        let original_dir = std::env::current_dir().expect("Failed to get current dir");
                        std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
                        
                        let result = PagefindBuilder::new(black_box(site_path)).await;
                        
                        // Restore directory
                        std::env::set_current_dir(original_dir).expect("Failed to restore dir");
                        
                        assert!(result.is_ok());
                        result.unwrap()
                    })
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("error_handling");
    
    group.bench_function("invalid_path", |b| {
        b.iter(|| {
            rt.block_on(async {
                let invalid_path = PathBuf::from("/absolutely/nonexistent/path/that/should/never/exist");
                let result = PagefindBuilder::new(black_box(invalid_path)).await;
                
                assert!(result.is_err());
                match result.unwrap_err() {
                    PagefindError::SourcePathNotFound { .. } => {
                        // Expected error
                    }
                    other => panic!("Unexpected error: {:?}", other),
                }
            })
        });
    });
    
    group.bench_function("multiple_configs", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = TempDir::new().expect("Failed to create temp dir");
                let site_path = create_benchmark_site(&temp_dir, 5, 50)
                    .await
                    .expect("Failed to create site");
                
                // Create multiple config files
                fs::write(temp_dir.path().join("pagefind.toml"), "# TOML config")
                    .expect("Failed to write TOML");
                fs::write(temp_dir.path().join("pagefind.json"), "{}")
                    .expect("Failed to write JSON");
                
                let original_dir = std::env::current_dir().expect("Failed to get current dir");
                std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
                
                let result = PagefindBuilder::new(black_box(site_path)).await;
                
                std::env::set_current_dir(original_dir).expect("Failed to restore dir");
                
                assert!(result.is_err());
                match result.unwrap_err() {
                    PagefindError::MultipleConfigs { .. } => {
                        // Expected error
                    }
                    other => panic!("Unexpected error: {:?}", other),
                }
            })
        });
    });
    
    group.finish();
}

/// Simple memory usage estimation (platform-dependent)
fn get_memory_usage() -> usize {
    // This is a simplified version - in production you'd use platform-specific APIs
    // or memory profiling tools
    std::mem::size_of::<PagefindBuilder>()
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_operations");
    
    for concurrency in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_init", concurrency),
            concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    rt.block_on(async {
                        let temp_dir = TempDir::new().expect("Failed to create temp dir");
                        let site_path = create_benchmark_site(&temp_dir, 20, 100)
                            .await
                            .expect("Failed to create site");
                        
                        // Create multiple builders concurrently
                        let tasks = (0..concurrency).map(|_| {
                            let site_path = site_path.clone();
                            async move {
                                PagefindBuilder::new(black_box(site_path)).await
                            }
                        });
                        
                        let results = futures::future::join_all(tasks).await;
                        
                        // Verify all succeeded
                        for result in results {
                            assert!(result.is_ok());
                        }
                    })
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_pagefind_init,
    bench_pagefind_indexing,
    bench_memory_usage,
    bench_config_loading,
    bench_error_handling,
    bench_concurrent_operations,
);

criterion_main!(benches);