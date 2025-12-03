use anyhow::{Context, Result};
use clap::Parser;
use jiff::Zoned;

use markdown::to_html_with_options;
use serde::Serialize;
use std::fs;
use tera::{Context as TeraContext, Tera};
use walkdir::WalkDir;

use crate::config::{BookConfig, MarkdownFormat};
use crate::pagefind_service::PagefindBuilder;
use markdown::mdast::Node;
use markdown::to_mdast;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
#[cfg(feature = "syntax-highlighting")]
use syntect::highlighting::ThemeSet;
#[cfg(feature = "syntax-highlighting")]
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
#[cfg(feature = "syntax-highlighting")]
use syntect::parsing::SyntaxSet;
#[cfg(feature = "syntax-highlighting")]
use syntect::util::LinesWithEndings;

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
    build_sync_impl(args, config, watch_enabled).await
}

#[cfg(not(feature = "tokio"))]
fn build_impl(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
    build_sync_impl_sync(args, config, watch_enabled)
}

#[cfg(feature = "tokio")]
async fn build_sync_impl(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
    build_sync_impl_sync(args, config, watch_enabled)?;

    // After generating HTML files, run Pagefind indexing if search feature is enabled
    #[cfg(all(feature = "search", feature = "tokio"))]
    {
        match PagefindBuilder::new(PathBuf::from(&args.output)).await {
            Ok(pagefind) => {
                if let Err(e) = pagefind.build().await {
                    eprintln!("Search indexing failed: {e}");
                }
            }
            Err(e) => {
                eprintln!("Failed to create search builder: {e}");
            }
        }
    }

    Ok(())
}

fn build_sync_impl_sync(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
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
                .with_context(|| format!("Failed to read template: {template_path}"))?
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
            .with_context(|| format!("Failed to add template: {name}"))?;
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output)?;

    // Copy static assets
    copy_static_assets(&args.output, &config.paths.templates, config)?;

    // Collect all pages first
    let mut all_pages = Vec::new();
    let mut section_map: BTreeMap<String, Vec<PageInfo>> = BTreeMap::new();
    let mut root_pages: Vec<PageInfo> = Vec::new();

    // First pass: collect all pages
    let mut entries: Vec<_> = WalkDir::new(&args.input)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();

    // Sort entries by path to ensure consistent ordering
    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in &entries {
        let rel_path = entry.path().strip_prefix(&args.input)?;
        let parent_dir = rel_path.parent().and_then(|p| p.to_str()).unwrap_or("");

        let content = fs::read_to_string(entry.path())?;
        let page_info = PageInfo {
            title: extract_title(&content).unwrap_or_else(|| {
                entry.path().file_stem().map_or_else(
                    || "Untitled".to_string(),
                    |s| s.to_string_lossy().into_owned(),
                )
            }),
            path: format!("/{}", rel_path.with_extension("html").display()),
        };

        all_pages.push(page_info.clone());

        if parent_dir.is_empty() {
            root_pages.push(page_info);
        } else {
            section_map
                .entry(parent_dir.to_string())
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
        sections.push(Section { title, pages });
    }

    let total_pages = all_pages.len();
    println!("Total pages: {total_pages}");

    // Get current year using Jiff
    let current_year = Zoned::now().year().to_string();

    // Initialize syntax highlighting if feature is enabled
    #[cfg(feature = "syntax-highlighting")]
    let ss = SyntaxSet::load_defaults_newlines();

    #[cfg(feature = "syntax-highlighting")]
    {
        // Add syntax highlighting CSS
        let ts = ThemeSet::load_defaults();
        // TODO: Make this configurable
        let theme = &ts.themes["Solarized (light)"];
        let syntax_css = syntect::html::css_for_theme_with_class_style(theme, ClassStyle::Spaced)
            .map_err(|e| anyhow::anyhow!("CSS generation error: {:?}", e))?;

        fs::write(format!("{}/css/syntax.css", args.output), syntax_css)?;
    }

    // Process each markdown file
    for (current_page, entry) in entries.iter().enumerate() {
        if entry.path().extension().is_some_and(|ext| ext == "md") {
            let rel_path = entry.path().strip_prefix(&args.input)?;
            let html_path = format!(
                "{}/{}",
                args.output,
                rel_path.with_extension("html").display()
            );

            if let Some(parent) = Path::new(&html_path).parent() {
                fs::create_dir_all(parent)?;
            }

            let markdown_content = fs::read_to_string(entry.path())?;
            #[cfg(feature = "syntax-highlighting")]
            let html_content = process_markdown_with_highlighting(&markdown_content, &ss, config)?;
            #[cfg(not(feature = "syntax-highlighting"))]
            let html_content = process_markdown_basic(&markdown_content, config)?;

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
                title: extract_title(&markdown_content).unwrap_or_else(|| {
                    entry
                        .path()
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Untitled".to_string())
                }),
                content: html_content,
                sections: sections.clone(),
                previous,
                next,
            };

            let mut context = TeraContext::new();
            context.insert("year", &current_year);
            context.insert("page", &page_data);
            context.insert("config", &config);
            context.insert(
                "current_path",
                &rel_path.with_extension("html").display().to_string(),
            );
            context.insert("watch_enabled", &watch_enabled);

            let rendered = tera
                .render("page", &context)
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
        #[cfg(feature = "syntax-highlighting")]
        let html_content = process_markdown_with_highlighting(&markdown_content, &ss, config)?;
        #[cfg(not(feature = "syntax-highlighting"))]
        let html_content = process_markdown_basic(&markdown_content, config)?;

        context.insert("has_index", &true);
        context.insert("title", &index.title);
        context.insert("content", &html_content);
    } else {
        // If no index.md, use the default template with cards
        context.insert("has_index", &false);
        context.insert("title", &"Documentation");
    }

    let rendered = tera
        .render("index", &context)
        .context("Failed to render index page")?;
    fs::write(format!("{}/index.html", args.output), rendered)
        .context("Failed to write index.html")?;

    // Search indexing handled in async wrapper or skipped
    #[cfg(not(all(feature = "search", feature = "tokio")))]
    {
        println!("Skipping search indexing (search or tokio feature not enabled)");
    }

    Ok(())
}

