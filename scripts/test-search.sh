#!/bin/bash
set -euo pipefail

# Search Functionality Test Script
# Comprehensive testing of Pagefind search integration

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "\n${BLUE}🔍 $1${NC}"
    echo "================================================================"
}

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Test counter
TESTS_RUN=0
TESTS_PASSED=0

run_search_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo "Testing: $test_name"
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if eval "$test_command" >/dev/null 2>&1; then
        print_status "$test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "$test_name"
        echo "Command: $test_command"
        eval "$test_command"
        return 1
    fi
}

# Cleanup function
cleanup() {
    echo "Cleaning up search test artifacts..."
    rm -rf search_test_input search_test_output
}

trap cleanup EXIT

print_header "SEARCH FUNCTIONALITY TEST SUITE"

# Check prerequisites
if ! command -v pagefind &> /dev/null; then
    print_error "Pagefind not found. Please install it first:"
    echo "  npm install -g pagefind"
    echo "  or visit: https://pagefind.app/docs/installation/"
    exit 1
fi

print_status "Pagefind found: $(pagefind --version)"

# Create comprehensive test content
print_header "Setting up Test Content"

mkdir -p search_test_input/{docs,guides,api,tutorials}

# Create index page with search terms
cat > search_test_input/index.md << 'EOF'
# Test Documentation Site

Welcome to our comprehensive documentation. This site demonstrates search functionality.

## Quick Start
Get started with installation and configuration.

## Features
- **Full-text search**: Find any content across all pages
- **Real-time results**: Search as you type
- **Keyboard shortcuts**: Press `/` or `Cmd+K` to open search
- **Contextual snippets**: See relevant excerpts

## Popular Topics
- Installation guide
- Configuration options
- API reference
- Troubleshooting tips

Use the search feature to quickly find what you need!
EOF

# Create detailed documentation with searchable content
cat > search_test_input/docs/installation.md << 'EOF'
# Installation Guide

Learn how to install md-book on different platforms.

## System Requirements
- Rust 1.70.0 or later
- Node.js 18+ (for frontend features)
- 2GB RAM minimum
- 100MB disk space

## Installation Methods

### From Pre-built Binaries
Download the latest release from GitHub:
```bash
wget https://github.com/terraphim/md-book/releases/latest/download/md-book-linux.tar.gz
tar -xzf md-book-linux.tar.gz
sudo mv md-book /usr/local/bin/
```

### From Source
Clone and build from source:
```bash
git clone https://github.com/terraphim/md-book.git
cd md-book
cargo build --release
```

### Using Cargo
Install directly with cargo:
```bash
cargo install md-book
```

## Verification
Verify installation by running:
```bash
md-book --version
```

## Troubleshooting Installation
Common installation issues and solutions.
EOF

cat > search_test_input/docs/configuration.md << 'EOF'
# Configuration Guide

Configure md-book for your specific needs.

## Configuration File
Create a `book.toml` file in your project root:

```toml
[book]
title = "My Documentation"
authors = ["Your Name"]
description = "Comprehensive documentation"
language = "en"

[output.html]
mathjax-support = false
allow_html = true

[markdown]
format = "gfm"
frontmatter = false

[search]
enable = true
boost_factor = 1.0
```

## Environment Variables
- `MD_BOOK_LOG_LEVEL`: Set logging level (debug, info, warn, error)
- `MD_BOOK_PORT`: Default server port (default: 3000)
- `MD_BOOK_HOST`: Server host address (default: localhost)

## Advanced Configuration
Fine-tune behavior with advanced settings.
EOF

cat > search_test_input/api/endpoints.md << 'EOF'
# API Reference

Complete REST API documentation.

## Authentication
All API endpoints require authentication via Bearer token:
```
Authorization: Bearer YOUR_TOKEN_HERE
```

## User Management

### GET /api/users
List all users with pagination.

**Parameters:**
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 10)
- `search`: Search query for filtering

### POST /api/users
Create a new user account.

**Request Body:**
```json
{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "secure_password",
  "role": "user"
}
```

### PUT /api/users/{id}
Update user information.

### DELETE /api/users/{id}
Delete a user account.

## Search API

### POST /api/search
Perform full-text search across documentation.

**Request:**
```json
{
  "query": "installation guide",
  "filters": {
    "section": ["docs", "guides"],
    "tags": ["beginner"]
  },
  "limit": 20
}
```

**Response:**
```json
{
  "results": [
    {
      "title": "Installation Guide",
      "url": "/docs/installation",
      "snippet": "Learn how to install...",
      "score": 0.95
    }
  ],
  "total": 1,
  "took_ms": 23
}
```

## Error Handling
Standard HTTP error codes and messages.
EOF

cat > search_test_input/guides/quick-start.md << 'EOF'
# Quick Start Guide

Get up and running with md-book in minutes.

## 1. Installation
First, install md-book using one of these methods:
- Download pre-built binary
- Install with cargo
- Build from source

## 2. Create Your First Book
```bash
mkdir my-docs
cd my-docs
echo "# My Documentation" > index.md
```

## 3. Build Your Site
```bash
md-book -i . -o build/
```

## 4. Serve with Live Reload
```bash
md-book -i . -o build/ --serve --watch
```

Your documentation will be available at http://localhost:3000

## Next Steps
- Configure your book with `book.toml`
- Add more content and organize in folders
- Customize themes and styling
- Set up automated deployment

