//! Markdown → HTML fragment rendering (mdast splice + optional syntect).

use anyhow::Result;
use markdown::mdast::Node;
use markdown::to_html_with_options;
use markdown::to_mdast;

use crate::config::{BookConfig, MarkdownFormat};

#[cfg(feature = "syntax-highlighting")]
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
#[cfg(feature = "syntax-highlighting")]
use syntect::parsing::SyntaxSet;
#[cfg(feature = "syntax-highlighting")]
use syntect::util::LinesWithEndings;

/// A rendered markdown fragment plus what the page needs at runtime.
#[derive(Debug, Clone, Default)]
pub struct RenderedMarkdown {
    pub html: String,
    /// True when the source contains at least one ```mermaid fence, so the page
    /// template can load the (2.9MB) mermaid bundle only where it is used.
    pub has_mermaid: bool,
}

/// Whether `content` contains a mermaid code fence.
///
/// Detection is at fence level, over the parsed AST, not by searching the
/// rendered HTML for `language-mermaid`: a page that *documents* mermaid by
/// showing a fence inside a sample would match the latter and load the bundle
/// to render nothing.
///
/// Costs one extra mdast parse per page, which keeps a single definition of the
/// predicate shared by every rendering backend rather than one per path.
pub fn has_mermaid_fence(content: &str, config: &BookConfig) -> Result<bool> {
    fn walk(node: &Node) -> bool {
        if let Node::Code(code) = node {
            if code.lang.as_deref() == Some("mermaid") {
                return true;
            }
        }
        node.children()
            .map(|children| children.iter().any(walk))
            .unwrap_or(false)
    }

    let options = markdown_options(config);
    let ast = to_mdast(content, &options.parse)
        .map_err(|e| anyhow::anyhow!("Markdown parsing error: {:?}", e))?;
    Ok(walk(&ast))
}

/// Render markdown content to an HTML fragment, then rewrite internal `.md` links to `.html`.
///
/// When the `syntax-highlighting` feature is enabled, `syntax_set` must be `Some`.
pub fn render_markdown(
    content: &str,
    config: &BookConfig,
    #[cfg(feature = "syntax-highlighting")] syntax_set: Option<&SyntaxSet>,
    #[cfg(not(feature = "syntax-highlighting"))] _syntax_set: Option<&()>,
) -> Result<RenderedMarkdown> {
    #[cfg(feature = "syntax-highlighting")]
    {
        let ss = syntax_set.ok_or_else(|| {
            anyhow::anyhow!("syntax_set required when syntax-highlighting feature is enabled")
        })?;
        // This path already walks every code node, so detection is free; a
        // second parse per page was measurable on a 30-page book.
        let mut has_mermaid = false;
        let html = process_markdown_with_highlighting(content, ss, config, &mut has_mermaid)?;
        Ok(RenderedMarkdown {
            html: convert_md_links_to_html(&html),
            has_mermaid,
        })
    }
    #[cfg(not(feature = "syntax-highlighting"))]
    {
        // No splice walk here, so the predicate needs its own pass.
        let has_mermaid = has_mermaid_fence(content, config)?;
        Ok(RenderedMarkdown {
            html: convert_md_links_to_html(&process_markdown_basic(content, config)?),
            has_mermaid,
        })
    }
}

