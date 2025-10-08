#!/bin/bash

# Setup pre-commit hooks for md-book development
set -e

echo "🔧 Setting up pre-commit hooks for md-book..."

# Check if pre-commit is installed
if ! command -v pre-commit &> /dev/null; then
    echo "❌ pre-commit is not installed. Installing it now..."
    
    # Try different installation methods
    if command -v pip &> /dev/null; then
        pip install pre-commit
    elif command -v pip3 &> /dev/null; then
        pip3 install pre-commit
    elif command -v brew &> /dev/null; then
        brew install pre-commit
    elif command -v cargo &> /dev/null; then
        cargo install pre-commit
    else
        echo "❌ Could not install pre-commit automatically."
        echo "Please install pre-commit manually: https://pre-commit.com/#installation"
        exit 1
    fi
fi

echo "✅ pre-commit is installed"

# Install the pre-commit hooks
echo "📦 Installing pre-commit hooks..."
pre-commit install

# Run pre-commit on all files to test the setup
echo "🧪 Testing pre-commit hooks..."
pre-commit run --all-files

echo "✅ Pre-commit setup complete!"
echo ""
echo "The following hooks are now active:"
echo "  - cargo fmt (formatting check)"
echo "  - cargo clippy (linting)"
echo "  - cargo test (unit tests)"
echo "  - cargo check (compilation check)"
echo ""
echo "These will run automatically on every commit."
echo "To run them manually: pre-commit run --all-files"