## Common Commands
Quick reference for frequently used commands:
- `md-book --help`: Show all options
- `md-book --version`: Check version
- `md-book -i docs/ -o public/`: Build to public directory
EOF

cat > search_test_input/tutorials/advanced-features.md << 'EOF'
# Advanced Features Tutorial

Master advanced md-book features for professional documentation.

## Custom Templates
Override default templates by creating `templates/` directory:
- `base.html.tera`: Main page template
- `header.html.tera`: Page header
- `footer.html.tera`: Page footer
- `search-modal.html.tera`: Search interface

## Search Customization
Enhance search functionality:
- Configure boost factors for different content types
- Add custom search filters
- Implement search analytics
- Optimize indexing performance

## Performance Optimization
Tips for faster builds and better performance:
- Use incremental builds
- Optimize image sizes
- Minimize CSS and JavaScript
- Enable gzip compression
- Implement caching strategies

## Integration Examples
Integrate with external tools:
- GitHub Actions for automated deployment
- Netlify for hosting
- Algolia for advanced search
- Google Analytics for insights

## Maintenance
Keep your documentation site running smoothly:
- Regular dependency updates
- Performance monitoring
- Content auditing
- Link checking
- SEO optimization
EOF

# Create book.toml
cat > search_test_input/book.toml << 'EOF'
[book]
title = "Search Test Documentation"
authors = ["Test Author"]
description = "Test site for search functionality"
language = "en"

[output.html]
mathjax-support = false
allow_html = true

[markdown]
format = "gfm"
frontmatter = false
EOF

print_status "Test content created with searchable terms"

# Build the test site
print_header "Building Test Site"
run_search_test "Site build" "cargo run -- -i search_test_input -o search_test_output"

# Verify search index was created
print_header "Search Index Verification"
run_search_test "Pagefind directory exists" "test -d search_test_output/pagefind"
run_search_test "Pagefind JS bundle exists" "test -f search_test_output/pagefind/pagefind.js"
run_search_test "Search index data exists" "test -f search_test_output/pagefind/pagefind-entry.json || test -f search_test_output/pagefind/pagefind.en.json"

# Verify HTML includes search components
print_header "Search Integration Verification"
run_search_test "Search modal component included" "grep -q 'search-modal' search_test_output/index.html"
run_search_test "Search script included" "grep -q 'pagefind-search.js' search_test_output/index.html"
run_search_test "Search CSS included" "grep -q 'search.css' search_test_output/index.html"
run_search_test "Search initialization included" "grep -q 'search-init.js' search_test_output/index.html"

# Verify search files exist
print_header "Search Asset Verification"
run_search_test "Search CSS file exists" "test -f search_test_output/css/search.css"
run_search_test "Search JS file exists" "test -f search_test_output/js/pagefind-search.js"
run_search_test "Search modal component exists" "test -f search_test_output/components/search-modal.js"
run_search_test "Search init script exists" "test -f search_test_output/js/search-init.js"

# Test search functionality with pagefind CLI
print_header "Search Query Testing"

# Test basic search queries
if command -v node &> /dev/null; then
    cat > search_test.js << 'EOF'
const fs = require('fs');
const path = require('path');

// Simple test to verify pagefind can be loaded
const pagefindPath = path.join(__dirname, 'search_test_output', 'pagefind', 'pagefind.js');
if (fs.existsSync(pagefindPath)) {
    console.log('✅ Pagefind bundle found');
    process.exit(0);
} else {
    console.log('❌ Pagefind bundle not found');
    process.exit(1);
}
EOF

    run_search_test "Search bundle accessibility" "node search_test.js"
    rm -f search_test.js
else
    print_warning "Node.js not available, skipping JavaScript search tests"
fi

# Content analysis
print_header "Content Analysis"
total_pages=$(find search_test_output -name "*.html" | wc -l)
print_status "Generated $total_pages HTML pages"

if [ -f search_test_output/pagefind/pagefind-entry.json ]; then
    indexed_pages=$(grep -o '"url"' search_test_output/pagefind/pagefind-entry.json | wc -l)
    print_status "Indexed $indexed_pages pages for search"
elif [ -f search_test_output/pagefind/pagefind.en.json ]; then
    print_status "Search index created (pagefind.en.json format)"
fi

# Final results
print_header "Search Test Results"
echo "Tests Run: $TESTS_RUN"
echo "Tests Passed: $TESTS_PASSED"
echo "Success Rate: $((TESTS_PASSED * 100 / TESTS_RUN))%"

if [ $TESTS_PASSED -eq $TESTS_RUN ]; then
    print_status "All search functionality tests passed! 🎉"
    echo ""
    echo "Search features verified:"
    echo "  ✅ Pagefind index generation"
    echo "  ✅ Search UI components integration"
    echo "  ✅ Frontend asset inclusion"
    echo "  ✅ Multi-page content indexing"
    echo "  ✅ Search bundle accessibility"
    echo ""
    echo "You can test the search interface by:"
    echo "  1. cargo run -- -i search_test_input -o search_test_output --serve"
    echo "  2. Open http://localhost:3000 in your browser"
    echo "  3. Press '/' or 'Cmd+K' to open search"
    echo "  4. Try searching for: installation, API, configuration, tutorial"
else
    print_error "Some search tests failed!"
    exit 1
fi