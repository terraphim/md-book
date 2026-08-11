use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use twelf::{config, Layer};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum MarkdownFormat {
    #[default]
    Markdown,
    Gfm,
    Mdx,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkdownInput {
    #[serde(default)]
    pub format: MarkdownFormat,
    #[serde(default)]
    pub frontmatter: bool,
}

#[config]
#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct BookConfig {
    #[serde(default)]
    pub book: Book,
    #[serde(default)]
    pub rust: Rust,
    #[serde(default)]
    pub output: Output,
    #[serde(default)]
    pub markdown: MarkdownInput,
    #[serde(default)]
    pub paths: Paths,
    #[serde(default)]
    pub build: Build,
}

#[config]
#[derive(Debug, serde::Serialize, Clone)]
pub struct Book {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default = "default_logo")]
    pub logo: String,
    #[serde(default)]
    pub github_url: Option<String>,
    #[serde(default)]
    pub github_edit_url_base: Option<String>,
    /// Source directory relative to book root (mdBook `book.src`).
    #[serde(default)]
    pub src: Option<String>,
}

impl Default for Book {
    fn default() -> Self {
        Self {
            title: default_title(),
            description: None,
            authors: Vec::new(),
            language: default_language(),
            base_url: None,
            logo: default_logo(),
            github_url: None,
            github_edit_url_base: None,
            src: None,
        }
    }
}