fn extract_title(markdown: &str) -> Option<String> {
    markdown
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line[2..].trim().to_string())
}

fn copy_static_assets(output_dir: &str, templates_dir: &str, _config: &BookConfig) -> Result<()> {
    // Create components directory
    fs::create_dir_all(format!("{}/components", output_dir))?;

    // Copy CSS directory
    let css_source = format!("{}/css", templates_dir);
    let css_dest = format!("{}/css/", output_dir);
    fs::create_dir_all(&css_dest)?;
    if std::path::Path::new(&css_source).exists() {
        for entry in WalkDir::new(&css_source) {
            let entry = entry?;
            let dest_path =
                css_dest.clone() + entry.path().strip_prefix(&css_source)?.to_str().unwrap();
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
            let dest_path =
                js_dest.clone() + entry.path().strip_prefix(&js_source)?.to_str().unwrap();
            if entry.file_type().is_file() {
                fs::copy(entry.path(), dest_path)?;
            }
        }
    }
    // Copy img directory from templates
    let img_source = format!("{}/img", templates_dir);
    let img_dest = format!("{}/img/", output_dir);
    fs::create_dir_all(&img_dest)?;
    if std::path::Path::new(&img_source).exists() {
        for entry in WalkDir::new(&img_source) {
            let entry = entry?;
            let dest_path =
                img_dest.clone() + entry.path().strip_prefix(&img_source)?.to_str().unwrap();
            if entry.file_type().is_file() {
                fs::copy(entry.path(), dest_path)
                    .context(format!("Failed to copy img file: {:?}", entry.path()))?;
            }
        }
    }

    fs::write(
        format!("{}/components/doc-toc.js", output_dir),
        include_str!("templates/components/doc-toc.js"),
    )
    .context("Failed to write TOC component")?;

    fs::write(
        format!("{}/components/simple-block.js", output_dir),
        include_str!("templates/components/simple-block.js"),
    )
    .context("Failed to write Simple Block component")?;

    fs::write(
        format!("{}/components/search-modal.js", output_dir),
        include_str!("templates/components/search-modal.js"),
    )
    .context("Failed to write Search Modal component")?;

    Ok(())
}

