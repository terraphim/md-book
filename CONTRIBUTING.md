# Contributing to MD-Book

We welcome contributions to MD-Book! This document provides guidelines for contributing to the project.

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (check with `rustc --version`)
- Git
- Basic familiarity with Rust and web technologies

### Development Setup

```bash
# Clone the repository
git clone https://github.com/terraphim/md-book.git
cd md-book

# Install development dependencies
cargo install cargo-watch

# Run tests to ensure everything works
cargo test --all

# Build the project
cargo build

# Run locally
cargo run -- -i demo-docs -o demo-output --serve --watch
```

### Development Workflow

```bash
# Watch for changes and run tests automatically
cargo watch -x test

# Run integration tests with all features
cargo test --test integration --features "tokio,search,syntax-highlighting"

# Run E2E tests
cargo test --test e2e --features "tokio,search,syntax-highlighting"

# Run quality checks
make qa
```

## 📋 Contribution Types

### 🐛 Bug Reports

Found a bug? Please report it by:

1. **Search existing issues** to avoid duplicates
2. **Create a new issue** using the bug report template
3. **Provide minimal reproduction** with:
   - MD-Book version (`md-book --version`)
   - Operating system
   - Rust version
   - Sample code/files
   - Expected vs actual behavior

**Bug Report Template:**
```markdown
## Bug Description
Brief description of the issue

## Reproduction Steps
1. `md-book -i docs -o output`
2. Open generated site
3. Observe bug

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- MD-Book: 0.1.1
- OS: macOS 13.0
- Rust: 1.70.0
```

### ✨ Feature Requests

Have an idea? We'd love to hear it!

1. **Check existing issues** and feature requests
2. **Create a new issue** using the feature request template
3. **Describe the use case** and proposed solution

**Feature Request Template:**
```markdown
## Problem Statement
What problem does this solve?

## Proposed Solution
How should this work?

## Alternatives Considered
What other approaches did you consider?

## Additional Context
Any other relevant information
```

### 🧪 Code Contributions

#### Areas for Contribution

1. **Core Features** (`src/`)
   - Build process and template rendering
   - Search integration and indexing
   - Development server and live reload
   - Configuration system

2. **Templates and Styling** (`src/templates/`)
   - UI/UX improvements
   - Theme customization options
   - Responsive design enhancements
   - Web Components

3. **Documentation** (`demo-docs/`, `README.md`)
   - User documentation improvements
   - API documentation
   - Examples and tutorials
   - Deployment guides

4. **Testing** (`tests/`)
   - Unit tests for core functionality
   - Integration tests for workflows
   - E2E tests for user scenarios
   - Performance benchmarks

#### Pull Request Process

1. **Fork** the repository
2. **Create a feature branch**: `git checkout -b feature/your-feature`
3. **Make changes** following our coding standards
4. **Add tests** for new functionality
5. **Run all checks**: `make qa`
6. **Commit changes** using conventional commits
7. **Push to fork**: `git push origin feature/your-feature`
8. **Create Pull Request** with detailed description

#### Coding Standards

**Code Style:**
- Use `cargo fmt` for formatting
- Follow `clippy` recommendations
- Use `Result<T>` for error handling
- Document public APIs with `///` comments

**Example:**
```rust
/// Renders markdown content to HTML
/// 
/// # Arguments
/// 
/// * `content` - The markdown content to render
/// * `config` - Rendering configuration options
/// 
/// # Returns
/// 
/// Returns `Ok(html)` if rendering succeeds, `Err(error)` if it fails
/// 
/// # Examples
/// 
/// ```
/// use md_book::render_markdown;
/// 
/// let html = render_markdown("# Hello", &config)?;
/// assert!(html.contains("<h1>Hello</h1>"));
/// ```
pub fn render_markdown(content: &str, config: &RenderConfig) -> Result<String> {
    // Implementation
}
```

**Error Handling:**
```rust
use anyhow::{Context, Result};

