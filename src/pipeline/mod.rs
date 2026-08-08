//! Build pipeline: `collect` → `preprocess` → `render` → `index`.

pub mod preprocess;

use anyhow::{Context, Result};
use jiff::Zoned;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::book::flatten_title;
use crate::book::{load_book, Book};
use crate::config::BookConfig;
use crate::core::{Args, PageInfo};
use crate::pipeline::preprocess::{preprocess, PreprocessCtx};
use crate::render::html::{
    copy_static_assets, init_tera, render_index, render_page, write_syntax_css, Section,
};
use crate::render::markdown::render_markdown;

/// Run the synchronous pipeline stages.
pub fn run_sync(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
    let tera = init_tera(config)?;
    fs::create_dir_all(&args.output)?;
    copy_static_assets(&args.output, &config.paths.templates, config)?;

    let src_dir = Path::new(&args.input);
    let create_missing = config.build.create_missing;
    let (book, _created) = load_book(src_dir, create_missing)?;

    let chapters: Vec<&crate::book::Chapter> = book.iter_chapters().collect();
    println!("Total pages: {}", chapters.len());

    // Warn about orphan .md files when using SUMMARY
    if book.from_summary {
        warn_orphan_markdown(src_dir, &book)?;
        // Copy non-markdown assets through
        copy_non_markdown_assets(src_dir, Path::new(&args.output))?;
    }

    let sections = if book.from_summary {
        book.to_legacy_sections()
    } else {
        legacy_directory_sections(&book)
    };

    let current_year = Zoned::now().year().to_string();
    let no_section_label = config.output.html.no_section_label;

    #[cfg(feature = "syntax-highlighting")]
    let ss = {
        use syntect::parsing::SyntaxSet;
        let ss = SyntaxSet::load_defaults_newlines();
        write_syntax_css(&args.output)?;
        ss
    };

    let preprocess_ctx = PreprocessCtx;

    // Prev/next over page-producing chapters only
    for (idx, chapter) in chapters.iter().enumerate() {
        let source_rel = chapter
            .source_path
            .as_ref()
            .expect("iter_chapters only yields chapters with source");
        let source_abs = src_dir.join(source_rel);
        let output_rel = chapter
            .output_path
            .as_ref()
            .expect("source chapter has output path");
        let html_path = Path::new(&args.output).join(output_rel);

        if let Some(parent) = html_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let markdown_content = fs::read_to_string(&source_abs)
            .with_context(|| format!("Failed to read {}", source_abs.display()))?;
        let preprocessed = preprocess(&markdown_content, &preprocess_ctx)?;

        #[cfg(feature = "syntax-highlighting")]
        let html_content =
            crate::render::inject_heading_ids(&render_markdown(&preprocessed, config, Some(&ss))?);
        #[cfg(not(feature = "syntax-highlighting"))]
        let html_content =
            crate::render::inject_heading_ids(&render_markdown(&preprocessed, config, None)?);

        let previous = if idx > 0 {
            Some(chapter_to_pageinfo(chapters[idx - 1]))
        } else {
            None
        };
        let next = if idx + 1 < chapters.len() {
            Some(chapter_to_pageinfo(chapters[idx + 1]))
        } else {
            None
        };

        // SUMMARY link text wins as title
        let title = flatten_title(&chapter.name);

        let nav = book.to_nav(output_rel, no_section_label);

        render_page(
            &tera,
            &html_path.to_string_lossy(),
            title,
            html_content,
            &sections,
            previous,
            next,
            &current_year,
            config,
            &output_rel.display().to_string(),
            watch_enabled,
            Some(&nav),
        )?;
    }

    // Index page: prefer a chapter whose output is index.html
    let index_chapter = chapters.iter().find(|c| {
        c.output_path
            .as_ref()
            .map(|p| p.as_os_str() == "index.html")
            .unwrap_or(false)
    });

    let (index_page_info, index_content) = if let Some(ch) = index_chapter {
        let source_abs = src_dir.join(ch.source_path.as_ref().unwrap());
        let markdown_content = fs::read_to_string(&source_abs)?;
        let preprocessed = preprocess(&markdown_content, &preprocess_ctx)?;
        #[cfg(feature = "syntax-highlighting")]
        let html =
            crate::render::inject_heading_ids(&render_markdown(&preprocessed, config, Some(&ss))?);
        #[cfg(not(feature = "syntax-highlighting"))]
        let html =
            crate::render::inject_heading_ids(&render_markdown(&preprocessed, config, None)?);
        (Some(chapter_to_pageinfo(ch)), Some(html))
    } else {
        // Fallback: look for index.md even if not in book (directory edge case)
        let index_path = src_dir.join("index.md");
        if index_path.exists() {
            let markdown_content = fs::read_to_string(&index_path)?;
            let preprocessed = preprocess(&markdown_content, &preprocess_ctx)?;
            #[cfg(feature = "syntax-highlighting")]
            let html = crate::render::inject_heading_ids(&render_markdown(
                &preprocessed,
                config,
                Some(&ss),
            )?);
            #[cfg(not(feature = "syntax-highlighting"))]
            let html =
                crate::render::inject_heading_ids(&render_markdown(&preprocessed, config, None)?);
            let info = PageInfo {
                title: "Documentation".into(),
                path: "/index.html".into(),
            };
            (Some(info), Some(html))
        } else {
            (None, None)
        }
    };

    render_index(
        &tera,
        &args.output,
        index_page_info.as_ref(),
        index_content,
        &sections,
        &current_year,
        config,
        Some(&book.to_nav(Path::new("index.html"), no_section_label)),
    )?;

    // 404 page
    {
        let mut ctx = tera::Context::new();
        ctx.insert("config", &config);
        ctx.insert("path_to_root", &"");
        ctx.insert("year", &current_year);
        match tera.render("404", &ctx) {
            Ok(html) => {
                if let Err(e) = fs::write(format!("{}/404.html", args.output), html) {
                    eprintln!("warning: failed to write 404.html: {e}");
                }
            }
            Err(e) => eprintln!("warning: failed to render 404 template: {e}"),
        }
    }

    // Redirect stubs (path-contained + HTML-escaped)
    let out_root = Path::new(&args.output);
    for (from, to) in &config.output.html.redirect {
        let rel = from.trim_start_matches('/');
        // Reject absolute / parent-escaping redirect sources
        let rel_path = Path::new(rel);
        if rel_path.is_absolute()
            || rel_path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            eprintln!("warning: ignoring unsafe redirect source path: {from}");
            continue;
        }
        let dest = out_root.join(rel_path);
        // Containment check after join
        let dest_canon_parent = dest.parent().and_then(|p| {
            let _ = fs::create_dir_all(p);
            p.canonicalize().ok()
        });
        let out_canon = out_root.canonicalize().ok();
        if let (Some(parent), Some(root)) = (dest_canon_parent, out_canon) {
            if !parent.starts_with(&root) {
                eprintln!("warning: redirect escapes build dir, skipped: {from}");
                continue;
            }
        }
        // Only allow relative, root-relative, or http(s) targets
        let to_safe = to.trim();
        let to_ok = to_safe.starts_with('/')
            || to_safe.starts_with("./")
            || to_safe.starts_with("../")
            || to_safe.starts_with("http://")
            || to_safe.starts_with("https://")
            || (!to_safe.contains(':') && !to_safe.is_empty());
        if !to_ok || to_safe.contains('\n') || to_safe.contains('"') || to_safe.contains('<') {
            eprintln!("warning: ignoring unsafe redirect target: {to}");
            continue;
        }
        let escaped = html_escape::encode_double_quoted_attribute(to_safe);
        let body = format!(
            r#"<!DOCTYPE html><html><head><meta http-equiv="refresh" content="0; url={escaped}"><link rel="canonical" href="{escaped}"></head><body><a href="{escaped}">Redirect</a></body></html>"#
        );
        if let Err(e) = fs::write(&dest, body) {
            eprintln!("warning: failed to write redirect {from}: {e}");
        }
    }

    // Print page (all chapters concatenated)
    if config.output.html.print.enable {
        #[derive(serde::Serialize)]
        struct PrintChapter {
            title: String,
            content: String,
            anchor: String,
        }
        let mut print_chapters = Vec::new();
        for ch in &chapters {
            let source_abs = src_dir.join(ch.source_path.as_ref().unwrap());
            if let Ok(md) = fs::read_to_string(&source_abs) {
                if let Ok(pre) = preprocess(&md, &preprocess_ctx) {
                    #[cfg(feature = "syntax-highlighting")]
                    let html = crate::render::inject_heading_ids(
                        &render_markdown(&pre, config, Some(&ss)).unwrap_or_default(),
                    );
                    #[cfg(not(feature = "syntax-highlighting"))]
                    let html = crate::render::inject_heading_ids(
                        &render_markdown(&pre, config, None).unwrap_or_default(),
                    );
                    print_chapters.push(PrintChapter {
                        title: flatten_title(&ch.name),
                        content: html,
                        anchor: ch
                            .output_path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default(),
                    });
                }
            }
        }
        let mut ctx = tera::Context::new();
        ctx.insert("config", &config);
        ctx.insert("path_to_root", &"");
        ctx.insert("print_chapters", &print_chapters);
        ctx.insert("page_break", &config.output.html.print.page_break);
        if let Ok(html) = tera.render("print", &ctx) {
            let _ = fs::write(format!("{}/print.html", args.output), html);
        }
    }

    #[cfg(not(all(feature = "search", feature = "tokio")))]
    {
        println!("Skipping search indexing (search or tokio feature not enabled)");
    }

    Ok(())
}

