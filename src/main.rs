use anyhow::{Context, Result};
use clap::Parser;
use jiff::Zoned;
use markdown::to_html_with_options;
use serde::Serialize;
use std::fs;
use tera::{Context as TeraContext, Tera};
use walkdir::WalkDir;
use syntect::highlighting::ThemeSet;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use markdown::mdast::Node;
use markdown::to_mdast;
mod config;
use config::{BookConfig, MarkdownFormat};
use tokio;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::time::Duration;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
mod server; // Import the server module

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing markdown files
    #[arg(short, long)]
    input: String,

    /// Output directory for HTML files
    #[arg(short, long)]
    output: String,

    /// Optional path to config file
    #[arg(short, long)]
    config: Option<String>,

    /// Watch for changes and rebuild
    #[arg(short, long)]
    watch: bool,

    /// Serve the book at http://localhost:3000
    #[arg(short, long)]
    serve: bool,

    /// Port to serve on when using --serve (default: 3000)
    #[arg(long, default_value = "3000")]
    port: u16,
}

#[derive(Serialize, Debug, Clone)]
struct PageData {
    title: String,
    content: String,
    sections: Vec<Section>,
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let watch_enabled = args.watch;
    // Load configuration
    let config = config::load_config(args.config.as_deref())?;
    
    // Initial build
    build(&args, &config, watch_enabled)?;

    if args.watch || args.serve {
        let (reload_tx, _) = broadcast::channel(16);
        
        let mut handles = vec![];

        // Start server if requested
        if args.serve {
            let output_dir = args.output.clone();
            let port = args.port;
            let reload_tx = reload_tx.clone();
            
            handles.push(tokio::spawn(async move {
                if let Err(e) = server::serve_book(output_dir, port, reload_tx).await {
                    eprintln!("Server error: {}", e);
                }
            }));
        }

        // Start watcher if requested
        if args.watch {
            let mut watch_paths = vec![args.input.clone()];
            if let Some(templates_dir) = get_templates_dir(&config) {
                println!("Adding template dir to watch: {}", templates_dir);
                watch_paths.push(templates_dir);
            }

            let args = args.clone();
            let config = config.clone();
            let reload_tx = reload_tx.clone();

            handles.push(tokio::spawn(async move {
                if let Err(e) = watch_files(watch_paths, move || {
                    build(&args, &config, watch_enabled)
                }, reload_tx).await {
                    eprintln!("Watch error: {}", e);
                }
            }));
        }

        // Keep the main task running
        futures::future::join_all(handles).await;
    }

    Ok(())
}

fn get_templates_dir(config: &BookConfig) -> Option<String> {
    let templates_dir = &config.paths.templates;
    if Path::new(templates_dir).exists() {
        Some(templates_dir.clone())
    } else {
        None
    }
}

async fn watch_files<F>(paths: Vec<String>, rebuild: F, reload_tx: broadcast::Sender<()>) -> Result<()>
where
    F: Fn() -> Result<()> + Send + Sync + 'static,
{
    use tokio::time::Duration;
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                println!("Change detected: {:?}", event);
                let _ = tx.blocking_send(());
            }
        },
        notify::Config::default(),
    )?;

    // Watch all paths
    for path in &paths {
        println!("Watching {}", path);
        watcher.watch(std::path::Path::new(path), RecursiveMode::Recursive)?;
    }

    // Debounce timer
    let mut debounce = tokio::time::interval(Duration::from_millis(500));
    let mut pending = false;

    loop {
        tokio::select! {
            Some(_) = rx.recv() => {
                pending = true;
            }
            _ = debounce.tick() => {
                if pending {
                    pending = false;
                    println!("Rebuilding...");
                    if let Err(e) = rebuild() {
                        eprintln!("Rebuild error: {}", e);
                    } else {
                        let _ = reload_tx.send(());
                    }
                }
            }
        }
    }
}

