# 🦀 MD-Book: Modern Documentation Generator

[![Crates.io](https://img.shields.io/crates/v/md-book.svg)](https://crates.io/crates/md-book)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/terraphim/md-book/workflows/CI/badge.svg)](https://github.com/terraphim/md-book/actions)

> **A modern mdBook replacement written in Rust that generates beautiful HTML documentation from Markdown files**

Welcome to the **official MD-Book documentation** - this site itself is generated using MD-Book, demonstrating all its capabilities in real-world usage.

## 🎯 What You're Looking At

This documentation demonstrates MD-Book's full feature set:

- **📱 Responsive Design** - Desktop, tablet, and mobile optimized
- **🔍 Instant Search** - Full-text search with Pagefind integration
- **⚡ Performance** - Lightning-fast builds and static generation
- **🎨 Beautiful UI** - Modern, clean interface out of the box
- **⚙️ Flexible Configuration** - Extensive customization options

## 🚀 Quick Demo

Try the search feature now - press `/` to focus search and type anything like "configuration" or "deployment". The results appear instantly!

## 📚 Real-World Examples

This documentation showcases:

- **Multiple Content Types** - Guides, API docs, configuration references
- **Code Examples** - Syntax highlighting for Rust, JavaScript, YAML, and more
- **Interactive Elements** - Table of contents, navigation, search
- **Production Deployment** - This site is deployed on Cloudflare Pages

## 🔧 Architecture Overview

MD-Book's core components:

```rust
src/
  main.rs              // CLI entry point and orchestration
  lib.rs               // Public API exports
  config.rs            // Layered configuration (twelf)
  core.rs              // Build logic and markdown processing
  server.rs            // Development server with WebSocket live reload
  pagefind_service.rs  // Search indexing integration
  templates/           // Tera templates, CSS, JS, Web Components
```

**Build Pipeline:**
1. Load configuration (env vars → config files → defaults)
2. Initialize Tera templates (custom or embedded)
3. Copy static assets (CSS, JS, images, components)
4. Process markdown with syntax highlighting
5. Generate HTML with navigation structure
6. Run Pagefind search indexing
7. Optional: Start development server with live reload

## 🌟 Key Features

### Modern Markdown Support
- **Standard Markdown** - Full CommonMark compliance
- **GitHub Flavored Markdown** - Tables, task lists, strikethrough
- **MDX Support** - React components in markdown

### Developer Experience
- **Live Development Server** - WebSocket-based live reload
- **File Watching** - Automatic rebuilds on changes
- **Syntax Highlighting** - Server-side highlighting with syntect
- **Cross-platform** - Linux, macOS, Windows support

### Production Ready
- **Static Site Generation** - Deploy anywhere
- **Search Integration** - Full-text search with Pagefind
- **Performance Optimized** - Fast builds and small bundles
- **Feature Flags** - Include only what you need

---

**Ready to start using MD-Book?** Jump to [Getting Started](getting-started.md) for installation and setup instructions.

> 💡 **This very documentation is generated with MD-Book!** Every feature you see - search, navigation, responsive design, code highlighting - works out of the box.