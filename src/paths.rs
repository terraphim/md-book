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
        let p = PathBuf::from(i);
        if p.is_absolute() {
            p
        } else {
            // Relative -i is resolved from CWD (explicit override), not book root
            p
        }
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

impl BookPaths {
    /// Verify the resolved source directory actually exists.
    ///
    /// Resolution itself is pure; this is the I/O boundary check, called before a
    /// build so that running `md-book` outside a book directory fails loudly
    /// instead of emitting an empty book.
    ///
    /// # Errors
    ///
    /// Returns an error naming the path that was expected, and how to fix it.
    pub fn validate_for_build(&self, had_input_override: bool) -> Result<()> {
        if self.src.is_dir() {
            return Ok(());
        }

        if had_input_override {
            anyhow::bail!("input directory does not exist: {}", self.src.display());
        }

        anyhow::bail!(
            "no book found in {}\n  \
             expected a source directory at {}\n  \
             run `md-book init` to scaffold one, or pass --input <dir>",
            self.root.display(),
            self.src.display()
        )
    }
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

    #[test]
    fn test_validate_errors_when_src_missing() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = BookConfig::default();
        let paths = resolve(Some(dir.path()), None, None, &config).unwrap();

        let err = paths
            .validate_for_build(false)
            .expect_err("a directory with no src/ is not a book");
        let msg = err.to_string();
        assert!(msg.contains("no book found"), "got: {msg}");
        assert!(msg.contains("md-book init"), "got: {msg}");
    }

    #[test]
    fn test_validate_errors_name_the_input_override() {
        let config = BookConfig::default();
        let paths = resolve(None, Some("definitely-not-here"), None, &config).unwrap();

        let err = paths
            .validate_for_build(true)
            .expect_err("an explicit --input that does not exist is an error");
        assert!(
            err.to_string().contains("definitely-not-here"),
            "error should name the path the user passed, got: {err}"
        );
    }

    #[test]
    fn test_validate_accepts_existing_src() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        let config = BookConfig::default();
        let paths = resolve(Some(dir.path()), None, None, &config).unwrap();

        assert!(paths.validate_for_build(false).is_ok());
    }
}
