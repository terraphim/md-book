//! HTML page assembly: Tera templates, static assets, page/index writers.

use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;
use tera::{Context as TeraContext, Tera};
use walkdir::WalkDir;

use crate::config::BookConfig;
use crate::core::PageInfo;

#[derive(Serialize, Debug, Clone)]
pub struct PageData {
    pub title: String,
    pub content: String,
    pub sections: Vec<Section>,
    pub previous: Option<PageInfo>,
    pub next: Option<PageInfo>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Section {
    pub title: String,
    pub pages: Vec<PageInfo>,
}

/// Initialise Tera from the configured templates directory, falling back to embedded defaults.
pub fn init_tera(config: &BookConfig) -> Result<Tera> {
    let mut tera = Tera::default();

    // Names end in .html so Tera autoescapes them. Registered as bare "page",
    // "index", … nothing was escaped anywhere, and authored SUMMARY.md text
    // reached the sidebar and <title> as raw markup.
    let template_files = [
        ("page.html", "page.html.tera"),
        ("index.html", "index.html.tera"),
        ("sidebar.html", "sidebar.html.tera"),
        ("footer.html", "footer.html.tera"),
        ("header.html", "header.html.tera"),
        ("404.html", "404.html.tera"),
        ("print.html", "print.html.tera"),
    ];

    for (name, file) in template_files {
        let template_path = format!("{}/{}", config.paths.templates, file);
        let template_content = if Path::new(&template_path).exists() {
            fs::read_to_string(&template_path)
                .with_context(|| format!("Failed to read template: {template_path}"))?
        } else {
            match file {
                "page.html.tera" => include_str!("../templates/page.html.tera").to_string(),
                "index.html.tera" => include_str!("../templates/index.html.tera").to_string(),
                "sidebar.html.tera" => include_str!("../templates/sidebar.html.tera").to_string(),
                "footer.html.tera" => include_str!("../templates/footer.html.tera").to_string(),
                "header.html.tera" => include_str!("../templates/header.html.tera").to_string(),
                "404.html.tera" => include_str!("../templates/404.html.tera").to_string(),
                "print.html.tera" => include_str!("../templates/print.html.tera").to_string(),
                _ => return Err(anyhow::anyhow!("Unknown template file: {}", file)),
            }
        };

        tera.add_raw_template(name, &template_content)
            .with_context(|| format!("Failed to add template: {name}"))?;
    }

    Ok(tera)
}

/// Extract the first H1 title from markdown, if any.
pub fn extract_title(markdown: &str) -> Option<String> {
    markdown
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line[2..].trim().to_string())
}

/// `../`-style prefix from a page to the build-dir root; `""` at the root.
/// Resolve the configured logo to a URL for a page at `root` depth.
///
/// External logos pass through; local ones become relative to the page. Built
/// here rather than in the template so the value can be escaped once, as an
/// attribute, without Tera also rewriting `/` as `&#x2F;`.
#[must_use]
pub fn logo_url(logo: &str, root: &str) -> String {
    let url = if logo.starts_with("http://") || logo.starts_with("https://") {
        logo.to_string()
    } else {
        format!("{root}{}", logo.trim_start_matches('/'))
    };
    escape_url_attr(&url)
}

/// Resolve the "edit this page" URL for a chapter, or `None` when unconfigured.
#[must_use]
pub fn edit_url(base: Option<&str>, current_path: &str) -> Option<String> {
    let base = base?;
    let source = current_path
        .strip_suffix(".html")
        .map_or_else(|| current_path.to_string(), |stem| format!("{stem}.md"));
    Some(escape_url_attr(&format!("{base}{source}")))
}

/// Escape a URL for use inside a double-quoted HTML attribute.
///
/// Escapes `&` and `"` only, leaving `/` readable. Without this a chapter whose
/// SUMMARY.md target contains a quote could close the attribute and inject
/// event handlers.
#[must_use]
pub fn escape_url_attr(url: &str) -> String {
    html_escape::encode_double_quoted_attribute(url).into_owned()
}

