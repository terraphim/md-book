#!/bin/bash

# Comprehensive deployment script for MD-Book
# This script builds and deploys both the static site and worker to Cloudflare

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT=${1:-production}
INPUT_DIR=${2:-test_input}
OUTPUT_DIR=${3:-dist}
SKIP_TESTS=${SKIP_TESTS:-false}
DEPLOY_WORKER=${DEPLOY_WORKER:-true}
USE_1PASSWORD=${USE_1PASSWORD:-auto}

echo -e "${BLUE}🚀 MD-Book Deployment Pipeline${NC}"
echo -e "${BLUE}================================${NC}"
echo "Environment: $ENVIRONMENT"
echo "Input Dir: $INPUT_DIR"
echo "Output Dir: $OUTPUT_DIR"
echo "Skip Tests: $SKIP_TESTS"
echo "Deploy Worker: $DEPLOY_WORKER"
echo "1Password: $USE_1PASSWORD"
echo ""

# Load secrets from 1Password if available
load_secrets_from_1password() {
    # Skip if explicitly disabled
    if [ "$USE_1PASSWORD" = "false" ] || [ "$USE_1PASSWORD" = "no" ]; then
        return 0
    fi
    
    # Check if 1Password CLI is available
    if ! command -v op &> /dev/null; then
        if [ "$USE_1PASSWORD" = "true" ] || [ "$USE_1PASSWORD" = "yes" ]; then
            echo -e "${RED}❌ 1Password CLI required but not found${NC}"
            echo "  Install with: brew install 1password-cli"
            exit 1
        fi
        return 0
    fi
    
    # Check if authenticated
    if ! op account list &> /dev/null; then
        if [ "$USE_1PASSWORD" = "true" ] || [ "$USE_1PASSWORD" = "yes" ]; then
            echo -e "${RED}❌ Not authenticated with 1Password${NC}"
            echo "  Run: op signin"
            exit 1
        fi
        return 0
    fi
    
    # Try to load secrets from 1Password
    echo -e "${YELLOW}🔐 Loading secrets from 1Password...${NC}"
    
    # Set vault and item names
    VAULT_NAME="MD-Book-Deployment"
    CLOUDFLARE_ITEM="Cloudflare"
    
    # Try to read secrets
    if op vault get "$VAULT_NAME" &> /dev/null && op item get "$CLOUDFLARE_ITEM" --vault="$VAULT_NAME" &> /dev/null; then
        # Load API token if not already set
        if [ -z "$CLOUDFLARE_API_TOKEN" ]; then
            if CLOUDFLARE_API_TOKEN=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/api_token" 2>/dev/null); then
                export CLOUDFLARE_API_TOKEN
                echo -e "${GREEN}✅ Loaded CLOUDFLARE_API_TOKEN from 1Password${NC}"
            fi
        fi
        
        # Load Account ID if not already set
        if [ -z "$CLOUDFLARE_ACCOUNT_ID" ]; then
            if CLOUDFLARE_ACCOUNT_ID=$(op read "op://$VAULT_NAME/$CLOUDFLARE_ITEM/account_id" 2>/dev/null); then
                export CLOUDFLARE_ACCOUNT_ID
                echo -e "${GREEN}✅ Loaded CLOUDFLARE_ACCOUNT_ID from 1Password${NC}"
            fi
        fi
    else
        if [ "$USE_1PASSWORD" = "true" ] || [ "$USE_1PASSWORD" = "yes" ]; then
            echo -e "${RED}❌ Cannot access 1Password vault or Cloudflare item${NC}"
            echo "  Run: ./scripts/setup-1password.sh"
            exit 1
        else
            echo -e "${YELLOW}⚠️  1Password vault not accessible, falling back to environment variables${NC}"
        fi
    fi
}

# Check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}🔍 Checking prerequisites...${NC}"
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust/Cargo not found. Please install Rust.${NC}"
        exit 1
    fi
    
    # Check Node.js for Cloudflare CLI
    if ! command -v node &> /dev/null; then
        echo -e "${RED}❌ Node.js not found. Please install Node.js.${NC}"
        exit 1
    fi
    
    # Check if input directory exists
    if [ ! -d "$INPUT_DIR" ]; then
        echo -e "${YELLOW}⚠️  Input directory not found, creating minimal example...${NC}"
        mkdir -p "$INPUT_DIR"
        cat > "$INPUT_DIR/index.md" << 'EOF'
# Welcome to MD-Book

This is a modern mdbook replacement written in Rust that generates beautiful HTML documentation from Markdown files.

