//! Preprocessing stage — designated seam for mdBook-style directives.
//!
//! Today this is an identity function. P2 directives (`{{#include}}`,
//! `{{#rustdoc_include}}`, `{{#playground}}`, hidden lines, etc.) land here
//! in a later cycle without touching the collect/render stages.

use anyhow::Result;

/// Context available to preprocessors.
///
/// Empty for now; later increments may carry chapter path, book root, and
/// config knobs that directives need.
#[derive(Debug, Default, Clone)]
pub struct PreprocessCtx;

/// Identity preprocessor.
///
/// Called on every chapter's markdown text before mdast parsing. Must preserve
/// input exactly until P2 lands — that is the Increment A contract.
///
/// # Errors
///
/// Never errors today; the `Result` is for future I/O-bound directives.
pub fn preprocess(md: &str, _ctx: &PreprocessCtx) -> Result<String> {
    Ok(md.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_identity_preserves_input() {
        let input = "# Title\n\n{{#include foo.md}}\n\nSome text with **bold**.";
        let out = preprocess(input, &PreprocessCtx).unwrap();
        assert_eq!(out, input);
    }

    #[test]
    fn test_preprocess_identity_empty() {
        let out = preprocess("", &PreprocessCtx).unwrap();
        assert_eq!(out, "");
    }

    #[test]
    fn test_preprocess_identity_preserves_whitespace() {
        let input = "  \n\t# A\n\n";
        let out = preprocess(input, &PreprocessCtx).unwrap();
        assert_eq!(out, input);
    }
}
