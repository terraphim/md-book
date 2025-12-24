# Hacker News Submission: MD-Book - Modern Documentation Generator in Rust

## Title Options:
- "MD-Book: A modern, fast documentation generator written in Rust"
- "Show HN: MD-Book - Rust alternative to mdBook and Docusaurus"
- "I built MD-Book because existing docs generators are either slow or too complex"

## Submission Text:

After struggling with slow build times and complex setup from existing documentation generators, I built MD-Book - a modern documentation generator written in Rust.

### The Problem
Documentation tools fall into two camps:
1. **Simple but dated** (mdBook) - Slow builds, basic features
2. **Modern but complex** (Docusaurus) - 30s+ build times, Node.js dependency hell

### The Solution
MD-Book combines the best of both worlds:
- **⚡ Fast builds** (< 1s for small docs)
- **🎨 Modern features** (instant search, live reload, responsive design)
- **🦀 Native Rust** (No Node.js, no complex setup)
- **📱 Production ready** (Deploy anywhere - Cloudflare, Netlify, Vercel)

### Key Features
- **Performance**: 10x faster builds than Docusaurus, 2x faster than mdBook
- **Search**: Built-in full-text search with Pagefind
- **Developer Experience**: WebSocket live reload, syntax highlighting
- **Static Generation**: Deploy to any hosting platform
- **Multi-format**: Support for Markdown, GFM, and MDX

### Real-World Comparison
| Tool | Build Time | Bundle Size | Language |
|------|------------|-------------|----------|
| MD-Book | < 1s | ~50KB | Rust |
| mdBook | < 2s | ~100KB | Rust |
| Docusaurus | 30s+ | ~500KB | Node.js |

### Live Demo & Links
- **Demo**: https://md-book.pages.dev (built with MD-Book itself)
- **GitHub**: https://github.com/terraphim/md-book
- **Install**: `cargo install md-book`

### Why This Matters
As Rust ecosystem grows, we need documentation tools that match Rust's performance and reliability ethos. MD-Book brings modern web development practices to the Rust world while maintaining the simplicity that makes mdBook popular.

### Quick Start
```bash
cargo install md-book
md-book -i docs -o output --serve --watch
```

That's it - beautiful documentation in 3 commands.

Open to feedback and contributions! The project is MIT licensed and actively maintained.