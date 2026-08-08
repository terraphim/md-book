//! CLI args and build orchestration.
//!
//! Pipeline stages live in [`crate::pipeline`] and [`crate::render`].

use anyhow::Result;
use clap::Parser;
use serde::Serialize;

use crate::config::BookConfig;
use crate::pipeline;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input directory containing markdown files
    #[arg(short, long)]
    pub input: String,

    /// Output directory for HTML files
    #[arg(short, long)]
    pub output: String,

    /// Optional path to config file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Watch for changes and rebuild
    #[arg(short, long)]
    #[cfg(feature = "watcher")]
    pub watch: bool,

    /// Serve the book at <http://localhost:3000>
    #[arg(short, long)]
    #[cfg(feature = "server")]
    pub serve: bool,

    /// Port to serve on when using --serve (default: 3000)
    #[arg(long, default_value = "3000")]
    #[cfg(feature = "server")]
    pub port: u16,
}

#[derive(Serialize, Debug, Clone)]
pub struct PageInfo {
    pub title: String,
    pub path: String,
}

#[cfg(feature = "tokio")]
/// Build the book from markdown files to HTML
///
/// # Errors
///
/// Returns an error if the build process fails, including template rendering,
/// file I/O errors, or search indexing failures
pub async fn build(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
    build_impl(args, config, watch_enabled).await
}

#[cfg(not(feature = "tokio"))]
pub fn build(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
    build_impl(args, config, watch_enabled)
}

#[cfg(feature = "tokio")]
async fn build_impl(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
    pipeline::run_sync(args, config, watch_enabled)?;

    #[cfg(all(feature = "search", feature = "tokio"))]
    {
        pipeline::index(&args.output).await?;
    }

    Ok(())
}

#[cfg(not(feature = "tokio"))]
fn build_impl(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
    pipeline::run_sync(args, config, watch_enabled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BookConfig;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_args_default_values() {
        use clap::Parser;

        let args = Args::try_parse_from(["md-book", "-i", "input", "-o", "output"]).unwrap();
        assert_eq!(args.input, "input");
        assert_eq!(args.output, "output");
        assert_eq!(args.config, None);

        #[cfg(feature = "watcher")]
        assert!(!args.watch);

        #[cfg(feature = "server")]
        {
            assert!(!args.serve);
            assert_eq!(args.port, 3000);
        }
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_args_with_server_options() {
        use clap::Parser;

        let args = Args::try_parse_from([
            "md-book", "-i", "input", "-o", "output", "--serve", "--port", "8080",
        ])
        .unwrap();

        assert!(args.serve);
        assert_eq!(args.port, 8080);
    }

    #[test]
    #[ignore = "MathJax support not implemented yet"]
    fn test_process_markdown_with_mathjax() -> Result<()> {
        let mut config = BookConfig::default();
        config.output.html.mathjax_support = true;

        let markdown = "# Math Test\n\n$$E = mc^2$$";
        let html = markdown::to_html(markdown);
        assert!(html.contains("E = mc^2"));
        Ok(())
    }

    // WASM-specific tests
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_wasm_process_markdown() {
        use crate::wasm_process_markdown;

        let markdown = "# WASM Test\n\nThis is **bold** text for WASM.";
        let html = wasm_process_markdown(markdown);

        assert!(html.contains("<h1>WASM Test</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(!html.is_empty());
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_wasm_process_markdown_empty() {
        use crate::wasm_process_markdown;

        let html = wasm_process_markdown("");
        assert!(html.is_empty() || html == "<p></p>\n");
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_wasm_process_markdown_code_blocks() {
        use crate::wasm_process_markdown;

        let markdown = "```rust\nfn main() {\n    println!(\"Hello, WASM!\");\n}\n```";
        let html = wasm_process_markdown(markdown);

        assert!(html.contains("<pre>") || html.contains("<code>"));
        assert!(html.contains("fn main"));
        assert!(html.contains("Hello, WASM!"));
    }

    #[cfg(all(feature = "tokio", not(target_arch = "wasm32")))]
    #[tokio::test]
    async fn test_build_simple_book() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("book");

        fs::create_dir_all(&input_dir)?;
        fs::create_dir_all(&output_dir)?;

        fs::write(input_dir.join("test.md"), "# Test Page\n\nThis is a test.")?;

        let args = Args {
            input: input_dir.to_string_lossy().to_string(),
            output: output_dir.to_string_lossy().to_string(),
            config: None,
            #[cfg(feature = "watcher")]
            watch: false,
            #[cfg(feature = "server")]
            serve: false,
            #[cfg(feature = "server")]
            port: 3000,
        };

        let config = BookConfig::default();
        build(&args, &config, false).await?;

        assert!(output_dir.exists());
        Ok(())
    }

    #[cfg(all(not(feature = "tokio"), not(target_arch = "wasm32")))]
    #[test]
    fn test_build_simple_book() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("book");

        fs::create_dir_all(&input_dir)?;
        fs::create_dir_all(&output_dir)?;

        fs::write(input_dir.join("test.md"), "# Test Page\n\nThis is a test.")?;

        let args = Args {
            input: input_dir.to_string_lossy().to_string(),
            output: output_dir.to_string_lossy().to_string(),
            config: None,
            #[cfg(feature = "watcher")]
            watch: false,
            #[cfg(feature = "server")]
            serve: false,
            #[cfg(feature = "server")]
            port: 3000,
        };

        let config = BookConfig::default();
        build(&args, &config, false)?;

        assert!(output_dir.exists());
        Ok(())
    }
}
