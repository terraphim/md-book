//! Book-directory / `src` / `build-dir` resolution and precedence.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::BookConfig;

/// Resolved book locations. Precedence: CLI flag > book.toml > default.
#[derive(Debug, Clone)]
pub struct BookPaths {
    pub root: PathBuf,
    pub src: PathBuf,
    pub build: PathBuf,
}

/// Resolve book paths.
///
/// - `book_dir`: directory containing `book.toml` (subcommand positional)
/// - `input_override`: `-i` / `--input`
/// - `output_override`: `-o` / `--output` / `-d` / `--dest-dir`
pub fn resolve(
    book_dir: Option<&Path>,
    input_override: Option<&str>,
    output_override: Option<&str>,
    config: &BookConfig,
) -> Result<BookPaths> {
    let root = book_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let src = if let Some(i) = input_override {
        PathBuf::from(i)
    } else if let Some(ref s) = config.book.src {
        root.join(s)
    } else {
        root.join("src")
    };

    let build = if let Some(o) = output_override {
        PathBuf::from(o)
    } else if let Some(ref d) = config.build.build_dir {
        root.join(d)
    } else {
        root.join("book")
    };

    Ok(BookPaths { root, src, build })
}

/// Scaffold a new book (mdBook-compatible init).
pub fn init_book(dir: &Path) -> Result<()> {
    use std::fs;
    fs::create_dir_all(dir.join("src"))?;
    let book_toml = dir.join("book.toml");
    if !book_toml.exists() {
        fs::write(
            &book_toml,
            r#"[book]
title = "My Book"
authors = []
language = "en"

[build]
build-dir = "book"
"#,
        )?;
    }
    let summary = dir.join("src/SUMMARY.md");
    if !summary.exists() {
        fs::write(
            &summary,
            r#"# Summary

- [Chapter 1](./chapter_1.md)
"#,
        )?;
    }
    let ch1 = dir.join("src/chapter_1.md");
    if !ch1.exists() {
        fs::write(&ch1, "# Chapter 1\n\n")?;
    }
    let gitignore = dir.join(".gitignore");
    if !gitignore.exists() {
        fs::write(&gitignore, "book\n")?;
    }
    println!("Initialised book in {}", dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BookConfig;

    #[test]
    fn test_resolve_precedence_cli_over_toml() {
        let mut config = BookConfig::default();
        config.book.src = Some("toml-src".into());
        config.build.build_dir = Some("toml-out".into());

        let paths = resolve(
            Some(Path::new("/book")),
            Some("cli-src"),
            Some("cli-out"),
            &config,
        )
        .unwrap();
        assert_eq!(paths.src, PathBuf::from("cli-src"));
        assert_eq!(paths.build, PathBuf::from("cli-out"));
    }

    #[test]
    fn test_resolve_defaults() {
        let config = BookConfig::default();
        let paths = resolve(Some(Path::new("/book")), None, None, &config).unwrap();
        assert_eq!(paths.src, PathBuf::from("/book/src"));
        assert_eq!(paths.build, PathBuf::from("/book/book"));
    }
}
