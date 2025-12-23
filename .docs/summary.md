# MD-Book Project Summary

## Overview
MD-book is a modern mdbook replacement written in Rust that generates HTML documentation from Markdown files. It supports multiple markdown formats (Markdown, GFM, MDX) with server-side syntax highlighting, live development server, and integrated search functionality via Pagefind.

## Architecture

### Core Components
```
src/
  main.rs          - CLI entry point, orchestrates build/watch/serve
  lib.rs           - Library root, public API exports
  config.rs        - Layered configuration system (twelf)
  core.rs          - Build logic, markdown processing, template rendering
  server.rs        - Development server with live reload
  pagefind_service.rs - Search indexing integration
  templates/       - Tera templates, CSS, JS, Web Components
```

### Build Pipeline
1. **Configuration Loading** - Merges env vars, config files, defaults
2. **Template Setup** - Tera templates (embedded or custom)
3. **Asset Copying** - CSS, JS, images, components to output
4. **Markdown Processing** - AST parsing, syntax highlighting, HTML generation
5. **Navigation Building** - Section/page hierarchy from directory structure
6. **Search Indexing** - Pagefind integration for full-text search
7. **Development Features** - File watching, live reload server

### Feature Flags
- `default`: server, watcher, search, syntax-highlighting
- `server`: HTTP server with WebSocket live reload
- `watcher`: File system watching for auto-rebuild
- `search`: Pagefind search indexing
- `syntax-highlighting`: Syntect code highlighting
- `wasm`: WebAssembly support

### Key Dependencies
- **markdown**: Multi-format parsing (standard, GFM, MDX)
- **tera**: Template engine
- **syntect**: Server-side syntax highlighting
- **pagefind**: Full-text search
- **warp**: Development server
- **notify**: File watching
- **twelf**: Configuration management
- **jiff**: Date/time handling

### Configuration System
Layered configuration via twelf (priority high to low):
1. CLI arguments
2. Environment variables (MDBOOK_ prefix)
3. Custom config file (--config)
4. book.toml in current directory
5. Default values

Supports TOML and JSON config formats.

## Testing
- Unit tests: `cargo test --lib --bins`
- Integration tests: `cargo test --test integration --features "tokio,search,syntax-highlighting"`
- E2E tests: `cargo test --test e2e`
- All tests: `make test-all`

## Development Workflow
- `make qa` - Format check, clippy, tests
- `make dev-check` - Complete development check
- `make ci-local` - Simulate CI locally
- Pre-commit hooks available via `make install-pre-commit`

## Deployment
- Cloudflare Pages (primary)
- Netlify (alternative)
- GitHub Actions workflows for CI/CD