/// Converts internal .md links to .html links in HTML content.
/// External links (http://, https://, mailto:, etc.) are not modified.
pub fn convert_md_links_to_html(html: &str) -> String {
    let mut result = html.to_string();

    // Convert href="...md" to href="...html" (double quotes)
    let mut start = 0;
    while let Some(href_pos) = result[start..].find("href=\"") {
        let abs_pos = start + href_pos;
        let url_start = abs_pos + 6;

        if let Some(quote_end) = result[url_start..].find('"') {
            let url_end = url_start + quote_end;
            let url = &result[url_start..url_end];

            if url.ends_with(".md")
                && !url.starts_with("http://")
                && !url.starts_with("https://")
                && !url.starts_with("mailto:")
                && !url.starts_with("//")
            {
                let new_url = format!("{}.html", &url[..url.len() - 3]);
                result = format!(
                    "{}href=\"{}\"{}",
                    &result[..abs_pos],
                    new_url,
                    &result[url_end + 1..]
                );
                start = abs_pos + 6 + new_url.len() + 1;
            } else {
                start = url_end + 1;
            }
        } else {
            break;
        }
    }

    // Also handle single quotes
    start = 0;
    while let Some(href_pos) = result[start..].find("href='") {
        let abs_pos = start + href_pos;
        let url_start = abs_pos + 6;

        if let Some(quote_end) = result[url_start..].find('\'') {
            let url_end = url_start + quote_end;
            let url = &result[url_start..url_end];

            if url.ends_with(".md")
                && !url.starts_with("http://")
                && !url.starts_with("https://")
                && !url.starts_with("mailto:")
                && !url.starts_with("//")
            {
                let new_url = format!("{}.html", &url[..url.len() - 3]);
                result = format!(
                    "{}href='{}'{}",
                    &result[..abs_pos],
                    new_url,
                    &result[url_end + 1..]
                );
                start = abs_pos + 6 + new_url.len() + 1;
            } else {
                start = url_end + 1;
            }
        } else {
            break;
        }
    }

    result
}

/// Parse/compile options for the configured markdown flavour.
pub(crate) fn options_for(config: &BookConfig) -> markdown::Options {
    markdown_options(config)
}

fn markdown_options(config: &BookConfig) -> markdown::Options {
    let parse_options = match config.markdown.format {
        MarkdownFormat::Mdx => markdown::ParseOptions::mdx(),
        MarkdownFormat::Gfm => markdown::ParseOptions::gfm(),
        MarkdownFormat::Markdown => markdown::ParseOptions::default(),
    };

    let compile_options = if matches!(config.markdown.format, MarkdownFormat::Gfm) {
        markdown::CompileOptions::gfm()
    } else {
        markdown::CompileOptions::default()
    };

    let mut options = markdown::Options {
        parse: parse_options,
        compile: compile_options,
    };

    options.parse.constructs.frontmatter = config.markdown.frontmatter;
    options.parse.constructs.html_flow = config.output.html.allow_html;
    options.parse.constructs.html_text = config.output.html.allow_html;
    options.compile.allow_dangerous_html = config.output.html.allow_html;
    options.compile.allow_dangerous_protocol = config.output.html.allow_html;

    options
}

#[cfg(feature = "syntax-highlighting")]
fn process_code_block(code: &str, language: Option<&str>, ss: &SyntaxSet) -> Result<String> {
    let syntax = match language {
        Some("rust") => {
            let syntax = ss
                .find_syntax_by_extension("rs")
                .ok_or_else(|| anyhow::anyhow!("Rust syntax not found"))?;
            if code.contains("<--editable-->") {
                let code_with_comment = format!("{}\n// <--editable-->", code);
                process_rust_code(&code_with_comment, syntax, ss)?
            } else {
                process_rust_code(code, syntax, ss)?
            }
        }
        Some("mermaid") => {
            format!(
                "<pre class=\"code\"><code class=\"language-mermaid\">{}</code></pre>",
                html_escape::encode_text(code)
            )
        }
        Some(lang) => {
            let syntax = ss
                .find_syntax_by_extension(lang)
                .or_else(|| ss.find_syntax_by_name(lang))
                .or_else(|| ss.find_syntax_by_token(lang))
                .or_else(|| Some(ss.find_syntax_plain_text()))
                .ok_or_else(|| anyhow::anyhow!("Syntax not found for language: {:?}", lang))?;
            process_generic_code(code, syntax, ss)?
        }
        None => {
            let syntax = ss.find_syntax_plain_text();
            process_generic_code(code, syntax, ss)?
        }
    };
    Ok(syntax)
}

