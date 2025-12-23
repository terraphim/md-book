# src/config.rs Summary

## Purpose
Configuration management using the `twelf` crate for layered configuration from multiple sources.

## Key Types
- `BookConfig`: Root config struct with book metadata, output settings, markdown options, paths
- `Book`: Title, description, authors, language, logo, GitHub URLs
- `Output`/`HtmlOutput`: HTML output settings including MathJax, playground, search config
- `SearchConfig`: Pagefind search parameters (limit, boost weights, heading split level)
- `MarkdownFormat`: Enum for markdown, GFM, or MDX format
- `Paths`: Template directory configuration

## Configuration Priority (highest to lowest)
1. Environment variables (MDBOOK_ prefix)
2. Custom config file (--config)
3. Default book.toml
4. Default values

## Load Function
`load_config(config_path: Option<&str>)` - Loads configuration with layer merging, supports TOML and JSON formats.

## Default Values
- Title: "My Book"
- Language: "en"
- Rust edition: "2021"
- Search limit: 20 results
- Templates dir: "templates"