## Features

- **Fast**: Built with Rust for maximum performance
- **Modern**: Uses contemporary web technologies
- **Flexible**: Support for multiple markdown formats
- **Search**: Integrated full-text search with Pagefind
- **Live Reload**: Development server with automatic refresh
- **Customizable**: Themeable with custom CSS and JavaScript

## Quick Start

1. Install MD-Book
2. Create your content in Markdown
3. Run the build command
4. Deploy to any static hosting service

Start writing your documentation in Markdown and let MD-Book handle the rest!
EOF
        
        cat > "$INPUT_DIR/getting-started.md" << 'EOF'
# Getting Started

Learn how to use MD-Book to create beautiful documentation sites.

## Installation

You can install MD-Book by cloning the repository and building from source:

```bash
git clone https://github.com/terraphim/md-book.git
cd md-book
cargo build --release
```

## Building Your Documentation

1. Create a directory for your Markdown files
2. Add your content as `.md` files
3. Run the build command:

```bash
./target/release/md-book -i input_dir -o output_dir
```

## Development Mode

For development, you can use the watch and serve modes:

```bash
./target/release/md-book -i input_dir -o output_dir --watch --serve
```

This will start a development server at `http://localhost:3000` with live reload.

## Configuration

MD-Book can be configured using a `book.toml` file in your project root. See the example configuration for available options.
EOF
    fi
    
    echo -e "${GREEN}✅ Prerequisites check passed${NC}"
}

# Run tests
run_tests() {
    if [ "$SKIP_TESTS" = "true" ]; then
        echo -e "${YELLOW}⚠️  Skipping tests as requested${NC}"
        return
    fi
    
    echo -e "${YELLOW}🧪 Running tests...${NC}"
    
    # Format check
    echo "Checking code formatting..."
    cargo fmt --all -- --check
    
    # Clippy lints
    echo "Running clippy..."
    cargo clippy --all-targets --all-features -- -D warnings
    
    # Unit tests
    echo "Running unit tests..."
    cargo test --lib --bins
    
    # Install and run frontend tests if available
    if [ -f "package.json" ]; then
        echo "Running frontend tests..."
        if command -v bun &> /dev/null; then
            bun install
            bun test
        elif command -v npm &> /dev/null; then
            npm install
            npm test
        fi
    fi
    
    echo -e "${GREEN}✅ All tests passed${NC}"
}

# Build the project
build_project() {
    echo -e "${YELLOW}🔨 Building MD-Book...${NC}"
    
    # Clean previous builds
    cargo clean
    rm -rf "$OUTPUT_DIR"
    
    # Build release binary
    echo "Building optimized release binary..."
    cargo build --release --all-features
    
    # Generate static site
    echo "Generating static site..."
    ./target/release/md-book -i "$INPUT_DIR" -o "$OUTPUT_DIR"
    
    # Verify build output
    if [ ! -d "$OUTPUT_DIR" ]; then
        echo -e "${RED}❌ Build output directory not found: $OUTPUT_DIR${NC}"
        exit 1
    fi
    
    # Check for essential files
    if [ ! -f "$OUTPUT_DIR/index.html" ]; then
        echo -e "${RED}❌ index.html not found in output directory${NC}"
        exit 1
    fi
    
    # Create 404 page if it doesn't exist
    if [ ! -f "$OUTPUT_DIR/404.html" ]; then
        echo "Creating 404 page..."
        cat > "$OUTPUT_DIR/404.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Page Not Found - MD-Book</title>
    <link rel="stylesheet" href="/css/variables.css">
    <link rel="stylesheet" href="/css/general.css">
    <link rel="stylesheet" href="/css/chrome.css">
</head>
<body>
    <div id="wrapper" class="page-wrapper">
        <div class="page">
            <main>
                <h1>Page Not Found</h1>
                <p>The page you're looking for doesn't exist.</p>
                <p><a href="/">Return to the homepage</a></p>
            </main>
        </div>
    </div>
</body>
</html>
EOF
    fi
    
    echo -e "${GREEN}✅ Build completed successfully${NC}"
    echo "Generated $(find "$OUTPUT_DIR" -type f | wc -l) files"
}

