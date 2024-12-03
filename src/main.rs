use anyhow::{Context, Result};
use clap::Parser;
use jiff::{Zoned, Unit};
use markdown::{to_html_with_options, Options};
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
use config::BookConfig;

#[derive(Parser, Debug)]
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
    
    // Load configuration
    let config = config::load_config(args.config.as_deref())?;
    println!("{:#?}", config);
    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output)?;
    
    // Copy static assets
    copy_static_assets(&args.output, &config)?;
    
    // Initialize Tera 
    let mut tera = Tera::default();
    tera.add_raw_template("page", include_str!("templates/page.html.tera"))?;
    tera.add_raw_template("index", include_str!("templates/index.html.tera"))?;
    tera.add_raw_template("sidebar", include_str!("templates/sidebar.html.tera"))?;
    tera.add_raw_template("footer", include_str!("templates/footer.html.tera"))?;
    tera.add_raw_template("header", include_str!("templates/header.html.tera"))?;
    


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
                path: format!("/{}", rel_path.with_extension("html").display().to_string()),
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
                path: format!("/{}", rel_path.with_extension("html").display().to_string()),
            };
            
            all_pages.push(page_info);
        }
    }
    
    // Second pass: generate pages with navigation
    let total_pages = all_pages.len();
    let mut current_page = 0;
    
    println!("Total pages: {}", total_pages);
    let mut context = TeraContext::new();
    // Add current year to all contexts
    let now = Zoned::now().round(Unit::Second)?;
    let current_year = now.year();
    context.insert("year", &current_year);
    context.insert("sections", &all_pages);

    // Initialize SyntaxSet once
    let ss = SyntaxSet::load_defaults_newlines();
    
    // Add syntax highlighting CSS
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["Solarized (light)"];
    let syntax_css = syntect::html::css_for_theme_with_class_style(theme, ClassStyle::Spaced)
        .map_err(|e| anyhow::anyhow!("CSS generation error: {:?}", e))?;
    
    fs::write(format!("{}/css/syntax.css", args.output), syntax_css)?;

    for entry in WalkDir::new(&args.input) {
        let entry = entry?;
        if entry.path().extension().map_or(false, |e| e == "md") {
            let rel_path = entry.path().strip_prefix(&args.input)?;
            let html_path = format!("{}/{}", args.output, rel_path.with_extension("html").display());
            
            if let Some(parent) = std::path::Path::new(&html_path).parent() {
                fs::create_dir_all(parent)?;
            }
            
            let markdown_content = fs::read_to_string(entry.path())?;
            let html_content = process_markdown_with_highlighting(&markdown_content, &ss, &config)?;
            
            
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
            context.insert("year", &current_year);
            context.insert("page", &page_data);
            context.insert("config", &config);
            context.insert("current_path", &rel_path.with_extension("html").display().to_string());
            
            let rendered = tera.render("page", &context)?;
            fs::write(html_path, rendered)?;
            
            current_page += 1;
        }
    }
    
    // Generate index page
    let mut context = TeraContext::new();
    
    // Check if index.md exists in root pages
    let index_page = all_pages.iter().find(|p| p.path == "/index.html");
    
    if let Some(index) = index_page {
        // If index.md exists, use its content
        let index_path = std::path::Path::new(&args.input).join("index.md");
        let markdown_content = fs::read_to_string(index_path)?;
        println!("markdown_content index: {}", markdown_content);
        let html_content = process_markdown_with_highlighting(&markdown_content, &ss, &config)?;
        println!("html_content index: {}", html_content);
        context.insert("has_index", &true);
        context.insert("title", &index.title);
        context.insert("content", &html_content);
    } else {
        // If no index.md, use the default template with cards
        context.insert("has_index", &false);
        context.insert("title", &"Documentation");
    }

    context.insert("config", &config);
    context.insert("sections", &sections);
    context.insert("current_path", &"index.html");
    
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