/// Render a relative path as a URL path.
///
/// `Path::display()` uses the platform separator, so on Windows it yields
/// `individual\heading.html` -- a broken href in generated HTML. URLs always
/// use `/`, whatever built the book.
#[must_use]
pub fn to_url_path(path: &Path) -> String {
    path.components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn path_to_root(page: &Path) -> String {
    let depth = page.components().count().saturating_sub(1);
    if depth == 0 {
        String::new()
    } else {
        "../".repeat(depth)
    }
}

#[cfg(test)]
mod path_to_root_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_path_to_root_depths() {
        assert_eq!(path_to_root(Path::new("index.html")), "");
        assert_eq!(path_to_root(Path::new("a.html")), "");
        assert_eq!(path_to_root(Path::new("dir/a.html")), "../");
        assert_eq!(path_to_root(Path::new("a/b/c.html")), "../../");
    }
}

/// Write a single chapter HTML page.
#[allow(clippy::too_many_arguments)]
pub fn render_page(
    tera: &Tera,
    html_path: &str,
    title: String,
    content: String,
    sections: &[Section],
    previous: Option<PageInfo>,
    next: Option<PageInfo>,
    year: &str,
    config: &BookConfig,
    current_path: &str,
    watch_enabled: bool,
    chapters: Option<&[crate::book::NavEntry]>,
    has_mermaid: bool,
    additional: &crate::pipeline::AdditionalAssets,
) -> Result<()> {
    let page_data = PageData {
        title,
        content,
        sections: sections.to_vec(),
        previous,
        next,
    };

    let mut context = TeraContext::new();
    context.insert("year", &year);
    context.insert("page", &page_data);
    context.insert("config", &config);
    context.insert("current_path", &current_path);
    context.insert("watch_enabled", &watch_enabled);
    if let Some(nav) = chapters {
        context.insert("chapters", &nav);
    }
    let root = path_to_root(Path::new(current_path));
    context.insert("path_to_root", &root);
    // Gates the mermaid bundle: 2.9MB that most pages never need.
    context.insert("has_mermaid", &has_mermaid);
    context.insert("logo_url", &logo_url(&config.book.logo, &root));
    context.insert("additional_css", &additional.css);
    context.insert("additional_js", &additional.js);
    context.insert(
        "edit_url",
        &edit_url(config.book.github_edit_url_base.as_deref(), current_path),
    );
    context.insert("default_theme", &config.output.html.default_theme_name());
    context.insert(
        "preferred_dark_theme",
        &config.output.html.preferred_dark_theme_name(),
    );

    let rendered = tera
        .render("page.html", &context)
        .with_context(|| format!("Failed to render page: {}", html_path))?;
    fs::write(html_path, rendered)
        .with_context(|| format!("Failed to write file: {}", html_path))?;
    Ok(())
}

/// Write `index.html` for the book.
#[allow(clippy::too_many_arguments)]
pub fn render_index(
    tera: &Tera,
    output_dir: &str,
    index_page: Option<&PageInfo>,
    index_content: Option<String>,
    sections: &[Section],
    year: &str,
    config: &BookConfig,
    chapters: Option<&[crate::book::NavEntry]>,
    has_mermaid: bool,
    additional: &crate::pipeline::AdditionalAssets,
) -> Result<()> {
    let mut context = TeraContext::new();
    context.insert("year", &year);
    context.insert("config", &config);
    context.insert("sections", &sections);
    context.insert("has_mermaid", &has_mermaid);
    context.insert("logo_url", &logo_url(&config.book.logo, ""));
    context.insert("additional_css", &additional.css);
    context.insert("additional_js", &additional.js);
    context.insert(
        "edit_url",
        &edit_url(config.book.github_edit_url_base.as_deref(), "index.html"),
    );
    context.insert("default_theme", &config.output.html.default_theme_name());
    context.insert(
        "preferred_dark_theme",
        &config.output.html.preferred_dark_theme_name(),
    );
    context.insert("current_path", &"index.html");
    if let Some(nav) = chapters {
        context.insert("chapters", &nav);
    }
    context.insert("path_to_root", &"");

    if let (Some(index), Some(html_content)) = (index_page, index_content) {
        context.insert("has_index", &true);
        context.insert("title", &index.title);
        context.insert("content", &html_content);
    } else {
        context.insert("has_index", &false);
        context.insert("title", &"Documentation");
    }

    let rendered = tera
        .render("index.html", &context)
        .context("Failed to render index page")?;
    fs::write(format!("{}/index.html", output_dir), rendered)
        .context("Failed to write index.html")?;
    Ok(())
}