# Deploy to Cloudflare Pages
deploy_pages() {
    echo -e "${YELLOW}🌐 Deploying to Cloudflare Pages...${NC}"
    
    # Install wrangler if not available
    if ! command -v wrangler &> /dev/null; then
        echo "Installing Wrangler CLI..."
        npm install -g wrangler
    fi
    
    # Load secrets from 1Password if available
    load_secrets_from_1password
    
    # Check required environment variables
    if [ -z "$CLOUDFLARE_API_TOKEN" ] || [ -z "$CLOUDFLARE_ACCOUNT_ID" ]; then
        echo -e "${RED}❌ Missing required environment variables:${NC}"
        echo "   CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID must be set"
        echo ""
        echo "   Get your API token from: https://dash.cloudflare.com/profile/api-tokens"
        echo "   Get your Account ID from: https://dash.cloudflare.com/"
        exit 1
    fi
    
    # Deploy based on environment
    case $ENVIRONMENT in
        "production")
            echo "Deploying to production..."
            wrangler pages deploy "$OUTPUT_DIR" \
                --project-name=md-book \
                --compatibility-date=2024-09-06
            ;;
        "staging")
            echo "Deploying to staging..."
            wrangler pages deploy "$OUTPUT_DIR" \
                --project-name=md-book-staging \
                --compatibility-date=2024-09-06
            ;;
        *)
            echo -e "${RED}❌ Unknown environment: $ENVIRONMENT${NC}"
            echo "   Supported environments: production, staging"
            exit 1
            ;;
    esac
    
    echo -e "${GREEN}✅ Pages deployment completed${NC}"
}

# Deploy worker
deploy_worker() {
    if [ "$DEPLOY_WORKER" != "true" ]; then
        echo -e "${YELLOW}⚠️  Skipping worker deployment${NC}"
        return
    fi
    
    echo -e "${YELLOW}⚙️  Deploying Cloudflare Worker...${NC}"
    
    if [ -f "scripts/deploy-worker.sh" ]; then
        ./scripts/deploy-worker.sh "$ENVIRONMENT"
    else
        echo -e "${YELLOW}⚠️  Worker deployment script not found, skipping...${NC}"
    fi
}

# Show deployment summary
show_summary() {
    echo ""
    echo -e "${BLUE}🎉 Deployment Summary${NC}"
    echo -e "${BLUE}===================${NC}"
    echo "Environment: $ENVIRONMENT"
    echo "Build completed: $(date)"
    echo "Files deployed: $(find "$OUTPUT_DIR" -type f | wc -l)"
    echo ""
    
    case $ENVIRONMENT in
        "production")
            echo -e "${GREEN}🔗 Live URL: https://md-book.pages.dev${NC}"
            ;;
        "staging")
            echo -e "${YELLOW}🔗 Staging URL: https://md-book-staging.pages.dev${NC}"
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}✅ Deployment completed successfully!${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}Starting deployment process...${NC}"
    
    check_prerequisites
    run_tests
    build_project
    deploy_pages
    deploy_worker
    show_summary
    
    echo -e "${GREEN}🚀 All done! Your MD-Book documentation is now live.${NC}"
}

# Handle script arguments and help
case ${1:-""} in
    "-h"|"--help"|"help")
        echo "MD-Book Deployment Script"
        echo ""
        echo "Usage: $0 [environment] [input_dir] [output_dir]"
        echo ""
        echo "Arguments:"
        echo "  environment    Deployment environment (production|staging) [default: production]"
        echo "  input_dir      Input directory with Markdown files [default: test_input]"
        echo "  output_dir     Output directory for generated HTML [default: dist]"
        echo ""
        echo "Environment Variables:"
        echo "  SKIP_TESTS           Skip running tests [default: false]"
        echo "  DEPLOY_WORKER        Deploy Cloudflare Worker [default: true]"
        echo "  USE_1PASSWORD        Use 1Password for secrets (auto|true|false) [default: auto]"
        echo "  CLOUDFLARE_API_TOKEN Required Cloudflare API token (or from 1Password)"
        echo "  CLOUDFLARE_ACCOUNT_ID Required Cloudflare Account ID (or from 1Password)"
        echo ""
        echo "1Password Integration:"
        echo "  This script can automatically load secrets from 1Password if available."
        echo "  Set USE_1PASSWORD=false to disable, or run ./scripts/setup-1password.sh first."
        echo ""
        echo "Examples:"
        echo "  $0 production                    # Auto-detect 1Password"
        echo "  $0 staging docs dist             # Use custom directories"
        echo "  SKIP_TESTS=true $0 production    # Skip tests"
        echo "  USE_1PASSWORD=false $0 production # Force environment variables"
        echo ""
        echo "1Password Usage:"
        echo "  op run --env-file=.env.1password -- $0 production"
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac