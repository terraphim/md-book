//! Configuration types for the book generator.
//! These types are used to parse and store configuration from various sources
//! including TOML files and environment variables.

use serde::{Deserialize, Serialize};
use twelf::{config, Layer};

/// Main configuration structure for the book
#[config]
#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct BookConfig {
    pub book: Book,
    #[serde(default)]
    pub rust: Rust,
    #[serde(default)]
    pub output: Output,
    #[serde(default)]
    pub paths: Paths,
}

/// Book configuration
#[config]
#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct Book {
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
}

fn default_language() -> String {
    "en".to_string()
}

fn default_logo() -> String {
    "/img/default_logo.svg".to_string()
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Rust {
    #[serde(default = "default_edition")]
    pub edition: String,
}

fn default_edition() -> String {
    "2021".to_string()
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Output {
    pub html: HtmlOutput,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct HtmlOutput {
    #[serde(default)]
    pub mathjax_support: bool,
    pub playground: PlaygroundConfig,
    pub search: SearchConfig,
    pub redirect: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PlaygroundConfig {
    #[serde(default)]
    pub editable: bool,
    #[serde(default)]
    pub line_numbers: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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

fn default_limit_results() -> u32 {
    20
}
fn default_boost_title() -> u32 {
    2
}
fn default_boost_hierarchy() -> u32 {
    2
}
fn default_boost_paragraph() -> u32 {
    1
}
fn default_heading_split_level() -> u32 {
    2
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub(crate) struct Paths {
    #[serde(default = "default_templates_dir")]
    pub templates: String,
}

fn default_templates_dir() -> String {
    "templates".to_string()
}

pub(crate) fn load_config(config_path: Option<&str>) -> anyhow::Result<BookConfig> {
    let mut layers = vec![Layer::Env(Some("MDBOOK_".to_string()))];

    // Add default book.toml if it exists
    if std::path::Path::new("book.toml").exists() {
        layers.push(Layer::Toml("book.toml".into()));
    }

    // Add custom config file if provided
    if let Some(path) = config_path {
        if std::path::Path::new(path).exists() {
            // and is TOML
            if path.ends_with(".toml") {
                layers.push(Layer::Toml(path.into()));
            } else if path.ends_with(".json") {
                layers.push(Layer::Json(path.into()));
            } else {
                anyhow::bail!("Unsupported config file type: {}", path);
            }
        }
    }

    let config = BookConfig::with_layers(&layers)?;
    Ok(config)
}
