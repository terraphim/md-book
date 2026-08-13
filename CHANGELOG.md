# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Restored Web Awesome as an explicit, mutually exclusive template family via
  `paths.templates = "src/templates/webawesome"`. It is pinned to
  `3.0.0-beta.6` and currently uses a clearly labelled online-only CDN
  fallback; the default Shoelace family remains vendored and offline.

### Fixed
- Pagefind full-text search is available on the first successful build, uses
  instant 150 ms debounced results, ignores stale responses, and resolves its
  bundle relative to the generated site on root and nested pages.
- A failed Pagefind index now emits a clear build error, removes stale or
  partial index assets, and rerenders the book without broken search controls.
- `light`, `rust`, `coal`, `navy`, and `ayu` now apply coherent semantic colors
  to the page, header, sidebar, TOC, search, and both component families.
- Wide-screen prose keeps its multi-column layout, while mobile layouts no
  longer overflow horizontally and long book titles truncate in the header.

### Changed
- **Breaking (custom themes)**: custom templates must provide the semantic
  surface tokens `--bg`, `--fg`, `--sidebar-bg`, and `--accent`, together with
  the selected component family's color scale. Partial legacy overrides are no
  longer treated as a complete theme.

## [0.2.0] - 2026-08-11

mdBook contract parity for the `build` path: any valid mdBook book (bar those
using `{{#include}}`) now builds correctly, rendered through md-book's own
stack. Validated against terraphim-ai's real 129-file documentation set, not
only the test corpus.

### Added
- `SUMMARY.md` book model: prefix/suffix chapters, part titles, arbitrary
  nesting, draft chapters, separators, section numbering, and exclusion of
  files the summary does not list. A book without `SUMMARY.md` still builds
  by directory walk, so nothing existing breaks.
- Subcommands `build | serve | watch | init | clean [dir]`, honouring
  `book.src` and `build.build-dir`. `-i/-o` remain as overrides.
- Theme picker (light, rust, coal, navy, ayu) with `default-theme` and
  `preferred-dark-theme`; configurable syntect themes via `syntax-theme` and
  the new `syntax-theme-dark`, scoped so code follows the chosen theme.
- Print page, `[output.html.redirect]`, `[output.html.fold]`, keyboard
  shortcuts, `additional-css` / `additional-js`, `input-404` and `site-url`.
- Per-page `<meta name="description">` and `<link rel="canonical">`, a skip
  link, and search UI that appears only when a Pagefind index exists.
- mdBook's `git-repository-url` and `edit-url-template` config keys.
- Warnings for configuration keys that parse but have no effect, so a book
  cannot silently ask for something md-book does not do.

### Fixed
- Output is now relocatable and offline: every URL is relative to its page,
  and Shoelace is vendored (356KB subset) instead of loaded from a CDN.
- Default CSS, JS, images and components are embedded, so an installed
  binary produces a complete book without a templates directory. Previously
  a book built outside this repository had no stylesheets at all.
- Configuration defaults are applied: `title`, `language`, `logo`, `edition`
  and `templates` were empty for any book without a `book.toml`.
- Server-side heading IDs, preserving Unicode, so cross-page fragment links
  and search anchors resolve.
- HTML injection via `SUMMARY.md`: authored link text and targets are
  escaped, and Tera autoescaping is active (template names now end in
  `.html`, without which nothing was escaped anywhere).
- URLs are built with forward slashes, so books built on Windows are not
  emitted with backslash hrefs.
- Chapter paths are contained within the book directory, and output paths
  within the build directory.
- `create-missing` no longer triggers a rebuild loop under `--watch`.
- Mermaid (2.9MB) loads only on pages that contain a diagram.
- Building outside a book directory fails with an explanation instead of
  succeeding with an empty book.
- Accessibility: 0 axe violations on chapter, index and 404 pages.

### Changed
- **Breaking (library)**: `render_page` takes a `PageRender` struct;
  `render_markdown` returns `RenderedMarkdown`; `write_syntax_css` takes the
  config; `parse_summary` returns `SummaryErrors`.
- The flat `sections` template variable is deprecated in favour of
  `chapters`; it remains populated and is removed in 0.3.0.

### Known limitations
- `{{#include}}`, `{{#playground}}`, `{{#rustdoc_include}}` and `{{#title}}`
  are not implemented. md-book is not a drop-in replacement for books that
  use them.
- `mathjax-support` parses but does nothing.
- `md-book serve` is a development convenience and has no test coverage.

## [0.1.1] - 2025-12-08

### Fixed
- Config parsing for kebab-case TOML/JSON keys (mathjax-support, line-numbers, limit-results, etc.)
- CI pipeline hardened - removed `continue-on-error` flags that masked test failures
- Test flakiness resolved with proper mutex synchronization for tests that change working directory
- GitHub Pages documentation deployment permissions (contents:write)
- WASM core feature compilation by making main function conditional
- Wrangler.toml configuration for Cloudflare Pages deployment

### Added
- Comprehensive deployment documentation for 9+ platforms (Cloudflare Pages, Netlify, Vercel, etc.)
- 1Password integration for secure credential management
- Deploy script with 1Password support (`scripts/deploy-with-1password.sh`)
- Playwright verification for end-to-end testing

### Changed
- Replaced self-hosted runners with GitHub-hosted runners for CI
- Updated 1Password vault configuration to use TerraphimPlatform
- Improved sync-secrets workflow with GH_PAT requirement for writing repository secrets

### Infrastructure
- Cloudflare Pages project created and deployed at https://md-book.pages.dev
- Netlify deployment configured and working
- CI pipeline fully green across all platforms (Linux, macOS, Windows)

## [0.1.0] - 2025-11-XX

### Added
- Initial release of md-book
- Markdown to HTML documentation generation
- Support for multiple markdown formats (standard, GFM, MDX)
- Server-side syntax highlighting with syntect
- Live development server with WebSocket hot reload
- File watching for automatic rebuilds
- Pagefind integration for full-text search
- Tera template system for customizable output
- Web Components for table of contents and search modal
- Configuration via TOML, JSON, or environment variables
- mdBook compatibility mode for easy migration