fn copy_static_assets(output_dir: &str, config: &BookConfig) -> Result<()> {
    // Create components directory
    fs::create_dir_all(format!("{}/components", output_dir))?;
    
    // Copy CSS directory
    let css_source = "src/templates/css";
    let css_dest = format!("{}/css/", output_dir);
    fs::create_dir_all(&css_dest)?;
    for entry in WalkDir::new(css_source) {
        let entry = entry?;
        let dest_path = css_dest.clone() + entry.path().strip_prefix(css_source)?.to_str().unwrap();
        if entry.file_type().is_file() {
            fs::copy(entry.path(), dest_path).context(format!("Failed to copy CSS file: {:?}", entry.path()))?;
        }
    }

    // Copy JS directory
    let js_source = "src/templates/js";
    let js_dest = format!("{}/js/", output_dir);
    fs::create_dir_all(&js_dest)?;
    for entry in WalkDir::new(js_source) {
        let entry = entry?;
        let dest_path = js_dest.clone() + entry.path().strip_prefix(js_source)?.to_str().unwrap();
        if entry.file_type().is_file() {
            fs::copy(entry.path(), dest_path).context(format!("Failed to copy JS file: {:?}", entry.path()))?;
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
    let mut options = markdown::Options::gfm();
    
    // Parse options for HTML in markdown
    options.parse = markdown::ParseOptions {
        constructs: markdown::Constructs {
            html_flow: config.output.html.allow_html,
            html_text: config.output.html.allow_html,
            ..markdown::Constructs::gfm()
        },
        ..markdown::ParseOptions::gfm()
    };

    // Compile options to control HTML rendering
    options.compile = markdown::CompileOptions {
        allow_dangerous_html: config.output.html.allow_html,
        allow_dangerous_protocol: config.output.html.allow_html,
        ..markdown::CompileOptions::default()
    };

    let ast = to_mdast(content, &options.parse)
        .map_err(|e| anyhow::anyhow!("Markdown parsing error: {:?}", e))?;
    
    let mut parts = Vec::new();
    let mut last_pos = 0;
    
    fn process_node(node: &Node, ss: &SyntaxSet, content: &str, parts: &mut Vec<String>, last_pos: &mut usize) -> Result<()> {
        match node {
            Node::Code(code) => {
                // Add text before this code block
                if let Some(pos) = &code.position {
                    if *last_pos < pos.start.offset {
                        let text = &content[*last_pos..pos.start.offset];
                        if !text.trim().is_empty() {
                            // Use the same HTML-enabled options here
                            let mut options = markdown::Options::gfm();
                            // Compile options to control HTML rendering
                            options.compile = markdown::CompileOptions {
                                allow_dangerous_html: true,
                                allow_dangerous_protocol: true,
                                ..markdown::CompileOptions::default()
                            };

                            options.parse = markdown::ParseOptions {
                                // constructs: markdown::Constructs {
                                //     html_flow: true,
                                //     html_text: true,
                                //     ..markdown::Constructs::gfm()
                                // },
                                ..markdown::ParseOptions::gfm()
                            };
                            let temp_html = to_html_with_options(text, &options)
                                .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))?;
                            println!("temp_html: {}", temp_html);
                            parts.push(temp_html);
                        }
                    }
                    
                    // Process code block with syntax highlighting
                    let highlighted = process_code_block(&code.value, code.lang.as_deref(), ss)?;
                    parts.push(highlighted);
                    
                    *last_pos = pos.end.offset;
                }
            },
            _ => {
                // Process children recursively
                if let Some(children) = node.children() {
                    for child in children {
                        process_node(child, ss, content, parts, last_pos)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    // Process the AST
    process_node(&ast, ss, content, &mut parts, &mut last_pos)?;
    
    // Add any remaining content
    if last_pos < content.len() {
        let remaining = &content[last_pos..];
        if !remaining.trim().is_empty() {
            let mut options = markdown::Options::gfm();
            options.compile = markdown::CompileOptions {
                allow_dangerous_html: true,
                allow_dangerous_protocol: true,
                ..markdown::CompileOptions::default()
            };
            parts.push(to_html_with_options(remaining, &options)
                .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))?);
        }
    }
    
    Ok(parts.join(""))
}