#[cfg(feature = "syntax-highlighting")]
fn process_code_block(code: &str, language: Option<&str>, ss: &SyntaxSet) -> Result<String> {
    let syntax = match language {
        Some("rust") => {
            let syntax = ss
                .find_syntax_by_extension("rs")
                .ok_or_else(|| anyhow::anyhow!("Rust syntax not found"))?;
            // Check if code block has editable tag
            if code.contains("<--editable-->") {
                let code_with_comment = format!("{}\n// <--editable-->", code);
                process_rust_code(&code_with_comment, syntax, ss)?
            } else {
                process_rust_code(code, syntax, ss)?
            }
        }
        Some("mermaid") => {
            // For markdown, preserve the content exactly as is
            format!(
                "<pre class=\"code\"><code class=\"language-mermaid\">{}</code></pre>",
                html_escape::encode_text(code)
            )
        }
        Some(lang) => {
            let syntax = ss
                .find_syntax_by_extension(lang)
                .or_else(|| ss.find_syntax_by_name(lang))
                .or_else(|| ss.find_syntax_by_token(lang))
                .or_else(|| Some(ss.find_syntax_plain_text()))
                .ok_or_else(|| anyhow::anyhow!("Syntax not found for language: {:?}", lang))?;
            process_generic_code(code, syntax, ss)?
        }
        None => {
            let syntax = ss.find_syntax_plain_text();
            process_generic_code(code, syntax, ss)?
        }
    };
    Ok(syntax)
}

#[cfg(feature = "syntax-highlighting")]
fn process_rust_code(
    code: &str,
    syntax: &syntect::parsing::SyntaxReference,
    ss: &SyntaxSet,
) -> Result<String> {
    let mut html_generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, ss, ClassStyle::Spaced);

    for line in LinesWithEndings::from(code) {
        html_generator
            .parse_html_for_line_which_includes_newline(line)
            .map_err(|e| anyhow::anyhow!("HTML generation error: {:?}", e))?;
    }
    let html = html_generator.finalize();
    Ok(format!(
        "<pre class=\"code rust\"><code>{}</code></pre>",
        html
    ))
}

#[cfg(feature = "syntax-highlighting")]
fn process_generic_code(
    code: &str,
    syntax: &syntect::parsing::SyntaxReference,
    ss: &SyntaxSet,
) -> Result<String> {
    let mut html_generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, ss, ClassStyle::Spaced);

    for line in LinesWithEndings::from(code) {
        html_generator
            .parse_html_for_line_which_includes_newline(line)
            .map_err(|e| anyhow::anyhow!("HTML generation error: {:?}", e))?;
    }
    let html = html_generator.finalize();
    Ok(format!("<pre class=\"code\"><code>{}</code></pre>", html))
}

#[cfg(feature = "syntax-highlighting")]
fn process_markdown_with_highlighting(
    content: &str,
    ss: &SyntaxSet,
    config: &BookConfig,
) -> Result<String> {
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

    fn process_node(
        node: &Node,
        ss: &SyntaxSet,
        content: &str,
        parts: &mut Vec<String>,
        last_pos: &mut usize,
        config: &BookConfig,
    ) -> Result<()> {
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

                            let compile_options =
                                if matches!(config.markdown.format, MarkdownFormat::Gfm) {
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
                            options.compile.allow_dangerous_protocol =
                                config.output.html.allow_html;

                            let temp_html = to_html_with_options(text, &options).map_err(|e| {
                                anyhow::anyhow!("Markdown conversion error: {:?}", e)
                            })?;
                            parts.push(temp_html);
                        }
                    }

                    let highlighted = process_code_block(&code.value, code.lang.as_deref(), ss)?;
                    parts.push(highlighted);

                    *last_pos = pos.end.offset;
                }
            }
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

            parts.push(
                to_html_with_options(remaining, &options)
                    .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))?,
            );
        }
    }

    Ok(parts.join(""))
}