/// Emit syntect theme CSS into the output directory.
#[cfg(feature = "syntax-highlighting")]
pub fn write_syntax_css(output_dir: &str, config: &BookConfig) -> Result<()> {
    use syntect::highlighting::ThemeSet;
    use syntect::html::ClassStyle;

    let ts = ThemeSet::load_defaults();

    let pick = |configured: Option<&str>, fallback: &str| -> String {
        match configured {
            Some(name) if ts.themes.contains_key(name) => name.to_string(),
            Some(name) => {
                let mut available: Vec<&str> = ts.themes.keys().map(String::as_str).collect();
                available.sort_unstable();
                eprintln!(
                    "warning: unknown syntax theme '{name}', using '{fallback}'. Available: {}",
                    available.join(", ")
                );
                fallback.to_string()
            }
            None => fallback.to_string(),
        }
    };

    let light = pick(
        config.output.html.syntax_theme.as_deref(),
        "Solarized (light)",
    );
    let dark = pick(
        config.output.html.syntax_theme_dark.as_deref(),
        "base16-ocean.dark",
    );

    let css_for = |name: &str| -> Result<String> {
        syntect::html::css_for_theme_with_class_style(&ts.themes[name], ClassStyle::Spaced)
            .map_err(|e| anyhow::anyhow!("CSS generation error: {e:?}"))
    };

    // The dark rules are scoped to the dark themes so a single stylesheet
    // serves every theme the picker offers.
    let dark_scopes = ["coal", "navy", "ayu"];
    let mut css = css_for(&light)?;
    css.push_str("\n\n/* Dark themes */\n");
    css.push_str(&scope_css(&css_for(&dark)?, &dark_scopes));

    fs::write(format!("{}/css/syntax.css", output_dir), css)?;
    Ok(())
}

/// Prefix every selector in `css` with each `[data-theme="…"]` scope.
///
/// syntect emits flat rules, so without this the dark theme would override the
/// light one for every reader regardless of their choice.
#[must_use]
pub fn scope_css(css: &str, themes: &[&str]) -> String {
    // Comments sit outside any rule, so they would otherwise be parsed as part
    // of the next selector.
    let mut stripped = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        stripped.push_str(&rest[..start]);
        match rest[start..].find("*/") {
            Some(end) => rest = &rest[start + end + 2..],
            None => {
                rest = "";
                break;
            }
        }
    }
    stripped.push_str(rest);

    let mut out = String::with_capacity(stripped.len() * 2);

    for block in stripped.split_inclusive('}') {
        let Some((selectors, body)) = block.split_once('{') else {
            // Trailing whitespace after the last rule.
            continue;
        };

        let scoped: Vec<String> = selectors
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .flat_map(|selector| {
                themes.iter().map(move |theme| {
                    if selector == ":root" || selector.eq_ignore_ascii_case("html") {
                        format!("[data-theme=\"{theme}\"]")
                    } else {
                        format!("[data-theme=\"{theme}\"] {selector}")
                    }
                })
            })
            .collect();

        if scoped.is_empty() {
            continue;
        }
        out.push_str(&scoped.join(",\n"));
        out.push_str(" {");
        out.push_str(body);
    }

    out
}

#[cfg(not(feature = "syntax-highlighting"))]
pub fn write_syntax_css(_output_dir: &str, _config: &BookConfig) -> Result<()> {
    Ok(())
}

/// Default templates, embedded at compile time.
///
/// Everything a book needs to render is baked into the binary, so an installed
/// md-book produces a styled, offline-capable book with no templates directory
/// on disk. A templates directory, when present, is copied over the top and
/// wins per file.
static DEFAULT_CSS: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/templates/css");
static DEFAULT_JS: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/templates/js");
static DEFAULT_IMG: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/templates/img");
static DEFAULT_COMPONENTS: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/templates/components");
/// Vendored third-party assets (local Shoelace build).
static VENDOR_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/templates/vendor");

/// Write an embedded directory into `output_dir/prefix`, preserving structure.
fn write_embedded(dir: &include_dir::Dir<'_>, output_dir: &Path, prefix: &str) -> Result<()> {
    for file in dir.files() {
        let dest = output_dir.join(prefix).join(file.path());
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, file.contents())
            .with_context(|| format!("Failed to write {}", dest.display()))?;
    }
    for sub in dir.dirs() {
        write_embedded(sub, output_dir, prefix)?;
    }
    Ok(())
}

