# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MD-book is a modern mdbook replacement written in Rust that generates HTML documentation from Markdown files. It supports multiple markdown formats (Markdown, GFM, MDX) with server-side syntax highlighting, live development server, and integrated search functionality via Pagefind.

## Common Development Commands

### Build and Run
```bash
# Basic build - converts markdown to HTML
cargo run -- -i input_dir -o output_dir

# Development mode with file watching
cargo run -- -i input_dir -o output_dir --watch

# Development with built-in server
cargo run -- -i input_dir -o output_dir --serve

# Full development mode (watch + serve on custom port)
cargo run -- -i input_dir -o output_dir --watch --serve --port 8080
```

### Testing
```bash
# Unit tests only
cargo test --lib --bins

# Run a single test
cargo test test_name

# Scope runs to specific crate/test
cargo test -p md_book test_extract_title

# Integration tests (requires features)
cargo test --test integration --features "tokio,search,syntax-highlighting"

# E2E tests
cargo test --test e2e --features "tokio,search,syntax-highlighting"

# All tests
cargo test --all-targets --features "tokio,search,syntax-highlighting"

# Run mdBook compatibility tests
cargo test --test mdbook_test_book
cargo test --test mdbook_compatibility
```

### Quality Checks
```bash
make qa              # Format check, clippy, unit tests
make dev-check       # Complete development check
make ci-local        # Simulate CI checks locally
cargo fmt            # Format code
cargo clippy --all-targets --all-features -- -D warnings
```

### Pagefind Search
Pagefind provides full-text search for generated documentation:
```bash
# Install pagefind CLI (required for search feature)
cargo install pagefind

# Manual indexing (automatically runs during build with search feature)
pagefind --site output_dir
```

## Architecture Overview

### Module Structure
- **main.rs**: CLI entry point, orchestrates build/watch/serve via async tasks
- **lib.rs**: Library root with public API exports
- **config.rs**: Layered configuration using `twelf` (env vars, TOML, JSON)
- **core.rs**: Build logic - markdown processing, template rendering, navigation
- **server.rs**: Warp-based dev server with WebSocket live reload
- **pagefind_service.rs**: Search indexing via Pagefind CLI subprocess

### Build Process Flow
1. Load configuration (env → config file → book.toml → defaults)
2. Initialize Tera templates (custom or embedded defaults)
3. Copy static assets (CSS, JS, images, Web Components)
4. Walk input directory, extract titles, build navigation structure
5. Process each markdown file with syntax highlighting → render HTML
6. Generate index.html (from index.md or card-based default)
7. Run Pagefind search indexing (async, requires `pagefind` CLI in PATH)

### Feature Flags
```toml
default = ["server", "watcher", "search", "syntax-highlighting"]
server = ["warp", "tokio/full", "futures", "futures-util"]   # Dev server
watcher = ["notify", "tokio/full"]                            # File watching
search = ["pagefind", "tokio/rt", "tokio/macros"]            # Pagefind search
syntax-highlighting = ["syntect"]                             # Code highlighting
wasm = ["wasm-bindgen"]                                       # WASM support
```

Build without optional features: `cargo build --no-default-features`

### Configuration Priority (highest to lowest)
1. CLI arguments
2. Environment variables (MDBOOK_ prefix)
3. Custom config file (--config flag)
4. book.toml in current directory
5. Default values

### Template System
Templates in `src/templates/` or custom directory via config:
- `page.html.tera` - Individual page layout
- `index.html.tera` - Home page
- `sidebar.html.tera`, `header.html.tera`, `footer.html.tera` - Layout partials
- `components/` - Web Components (doc-toc, search-modal, simple-block)

### Key Dependencies
- **markdown**: Multi-format parsing (standard, GFM, MDX)
- **tera**: Template engine
- **syntect**: Server-side syntax highlighting
- **pagefind**: Full-text search engine (requires CLI: `cargo install pagefind`)
- **warp**: HTTP server with WebSocket
- **notify**: File system watching
- **twelf**: Configuration management
- **jiff**: Date/time operations (use instead of chrono)

## Code Patterns

### Async/Feature-Gated Code
The codebase uses extensive conditional compilation:
```rust
#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "tokio")]
pub async fn build(...) -> Result<()> { ... }

#[cfg(not(feature = "tokio"))]
pub fn build(...) -> Result<()> { ... }
```

### Error Handling
Uses `anyhow::Result` for application errors and `thiserror` for library error types (see `PagefindError`).

### Configuration Loading
```rust
let config = config::load_config(args.config.as_deref())?;
```

## Testing Guidelines

- Keep fast unit tests inline with `mod tests {}`; put multi-crate checks in `tests/` or `test_*.sh`
- Scope runs with `cargo test -p md_book test_name`; add regression coverage for new failure modes
- Write tests using `#[tokio::test]` for async code
- Never use mocks - use real implementations or test doubles
- Run `make ci-local` before pushing to verify CI will pass

## Rust Performance Practices

- Profile first (`cargo bench`, `cargo flamegraph`, `perf`) and land only measured wins
- Borrow ripgrep tactics: reuse buffers with `with_capacity`, favor iterators, reach for `memchr`/SIMD, and hoist allocations out of loops
- Apply inline directives sparingly—mark tiny wrappers `#[inline]`, keep cold errors `#[cold]`, and guard rayon-style parallel loops with `#[inline(never)]`
- Prefer zero-copy types (`&[u8]`, `bstr`) and parallelize CPU-bound work with `rayon`, feature-gated for graceful fallback
- Benchmark file: `benches/pagefind_bench.rs`

## Commit & Pull Request Guidelines

- Use Conventional Commit prefixes (`fix:`, `feat:`, `refactor:`) and keep changes scoped
- Ensure commits pass `cargo fmt`, `cargo clippy`, required `cargo test`, and desktop checks
- PRs should explain motivation, link issues, list manual verification commands, and attach UI screenshots or logs when behavior shifts

## Configuration & Security

- Keep secrets in 1Password or `.env` (never commit `.env`)
- Use `scripts/` helpers to bootstrap integrations (e.g., `scripts/deploy.sh`, `scripts/setup-cloudflare.sh`)
- Wrap optional features with graceful fallbacks for network failures
- See `.env.example` for required environment variables template

### 1Password CLI Usage
Use `op run` with an env file to inject secrets from 1Password:
```bash
# Using .env.1password file (recommended)
op run --env-file=.env.1password --no-masking -- wrangler pages deploy dist --project-name md-book

# Or export env vars first, then use op run
export CLOUDFLARE_ACCOUNT_ID="op://TerraphimPlatform/md-book-cloudflare/account_id"
export CLOUDFLARE_API_TOKEN="op://TerraphimPlatform/md-book-cloudflare/api_token"
op run --no-masking -- wrangler pages deploy dist --project-name md-book
```
- Use `op://Vault/Item/field` syntax for secret references
- Secrets must be in env vars or `--env-file` before `op run` scans them
- `--no-masking` shows the actual output (useful for debugging)

## Deployment

See [DEPLOYMENT.md](DEPLOYMENT.md) for comprehensive deployment documentation including:
- Cloudflare Pages setup (primary deployment target)
- Netlify configuration
- GitHub Actions workflows
- 1Password secret management integration

Quick deploy:
```bash
./scripts/deploy.sh production    # Cloudflare Pages
netlify deploy --prod --dir=dist  # Netlify
```

## Time Handling

Always use `jiff` instead of `chrono` for date/time operations.