pub fn build_site(config: &Config) -> Result<()> {
    // Use context for better error messages
    std::fs::create_dir_all(&config.output_dir)
        .context("Failed to create output directory")?;
    
    // Use ? operator for error propagation
    process_files(&config.input_dir, &config.output_dir)
        .context("Failed to process files")?;
    
    Ok(())
}
```

## 🧪 Testing

### Test Structure

```
tests/
├── common/                 # Test utilities and fixtures
├── integration/            # Integration tests
├── e2e/                   # End-to-end tests
└── integration/
    ├── build_test.rs        # Build process tests
    ├── mdbook_test_book.rs # Compatibility tests
    └── mdbook_compatibility.rs # Feature compatibility
```

### Running Tests

```bash
# Unit tests only
cargo test --lib --bins

# Integration tests
cargo test --test integration --features "tokio,search,syntax-highlighting"

# E2E tests
cargo test --test e2e --features "tokio,search,syntax-highlighting"

# All tests with coverage
cargo test --all-features

# Single test
cargo test test_render_markdown
```

### Writing Tests

**Unit Test Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_rendering() {
        let result = render_markdown("# Hello", &Config::default());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("<h1>Hello</h1>"));
    }

    #[test]
    fn test_invalid_input() {
        let result = parse_config("invalid::toml");
        assert!(result.is_err());
    }
}
```

**Integration Test Example:**
```rust
#[tokio::test]
async fn test_full_build_process() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create test content
    std::fs::create_dir_all(&input_dir)?;
    std::fs::write(input_dir.join("index.md"), "# Test")?;
    
    // Run build process
    let config = Config::new(&input_dir, &output_dir);
    build_site(&config).await?;
    
    // Verify output
    assert!(output_dir.join("index.html").exists());
    assert!(output_dir.join("pagefind").exists());
    
    Ok(())
}
```

## 📦 Release Process

### Versioning

MD-Book follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backwards compatible)
- **PATCH**: Bug fixes

### Release Checklist

1. **Update version** in `Cargo.toml`
2. **Update CHANGELOG.md** with release notes
3. **Run full test suite**: `make ci-local`
4. **Create Git tag**: `git tag v0.x.x`
5. **Push tag**: `git push origin v0.x.x`
6. **Create GitHub Release**
7. **Deploy to crates.io** (automated via GitHub Actions)

### Automated Releases

The project uses GitHub Actions for automated releases:

```yaml
# .github/workflows/release.yml
on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'
      
jobs:
  create-release:
    # Creates GitHub release
  build-release:
    # Builds binaries for all platforms
  publish-crate:
    # Publishes to crates.io
```

## 🛠️ Development Tools

### Make Commands

```bash
make help          # Show all available commands
make qa             # Run all quality checks
make dev-check      # Complete development check
make ci-local       # Simulate CI locally
make test           # Run unit tests
make test-integration # Run integration tests
make test-all       # Run all tests
make fmt            # Check code formatting
make fmt-fix        # Fix code formatting
make clippy         # Run clippy lints
make clean          # Clean build artifacts
```

### Pre-commit Hooks

Install pre-commit hooks for automatic quality checks:

```bash
make install-pre-commit
# or manually:
./scripts/setup-pre-commit.sh
```

Hooks run:
- `cargo fmt --all -- --check` - Formatting check
- `cargo clippy --all-targets --all-features -- -D warnings` - Linting
- `cargo test --lib --bins` - Unit tests
- `cargo check --all-targets --all-features` - Compilation check

### IDE Configuration

**VS Code (.vscode/settings.json):**
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.loadOutDirsFromCheck": true,
    "rust-analyzer.imports.granularity.group": "module",
    "rust-analyzer.completion.addCallParentheses": true
}
```

**Neovim (.config/nvim/after/plugin/rust.lua):**
```lua
-- Rust development setup
vim.g.rustaceanvim = {
    tools = {
        enable = true,
        test_runner = "cargo",
        cargo_watch = {
            enable = true,
        },
    },
    server = {
        standalone = true,
    },
}
```

## 🎯 Good First Issues

### Beginner-Friendly Areas

1. **Documentation Improvements**
   - Improve README examples
   - Add more code examples
   - Fix typos and grammar

2. **Test Coverage**
   - Add missing unit tests
   - Improve test coverage
   - Add edge case tests

3. **UI/UX Enhancements**
   - Improve mobile responsiveness
   - Add theme customization options
   - Enhance accessibility

### Current Good First Issues

Check the [GitHub Issues](https://github.com/terraphim/md-book/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) for current opportunities.

### Example Contribution Flow

Let's walk through adding a new feature:

#### 1. Choose an Issue

Example: "Add dark mode toggle to documentation"

#### 2. Understand the Codebase

```bash
# Explore relevant files
find src/templates -name "*.html" | xargs grep -l "theme"
find src/templates -name "*.css" | xargs grep -l "dark"
```

#### 3. Implementation

**Add CSS variables** (`src/templates/css/styles.css`):
```css
:root {
    --bg-color: #ffffff;
    --text-color: #333333;
    --primary-color: #0066cc;
}