fn default_title() -> String {
    "My Book".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

fn default_logo() -> String {
    "/img/default_logo.svg".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rust {
    #[serde(default = "default_edition")]
    pub edition: String,
}

impl Default for Rust {
    fn default() -> Self {
        Self {
            edition: default_edition(),
        }
    }
}

fn default_edition() -> String {
    "2021".to_string()
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Output {
    pub html: HtmlOutput,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct HtmlOutput {
    #[serde(default)]
    pub mathjax_support: bool,
    #[serde(default)]
    pub allow_html: bool,
    #[serde(default)]
    pub playground: PlaygroundConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub default_theme: Option<String>,
    #[serde(default)]
    pub preferred_dark_theme: Option<String>,
    #[serde(default)]
    pub additional_css: Vec<String>,
    #[serde(default)]
    pub additional_js: Vec<String>,
    #[serde(default)]
    pub input_404: Option<String>,
    #[serde(default)]
    pub site_url: Option<String>,
    #[serde(default)]
    pub no_section_label: bool,
    #[serde(default)]
    pub syntax_theme: Option<String>,
    /// mdBook's spelling of `book.github_url`.
    #[serde(default)]
    pub git_repository_url: Option<String>,
    /// mdBook's edit link, with a `{path}` placeholder for the source file.
    #[serde(default)]
    pub edit_url_template: Option<String>,
    /// Syntect theme used under the dark themes (coal, navy, ayu).
    #[serde(default)]
    pub syntax_theme_dark: Option<String>,
    #[serde(default)]
    pub fold: FoldConfig,
    #[serde(default)]
    pub print: PrintConfig,
    #[serde(default)]
    pub redirect: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct FoldConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default = "default_fold_level")]
    pub level: u32,
}

fn default_fold_level() -> u32 {
    0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PrintConfig {
    #[serde(default = "default_true")]
    pub enable: bool,
    #[serde(default = "default_true")]
    pub page_break: bool,
}

fn default_true() -> bool {
    true
}

impl Default for PrintConfig {
    fn default() -> Self {
        Self {
            enable: true,
            page_break: true,
        }
    }
}

impl HtmlOutput {
    /// Repository URL, accepting mdBook's `git-repository-url` as well as
    /// md-book's own `book.github_url`.
    #[must_use]
    pub fn repository_url<'a>(&'a self, book: &'a Book) -> Option<&'a str> {
        book.github_url
            .as_deref()
            .or(self.git_repository_url.as_deref())
    }

    /// Theme applied when the reader has expressed no preference.
    #[must_use]
    pub fn default_theme_name(&self) -> &str {
        self.default_theme.as_deref().unwrap_or("light")
    }

    /// Theme applied when the OS reports a dark colour-scheme preference.
    #[must_use]
    pub fn preferred_dark_theme_name(&self) -> &str {
        self.preferred_dark_theme.as_deref().unwrap_or("navy")
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PlaygroundConfig {
    #[serde(default)]
    pub editable: bool,
    #[serde(default)]
    pub line_numbers: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct SearchConfig {
    #[serde(default = "default_limit_results")]
    pub limit_results: u32,
    #[serde(default)]
    pub use_boolean_and: bool,
    #[serde(default = "default_boost_title")]
    pub boost_title: u32,
    #[serde(default = "default_boost_hierarchy")]
    pub boost_hierarchy: u32,
    #[serde(default = "default_boost_paragraph")]
    pub boost_paragraph: u32,
    #[serde(default)]
    pub expand: bool,
    #[serde(default = "default_heading_split_level")]
    pub heading_split_level: u32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            limit_results: default_limit_results(),
            use_boolean_and: false,
            boost_title: default_boost_title(),
            boost_hierarchy: default_boost_hierarchy(),
            boost_paragraph: default_boost_paragraph(),
            expand: false,
            heading_split_level: default_heading_split_level(),
        }
    }
}

const fn default_limit_results() -> u32 {
    20
}
const fn default_boost_title() -> u32 {
    2
}
const fn default_boost_hierarchy() -> u32 {
    2
}
const fn default_boost_paragraph() -> u32 {
    1
}
const fn default_heading_split_level() -> u32 {
    2
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Build {
    /// Output directory relative to book root (mdBook `build.build-dir`).
    #[serde(default)]
    pub build_dir: Option<String>,
    /// Extra directories to watch in watch/serve mode.
    #[serde(default)]
    pub extra_watch_dirs: Vec<String>,
    /// Create missing SUMMARY targets (mdBook default true).
    #[serde(default = "default_create_missing")]
    pub create_missing: bool,
}

fn default_create_missing() -> bool {
    true
}

impl Default for Build {
    fn default() -> Self {
        Self {
            build_dir: None,
            extra_watch_dirs: Vec::new(),
            create_missing: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paths {
    #[serde(default = "default_templates_dir")]
    pub templates: String,
}

impl Default for Paths {
    fn default() -> Self {
        Self {
            templates: default_templates_dir(),
        }
    }
}

fn default_templates_dir() -> String {
    "templates".to_string()
}

/// A configuration key md-book accepts but does not act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedKey {
    /// Dotted path as it appears in `book.toml`.
    pub path: String,
    /// Why it has no effect.
    pub reason: &'static str,
}

/// Keys that parse but change nothing, with the reason to report.
///
/// A prefix entry covers its whole table: `output.html.playground` also reports
/// `output.html.playground.editable`.
const UNSUPPORTED: &[(&str, &str)] = &[
    (
        "output.html.mathjax-support",
        "MathJax rendering is not implemented",
    ),
    (
        "output.html.playground",
        "the Rust Playground runtime is out of scope for md-book",
    ),
    (
        "output.html.search.use-boolean-and",
        "md-book searches with Pagefind, which has no equivalent setting",
    ),
    (
        "output.html.search.boost-title",
        "md-book searches with Pagefind, which has no equivalent setting",
    ),
    (
        "output.html.search.boost-hierarchy",
        "md-book searches with Pagefind, which has no equivalent setting",
    ),
    (
        "output.html.search.boost-paragraph",
        "md-book searches with Pagefind, which has no equivalent setting",
    ),
    (
        "output.html.search.expand",
        "md-book searches with Pagefind, which has no equivalent setting",
    ),
    (
        "output.html.search.teaser-word-count",
        "md-book searches with Pagefind, which has no equivalent setting",
    ),
];

/// Find unsupported keys the author actually set in a config document.
///
/// Works on the parsed document rather than the loaded `BookConfig` because a
/// deserialised default is indistinguishable from a value the author typed --
/// warning on defaults would fire for every book.
#[must_use]
pub fn unsupported_keys_in(doc: &serde_json::Value) -> Vec<UnsupportedKey> {
    let mut found = Vec::new();

    for (path, reason) in UNSUPPORTED {
        let mut cursor = doc;
        let mut present = true;
        for segment in path.split('.') {
            match cursor.get(segment) {
                Some(next) => cursor = next,
                None => {
                    present = false;
                    break;
                }
            }
        }
        if present && !cursor.is_null() {
            found.push(UnsupportedKey {
                path: (*path).to_string(),
                reason,
            });
        }
    }

    found
}

/// Read a config file and report any keys that have no effect.
///
/// Failure to read or parse is silent: the layering itself will surface a
/// genuine problem, and this is only advisory.
fn warn_unsupported_keys(path: &Path) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };

    let doc: Option<serde_json::Value> = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("toml") => toml::from_str(&text).ok(),
        Some(ext) if ext.eq_ignore_ascii_case("json") => serde_json::from_str(&text).ok(),
        _ => None,
    };
    let Some(doc) = doc else { return };

    for key in unsupported_keys_in(&doc) {
        eprintln!(
            "warning: {}: '{}' is ignored -- {}",
            path.display(),
            key.path,
            key.reason
        );
    }
}

/// Load configuration from file or use defaults
///
/// # Errors
///
/// Returns an error if the configuration file cannot be read or parsed
pub fn load_config(config_path: Option<&str>) -> anyhow::Result<BookConfig> {
    load_config_from(None, config_path)
}

/// Load configuration for a book directory.
///
/// Layers, lowest precedence first: `MDBOOK_` environment variables, the
/// book directory's `book.toml` (or the current directory's when `book_dir` is
/// `None`), then an explicit config file. Unset scalars are then filled from the
/// documented defaults, and keys that have no effect are reported.
///
/// # Errors
///
/// Returns an error if a layer cannot be read or parsed, or if `config_path` has
/// an extension other than `.toml` or `.json`.
pub fn load_config_from(
    book_dir: Option<&Path>,
    config_path: Option<&str>,
) -> anyhow::Result<BookConfig> {
    let mut layers = vec![Layer::Env(Some("MDBOOK_".to_string()))];

    let book_toml = book_dir
        .map(|dir| dir.join("book.toml"))
        .unwrap_or_else(|| PathBuf::from("book.toml"));
    if book_toml.exists() {
        warn_unsupported_keys(&book_toml);
        layers.push(Layer::Toml(book_toml));
    }

    if let Some(path) = config_path {
        let path = Path::new(path);
        if path.exists() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            if ext.eq_ignore_ascii_case("toml") {
                warn_unsupported_keys(path);
                layers.push(Layer::Toml(path.to_path_buf()));
            } else if ext.eq_ignore_ascii_case("json") {
                warn_unsupported_keys(path);
                layers.push(Layer::Json(path.to_path_buf()));
            } else {
                anyhow::bail!("Unsupported config file type: {}", path.display());
            }
        }
    }

    let mut config = BookConfig::with_layers(&layers)?;
    fill_unset_with_defaults(&mut config);
    Ok(config)
}

/// Replace unset scalar fields with their documented defaults.
///
/// `#[serde(default = "...")]` does not survive two hops here: twelf's layering
/// yields empty strings for absent keys, and `#[serde(default)]` on a container
/// field constructs it with `Default::default()`, bypassing the per-field
/// defaults inside it. Without this, a book with no `book.toml` renders
/// `<html lang="">`, an empty `<title>` and a broken logo `src=""`.
fn fill_unset_with_defaults(config: &mut BookConfig) {
    let d = BookConfig::default();

    if config.book.title.is_empty() {
        config.book.title = d.book.title;
    }
    if config.book.language.is_empty() {
        config.book.language = d.book.language;
    }
    if config.book.logo.is_empty() {
        config.book.logo = d.book.logo;
    }
    if config.rust.edition.is_empty() {
        config.rust.edition = d.rust.edition;
    }
    if config.paths.templates.is_empty() {
        config.paths.templates = d.paths.templates;
    }

    // Zero is never a meaningful value for these, so it means "absent".
    let search = &mut config.output.html.search;
    let ds = d.output.html.search;
    if search.limit_results == 0 {
        search.limit_results = ds.limit_results;
    }
    if search.heading_split_level == 0 {
        search.heading_split_level = ds.heading_split_level;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to serialize tests that change the current working directory
    // This prevents race conditions when tests run in parallel
    static CWD_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_markdown_format_default() {
        let format = MarkdownFormat::default();
        assert!(matches!(format, MarkdownFormat::Markdown));
    }

    #[test]
    fn test_markdown_format_serde() {
        // Test deserialization from lowercase strings
        let json = r#""gfm""#;
        let format: MarkdownFormat = serde_json::from_str(json).unwrap();
        assert!(matches!(format, MarkdownFormat::Gfm));

        // Test serialization
        let serialized = serde_json::to_string(&MarkdownFormat::Mdx).unwrap();
        assert_eq!(serialized, r#""mdx""#);
    }

    #[test]
    fn test_markdown_input_default() {
        let input = MarkdownInput::default();
        assert!(matches!(input.format, MarkdownFormat::Markdown));
        assert!(!input.frontmatter);
    }

    #[test]
    fn test_book_config_defaults() {
        // Test basic config loading works
        let mut config =
            BookConfig::with_layers(&[Layer::Env(Some("MDBOOK_".to_string()))]).unwrap();
        fill_unset_with_defaults(&mut config);

        // Documented defaults must be real values, not empty strings.
        assert_eq!(config.book.title, "My Book");
        assert_eq!(config.book.language, "en");
        assert_eq!(config.book.logo, "/img/default_logo.svg");
        assert_eq!(config.rust.edition, "2021");
        assert_eq!(config.paths.templates, "templates");
        assert_eq!(config.output.html.search.limit_results, 20);
    }

    #[test]
    fn test_load_config_no_files() -> anyhow::Result<()> {
        // Lock mutex to prevent race conditions with other tests
        let _guard = CWD_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new()?;
        let original_dir = std::env::current_dir()?;

        // Change to temp directory so no book.toml exists
        std::env::set_current_dir(temp_dir.path())?;

        let config = load_config(None)?;

        // Restore original directory
        std::env::set_current_dir(original_dir)?;

        // With no book.toml at all, defaults still apply.
        assert_eq!(config.book.language, "en");
        assert_eq!(config.rust.edition, "2021");
        assert_eq!(config.book.title, "My Book");

        Ok(())
    }

    #[test]
    fn test_load_config_with_book_toml() -> anyhow::Result<()> {
        // Lock mutex to prevent race conditions with other tests that change cwd
        let _guard = CWD_MUTEX.lock().unwrap();

        // Test loading config with a custom book.toml file
        // Use CARGO_MANIFEST_DIR to get absolute path for test reliability
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let book_toml_path =
            std::path::PathBuf::from(&manifest_dir).join("test_book_mdbook/book.toml");
        if !book_toml_path.exists() {
            // Skip test if mdBook test book is not available
            return Ok(());
        }

        let config = load_config(Some(book_toml_path.to_str().unwrap()))?;

        assert_eq!(config.book.title, "mdBook test book");
        assert_eq!(
            config.book.description,
            Some("A demo book to test and validate changes".to_string())
        );
        assert_eq!(config.book.authors, vec!["YJDoc2"]);
        assert_eq!(config.book.language, "en");
        assert_eq!(config.rust.edition, "2018");
        assert_eq!(config.output.html.search.limit_results, 20);
        assert_eq!(config.output.html.search.boost_title, 2);

        Ok(())
    }

    #[test]
    fn test_load_config_with_custom_toml() -> anyhow::Result<()> {
        // load_config resolves a relative "book.toml" against the process CWD,
        // which sibling tests change; serialise against them.
        let _guard = CWD_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new()?;

        let custom_toml_content = r#"
[book]
title = "Custom Config Book"
language = "es"

[markdown]
format = "gfm"
frontmatter = true
"#;

        let custom_toml_path = temp_dir.path().join("custom.toml");
        fs::write(&custom_toml_path, custom_toml_content)?;

        let config = load_config(Some(custom_toml_path.to_str().unwrap()))?;

        assert_eq!(config.book.title, "Custom Config Book");
        assert_eq!(config.book.language, "es");
        assert!(matches!(config.markdown.format, MarkdownFormat::Gfm));
        assert!(config.markdown.frontmatter);

        Ok(())
    }

    #[test]
    fn test_load_config_with_custom_json() -> anyhow::Result<()> {
        // Lock mutex to prevent race conditions with other tests that change cwd
        let _guard = CWD_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new()?;

        let custom_json_content = r#"
{
  "book": {
    "title": "JSON Config Book",
    "language": "de",
    "authors": ["JSON Author"]
  },
  "markdown": {
    "format": "mdx",
    "frontmatter": false
  },
  "output": {
    "html": {
      "mathjax-support": true,
      "search": {
        "limit-results": 100
      }
    }
  }
}
"#;

        let custom_json_path = temp_dir.path().join("custom.json");
        fs::write(&custom_json_path, custom_json_content)?;

        let config = load_config(Some(custom_json_path.to_str().unwrap()))?;

        assert_eq!(config.book.title, "JSON Config Book");
        assert_eq!(config.book.language, "de");
        assert_eq!(config.book.authors, vec!["JSON Author"]);
        assert!(matches!(config.markdown.format, MarkdownFormat::Mdx));
        assert!(!config.markdown.frontmatter);
        assert!(config.output.html.mathjax_support);
        assert_eq!(config.output.html.search.limit_results, 100);

        Ok(())
    }

    #[test]
    fn test_load_config_unsupported_format() {
        // See test_load_config_with_custom_toml: load_config touches the CWD.
        let _guard = CWD_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let unsupported_path = temp_dir.path().join("config.yaml");
        fs::write(&unsupported_path, "title: test").unwrap();

        let result = load_config(Some(unsupported_path.to_str().unwrap()));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported config file type"));
    }

    #[test]
    fn test_load_config_nonexistent_custom_file() -> anyhow::Result<()> {
        // Lock mutex to prevent race conditions with other tests
        let _guard = CWD_MUTEX.lock().unwrap();

        // Change to a temporary directory to avoid interference from other tests
        let temp_dir = TempDir::new()?;
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(temp_dir.path())?;

        // Should succeed even if custom file doesn't exist
        let config = load_config(Some("nonexistent.toml"));

        // Always restore directory
        std::env::set_current_dir(original_dir)?;

        let config = config?;
        assert_eq!(config.book.language, "en");
        Ok(())
    }

    #[test]
    fn test_config_serialization() -> anyhow::Result<()> {
        let mut config = BookConfig::default();
        config.book.title = "Serialization Test".to_string();
        config.book.authors = vec!["Test Author".to_string()];
        config.markdown.format = MarkdownFormat::Gfm;

        let serialized = serde_json::to_string_pretty(&config)?;
        assert!(serialized.contains("Serialization Test"));
        assert!(serialized.contains("Test Author"));
        assert!(serialized.contains("gfm"));

        Ok(())
    }

    #[test]
    fn test_playground_config_defaults() {
        let config = PlaygroundConfig::default();
        assert!(!config.editable);
        assert!(!config.line_numbers);
    }

    #[test]
    fn test_search_config_defaults() {
        let config: SearchConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(config.limit_results, 20);
        assert!(!config.use_boolean_and);
        assert_eq!(config.boost_title, 2);
        assert_eq!(config.boost_hierarchy, 2);
        assert_eq!(config.boost_paragraph, 1);
        assert!(!config.expand);
        assert_eq!(config.heading_split_level, 2);
    }

    #[test]
    fn test_html_output_defaults() {
        let output = HtmlOutput::default();
        assert!(!output.mathjax_support);
        assert!(!output.allow_html);
    }
}

#[cfg(test)]
mod unsupported_key_tests {
    use super::*;

    fn doc(toml_text: &str) -> serde_json::Value {
        toml::from_str(toml_text).expect("valid toml")
    }

    #[test]
    fn test_reports_keys_the_author_set() {
        let found = unsupported_keys_in(&doc(r#"
[output.html]
mathjax-support = true

[output.html.search]
boost-title = 5

[output.html.playground]
editable = true
"#));

        let paths: Vec<&str> = found.iter().map(|k| k.path.as_str()).collect();
        assert!(paths.contains(&"output.html.mathjax-support"), "{paths:?}");
        assert!(paths.contains(&"output.html.playground"), "{paths:?}");
        assert!(
            paths.contains(&"output.html.search.boost-title"),
            "{paths:?}"
        );
    }

    #[test]
    fn test_silent_for_a_config_that_sets_nothing_unsupported() {
        let found = unsupported_keys_in(&doc(r#"
[book]
title = "Fine"
language = "en"

[output.html]
default-theme = "navy"
no-section-label = true
"#));
        assert!(found.is_empty(), "unexpected warnings: {found:?}");
    }

    #[test]
    fn test_reports_a_whole_unsupported_table() {
        let found = unsupported_keys_in(&doc(r#"
[output.html.playground]
editable = true
"#));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, "output.html.playground");
    }

    #[test]
    fn test_supported_keys_never_warn() {
        // Every key md-book acts on must stay silent, or the warning becomes
        // noise that authors learn to ignore.
        let found = unsupported_keys_in(&doc(r#"
[book]
title = "T"
src = "src"

[build]
build-dir = "book"
create-missing = false

[output.html]
site-url = "/docs/"
input-404 = "404.md"
preferred-dark-theme = "coal"
syntax-theme = "InspiredGitHub"
syntax-theme-dark = "base16-ocean.dark"
additional-css = ["extra.css"]
additional-js = ["extra.js"]

[output.html.fold]
enable = true
level = 1

[output.html.print]
enable = true

[output.html.redirect]
"old.html" = "new.html"

[output.html.search]
limit-results = 10
heading-split-level = 2
"#));
        assert!(found.is_empty(), "unexpected warnings: {found:?}");
    }
}
