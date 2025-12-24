# Configuration Guide

MD-Book offers flexible configuration options through multiple sources. This documentation itself uses these configuration options!

## 🏗️ Configuration Priority

Configuration is loaded in this order (highest to lowest priority):

1. **Command Line Arguments** - Direct CLI flags
2. **Environment Variables** - `MDBOOK_` prefixed  
3. **Custom Config File** - Specified with `--config`
4. **book.toml** - Default configuration file
5. **Default Values** - Built-in fallbacks

### Example Priority in Action

```bash
# This will override book.toml and defaults
MDBOOK_BOOK_TITLE="Override Title" md-book -i docs -o output

# Custom config file
md-book -i docs -o output --config /path/to/custom.toml

# CLI arguments (highest priority)
md-book -i docs -o output --title "CLI Override"
```

## 📄 Basic Configuration

### book.toml Structure

This is the configuration used for this documentation:

```toml
[book]
title = "MD-Book: Modern Documentation Generator"
description = "A modern mdbook replacement written in Rust"
authors = ["Terraphim Team"]
language = "en"
src = "demo-docs"

[output.html]
default-theme = "light"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/terraphim/md-book"
edit-url-template = "https://github.com/terraphim/md-book/edit/main/demo-docs/{path}"

[search]
enable = true
limit-results = 30
teaser-word-count = 30
use-boolean-and = true
boost-title = 2
boost-hierarchy = 1
```

### Core Book Settings

```toml
[book]
title = "My Documentation"                    # Book title
description = "Complete guide for developers"    # Meta description
authors = ["John Doe <john@example.com>"]      # Authors list
language = "en"                               # Content language
src = "docs"                                  # Source directory
multilingual = false                           # Multi-language support
```

### Build Configuration

```toml
[build]
build-dir = "output"               # Output directory
extra-mime-types = []              # Additional MIME types
use-default-preprocessors = true     # Use built-in preprocessors
create-missing = true              # Create missing directories
```

## 🎨 Theme Configuration

### Custom CSS and JavaScript

```toml
[output.html]
additional-css = [
    "theme/custom.css",
    "theme/highlight.css",
    "theme/branding.css"
]

additional-js = [
    "theme/custom.js", 
    "theme/analytics.js",
    "theme/search-enhancements.js"
]
```

### Theme Customization

```toml
[output.html]
default-theme = "light"              # Light, dark, or custom
preferred-dark-theme = "navy"        # Dark mode preference
theme = "src/theme"                  # Custom theme directory

[output.html.themes]
light = "src/themes/light.css"
dark = "src/themes/dark.css" 
navy = "src/themes/navy.css"
```

### Branding and Assets

```toml
[output.html]
favicon = "favicon.ico"             # Site favicon
logo = "logo.png"                   # Header logo
cname = "docs.example.com"          # Custom domain name

# Custom content
content-404 = "404.md"             # Custom 404 page
landing-page = "landing.md"         # Custom landing page
```

## 🔍 Search Configuration

### Pagefind Integration

```toml
[search]
enable = true                          # Enable search functionality
limit-results = 30                      # Max search results
teaser-word-count = 30                  # Words in search snippet
use-boolean-and = true                   # AND operator support
boost-title = 2                        # Title search boost
boost-hierarchy = 1                     # Heading search boost
boost-paragraph = 1                    # Content search boost
expand = true                          # Show all results by default
```

### Advanced Search Options

```toml
[search.indexing]
indexing = "all"                       # What to index
root-selector = "html"                 # Root element for indexing
exclude-selectors = []                  # Elements to exclude
force-language = ""                     # Force search language
translation = ""                       # Translation file path
custom-styles = false                   # Custom search styles
excerpt-length = 30                    # Search result excerpt length
filter-tags = []                       # Available search filters
```

### Search UI Customization

```toml
[search.ui]
display-results = true                 # Show search results
placeholder-text = "Search documentation..." 
empty-results-text = "No results found"
filter-clear-text = "Clear"
show-images = false                     # Show images in results
show-subtitle = false                  # Show page subtitles
loading-text = "Searching..."           # Loading indicator
```

## 🌐 Multi-language Support

### Basic I18n Setup

```toml
[book]
title = "My Book"
authors = ["John Doe"] 
language = "en"
multilingual = true                     # Enable i18n
src = "src"

[book.languages.en]
title = "My Book (English)"
src = "src"

[book.languages.es]
title = "Mi Libro (Spanish)"
src = "src/es"

[book.languages.fr]
title = "Mon Livre (French)"
src = "src/fr"
```

### Language-Specific Settings

```toml
[book.languages.en.output.html]
default-theme = "light"
git-repository-url = "https://github.com/user/docs"

[book.languages.es.output.html]
default-theme = "dark"
git-repository-url = "https://github.com/user/docs-es"
```

## 🔧 Advanced Configuration

### Environment Variables

```bash
# Book configuration
export MDBOOK_BOOK_TITLE="My Project Docs"
export MDBOOK_BOOK_DESCRIPTION="API Documentation"
export MDBOOK_BOOK_AUTHORS="Team Name"

# Build configuration  
export MDBOOK_BUILD_BUILD_DIR="public"
export MDBOOK_OUTPUT_HTML_DEST="dist"

# Feature toggles
export MDBOOK_FEATURES="server,search,syntax-highlighting"
export MDBOOK_OUTPUT_HTML_SEARCH_ENABLE=true
```