fn build(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
    // Initialize Tera with configured templates directory
    let mut tera = Tera::default();
    
    // Add template files from the configured directory
    let template_files = [
        ("page", "page.html.tera"),
        ("index", "index.html.tera"),
        ("sidebar", "sidebar.html.tera"),
        ("footer", "footer.html.tera"),
        ("header", "header.html.tera"),
    ];

    for (name, file) in template_files {
        let template_path = format!("{}/{}", config.paths.templates, file);
        let template_content = if Path::new(&template_path).exists() {
            fs::read_to_string(&template_path)
                .with_context(|| format!("Failed to read template: {}", template_path))?
        } else {
            // Load default template content directly
            match file {
                "page.html.tera" => include_str!("templates/page.html.tera").to_string(),
                "index.html.tera" => include_str!("templates/index.html.tera").to_string(),
                "sidebar.html.tera" => include_str!("templates/sidebar.html.tera").to_string(),
                "footer.html.tera" => include_str!("templates/footer.html.tera").to_string(),
                "header.html.tera" => include_str!("templates/header.html.tera").to_string(),
                _ => return Err(anyhow::anyhow!("Unknown template file: {}", file)),
            }
        };
        
        tera.add_raw_template(name, &template_content)
            .with_context(|| format!("Failed to add template: {}", name))?;
    }
    
    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output)?;
    
    // Copy static assets
    copy_static_assets(&args.output, &config.paths.templates, &config)?;

    // Collect all pages first
    let mut all_pages = Vec::new();
    let mut section_map: BTreeMap<String, Vec<PageInfo>> = BTreeMap::new();
    let mut root_pages: Vec<PageInfo> = Vec::new();

    // First pass: collect all pages
    let mut entries: Vec<_> = WalkDir::new(&args.input)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .collect();

    // Sort entries by path to ensure consistent ordering
    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in &entries {
        let rel_path = entry.path().strip_prefix(&args.input)?;
        let parent_dir = rel_path.parent().and_then(|p| p.to_str()).unwrap_or("");
        
        let content = fs::read_to_string(entry.path())?;
        let page_info = PageInfo {
            title: extract_title(&content)
                .unwrap_or_else(|| entry.path().file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Untitled".to_string())),
            path: format!("/{}", rel_path.with_extension("html").display().to_string()),
        };

        all_pages.push(page_info.clone());

        if parent_dir.is_empty() {
            root_pages.push(page_info);
        } else {
            section_map.entry(parent_dir.to_string())
                .or_default()
                .push(page_info);
        }
    }

    // Convert the map to sections
    let mut sections = Vec::new();
    
    // Add root pages first if they exist
    if !root_pages.is_empty() {
        sections.push(Section {
            title: "Guide".to_string(),
            pages: root_pages,
        });
    }

    // Add other sections
    for (title, pages) in section_map {
        sections.push(Section {
            title,
            pages,
        });
    }

    let total_pages = all_pages.len();
    println!("Total pages: {}", total_pages);

    // Get current year using Jiff
    let current_year = Zoned::now().year().to_string();

    // Initialize SyntaxSet once
    let ss = SyntaxSet::load_defaults_newlines();
    
    // Add syntax highlighting CSS
    let ts = ThemeSet::load_defaults();
    // TODO: Make this configurable
    let theme = &ts.themes["Solarized (light)"];
    let syntax_css = syntect::html::css_for_theme_with_class_style(theme, ClassStyle::Spaced)
        .map_err(|e| anyhow::anyhow!("CSS generation error: {:?}", e))?;
    
    fs::write(format!("{}/css/syntax.css", args.output), syntax_css)?;

    // Process each markdown file
    for (current_page, entry) in entries.iter().enumerate() {
        if entry.path().extension().map_or(false, |ext| ext == "md") {
            let rel_path = entry.path().strip_prefix(&args.input)?;
            let html_path = format!("{}/{}", args.output, rel_path.with_extension("html").display());
            
            if let Some(parent) = Path::new(&html_path).parent() {
                fs::create_dir_all(parent)?;
            }
            
            let markdown_content = fs::read_to_string(entry.path())?;
            let html_content = process_markdown_with_highlighting(&markdown_content, &ss, &config)?;
            
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
            context.insert("year", &current_year);
            context.insert("page", &page_data);
            context.insert("config", &config);
            context.insert("current_path", &rel_path.with_extension("html").display().to_string());
            context.insert("watch_enabled", &watch_enabled);
            
            let rendered = tera.render("page", &context)
                .with_context(|| format!("Failed to render page: {}", html_path))?;
            fs::write(&html_path, rendered)
                .with_context(|| format!("Failed to write file: {}", html_path))?;
        }
    }

    // Generate index page
    let mut context = TeraContext::new();
    context.insert("year", &current_year);
    context.insert("config", &config);
    context.insert("sections", &sections);
    context.insert("current_path", &"index.html");

    let index_page = all_pages.iter().find(|p| p.path == "/index.html");
    
    if let Some(index) = index_page {
        // If index.md exists, use its content
        let index_path = Path::new(&args.input).join("index.md");
        let markdown_content = fs::read_to_string(&index_path)
            .with_context(|| format!("Failed to read index file: {}", index_path.display()))?;
        let html_content = process_markdown_with_highlighting(&markdown_content, &ss, &config)?;
        
        context.insert("has_index", &true);
        context.insert("title", &index.title);
        context.insert("content", &html_content);
    } else {
        // If no index.md, use the default template with cards
        context.insert("has_index", &false);
        context.insert("title", &"Documentation");
    }
    
    let rendered = tera.render("index", &context)
        .context("Failed to render index page")?;
    fs::write(format!("{}/index.html", args.output), rendered)
        .context("Failed to write index.html")?;

    Ok(())
}

