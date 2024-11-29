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

#[derive(Serialize, Debug, Clone)]
struct PageData {
    title: String,
    content: String,   // Headers within the current page
    sections: Vec<Section>,    // Global navigation
    previous: Option<PageInfo>,
    next: Option<PageInfo>,
}


#[derive(Serialize, Debug, Clone)]
struct Section {
    title: String,
    pages: Vec<PageInfo>,
}

#[derive(Serialize, Debug, Clone)]
struct PageInfo {
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
    
    
    // Create sections based on directory structure
    let mut sections: Vec<Section> = Vec::new();
    let mut root_pages: Vec<PageInfo> = Vec::new();

    for entry in WalkDir::new(&args.input) {
        let entry = entry?;
        if entry.path().extension().map_or(false, |e| e == "md") {
            let rel_path = entry.path().strip_prefix(&args.input)?;
            let parent_dir = rel_path.parent().and_then(|p| p.to_str()).unwrap_or("");
            
            let page_info = PageInfo {
                title: extract_title(&fs::read_to_string(entry.path())?)
                    .unwrap_or_else(|| entry.path().file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Untitled".to_string())),
                path: rel_path.with_extension("html").display().to_string(),
            };

            if parent_dir.is_empty() {
                root_pages.push(page_info);
            } else {
                if let Some(section) = sections.iter_mut().find(|s| s.title == parent_dir) {
                    section.pages.push(page_info);
                } else {
                    sections.push(Section {
                        title: parent_dir.to_string(),
                        pages: vec![page_info],
                    });
                }
            }
        }
    }

    // Add root pages as a section if they exist
    if !root_pages.is_empty() {
        sections.insert(0, Section {
            title: "Guide".to_string(),
            pages: root_pages,
        });
    }

    // Process markdown files
    let mut all_pages: Vec<PageInfo> = Vec::new();
    
    // Collect only markdown files first
    for entry in WalkDir::new(&args.input) {
        let entry = entry?;
        if entry.path().extension().map_or(false, |e| e == "md") {
            let markdown_content = fs::read_to_string(entry.path())?;
            let rel_path = entry.path().strip_prefix(&args.input)?;
            
            let page_info = PageInfo {
                title: extract_title(&markdown_content)
                    .unwrap_or_else(|| entry.path().file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Untitled".to_string())),
                path: rel_path.with_extension("html").display().to_string(),
            };
            
            all_pages.push(page_info);
        }
    }
    
    // Second pass: generate pages with navigation
    let total_pages = all_pages.len();
    let mut current_page = 0;
    
    for entry in WalkDir::new(&args.input) {
        let entry = entry?;
        if entry.path().extension().map_or(false, |e| e == "md") {
            let rel_path = entry.path().strip_prefix(&args.input)?;
            let html_path = format!("{}/{}", args.output, rel_path.with_extension("html").display());
            
            if let Some(parent) = std::path::Path::new(&html_path).parent() {
                fs::create_dir_all(parent)?;
            }
            
            let markdown_content = fs::read_to_string(entry.path())?;
            let options = markdown::Options::gfm();
            let html_content = to_html_with_options(&markdown_content, &options)
                .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))?;
            
            
            // Safe navigation handling
            let previous = if current_page > 0 {
                Some(all_pages[current_page - 1].clone())
            } else {
                None
            };
            
            let next = if current_page + 1 < total_pages {
                Some(all_pages[current_page + 1].clone())
            } else {
                None
            };
            
            let page_data = PageData {
                title: extract_title(&markdown_content)
                    .unwrap_or_else(|| entry.path().file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Untitled".to_string())),
                content: html_content,
                sections: sections.clone(),
                previous,
                next,
            };
            
            let mut context = TeraContext::new();
            context.insert("page", &page_data);
            
            let rendered = tera.render("page", &context)?;
            fs::write(html_path, rendered)?;
            
            current_page += 1;
        }
    }
    
    // Generate index page
    let mut context = TeraContext::new();
    context.insert("pages", &all_pages);
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