#[cfg(feature = "syntax-highlighting")]
fn process_rust_code(
    code: &str,
    syntax: &syntect::parsing::SyntaxReference,
    ss: &SyntaxSet,
) -> Result<String> {
    let mut html_generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, ss, ClassStyle::Spaced);

    for line in LinesWithEndings::from(code) {
        html_generator
            .parse_html_for_line_which_includes_newline(line)
            .map_err(|e| anyhow::anyhow!("HTML generation error: {:?}", e))?;
    }
    let html = html_generator.finalize();
    Ok(format!(
        "<pre class=\"code rust\"><code>{}</code></pre>",
        html
    ))
}

#[cfg(feature = "syntax-highlighting")]
fn process_generic_code(
    code: &str,
    syntax: &syntect::parsing::SyntaxReference,
    ss: &SyntaxSet,
) -> Result<String> {
    let mut html_generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, ss, ClassStyle::Spaced);

    for line in LinesWithEndings::from(code) {
        html_generator
            .parse_html_for_line_which_includes_newline(line)
            .map_err(|e| anyhow::anyhow!("HTML generation error: {:?}", e))?;
    }
    let html = html_generator.finalize();
    Ok(format!("<pre class=\"code\"><code>{}</code></pre>", html))
}

#[cfg(feature = "syntax-highlighting")]
fn process_markdown_with_highlighting(
    content: &str,
    ss: &SyntaxSet,
    config: &BookConfig,
    saw_mermaid: &mut bool,
) -> Result<String> {
    let options = markdown_options(config);

    let ast = to_mdast(content, &options.parse)
        .map_err(|e| anyhow::anyhow!("Markdown parsing error: {:?}", e))?;

    let mut parts = Vec::new();
    let mut last_pos = 0;

    #[allow(clippy::too_many_arguments)]
    fn process_node(
        node: &Node,
        ss: &SyntaxSet,
        content: &str,
        parts: &mut Vec<String>,
        last_pos: &mut usize,
        config: &BookConfig,
        saw_mermaid: &mut bool,
    ) -> Result<()> {
        match node {
            Node::Code(code) => {
                if let Some(pos) = &code.position {
                    if *last_pos < pos.start.offset {
                        let text = &content[*last_pos..pos.start.offset];
                        if !text.trim().is_empty() {
                            let options = markdown_options(config);
                            let temp_html = to_html_with_options(text, &options).map_err(|e| {
                                anyhow::anyhow!("Markdown conversion error: {:?}", e)
                            })?;
                            parts.push(temp_html);
                        }
                    }

                    if code.lang.as_deref() == Some("mermaid") {
                        *saw_mermaid = true;
                    }
                    let highlighted = process_code_block(&code.value, code.lang.as_deref(), ss)?;
                    parts.push(highlighted);

                    *last_pos = pos.end.offset;
                }
            }
            _ => {
                if let Some(children) = node.children() {
                    for child in children {
                        process_node(child, ss, content, parts, last_pos, config, saw_mermaid)?;
                    }
                }
            }
        }
        Ok(())
    }

    process_node(
        &ast,
        ss,
        content,
        &mut parts,
        &mut last_pos,
        config,
        saw_mermaid,
    )?;

    if last_pos < content.len() {
        let remaining = &content[last_pos..];
        if !remaining.trim().is_empty() {
            let options = markdown_options(config);
            parts.push(
                to_html_with_options(remaining, &options)
                    .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))?,
            );
        }
    }

    Ok(parts.join(""))
}