fn chapter_to_pageinfo(ch: &crate::book::Chapter) -> PageInfo {
    PageInfo {
        title: flatten_title(&ch.name),
        path: format!(
            "/{}",
            ch.output_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        ),
    }
}

/// Directory-mode sections: root → "Guide", other parents → dir name (old behaviour).
fn legacy_directory_sections(book: &Book) -> Vec<Section> {
    use std::collections::BTreeMap;
    let mut section_map: BTreeMap<String, Vec<PageInfo>> = BTreeMap::new();
    let mut root_pages = Vec::new();

    for ch in book.iter_chapters() {
        let out = match &ch.output_path {
            Some(p) => p,
            None => continue,
        };
        let info = chapter_to_pageinfo(ch);
        let parent = ch
            .source_path
            .as_ref()
            .and_then(|p| p.parent())
            .and_then(|p| p.to_str())
            .unwrap_or("");
        if parent.is_empty() {
            root_pages.push(info);
        } else {
            section_map
                .entry(parent.to_string())
                .or_default()
                .push(info);
        }
        let _ = out;
    }

    let mut sections = Vec::new();
    if !root_pages.is_empty() {
        sections.push(Section {
            title: "Guide".to_string(),
            pages: root_pages,
        });
    }
    for (title, pages) in section_map {
        sections.push(Section { title, pages });
    }
    sections
}