[data-theme="dark"] {
    --bg-color: #1a1a1a;
    --text-color: #e0e0e0;
    --primary-color: #4d94ff;
}
```

**Add JavaScript toggle** (`src/templates/js/theme-toggle.js`):
```javascript
function toggleTheme() {
    const current = document.documentElement.getAttribute('data-theme');
    const newTheme = current === 'dark' ? 'light' : 'dark';
    
    document.documentElement.setAttribute('data-theme', newTheme);
    localStorage.setItem('theme', newTheme);
}

// Initialize theme from localStorage
document.addEventListener('DOMContentLoaded', () => {
    const savedTheme = localStorage.getItem('theme') || 'light';
    document.documentElement.setAttribute('data-theme', savedTheme);
});
```

**Update template** (`src/templates/page.html.tera`):
```html
<button onclick="toggleTheme()" class="theme-toggle">
    <span id="theme-icon">🌙</span>
</button>

<script src="/js/theme-toggle.js"></script>
```

#### 4. Add Tests

**Unit test** (`src/tests/theme_tests.rs`):
```rust
#[test]
fn test_theme_toggle_persistence() {
    // Test theme is saved to localStorage
    // Test theme is applied correctly
    // Test default theme handling
}
```

#### 5. Update Documentation

Add to configuration documentation:
```toml
[output.html.theme]
enable-dark-mode = true
default-theme = "light"  # light, dark, auto
```

#### 6. Quality Checks

```bash
make qa
cargo test
cargo clippy
```

#### 7. Pull Request

```markdown
## Description
Added dark mode toggle to documentation with:

- CSS custom properties for theme switching
- JavaScript for theme persistence
- Template integration
- Configuration option
- Mobile-friendly toggle button

## Testing
- Manual testing in browsers
- Unit tests for theme logic
- Accessibility testing

## Screenshots
[Add screenshots if applicable]
```

## 🤝 Community Guidelines

### Code of Conduct

We are committed to providing a friendly, safe, and welcoming environment for all participants. Please read our full [Code of Conduct](CODE_OF_CONDUCT.md).

### Communication

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: General questions, ideas
- **Twitter**: @terraphim (announcements)
- **Discord**: For real-time chat (coming soon)

### Getting Help

- **Documentation**: https://md-book.pages.dev
- **GitHub Issues**: For bugs and feature requests
- **Discussions**: For questions and ideas
- **Email**: support@terraphim.io (for support inquiries)

## 📊 Project Goals

### Short Term (v0.2.x)
- [ ] Enhanced theme system
- [ ] MathJax/LaTeX support
- [ ] Improved search capabilities
- [ ] Plugin system improvements

### Medium Term (v0.3.x)
- [ ] Multi-language support
- [ ] Advanced search features
- [ ] Visual editor integration
- [ ] Analytics integration

### Long Term (v1.0.x)
- [ ] Complete mdBook compatibility
- [ ] Comprehensive plugin ecosystem
- [ ] WebAssembly-based rendering
- [ ] Real-time collaboration features

## 🙏 Acknowledgments

Thank you to all contributors who have helped make MD-Book better:

- **Rust Community**: For inspiration and feedback
- **mdBook Team**: For pioneering documentation generation in Rust
- **Tera Team**: For the excellent templating engine
- **Pagefind Team**: For the amazing search library
- **All Contributors**: For bug reports, features, and improvements

## 📄 License

By contributing to MD-Book, you agree that your contributions will be licensed under the [MIT License](LICENSE).

---

Thank you for contributing to MD-Book! Every contribution, no matter how small, helps make the project better for everyone.

Have questions? Feel free to ask in GitHub Discussions or open an issue.