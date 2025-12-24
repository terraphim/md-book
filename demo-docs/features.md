# Features

MD-Book comes packed with modern features that make documentation creation a breeze. Every feature demonstrated here is built into MD-Book!

## 🎨 Beautiful Design

### Responsive Layout
- **Desktop**: Full sidebar navigation with content TOC (try resizing this window!)
- **Mobile**: Collapsible hamburger menu, touch-friendly interface
- **Tablet**: Adaptive layouts for medium screens

### Modern Styling
- Clean, professional appearance out of box
- Card-based navigation for home pages
- Smooth animations and transitions
- Typography optimized for readability

### Theme System
```rust
// Example theme customization
let theme = Theme {
    primary_color: "#0066cc",
    secondary_color: "#f8f9fa",
    font_family: "Inter, system-ui, sans-serif",
    code_theme: "github-light",
};
```

## ⚡ Developer Experience

### Live Development Server
```bash
# Start with all features enabled
md-book -i docs -o output --serve --watch --port 8080

# Development with custom port
md-book -i docs -o output --serve --port 3000

# Watch mode without server
md-book -i docs -o output --watch
```

**Features:**
- WebSocket-based live reload (no page refresh needed!)
- Automatic browser refresh when files change
- Intelligent rebuilds (only rebuild changed files)
- Custom port and hostname support

### Fast Builds
- **Rust-native performance** - Built for speed
- **Incremental builds** - Only rebuild what changed
- **Parallel processing** - When possible, process files in parallel
- **Static generation** - Deploy anywhere, no runtime dependencies

```bash
# Performance comparison
$ time md-book -i docs -o output
real    0m0.234s  # Typical for small docs
user    0m0.156s
sys     0m0.078s
```

## 🔍 Powerful Search

### Pagefind Integration
- **Full-text search** across all documentation
- **Instant results** as you type (try pressing `/` now!)
- **Highlighted matches** in search context
- **No backend** - client-side JavaScript only

### Search Features
```javascript
// Search configuration (in book.toml)
[search]
enable = true
limit-results = 30
teaser-word-count = 30
use-boolean-and = true
boost-title = 2        # Boost titles in results
boost-hierarchy = 1     # Boost headings
expand = true           # Show all results
```

## 📝 Rich Content Support

### Multiple Markdown Formats
```markdown
# Standard Markdown
## GitHub Flavored Markdown
- [x] Task lists
- ~~Strikethrough text~~
- `inline code`

### Tables
| Feature | Status | Notes |
|---------|--------|-------|
| Search | ✅ | Pagefind integration |
| Themes | ✅ | Customizable |
| Plugins | 🚧 | Coming soon |

### Code Blocks with Syntax Highlighting
```rust
fn main() {
    println!("Hello, MD-Book!");
}
```
```

### Advanced Content Features
- **Mermaid Diagrams** - Charts, flowcharts, graphs
- **Mathematical Expressions** - MathJax/LaTeX support  
- **Embedded HTML** - With security controls
- **Table of Contents** - Auto-generated from headings
- **Code Copy** - One-click code snippet copying

## 🏗️ Flexible Architecture

### Template System
MD-Book uses Tera templating engine:

```html
<!-- src/templates/page.html.tera -->
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
    <link rel="stylesheet" href="/css/styles.css">
</head>
<body>
    {% include "header.html" %}
    <main>{{ content }}</main>
    {% include "footer.html" %}
</body>
</html>
```

### Configuration Management
```toml
[book]
title = "My Documentation"
description = "Complete guide for my project"
authors = ["Your Name"]

[output.html]
default-theme = "light"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/user/repo"

[output.html.search]
enable = true
limit-results = 30

[preprocessor.custom]
command = "my-preprocessor"
```

### Plugin Architecture
- **Preprocessors** - Transform content before rendering
- **Custom Renderers** - Different output formats
- **Hook System** - Customize build process
- **Web Components** - Custom interactive elements

## 🚀 Production Ready

### Static Site Generation
```bash
$ md-book -i docs -o public
$ ls public/
index.html      css/           js/           
getting-started.html  img/           pagefind/
```

**Features:**
- **Pure HTML/CSS/JS** - No server requirements
- **CDN-friendly** - Optimized for global distribution
- **Search-ready** - Indexed and ready for instant search
- **Small footprint** - Minimal bundle sizes

### Multi-Platform Deployment
MD-Book is deployed to multiple platforms automatically:
- **Cloudflare Pages** - Primary (unlimited bandwidth)
- **Netlify** - Alternative with drag-and-drop
- **GitHub Pages** - For open source projects
- **Vercel** - Zero-config deployments
- **Any static hosting** - The output is just static files

### Performance Features
- **Asset optimization** - Minified CSS/JS
- **Lazy loading** - Images and heavy content
- **Caching headers** - Optimal browser caching
- **Compressed output** - gzip/brotli ready

## 🔧 Technical Features

### Feature Flags
Build only what you need:

```bash
# Minimal build
cargo build --no-default-features --features core

# Development build  
cargo build --features "server,watcher,search"

# Production build
cargo build --features "search,syntax-highlighting"
```

Available flags:
- `server` - HTTP development server
- `watcher` - File system watching
- `search` - Pagefind search integration
- `syntax-highlighting` - Code highlighting
- `wasm` - WebAssembly support

### Cross-Platform Support
- **Linux** (x86_64, ARM64, musl, glibc)
- **macOS** (Intel, Apple Silicon)
- **Windows** (MSVC, GNU)
- **WebAssembly** - Browser-based generation

### Security Features
- **HTML Sanitization** - Safe rendering of markdown
- **Content Security Policy** - XSS protection
- **No Remote Code Execution** - Static generation only
- **Dependency Auditing** - Regular security updates

## 📊 Performance Benchmarks

Typical performance metrics:

| Metric | MD-Book | mdBook | Docusaurus |
|--------|----------|--------|------------|
| **Build Time** | < 1s | < 2s | 10-30s |
| **Bundle Size** | ~50KB | ~100KB | ~500KB |
| **Search Latency** | < 50ms | N/A | < 200ms |
| **Page Load** | < 1.5s | < 2s | < 3s |

## 🌟 Real-World Examples

This documentation demonstrates:
- **Multiple sections** with different content types
- **Code examples** in various languages
- **Interactive elements** like search and navigation  
- **Responsive design** that adapts to your screen
- **Performance optimization** with fast loading
- **Production deployment** on multiple platforms

---

**Ready to customize?** Check out the [Configuration](configuration.md) section for detailed setup options, or see the [deployment guide](../DEPLOYMENT.md) for production deployment options.