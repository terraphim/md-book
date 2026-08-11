//! Page metadata derived from chapter content: descriptions and canonical URLs.

use anyhow::Result;
use markdown::mdast::Node;
use markdown::to_mdast;

use crate::config::BookConfig;

/// Longest description worth emitting; search engines truncate around here.
const MAX_DESCRIPTION: usize = 200;

/// Plain text of the first paragraph, for `<meta name="description">`.
///
/// Taken from the parsed AST rather than by scanning raw markdown, so emphasis,
/// links and inline code flatten to their text and non-ASCII survives. An
/// earlier implementation mapped every non-ASCII character to a space, which
/// turned "café" into "caf ".
///
/// # Errors
///
/// Returns an error if the markdown cannot be parsed.
pub fn first_paragraph_text(content: &str, config: &BookConfig) -> Result<Option<String>> {
    fn collect_text(node: &Node, out: &mut String) {
        match node {
            Node::Text(t) => out.push_str(&t.value),
            Node::InlineCode(c) => out.push_str(&c.value),
            _ => {
                if let Some(children) = node.children() {
                    for child in children {
                        collect_text(child, out);
                    }
                }
            }
        }
    }

    fn find_paragraph(node: &Node) -> Option<&Node> {
        if matches!(node, Node::Paragraph(_)) {
            return Some(node);
        }
        node.children()?.iter().find_map(find_paragraph)
    }

    let options = crate::render::markdown::options_for(config);
    let ast = to_mdast(content, &options.parse)
        .map_err(|e| anyhow::anyhow!("Markdown parsing error: {e:?}"))?;

    let Some(paragraph) = find_paragraph(&ast) else {
        return Ok(None);
    };

    let mut text = String::new();
    collect_text(paragraph, &mut text);
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

    if text.is_empty() {
        return Ok(None);
    }
    Ok(Some(truncate_on_word(&text, MAX_DESCRIPTION)))
}

fn truncate_on_word(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max).collect();
    match truncated.rsplit_once(' ') {
        Some((head, _)) if !head.is_empty() => format!("{head}…"),
        _ => format!("{truncated}…"),
    }
}

/// Description for a page: its first paragraph, else the book description,
/// else the book title.
pub fn page_description(content: &str, config: &BookConfig) -> String {
    first_paragraph_text(content, config)
        .ok()
        .flatten()
        .or_else(|| config.book.description.clone())
        .unwrap_or_else(|| config.book.title.clone())
}

/// Absolute canonical URL for a page, or `None` when no site URL is configured.
///
/// Never returns a relative URL: a relative `<link rel="canonical">` is worse
/// than none, since it resolves against whatever path served the page.
#[must_use]
pub fn canonical_url(site_prefix: &str, page_path: &str) -> Option<String> {
    if site_prefix.is_empty() {
        return None;
    }
    Some(format!(
        "{site_prefix}{}",
        page_path.trim_start_matches('/')
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> BookConfig {
        BookConfig::default()
    }

    #[test]
    fn test_description_from_first_paragraph() {
        let md = "# Title\n\nThe **first** paragraph with `code` and a [link](x.md).\n\nSecond.";
        let desc = page_description(md, &config());
        assert_eq!(desc, "The first paragraph with code and a link.");
    }

    #[test]
    fn test_description_preserves_non_ascii() {
        let md = "# Café\n\nUn café au lait, s'il vous plaît. Обзор. 日本語.";
        let desc = page_description(md, &config());
        assert!(desc.contains("café"), "{desc}");
        assert!(desc.contains("Обзор"), "{desc}");
        assert!(desc.contains("日本語"), "{desc}");
    }

    #[test]
    fn test_description_falls_back_to_book_then_title() {
        let mut c = config();
        c.book.description = Some("Book level".into());
        c.book.title = "The Title".into();

        assert_eq!(page_description("# Only a heading\n", &c), "Book level");

        c.book.description = None;
        assert_eq!(page_description("# Only a heading\n", &c), "The Title");
    }

    #[test]
    fn test_description_is_truncated_on_a_word_boundary() {
        let long = "word ".repeat(100);
        let md = format!("# T\n\n{long}");
        let desc = page_description(&md, &config());
        assert!(
            desc.chars().count() <= MAX_DESCRIPTION + 1,
            "{}",
            desc.len()
        );
        assert!(desc.ends_with('…'));
        assert!(!desc.contains("wor…"), "should cut between words: {desc}");
    }

    #[test]
    fn test_canonical_absent_without_site_url() {
        assert_eq!(canonical_url("", "a.html"), None);
    }

    #[test]
    fn test_canonical_joins_once() {
        assert_eq!(
            canonical_url("https://example.com/docs/", "/a/b.html"),
            Some("https://example.com/docs/a/b.html".into())
        );
    }
}
