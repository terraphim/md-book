# src/core.rs Summary

## Purpose
Core build logic for transforming markdown files into HTML documentation.

## Key Types
- `Args`: CLI arguments struct (input, output, config, watch, serve, port)
- `PageInfo`: Page metadata (title, path)
- `PageData`: Full page data with content, sections, prev/next navigation
- `Section`: Navigation section with title and pages

## Build Process (`build()` / `build_impl()`)
1. Initialize Tera templates (from config dir or embedded defaults)
2. Create output directory structure
3. Copy static assets (CSS, JS, images, components)
4. Walk input directory for .md files
5. Build navigation structure (sections, page ordering)
6. For each markdown file:
   - Extract title from first H1
   - Process markdown with syntax highlighting (if enabled)
   - Render page template with context
   - Write HTML output
7. Generate index.html (from index.md or card-based default)
8. Run Pagefind search indexing (async, if search feature enabled)

## Markdown Processing
- `process_markdown_with_highlighting()`: AST-based processing with syntect code highlighting
- `process_markdown_basic()`: Simple markdown-to-HTML conversion
- Supports markdown formats: standard, GFM, MDX
- Code block handling: Rust-specific highlighting, mermaid passthrough, generic highlighting

## Template System
Templates loaded from config.paths.templates or embedded defaults:
- page.html.tera, index.html.tera, sidebar.html.tera, footer.html.tera, header.html.tera

## Static Assets
Copies from templates dir:
- CSS (styles.css, syntax.css, search.css, highlight.css)
- JS (live-reload.js, search, mermaid, code-copy)
- Images
- Web Components (doc-toc.js, simple-block.js, search-modal.js)