#[cfg(not(feature = "syntax-highlighting"))]
fn process_markdown_basic(content: &str, config: &BookConfig) -> Result<String> {
    let options = markdown_options(config);
    to_html_with_options(content, &options)
        .map_err(|e| anyhow::anyhow!("Markdown conversion error: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_md_links_to_html() {
        let html = r#"<a href="page.md">Link</a>"#;
        let result = convert_md_links_to_html(html);
        assert_eq!(result, r#"<a href="page.html">Link</a>"#);

        let html = r#"<a href="dir/subdir/page.md">Link</a>"#;
        let result = convert_md_links_to_html(html);
        assert_eq!(result, r#"<a href="dir/subdir/page.html">Link</a>"#);

        let html = r#"<a href="page1.md">Link1</a> and <a href="page2.md">Link2</a>"#;
        let result = convert_md_links_to_html(html);
        assert_eq!(
            result,
            r#"<a href="page1.html">Link1</a> and <a href="page2.html">Link2</a>"#
        );

        let html = r#"<a href="https://example.com/page.md">External</a>"#;
        let result = convert_md_links_to_html(html);
        assert_eq!(
            result,
            r#"<a href="https://example.com/page.md">External</a>"#
        );

        let html = r#"<a href="http://example.com/page.md">External</a>"#;
        let result = convert_md_links_to_html(html);
        assert_eq!(
            result,
            r#"<a href="http://example.com/page.md">External</a>"#
        );

        let html = r#"<a href="page.html">Link</a>"#;
        let result = convert_md_links_to_html(html);
        assert_eq!(result, r#"<a href="page.html">Link</a>"#);

        let html = r#"<a href="local.md">Local</a> and <a href="https://ext.com/file.md">Ext</a>"#;
        let result = convert_md_links_to_html(html);
        assert_eq!(
            result,
            r#"<a href="local.html">Local</a> and <a href="https://ext.com/file.md">Ext</a>"#
        );
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_default() -> Result<()> {
        let config = BookConfig::default();
        let markdown = "# Hello World\n\nThis is **bold** text.";
        let html = process_markdown_basic(markdown, &config)?;
        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_gfm() -> Result<()> {
        let mut config = BookConfig::default();
        config.markdown.format = MarkdownFormat::Gfm;
        let markdown = "# GFM Test\n\n~~strikethrough~~\n\n- [ ] Task item";
        let html = process_markdown_basic(markdown, &config)?;
        assert!(html.contains("<h1>GFM Test</h1>"));
        assert!(html.contains("strikethrough"));
        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_mdx() -> Result<()> {
        let mut config = BookConfig::default();
        config.markdown.format = MarkdownFormat::Mdx;
        let markdown = "# MDX Test\n\nThis is **bold** text.";
        let html = process_markdown_basic(markdown, &config)?;
        assert!(html.contains("<h1>MDX Test</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_with_html_allowed() -> Result<()> {
        let mut config = BookConfig::default();
        config.output.html.allow_html = true;
        let markdown = "# Test\n\n<div>Raw HTML</div>";
        let html = process_markdown_basic(markdown, &config)?;
        assert!(html.contains("<div>Raw HTML</div>"));
        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_with_html_disallowed() -> Result<()> {
        let config = BookConfig::default();
        let markdown = "# Test\n\n<div>Raw HTML</div>";
        let html = process_markdown_basic(markdown, &config)?;
        assert!(!html.contains("<div>Raw HTML</div>"));
        Ok(())
    }

    #[cfg(not(feature = "syntax-highlighting"))]
    #[test]
    fn test_process_markdown_basic_with_frontmatter() -> Result<()> {
        let mut config = BookConfig::default();
        config.markdown.frontmatter = true;
        let markdown = "---\ntitle: Test\n---\n\n# Hello World";
        let html = process_markdown_basic(markdown, &config)?;
        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(!html.contains("---"));
        Ok(())
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn test_process_code_block_rust() -> Result<()> {
        let ss = SyntaxSet::load_defaults_newlines();
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let highlighted = process_code_block(code, Some("rust"), &ss)?;
        assert!(highlighted.contains("<pre"));
        assert!(!highlighted.is_empty());
        Ok(())
    }

    #[cfg(feature = "syntax-highlighting")]
    #[test]
    fn test_process_code_block_no_language() -> Result<()> {
        let ss = SyntaxSet::load_defaults_newlines();
        let code = "some plain text code";
        let highlighted = process_code_block(code, None, &ss)?;
        assert!(highlighted.contains("<pre"));
        assert!(highlighted.contains("some plain text code"));
        Ok(())
    }
}