fn extract_title(markdown: &str) -> Option<String> {
    markdown
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line[2..].trim().to_string())
}

fn copy_static_assets(output_dir: &str, templates_dir: &str, config: &BookConfig) -> Result<()> {
    // Create components directory
    fs::create_dir_all(format!("{}/components", output_dir))?;
    
    // Copy CSS directory
    let css_source = format!("{}/css", templates_dir);
    let css_dest = format!("{}/css/", output_dir);
    fs::create_dir_all(&css_dest)?;
    if std::path::Path::new(&css_source).exists() {
        for entry in WalkDir::new(&css_source) {
            let entry = entry?;
            let dest_path = css_dest.clone() + entry.path().strip_prefix(&css_source)?.to_str().unwrap();
            if entry.file_type().is_file() {
                fs::copy(entry.path(), dest_path)?;
            }
        }
    }

    // Copy JS directory
    let js_source = format!("{}/js", templates_dir);
    let js_dest = format!("{}/js/", output_dir);
    fs::create_dir_all(&js_dest)?;
    if std::path::Path::new(&js_source).exists() {
        for entry in WalkDir::new(&js_source) {
            let entry = entry?;
            let dest_path = js_dest.clone() + entry.path().strip_prefix(&js_source)?.to_str().unwrap();
            if entry.file_type().is_file() {
                fs::copy(entry.path(), dest_path)?;
            }
        }
    }
    // Copy img directory from templates
    let img_source = "src/templates/img";
    let img_dest = format!("{}/img/", output_dir);
    fs::create_dir_all(&img_dest)?;
    for entry in WalkDir::new(img_source) {
        let entry = entry?;
        let dest_path = img_dest.clone() + entry.path().strip_prefix(img_source)?.to_str().unwrap();
        if entry.file_type().is_file() {
            fs::copy(entry.path(), dest_path).context(format!("Failed to copy img file: {:?}", entry.path()))?;
        }
    }


        fs::write(
            format!("{}/components/doc-toc.js", output_dir),
            include_str!("templates/components/doc-toc.js"),
        ).context("Failed to write TOC component")?;

        fs::write(
            format!("{}/components/simple-block.js", output_dir),
            include_str!("templates/components/simple-block.js"),
        ).context("Failed to write Simple Block component")?;

    Ok(())
}

fn process_code_block(code: &str, language: Option<&str>, ss: &SyntaxSet) -> Result<String> {
    let syntax = match language {
        Some("rust") => {
            let syntax = ss.find_syntax_by_extension("rs")
                .ok_or_else(|| anyhow::anyhow!("Rust syntax not found"))?;
            // Check if code block has editable tag
            if code.contains("<--editable-->") {
                let code_with_comment = format!("{}\n// <--editable-->", code);
                process_rust_code(&code_with_comment, syntax, ss)?
            } else {
                process_rust_code(code, syntax, ss)?
            }
        },
        Some("mermaid") => {
            // For markdown, preserve the content exactly as is
            format!("<pre class=\"code\"><code class=\"language-mermaid\">{}</code></pre>", 
                   html_escape::encode_text(code))
        },
        Some(lang) => {
            let syntax = ss.find_syntax_by_extension(lang)
                .or_else(|| ss.find_syntax_by_name(lang))
                .or_else(|| ss.find_syntax_by_token(lang))
                .or_else(|| Some(ss.find_syntax_plain_text()))
                .ok_or_else(|| anyhow::anyhow!("Syntax not found for language: {:?}", lang))?;
            process_generic_code(code, syntax, ss)?
        },
        None => {
            let syntax = ss.find_syntax_plain_text();
            process_generic_code(code, syntax, ss)?
        }
    };
    Ok(syntax)
}

fn process_rust_code(code: &str, syntax: &syntect::parsing::SyntaxReference, ss: &SyntaxSet) -> Result<String> {
    let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        ss,
        ClassStyle::Spaced
    );

    for line in LinesWithEndings::from(code) {
        html_generator.parse_html_for_line_which_includes_newline(line)
            .map_err(|e| anyhow::anyhow!("HTML generation error: {:?}", e))?;
    }
    let html = html_generator.finalize();
    Ok(format!("<pre class=\"code rust\"><code>{}</code></pre>", html))
}

