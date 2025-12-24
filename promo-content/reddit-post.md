# 🦀 MD-Book: A Modern Documentation Generator Built in Rust

## Why It Matters

As developers, we spend countless hours writing documentation. But existing tools like mdBook feel dated, while complex solutions like Docusaurus are overkill for most projects.

MD-Book bridges this gap - a **modern, fast, and beautiful** documentation generator that just works.

## Key Features That Matter

### ⚡ Performance First
- **Build Time**: < 1s for small docs (vs mdBook's 2s, Docusaurus's 30s)
- **Bundle Size**: ~50KB (vs 100KB+ for alternatives)
- **Native Rust**: Zero JavaScript runtime dependencies

### 🎨 Developer Experience  
- **Live Reload**: WebSocket-based hot reloading
- **Instant Search**: Full-text search with Pagefind
- **Syntax Highlighting**: Server-side rendering for 100+ languages
- **Responsive Design**: Works beautifully on desktop, mobile, tablet

### 🚀 Production Ready
- **Static Generation**: Deploy anywhere (Cloudflare, Netlify, Vercel)
- **Multi-format Support**: Markdown, GitHub Flavored, MDX
- **Feature Flags**: Include only what you need
- **Docker Support**: Container-based deployment

## Real-World Comparison

| Feature | MD-Book | mdBook | Docusaurus |
|---------|----------|---------|------------|
| **Performance** | 🚀 < 1s build | ⚡ < 2s build | 🐢 30s build |
| **Bundle Size** | ✅ ~50KB | ⚠️ ~100KB | ❌ ~500KB |
| **Search** | ✅ Built-in | ❌ Basic | ✅ Algolia |
| **Live Reload** | ✅ WebSocket | ✅ Basic | ✅ HMR |
| **Deployment** | ✅ Anywhere | ✅ Anywhere | ⚠️ Complex |

## Why Now?

The Rust ecosystem needs better documentation tools. MD-Book brings modern web development practices to the Rust world:

1. **Modern Stack**: Rust + Tera templates + Pagefind search
2. **Developer Friendly**: Zero-config for simple use cases
3. **Extensible**: Plugin system for custom needs
4. **Open Source**: MIT license, community-driven

## See It in Action

👉 **Live Demo**: https://md-book.pages.dev  
👉 **GitHub**: https://github.com/terraphim/md-book  
👉 **Install**: `cargo install md-book`

## What Makes It Different?

### Not Just Another mdBook Clone
While maintaining compatibility, MD-Book adds:
- Modern web components
- Better search functionality  
- Enhanced performance
- Responsive design out of the box

### Built for 2024
- Mobile-first design
- Edge deployment ready
- Progressive enhancement
- Accessibility-first approach

## Quick Start

```bash
# Install
cargo install md-book

# Create docs
mkdir docs && echo "# My Docs" > docs/index.md

# Generate
md-book -i docs -o output --serve --watch
```

That's it - beautiful docs in 3 commands!

## Community & Roadmap

MD-Book is actively developed with:
- 🔄 Regular releases
- 🐛 Bug-fix releases  
- ✨ New features every month
- 🤝 Community contributions welcome

**Upcoming features:**
- MathJax/LaTeX support
- Dark mode themes
- Plugin marketplace
- Multi-language docs

## Why This Matters for Rust

As the Rust ecosystem grows, we need better tools to document our projects. MD-Book is:

1. **Fast** - Matching Rust's performance philosophy
2. **Reliable** - Built for production use
3. **Accessible** - Easy for anyone to use
4. **Extensible** - Grows with your needs

## Try It Now

**For your next project:**
```bash
cargo install md-book
```

**For existing docs:**
```bash
# Try it on your current mdbook project
md-book -i docs -o output
```

**For open source:**  
Star on GitHub to support development: https://github.com/terraphim/md-book

---

*Built with ❤️ by the Terraphim team for the Rust community*