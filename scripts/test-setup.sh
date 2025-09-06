#!/bin/bash
set -euo pipefail

# Test Setup Script
# Prepares the environment for testing md-book

echo "🔧 Setting up test environment for md-book..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust toolchain."
    exit 1
fi

print_status "Cargo found: $(cargo --version)"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Install required tools for testing
echo "📦 Installing required tools..."

# Install pagefind for search functionality tests
if ! command -v pagefind &> /dev/null; then
    print_warning "Pagefind not found. Installing..."
    # Try to install pagefind
    if command -v npm &> /dev/null; then
        npm install -g pagefind
    else
        print_error "npm not found. Please install pagefind manually: https://pagefind.app/docs/installation/"
        exit 1
    fi
else
    print_status "Pagefind found: $(pagefind --version)"
fi

# Install cargo-tarpaulin for coverage
if ! command -v cargo-tarpaulin &> /dev/null; then
    print_warning "cargo-tarpaulin not found. Installing for coverage reports..."
    cargo install cargo-tarpaulin
else
    print_status "cargo-tarpaulin found"
fi

# Install wasm-pack for WASM tests
if ! command -v wasm-pack &> /dev/null; then
    print_warning "wasm-pack not found. Installing for WASM tests..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
else
    print_status "wasm-pack found: $(wasm-pack --version)"
fi

# Install cargo-deny for security audits
if ! command -v cargo-deny &> /dev/null; then
    print_warning "cargo-deny not found. Installing for security audits..."
    cargo install cargo-deny
else
    print_status "cargo-deny found: $(cargo-deny --version)"
fi

# Create test directories
echo "📁 Creating test directories..."
mkdir -p test_input/docs
mkdir -p test_input/guides
mkdir -p test_output
mkdir -p test_temp

# Create sample test content
cat > test_input/index.md << 'EOF'
# Test Documentation

This is a test documentation site for md-book.

## Features
- Full-text search
- Live reload in serve mode
- Responsive design
- Syntax highlighting

Search for "test" to verify search functionality.
EOF

cat > test_input/docs/installation.md << 'EOF'
# Installation Guide

Learn how to install md-book on your system.

## Prerequisites
- Rust toolchain
- Node.js (for frontend dependencies)

## Building from Source
```bash
git clone https://github.com/terraphim/md-book.git
cd md-book
cargo build --release
```

The binary will be available at `target/release/md-book`.
EOF

cat > test_input/guides/usage.md << 'EOF'
# Usage Guide

This guide covers how to use md-book effectively.

## Basic Usage

### Building a Site
```bash
md-book -i input/ -o output/
```

### Serving with Live Reload
```bash
md-book -i input/ -o output/ --serve --watch
```

## Configuration

Create a `book.toml` file in your input directory:

```toml
[book]
title = "My Documentation"
authors = ["Your Name"]

[markdown]
format = "gfm"
```
EOF

cat > test_input/book.toml << 'EOF'
[book]
title = "Test Documentation"
authors = ["Test Author"]
description = "Test documentation for md-book"
language = "en"

[output.html]
mathjax-support = false

[markdown]
format = "gfm"
EOF

print_status "Test content created in test_input/"

# Install frontend dependencies
if [ -f "package.json" ]; then
    echo "📦 Installing frontend dependencies..."
    npm install
    print_status "Frontend dependencies installed"
fi

# Run initial checks
echo "🔍 Running initial checks..."

# Check Rust formatting
if cargo fmt --check > /dev/null 2>&1; then
    print_status "Code formatting is correct"
else
    print_warning "Code formatting issues detected. Run 'cargo fmt' to fix."
fi

# Run Clippy
echo "Running clippy..."
if cargo clippy --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
    print_status "Clippy checks passed"
else
    print_warning "Clippy warnings detected. Review with 'cargo clippy'"
fi

echo ""
echo "🎉 Test environment setup complete!"
echo ""
echo "Available test commands:"
echo "  ./scripts/run-tests.sh          - Run all tests"
echo "  ./scripts/test-search.sh        - Test search functionality"
echo "  ./scripts/benchmark.sh          - Run benchmarks"
echo "  cargo test                      - Run unit tests"
echo "  cargo test --test e2e           - Run e2e tests"
echo "  cargo test --test integration   - Run integration tests"
echo ""