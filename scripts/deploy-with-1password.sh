#!/bin/bash

# Deploy md-book to Cloudflare Pages using 1Password credentials
# This script uses op run to inject Cloudflare credentials from 1Password

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if op (1Password CLI) is installed
if ! command -v op &> /dev/null; then
    print_error "1Password CLI (op) is not installed. Please install it first."
    exit 1
fi

# Check if user is signed into 1Password
if ! op account list &> /dev/null; then
    print_error "You are not signed into 1Password. Please run 'op account add' first."
    exit 1
fi

# Check if wrangler is installed
if ! command -v wrangler &> /dev/null; then
    print_error "Wrangler CLI is not installed. Please install it first."
    exit 1
<arg_value>
print_status "Deploying md-book to Cloudflare Pages with 1Password credentials..."

# Build the project first
print_status "Building the project..."
cargo run -- -i book -o dist

if [ $? -ne 0 ]; then
    print_error "Build failed. Please fix the errors before deploying."
    exit 1
fi

print_status "Build completed successfully."

# Deploy using op run to inject credentials
print_status "Deploying to Cloudflare Pages..."
op run --no-masking -- \
    CLOUDFLARE_ACCOUNT_ID="op://TerraphimPlatform/md-book-cloudflare/account_id" \
    CLOUDFLARE_API_TOKEN="op://TerraphimPlatform/md-book-cloudflare/api_token" \
    wrangler pages deploy dist --project-name md-book

if [ $? -eq 0 ]; then
    print_status "Deployment completed successfully!"
    print_status "Your site is now live at: https://md-book.pages.dev"
else
    print_error "Deployment failed. Please check the error messages above."
    exit 1
fi