//! Build pipeline: `collect` → `preprocess` → `render` → `index`.
//!
//! Increment A extracts the stages from the former monolithic `build_sync_impl_sync`
//! with zero behaviour change. Later increments target a single stage.

pub mod preprocess;

use anyhow::{Context, Result};
use jiff::Zoned;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::config::BookConfig;
use crate::core::{Args, PageInfo};
use crate::pipeline::preprocess::{preprocess, PreprocessCtx};
use crate::render::html::{
    copy_static_assets, extract_title, init_tera, render_index, render_page, write_syntax_css,
    Section,
};
use crate::render::markdown::render_markdown;

/// Collected book structure before rendering (directory-walk mode).
///
/// Increment B replaces this with the `Book` model when `SUMMARY.md` is present.
#[derive(Debug, Clone)]
pub struct CollectedPages {
    /// Markdown source entries in path-sorted order (prev/next basis today).
    pub entries: Vec<DirEntry>,
    /// Flat page list in the same order as `entries`.
    pub all_pages: Vec<PageInfo>,
    /// Sidebar sections derived from parent directories.
    pub sections: Vec<Section>,
}

/// Run the synchronous pipeline stages: collect → render (with preprocess).
///
/// Indexing stays in the async wrapper in `core` when the search feature is on.
pub fn run_sync(args: &Args, config: &BookConfig, watch_enabled: bool) -> Result<()> {
    let tera = init_tera(config)?;
    fs::create_dir_all(&args.output)?;
    copy_static_assets(&args.output, &config.paths.templates, config)?;

    let collected = collect(&args.input)?;
    println!("Total pages: {}", collected.all_pages.len());

    let current_year = Zoned::now().year().to_string();

    #[cfg(feature = "syntax-highlighting")]
    let ss = {
        use syntect::parsing::SyntaxSet;
        let ss = SyntaxSet::load_defaults_newlines();
        write_syntax_css(&args.output)?;
        ss
    };

    let preprocess_ctx = PreprocessCtx;

    // Render each chapter
    for (current_page, entry) in collected.entries.iter().enumerate() {
        if !entry.path().extension().is_some_and(|ext| ext == "md") {
            continue;
        }

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
        let preprocessed = preprocess(&markdown_content, &preprocess_ctx)?;

        #[cfg(feature = "syntax-highlighting")]
        let html_content = render_markdown(&preprocessed, config, Some(&ss))?;
        #[cfg(not(feature = "syntax-highlighting"))]
        let html_content = render_markdown(&preprocessed, config, None)?;

        let previous = if current_page > 0 {
            Some(collected.all_pages[current_page - 1].clone())
        } else {
            None
        };

        let next = if current_page + 1 < collected.all_pages.len() {
            Some(collected.all_pages[current_page + 1].clone())
        } else {
            None
        };

        let title = extract_title(&markdown_content).unwrap_or_else(|| {
            entry
                .path()
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Untitled".to_string())
        });

        render_page(
            &tera,
            &html_path,
            title,
            html_content,
            &collected.sections,
            previous,
            next,
            &current_year,
            config,
            &rel_path.with_extension("html").display().to_string(),
            watch_enabled,
        )?;
    }

    // Index page
    let index_page = collected.all_pages.iter().find(|p| p.path == "/index.html");

    let index_content = if index_page.is_some() {
        let index_path = Path::new(&args.input).join("index.md");
        let markdown_content = fs::read_to_string(&index_path)
            .with_context(|| format!("Failed to read index file: {}", index_path.display()))?;
        let preprocessed = preprocess(&markdown_content, &preprocess_ctx)?;
        #[cfg(feature = "syntax-highlighting")]
        let html = render_markdown(&preprocessed, config, Some(&ss))?;
        #[cfg(not(feature = "syntax-highlighting"))]
        let html = render_markdown(&preprocessed, config, None)?;
        Some(html)
    } else {
        None
    };

    render_index(
        &tera,
        &args.output,
        index_page,
        index_content,
        &collected.sections,
        &current_year,
        config,
    )?;

    #[cfg(not(all(feature = "search", feature = "tokio")))]
    {
        println!("Skipping search indexing (search or tokio feature not enabled)");
    }

    Ok(())
}

/// Collect markdown pages by walking `input_dir`, sorting by path, grouping by parent.
///
/// This is today's directory-walk behaviour. Increment B moves it to
/// `book::directory` and selects on `SUMMARY.md` presence.
pub fn collect(input_dir: &str) -> Result<CollectedPages> {
    let mut all_pages = Vec::new();
    let mut section_map: BTreeMap<String, Vec<PageInfo>> = BTreeMap::new();
    let mut root_pages: Vec<PageInfo> = Vec::new();

    let mut entries: Vec<_> = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();

    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in &entries {
        let rel_path = entry.path().strip_prefix(input_dir)?;
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

    Ok(CollectedPages {
        entries,
        all_pages,
        sections,
    })
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
