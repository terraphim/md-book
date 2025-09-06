# Testing Guide for md-book

This directory contains comprehensive tests for the md-book project, designed to ensure robust functionality across all features and use cases.

## Test Structure

### Directory Organization

```
tests/
├── README.md              # This file
├── common/                # Shared test utilities
│   └── mod.rs             # Test helper functions and macros
├── integration/           # Integration tests
│   └── build_test.rs      # Tests for complete build process
├── e2e/                   # End-to-end tests
│   └── cli_test.rs        # Command-line interface tests
└── assets/                # Test assets and fixtures
    └── test_book_1/       # Sample book for testing
        ├── book.toml      # Test configuration
        └── src/           # Test markdown files
```

### Test Types

#### 1. Unit Tests (in `src/` modules)
- **Location**: Embedded in source files using `#[cfg(test)]`
- **Coverage**: Individual functions and modules
- **Examples**: 
  - `config.rs`: Configuration loading, parsing, defaults
  - `core.rs`: Markdown processing, title extraction, rendering
  - `pagefind_service.rs`: Search functionality (with `search` feature)

#### 2. Integration Tests (`tests/integration/`)
- **Purpose**: Test complete workflows and feature interactions
- **Key Files**:
  - `build_test.rs`: Complete book building process
  - Tests various markdown formats (Markdown, GFM, MDX)
  - Tests configuration scenarios
  - Tests asset handling

#### 3. End-to-End Tests (`tests/e2e/`)
- **Purpose**: Test the complete application from CLI perspective
- **Key Files**:
  - `cli_test.rs`: Command-line argument parsing and execution
  - Tests real CLI scenarios users would encounter
  - Tests error handling and edge cases

#### 4. Test Assets (`tests/assets/`)
- **Purpose**: Provide realistic test data
- **Contents**:
  - Sample book structures
  - Various markdown examples
  - Configuration files
  - Expected outputs

## Running Tests

### All Tests
```bash
cargo test
```

### Unit Tests Only
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test integration
```

### End-to-End Tests  
```bash
cargo test --test e2e
```

### Feature-Specific Tests

#### Without Optional Features
```bash
cargo test --no-default-features
```

#### With Specific Features
```bash
cargo test --features="search,syntax-highlighting"
cargo test --features="server,watcher"
```

#### WASM Tests
```bash
# For WebAssembly target
cargo test --target wasm32-unknown-unknown --features wasm
```

### Test Debugging

#### Run with Output
```bash
cargo test -- --nocapture
```

#### Run Specific Test
```bash
cargo test test_build_simple_book
```

#### Show Ignored Tests
```bash
cargo test -- --ignored
```

## Test Guidelines

### Writing Tests

1. **Use Test Utilities**: Leverage helpers in `tests/common/mod.rs`
   ```rust
   use common::*;
   let book = create_simple_book()?;
   book.build().await?;
   assert!(book.output_exists("README.html"));
   ```

2. **Feature Gates**: Use appropriate feature flags
   ```rust
   #[cfg(feature = "search")]
   #[test]
   fn test_search_functionality() { ... }
   ```

3. **Async Tests**: Use proper async test attributes
   ```rust
   #[tokio::test]
   async fn test_async_build() { ... }
   ```

4. **Temporary Files**: Always use `tempfile::TempDir` for test isolation
   ```rust
   let temp_dir = TempDir::new()?;
   let input_dir = temp_dir.path().join("src");
   ```

### Test Patterns

#### Assertion Macros
- `assert_contains!(text, pattern)` - Check if text contains pattern
- `assert_not_contains!(text, pattern)` - Check if text doesn't contain pattern

#### Error Testing
```rust
let result = some_operation();
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("expected error"));
```

#### File System Testing
```rust
assert!(output_dir.join("file.html").exists());
let content = fs::read_to_string(output_path.join("file.html"))?;
assert_contains!(content, "<h1>Expected Title</h1>");
```

## Test Coverage Areas

### Core Functionality
- ✅ Configuration loading (TOML, JSON, environment variables)
- ✅ Markdown processing (basic, GFM, MDX formats)
- ✅ HTML generation and templating
- ✅ File system operations and asset copying
- ✅ Title extraction and navigation building

### Feature-Specific
- ✅ Syntax highlighting (with `syntax-highlighting` feature)
- ✅ Search integration (with `search` feature)  
- ✅ Development server (with `server` feature)
- ✅ File watching (with `watcher` feature)
- ✅ WASM compatibility (with `wasm` target)

### Edge Cases and Error Handling
- ✅ Invalid input directories
- ✅ Malformed configuration files
- ✅ Permission errors
- ✅ Empty or missing files
- ✅ Invalid markdown content

### Cross-Platform Compatibility
- ✅ Path handling (Windows vs Unix)
- ✅ File permissions
- ✅ Line ending handling
- ✅ Unicode content

## Current Test Status

### Passing Tests
- Configuration parsing and defaults ✅
- Basic markdown processing ✅  
- Title extraction ✅
- Page data serialization ✅
- CLI argument parsing ✅
- Feature-gated functionality ✅

### Known Issues (Being Fixed)
- Some config default tests need adjustment 🔧
- Asset copying tests depend on template structure 🔧
- H2 title extraction needs implementation 🔧

### Ignored Tests
- MathJax support (not implemented yet) ⏸️

## Continuous Integration

### GitHub Actions
The project uses GitHub Actions for automated testing across:
- Multiple Rust versions (stable, beta, nightly)
- Multiple platforms (Linux, macOS, Windows)
- Different feature combinations
- WASM target compilation

### Test Commands in CI
```bash
# Format and lint
cargo fmt --check
cargo clippy -- -D warnings

# Core tests
cargo test --lib
cargo test --test integration
cargo test --test e2e

# Feature tests
cargo test --no-default-features
cargo test --all-features

# WASM tests
cargo test --target wasm32-unknown-unknown --features wasm
```

## Contributing

When adding new features:

1. **Add Unit Tests**: Test individual functions in the source module
2. **Add Integration Tests**: Test the feature in realistic scenarios
3. **Update Test Assets**: Add sample content if needed
4. **Test All Features**: Ensure compatibility with feature combinations
5. **Update Documentation**: Document any new test utilities or patterns

### Test Checklist
- [ ] Unit tests for new functions
- [ ] Integration tests for new features
- [ ] Feature gate compatibility
- [ ] Error case handling
- [ ] Cross-platform compatibility
- [ ] Documentation updates

## Performance Testing

### Benchmarks
```bash
cargo bench
```

Located in `benches/` directory, benchmarks test:
- Build performance on large books
- Markdown processing speed
- Search indexing performance
- Template rendering speed

### Load Testing
For server functionality:
```bash
# With server feature enabled
cargo run -- --serve --port 3000
# Use external tools like wrk or apache bench for load testing
```

This comprehensive test suite ensures md-book remains reliable, performant, and compatible across all supported platforms and use cases.