### Nested Configuration

```bash
# Use underscores for nested keys
MDBOOK_OUTPUT_HTML_MATHJAX_SUPPORT=true
MDBOOK_OUTPUT_HTML_GIT_REPOSITORY_URL="https://github.com/user/repo"
MDBOOK_OUTPUT_HTML_ADDITIONAL_CSS="theme/custom.css"
MDBOOK_DEV_SERVER_PORT=3000
```

### Custom Preprocessors

```toml
[preprocessor.toc]
command = "mdbook-toc"
renderer = ["html"]

[preprocessor.admonish]
command = "mdbook-admonish" 

[preprocessor.links]
# Built-in link checker

[preprocessor.custom]
command = "node my-preprocessor.js"
renderer = ["html", "pdf"]
```

### Custom Renderers

```toml
[output.epub]
command = "mdbook-epub"

[output.pdf] 
command = "mdbook-pdf"
optional = true

[output.json]
command = "mdbook-json"

[output.markdown]
command = "mdbook-markdown"
```

## 📱 Development Server Options

### Server Configuration

```toml
[dev-server]
hostname = "localhost"                 # Server hostname
port = 3000                          # Server port  
open = true                           # Open browser automatically
watcher = ["*.md", "src/**/*"]       # Files to watch
websocket-port = 3001                 # WebSocket port for live reload
```

### Watcher Settings

```toml
[dev-server.watcher]
watch-paths = ["src", "theme"]         # Directories to watch
ignore-paths = ["output", ".git"]     # Paths to ignore
debounce-ms = 300                     # Debounce delay (ms)
```

### Live Reload Options

```toml
[dev-server.live-reload]
enable = true                         # Enable live reload
websocket-path = "/livereload"         # WebSocket path
port = 3001                          # WebSocket port
host = "localhost"                    # WebSocket host
```

## 🔐 Security Settings

### HTML Security

```toml
[output.html]
# WARNING: Only enable if you trust all content authors
allow-html = true                      # Allow raw HTML in markdown
sanitize-html = true                   # Sanitize HTML content

[output.html.mathjax]
trusted-types = false                  # CSP for MathJax
```

### Content Security Policy

```toml
[security]
allow-remote-content = false           # Block remote content
sanitize-html = true                   # HTML sanitization
content-security-policy = "default-src 'self'; script-src 'self' 'unsafe-inline'"
```

### Input Validation

```toml
[security.validation]
max-file-size = "10MB"                # Max file size
allowed-extensions = [".md", ".txt"]  # Allowed file types
block-external-links = false            # Block external links
```

## 🎯 Performance Optimization

### Build Optimization

```toml
[build]
create-missing = true                  # Create missing directories
use-default-preprocessors = false        # Disable default preprocessors
parallel = true                       # Parallel processing
cache-dir = ".cache"                  # Build cache directory

[build.extra-watch-dirs] = ["theme"]
```

### Asset Optimization

```toml
[output.html]
copy-fonts = true                     # Copy font files
minify-css = true                     # Minify CSS
minify-js = true                      # Minify JavaScript
gzip-assets = true                    # Compress assets

[output.html.cdns]
# Use CDN for popular libraries
font-awesome = "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0"
highlight-js = "https://cdn.jsdelivr.net/npm/highlight.js@11.8.0"
```

### Caching Strategy

```toml
[cache]
enable = true                         # Enable build cache
cache-dir = ".mdbook-cache"           # Cache directory
max-age = "7d"                       # Cache max age
invalidate-on-config-change = true      # Invalidate cache on config change
```

## 📋 Real-World Examples

### API Documentation Setup

```toml
[book]
title = "My API Documentation"
description = "REST API reference and examples"
authors = ["API Team"]

[output.html]
default-theme = "coal"                 # Dark theme for API docs
git-repository-url = "https://github.com/user/api"
edit-url-template = "https://github.com/user/api/edit/main/docs/{path}"

[output.html.code]
line-numbers = true                  # Show line numbers
copy-button = true                    # Show copy button
highlight-language = "rust"             # Default language
```

### Project Documentation Setup

```toml
[book]
title = "Project Documentation"
description = "Complete guide for using our project"

[output.html]
default-theme = "light"
preferred-dark-theme = "ayu"
git-repository-url = "https://github.com/user/project"

[output.html.playground]
editable = true                       # Allow code editing
copyable = true                      # Allow code copying
copy-js = true                       # Copy with JS
```

### Blog/Article Setup

```toml
[book]
title = "Tech Blog"
description = "Technical articles and tutorials"

[output.html]
default-theme = "light"
git-repository-url = "https://github.com/user/blog"

[output.html.blog]
show-date = true                      # Show article dates
show-author = true                    # Show author names
show-tags = true                      # Show article tags
rss-feed = true                      # Generate RSS feed
```

### Enterprise Documentation

```toml
[book]
title = "Enterprise Platform"
description = "Internal platform documentation"

[output.html]
default-theme = "corporate"            # Custom theme
git-repository-url = "https://git.company.com/platform"

[output.html.enterprise]
analytics = true                      # Enable analytics
feedback-form = true                   # Enable feedback
search-analytics = true                # Track search usage
access-control = true                 # Restrict access
```

---

**Ready to deploy?** Check out the comprehensive [Deployment Guide](../DEPLOYMENT.md) for production setup across multiple platforms.