/// Copy a template subdirectory over the embedded defaults, per file.
///
/// A missing source is not an error: most books ship no templates at all.
fn copy_tree(templates_dir: &str, output_dir: &str, name: &str) -> Result<()> {
    let source = format!("{templates_dir}/{name}");
    let dest = Path::new(output_dir).join(name);
    fs::create_dir_all(&dest)?;
    if !Path::new(&source).exists() {
        return Ok(());
    }

    for entry in WalkDir::new(&source) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(&source)?;
        let dest_path = dest.join(rel);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(entry.path(), &dest_path)
            .with_context(|| format!("Failed to copy {}", entry.path().display()))?;
    }
    Ok(())
}

/// Write static assets (CSS, JS, images, web components, vendor) into the output.
pub fn copy_static_assets(
    output_dir: &str,
    templates_dir: &str,
    _config: &BookConfig,
) -> Result<()> {
    let out = Path::new(output_dir);

    // Embedded defaults first...
    write_embedded(&DEFAULT_CSS, out, "css")?;
    write_embedded(&DEFAULT_JS, out, "js")?;
    write_embedded(&DEFAULT_IMG, out, "img")?;
    write_embedded(&DEFAULT_COMPONENTS, out, "components")?;
    write_embedded(&VENDOR_DIR, out, "vendor")?;

    // ...then a user templates directory overrides them file by file.
    for tree in ["css", "js", "img", "components"] {
        copy_tree(templates_dir, output_dir, tree)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BookConfig;
    use tempfile::TempDir;

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

    #[test]
    fn test_copy_static_assets() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_dir = temp_dir.path().join("output");
        let templates_dir = project_root().join("src/templates");

        fs::create_dir_all(&output_dir)?;

        let config = BookConfig::default();
        copy_static_assets(
            output_dir.to_str().unwrap(),
            templates_dir.to_str().unwrap(),
            &config,
        )?;

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

        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod url_path_tests {
    use super::to_url_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_to_url_path_uses_forward_slashes() {
        // Path::display() would emit backslashes on Windows, producing hrefs
        // like `individual\heading.html` in generated HTML.
        let nested: PathBuf = ["individual", "heading.html"].iter().collect();
        assert_eq!(to_url_path(&nested), "individual/heading.html");

        let deep: PathBuf = ["a", "b", "c.html"].iter().collect();
        assert_eq!(to_url_path(&deep), "a/b/c.html");
    }

    #[test]
    fn test_to_url_path_flat_and_empty() {
        assert_eq!(to_url_path(Path::new("index.html")), "index.html");
        assert_eq!(to_url_path(Path::new("")), "");
    }
}

#[cfg(all(test, feature = "syntax-highlighting"))]
mod scope_css_tests {
    use super::scope_css;

    #[test]
    fn test_scope_css_prefixes_each_selector_for_each_theme() {
        let css = ".comment { color: red; }\n.string { color: blue; }";
        let scoped = scope_css(css, &["coal", "navy"]);

        assert!(scoped.contains("[data-theme=\"coal\"] .comment"));
        assert!(scoped.contains("[data-theme=\"navy\"] .comment"));
        assert!(scoped.contains("[data-theme=\"coal\"] .string"));
        // Unscoped rules would override the light theme for every reader.
        assert!(!scoped.contains("\n.comment"));
    }

    #[test]
    fn test_scope_css_rewrites_root_rather_than_nesting_it() {
        let scoped = scope_css(":root { background: #000; }", &["coal"]);
        assert!(scoped.contains("[data-theme=\"coal\"] {"));
        assert!(!scoped.contains("[data-theme=\"coal\"] :root"));
    }

    #[test]
    fn test_scope_css_drops_comments() {
        // syntect prefixes its output with a comment; treating it as a selector
        // produced `[data-theme="coal"] /* theme … */`.
        let scoped = scope_css("/* theme \"X\" generated */\n.a { color: red; }", &["coal"]);
        assert!(!scoped.contains("/*"));
        assert!(scoped.contains("[data-theme=\"coal\"] .a"));
    }

    #[test]
    fn test_scope_css_handles_comma_separated_selectors() {
        let scoped = scope_css(".a, .b { color: red; }", &["ayu"]);
        assert!(scoped.contains("[data-theme=\"ayu\"] .a"));
        assert!(scoped.contains("[data-theme=\"ayu\"] .b"));
    }
}
