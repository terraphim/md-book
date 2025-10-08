# MD book is a mdbook replacement with extra features to make docs beautiful
## Features
* parse md, mdx or gfm files thanks to markdown-rs
* tera templates for easy hacking
* Beautiful default styling
* Right hand TOC to navigate around the page.
* Create index.md to create a content for home page, alternatively it will create a list of cards with all the pages as index.
* Code blocks with syntax highlighting on server side using syntect

## Run
Checkout the source code and run:

```rust
cargo run -- -i ../mdBook/test_book  -o ./test_mdbook
```

-i is the input directory and -o is the output directory.
input directory is the directory with md files.

The tool will generate the input directory with markdown files and the output directory with HTML files ready to be deployed on any static site. 

Adjust the styling in the src/templates/css/styles.css file.

Or anything you want to change in the src/templates folder. It's a standard Tera template, so you can add your own custom stuff there.

## Styling

* Nicer default styling for content - multiple columns for horizontal layout,
* Right-hand TOC to navigate around the page.
* Create index.md to create content for the home page; alternatively, it will create a list of cards with all the pages as an index.


- Code blocks with syntax highlighting

- Better default styling

# Screenshots

![screen_resize](gif/screen_resize.gif)
![screen](gif/screen.gif)

# Development

## Quick Start

1. **Clone and build:**
   ```bash
   git clone https://github.com/terraphim/md-book.git
   cd md-book
   cargo build
   ```

2. **Set up pre-commit hooks (recommended):**
   ```bash
   make install-pre-commit
   # or manually:
   ./scripts/setup-pre-commit.sh
   ```

3. **Run quality checks:**
   ```bash
   make qa              # Run all checks (format, lint, test)
   make dev-check       # Complete development check
   make ci-local        # Simulate CI checks locally
   ```

## Available Commands

- `make help` - Show all available commands
- `make check` - Run cargo check
- `make fmt` - Check code formatting
- `make fmt-fix` - Fix code formatting
- `make clippy` - Run clippy lints
- `make test` - Run unit tests
- `make test-integration` - Run integration tests
- `make test-all` - Run all tests
- `make qa` - Run all quality checks
- `make clean` - Clean build artifacts

## Pre-commit Hooks

The project includes pre-commit hooks that automatically run:
- `cargo fmt --all -- --check` - Formatting check
- `cargo clippy --all-targets --all-features -- -D warnings` - Linting
- `cargo test --lib --bins` - Unit tests
- `cargo check --all-targets --all-features` - Compilation check

These run automatically on every commit to ensure code quality.

# Configuration

You can add a book.toml file to the input directory to configure the book.

Supports TOML configuration via book.toml
Allows overriding with environment variables (prefixed with MDBOOK_)
Supports command-line arguments
Enables shell expansion in config file paths
Provides default values for optional fields
Example usage:

```bash
# Using environment variables
MDBOOK_BOOK.TITLE="My Book" ./md-book -i input -o output

# Using custom config file
./md-book -i input -o output -c ~/my-config.toml

# Config values can be nested using an underscore
MDBOOK_OUTPUT.HTML.MATHJAX_SUPPORT=true ./md-book -i input -o output
```
The configuration system follows the priority order:
1. Command line arguments (highest priority)
2. Environment variables (prefixed with MDBOOK_)
3. Custom config file (if provided)
4. Default book.toml
5. Default values (lowest priority)
you shall be able to feed config into json and yaml files.

# Serve and Watch

## Just build
```bash
cargo run -- -i input -o output
```

## Build and watch
```bash
cargo run -- -i input -o output --watch
```

## Build and serve
```bash
cargo run -- -i input -o output --serve
```

## Build, watch and serve on custom port
```bash
cargo run -- -i input -o output --watch --serve --port 8080
```

## Deployment

MD-Book supports deployment to multiple platforms:

### Cloudflare Pages (Recommended)
```bash
# Setup (includes 1Password integration)
./scripts/setup-cloudflare.sh

# Deploy to production  
./scripts/deploy.sh production
```

### Netlify
```bash
# Build site
cargo run -- -i docs -o dist

# Deploy with CLI
netlify deploy --prod --dir=dist

# Or drag & drop at https://app.netlify.com/drop
```

See [DEPLOYMENT.md](DEPLOYMENT.md) for comprehensive deployment documentation including:
- 1Password integration for secure secret management
- GitHub Actions workflows
- Custom domains and SSL
- Performance optimization
- Platform comparison

# TODO

- [ ] Rust specific synax highlight. Good first issue.
- [x] Search [Done]
- [ ] Mathjax
