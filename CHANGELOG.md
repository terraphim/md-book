# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