fn warn_orphan_markdown(src_dir: &Path, book: &Book) -> Result<()> {
    use std::collections::HashSet;
    let listed: HashSet<PathBuf> = book
        .iter_all_chapters()
        .filter_map(|c| c.source_path.clone())
        .collect();

    for entry in WalkDir::new(src_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let rel = path.strip_prefix(src_dir)?;
        if rel == Path::new("SUMMARY.md") {
            continue;
        }
        if !listed.contains(rel) {
            eprintln!(
                "warning: markdown file not in SUMMARY.md (not published): {}",
                rel.display()
            );
        }
    }
    Ok(())
}

fn copy_non_markdown_assets(src_dir: &Path, out_dir: &Path) -> Result<()> {
    for entry in WalkDir::new(src_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            continue;
        }
        let rel = path.strip_prefix(src_dir)?;
        let dest = out_dir.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(path, &dest)
            .with_context(|| format!("Failed to copy asset {}", path.display()))?;
    }
    Ok(())
}

/// Run Pagefind indexing over the output directory (async path only).
#[cfg(all(feature = "search", feature = "tokio"))]
pub async fn index(output: &str) -> Result<()> {
    use crate::pagefind_service::PagefindBuilder;

    match PagefindBuilder::new(PathBuf::from(output)).await {
        Ok(pagefind) => {
            if let Err(e) = pagefind.build().await {
                eprintln!("Search indexing failed: {e}");
            }
        }
        Err(e) => {
            eprintln!("Failed to create search builder: {e}");
        }
    }
    Ok(())
}
