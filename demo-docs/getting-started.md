# Getting Started with MD-Book

MD-Book is a modern documentation generator that makes creating beautiful docs effortless. This very documentation is generated with MD-Book!

## 📦 Installation

### From Crates.io (Recommended)
```bash
cargo install md-book
```

### From Source
```bash
git clone https://github.com/terraphim/md-book.git
cd md-book
cargo install --path .
```

### Pre-built Binaries
Download from [GitHub Releases](https://github.com/terraphim/md-book/releases) for your platform:
- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)  
- Windows (x86_64)
- Debian packages (.deb)

### Docker
```bash
docker run --rm -v $(pwd):/workspace ghcr.io/terraphim/md-book:latest
```

## 🚀 Your First Documentation

### 1. Create Content
Create a directory with your markdown files:

```
docs/
├── index.md          # Home page
├── getting-started.md # This file
├── features.md
└── api/
    ├── overview.md
    └── reference.md
```

### 2. Generate Documentation
```bash
# Basic build
md-book -i docs -o output

# Development with live reload
md-book -i docs -o output --serve --watch
```

### 3. Deploy
The `output` directory contains static HTML ready for deployment to any hosting service.

## ⚙️ Configuration System

MD-Book uses layered configuration (highest to lowest priority):

1. **CLI Arguments** - Direct command line flags
2. **Environment Variables** - `MDBOOK_` prefixed
3. **Custom Config File** - Specified with `--config`
4. **book.toml** - Default in current directory
5. **Default Values** - Built-in fallbacks

### Basic book.toml

```toml
[book]
title = "My Project Documentation"
description = "Comprehensive guide for my project"
authors = ["Your Name <your@email.com>"]
language = "en"

[output.html]
default-theme = "light"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/user/repo"

[search]
enable = true
limit-results = 30
```

### Environment Variables
```bash
# Override book title
export MDBOOK_BOOK_TITLE="My API Documentation"

# Set output directory  
export MDBOOK_OUTPUT_HTML_DEST="public"

# Enable features
export MDBOOK_FEATURES="server,search,syntax-highlighting"
```

## 🎯 Development Workflow

### Local Development
```bash
# Start development server with all features
md-book -i docs -o output --serve --watch --port 8080

# Only build, no server
md-book -i docs -o output

# Custom configuration
md-book -i docs -o output --config custom.toml
```

### Quality Checks
MD-Book includes built-in development tools:

```bash
# Run all quality checks (format, lint, test)
make qa

# Complete development check
make dev-check

# Simulate CI locally
make ci-local

# Install pre-commit hooks
make install-pre-commit
```

## 📱 Responsive Design

MD-Book automatically generates responsive documentation:

- 🖥️ **Desktop** - Full sidebar navigation with content TOC
- 📱 **Mobile** - Collapsible hamburger menu, touch-friendly
- 📋 **Tablet** - Adaptive layouts for medium screens

**Try it now:** Resize your browser window to see the responsive layout in action!

## 🔍 Search Functionality

MD-Book includes built-in search powered by Pagefind:

- **Instant Results** - Search as you type (try pressing `/`)
- **Full-Text** - Search through all content
- **Highlighting** - Visual highlighting of matches
- **No Backend** - Client-side JavaScript only
- **Fuzzy Matching** - Finds results even with typos

**Try it now:** Press `/` to focus search and type "configuration" or "deployment".

## 🎨 Theming & Customization

### Custom CSS
```toml
[output.html]
additional-css = ["theme/custom.css", "theme/highlight.css"]
```

### Custom JavaScript
```toml
[output.html] 
additional-js = ["theme/custom.js", "theme/analytics.js"]
```

### Template Customization
```toml
[output.html]
theme = "src/theme"
```

MD-Book uses Tera templates. Customize:
- `src/templates/page.html.tera` - Main page layout
- `src/templates/index.html.tera` - Home page
- `src/templates/sidebar.html.tera` - Navigation
- `src/templates/css/styles.css` - Styling

## 🚀 Production Features

### Feature Flags
Build only what you need:

```bash
# Minimal build
cargo build --no-default-features --features core

# With search only
cargo build --features "core,search"

# Full features (default)
cargo build --features "server,watcher,search,syntax-highlighting"
```

Available features:
- `server` - Development server with WebSocket live reload
- `watcher` - File system watching for auto-rebuild  
- `search` - Pagefind search indexing
- `syntax-highlighting` - Code highlighting with syntect
- `wasm` - WebAssembly support

### Performance Optimizations
- **Static Generation** - Pure HTML/CSS/JS output
- **Asset Optimization** - Minified and compressed
- **Search Indexing** - Fast client-side search
- **Lazy Loading** - Optimized for large documentation

---

**Ready to explore more?** Check out the [Features](features.md) section to see what MD-Book can do, or [Configuration](configuration.md) for detailed setup options.