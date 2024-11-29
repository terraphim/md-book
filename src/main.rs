use anyhow::{Context, Result};
use clap::Parser;
use markdown::{to_html_with_options, Options};
use serde::Serialize;
use std::fs;
use tera::{Context as TeraContext, Tera};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing markdown files
    #[arg(short, long)]
    input: String,

    /// Output directory for HTML files
    #[arg(short, long)]
    output: String,
}

#[derive(Serialize)]
struct PageData {
    title: String,
    content: String,
    toc: Vec<TocItem>,
}

#[derive(Serialize, Clone)]
struct TocItem {
    title: String,
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output)?;
    
    // Copy static assets
    copy_static_assets(&args.output)?;
    
    // Initialize Tera template engine
    let mut tera = Tera::default();
    tera.add_raw_template("page", include_str!("templates/page.html.tera"))?;
    tera.add_raw_template("index", include_str!("templates/index.html.tera"))?;
    
    // Generate table of contents
    let mut toc = Vec::new();
    
    // Process markdown files
    for entry in WalkDir::new(&args.input) {
        let entry = entry?;
        if entry.path().extension().map_or(false, |e| e == "md") {
            let rel_path = entry.path().strip_prefix(&args.input)?;
            let html_path = args.output.clone() + "/" + &rel_path.with_extension("html").display().to_string();
            
            // Create necessary subdirectories
            if let Some(parent) = std::path::Path::new(&html_path).parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Read and convert markdown
            let markdown_content = fs::read_to_string(entry.path())?;

            // Use the predefined GFM options
            let options = markdown::Options::gfm();

            // Handle markdown conversion error
            let html_content = to_html_with_options(&markdown_content, &options)
                .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))?;
            
            // Fix 2: Proper string conversion for file name
            let title = extract_title(&markdown_content)
                .unwrap_or_else(|| {
                    entry.path()
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Untitled".to_string())
                });
            
            // Add to TOC
            toc.push(TocItem {
                title: title.clone(),
                path: rel_path.with_extension("html").display().to_string(),
            });
            
            // Render page
            let mut context = TeraContext::new();
            context.insert("page", &PageData {
                title,
                content: html_content,
                toc: toc.clone(),
            });
            
            let rendered = tera.render("page", &context)?;
            fs::write(html_path, rendered)?;
        }
    }
    
    // Generate index page
    let mut context = TeraContext::new();
    context.insert("toc", &toc);
    let rendered = tera.render("index", &context)?;
    fs::write(format!("{}/index.html", args.output), rendered)?;
    
    Ok(())
}

fn extract_title(markdown: &str) -> Option<String> {
    markdown
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line[2..].trim().to_string())
}

fn copy_static_assets(output_dir: &str) -> Result<()> {
    // Create components directory
    fs::create_dir_all(format!("{}/components", output_dir))?;
    
    // Copy CSS
    fs::write(
        format!("{}/styles.css", output_dir),
        include_str!("templates/styles.css"),
    ).context("Failed to write CSS file")?;
    
    // Copy web components
    fs::write(
        format!("{}/components/doc-sidebar.js", output_dir),
        include_str!("templates/components/doc-sidebar.js"),
    ).context("Failed to write sidebar component")?;
    
    fs::write(
        format!("{}/components/doc-toc.js", output_dir),
        include_str!("templates/components/doc-toc.js"),
    ).context("Failed to write TOC component")?;
    
    Ok(())
}