fn process_generic_code(code: &str, syntax: &syntect::parsing::SyntaxReference, ss: &SyntaxSet) -> Result<String> {
    let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        ss,
        ClassStyle::Spaced
    );

    for line in LinesWithEndings::from(code) {
        html_generator.parse_html_for_line_which_includes_newline(line)
            .map_err(|e| anyhow::anyhow!("HTML generation error: {:?}", e))?;
    }
    let html = html_generator.finalize();
    Ok(format!("<pre class=\"code\"><code>{}</code></pre>", html))
}

fn process_markdown_with_highlighting(content: &str, ss: &SyntaxSet, config: &BookConfig) -> Result<String> {
    let parse_options = match config.markdown.format {
        MarkdownFormat::Mdx => markdown::ParseOptions::mdx(),
        MarkdownFormat::Gfm => markdown::ParseOptions::gfm(),
        MarkdownFormat::Markdown => markdown::ParseOptions::default(),
    };

    let compile_options = if matches!(config.markdown.format, MarkdownFormat::Gfm) {
        markdown::CompileOptions::gfm()
    } else {
        markdown::CompileOptions::default()
    };

    let mut options = markdown::Options {
        parse: parse_options,
        compile: compile_options,
    };

    // Modify constructs for HTML and frontmatter
    options.parse.constructs.frontmatter = config.markdown.frontmatter;
    options.parse.constructs.html_flow = config.output.html.allow_html;
    options.parse.constructs.html_text = config.output.html.allow_html;
    options.compile.allow_dangerous_html = config.output.html.allow_html;
    options.compile.allow_dangerous_protocol = config.output.html.allow_html;

    let ast = to_mdast(content, &options.parse)
        .map_err(|e| anyhow::anyhow!("Markdown parsing error: {:?}", e))?;
    
    let mut parts = Vec::new();
    let mut last_pos = 0;
    
    fn process_node(node: &Node, ss: &SyntaxSet, content: &str, parts: &mut Vec<String>, last_pos: &mut usize, config: &BookConfig) -> Result<()> {
        match node {
            Node::Code(code) => {
                if let Some(pos) = &code.position {
                    if *last_pos < pos.start.offset {
                        let text = &content[*last_pos..pos.start.offset];
                        if !text.trim().is_empty() {
                            let parse_options = match config.markdown.format {
                                MarkdownFormat::Mdx => markdown::ParseOptions::mdx(),
                                MarkdownFormat::Gfm => markdown::ParseOptions::gfm(),
                                MarkdownFormat::Markdown => markdown::ParseOptions::default(),
                            };

                            let compile_options = if matches!(config.markdown.format, MarkdownFormat::Gfm) {
                                markdown::CompileOptions::gfm()
                            } else {
                                markdown::CompileOptions::default()
                            };

                            let mut options = markdown::Options {
                                parse: parse_options,
                                compile: compile_options,
                            };

                            options.parse.constructs.frontmatter = config.markdown.frontmatter;
                            options.parse.constructs.html_flow = config.output.html.allow_html;
                            options.parse.constructs.html_text = config.output.html.allow_html;
                            options.compile.allow_dangerous_html = config.output.html.allow_html;
                            options.compile.allow_dangerous_protocol = config.output.html.allow_html;

                            let temp_html = to_html_with_options(text, &options)
                                .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))?;
                            parts.push(temp_html);
                        }
                    }
                    
                    let highlighted = process_code_block(&code.value, code.lang.as_deref(), ss)?;
                    parts.push(highlighted);
                    
                    *last_pos = pos.end.offset;
                }
            },
            _ => {
                if let Some(children) = node.children() {
                    for child in children {
                        process_node(child, ss, content, parts, last_pos, config)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    process_node(&ast, ss, content, &mut parts, &mut last_pos, config)?;
    
    if last_pos < content.len() {
        let remaining = &content[last_pos..];
        if !remaining.trim().is_empty() {
            let parse_options = match config.markdown.format {
                MarkdownFormat::Mdx => markdown::ParseOptions::mdx(),
                MarkdownFormat::Gfm => markdown::ParseOptions::gfm(),
                MarkdownFormat::Markdown => markdown::ParseOptions::default(),
            };

            let compile_options = if matches!(config.markdown.format, MarkdownFormat::Gfm) {
                markdown::CompileOptions::gfm()
            } else {
                markdown::CompileOptions::default()
            };

            let mut options = markdown::Options {
                parse: parse_options,
                compile: compile_options,
            };

            options.parse.constructs.frontmatter = config.markdown.frontmatter;
            options.parse.constructs.html_flow = config.output.html.allow_html;
            options.parse.constructs.html_text = config.output.html.allow_html;
            options.compile.allow_dangerous_html = config.output.html.allow_html;
            options.compile.allow_dangerous_protocol = config.output.html.allow_html;

            parts.push(to_html_with_options(remaining, &options)
                .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))?);
        }
    }
    
    Ok(parts.join(""))
}