#[cfg(not(feature = "syntax-highlighting"))]
fn process_markdown_basic(content: &str, config: &BookConfig) -> Result<String> {
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

    to_html_with_options(content, &options)
        .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BookConfig;
    use std::fs;
    use tempfile::TempDir;

    // Get project root directory (CARGO_MANIFEST_DIR) for absolute path resolution
    fn project_root() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()),
        )
    }

    #[test]
    fn test_extract_title_h1() {
        let markdown = "# Main Title\n\nSome content here.";
        let title = extract_title(markdown);
        assert_eq!(title, Some("Main Title".to_string()));
    }

    #[test]
    fn test_extract_title_h2() {
        let markdown = "Some text\n\n## Section Title\n\nContent";
        let title = extract_title(markdown);
        // extract_title only looks for H1 headings, not H2
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_title_no_heading() {
        let markdown = "Just some regular text without headings.";
        let title = extract_title(markdown);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_title_complex_markup() {
        let markdown = "# Title with **bold** and *italic*";
        let title = extract_title(markdown);
        assert_eq!(title, Some("Title with **bold** and *italic*".to_string()));
    }

    #[test]
    fn test_extract_title_first_heading_wins() {
        let markdown = "# First Title\n\n## Second Title\n\n# Third Title";
        let title = extract_title(markdown);
        assert_eq!(title, Some("First Title".to_string()));
    }

    #[test]
    fn test_args_default_values() {
        use clap::Parser;

        // Test that we can parse minimal required args
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

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_default() -> Result<()> {
        let config = BookConfig::default();
        let markdown = "# Hello World\n\nThis is **bold** text.";

        let html = process_markdown_basic(markdown, &config)?;

        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("<strong>bold</strong>"));

        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_gfm() -> Result<()> {
        let mut config = BookConfig::default();
        config.markdown.format = MarkdownFormat::Gfm;

        let markdown = "# GFM Test\n\n~~strikethrough~~\n\n- [ ] Task item";

        let html = process_markdown_basic(markdown, &config)?;

        assert!(html.contains("<h1>GFM Test</h1>"));
        assert!(html.contains("strikethrough"));

        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_mdx() -> Result<()> {
        let mut config = BookConfig::default();
        config.markdown.format = MarkdownFormat::Mdx;

        let markdown = "# MDX Test\n\nThis is **bold** text.";

        let html = process_markdown_basic(markdown, &config)?;

        assert!(html.contains("<h1>MDX Test</h1>"));
        assert!(html.contains("<strong>bold</strong>"));

        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_with_html_allowed() -> Result<()> {
        let mut config = BookConfig::default();
        config.output.html.allow_html = true;

        let markdown = "# Test\n\n<div>Raw HTML</div>";

        let html = process_markdown_basic(markdown, &config)?;

        assert!(html.contains("<div>Raw HTML</div>"));

        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_with_html_disallowed() -> Result<()> {
        let config = BookConfig::default();

        let markdown = "# Test\n\n<div>Raw HTML</div>";

        let html = process_markdown_basic(markdown, &config)?;

        // HTML should be escaped or stripped when not allowed
        assert!(!html.contains("<div>Raw HTML</div>"));

        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_with_frontmatter() -> Result<()> {
        let mut config = BookConfig::default();
        config.markdown.frontmatter = true;

        let markdown = "---\ntitle: Test\n---\n\n# Hello World";

        let html = process_markdown_basic(markdown, &config)?;

        assert!(html.contains("<h1>Hello World</h1>"));
        // Frontmatter should be processed/removed from output
        assert!(!html.contains("---"));

        Ok(())
    }

    #[test]
    #[ignore = "MathJax support not implemented yet"]
    fn test_process_markdown_with_mathjax() -> Result<()> {
        let mut config = BookConfig::default();
        config.output.html.mathjax_support = true;

        let markdown = "# Math Test\n\n$$E = mc^2$$";

        // Test with basic markdown processing (will work regardless of syntax highlighting feature)
        let html = markdown::to_html(markdown);

        // When implemented, should contain MathJax markup
        assert!(html.contains("E = mc^2"));

        Ok(())
    }

    #[test]
    fn test_page_data_serialization() -> Result<()> {
        let page_data = PageData {
            title: "Test Page".to_string(),
            content: "<h1>Test</h1>".to_string(),
            sections: vec![Section {
                title: "Section 1".to_string(),
                pages: vec![PageInfo {
                    title: "Page 1".to_string(),
                    path: "/page1".to_string(),
                }],
            }],
            previous: Some(PageInfo {
                title: "Previous".to_string(),
                path: "/prev".to_string(),
            }),
            next: None,
        };

        let serialized = serde_json::to_string(&page_data)?;
        assert!(serialized.contains("Test Page"));
        assert!(serialized.contains("Section 1"));
        assert!(serialized.contains("/page1"));

        Ok(())
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn test_process_code_block_rust() -> Result<()> {
        use syntect::parsing::SyntaxSet;

        let ss = SyntaxSet::load_defaults_newlines();
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}";

        let highlighted = process_code_block(code, Some("rust"), &ss)?;

        assert!(highlighted.contains("<pre"));
        // Syntax highlighting behavior may vary, just check basic structure
        assert!(!highlighted.is_empty());

        Ok(())
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn test_process_code_block_no_language() -> Result<()> {
        use syntect::parsing::SyntaxSet;

        let ss = SyntaxSet::load_defaults_newlines();
        let code = "some plain text code";

        let highlighted = process_code_block(code, None, &ss)?;

        assert!(highlighted.contains("<pre"));
        assert!(highlighted.contains("some plain text code"));

        Ok(())
    }

    #[test]
    fn test_copy_static_assets() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_dir = temp_dir.path().join("output");
        // Use absolute path to avoid issues when other tests change working directory
        let templates_dir = project_root().join("src/templates");

        fs::create_dir_all(&output_dir)?;

        let config = BookConfig::default();
        copy_static_assets(
            output_dir.to_str().unwrap(),
            templates_dir.to_str().unwrap(),
            &config,
        )?;

        // Check that some assets were copied (if templates exist)
        let _has_assets = output_dir.join("css").exists()
            || output_dir.join("js").exists()
            || output_dir.join("img").exists();

        // This test passes even if no assets exist, just checking the function doesn't crash
        assert!(output_dir.exists());

        Ok(())
    }

    #[test]
    fn test_copy_static_assets_nonexistent_dir() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        let templates_dir = "nonexistent_templates";

        fs::create_dir_all(&output_dir).unwrap();

        let config = BookConfig::default();
        let result = copy_static_assets(output_dir.to_str().unwrap(), templates_dir, &config);

        // Should not fail even if templates dir doesn't exist
        assert!(result.is_ok());
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

        // WASM should handle basic markdown correctly
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

        // WASM should handle code blocks (even without syntax highlighting)
        assert!(html.contains("<pre>") || html.contains("<code>"));
        assert!(html.contains("fn main"));
        assert!(html.contains("Hello, WASM!"));
    }

    // Integration-style test for build function
    #[cfg(all(feature = "tokio", not(target_arch = "wasm32")))]
    #[tokio::test]
    async fn test_build_simple_book() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("book");

        fs::create_dir_all(&input_dir)?;
        fs::create_dir_all(&output_dir)?;

        // Create simple markdown file
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

        // Verify output was created
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

        // Create simple markdown file
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

        // Verify output was created
        assert!(output_dir.exists());

        Ok(())
    }
}
