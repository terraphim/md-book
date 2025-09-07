# @lessons-learned.md - Knowledge Retention

## Comprehensive Test Suite Implementation Insights

### Key Technical Learnings

#### 1. Test Architecture Patterns
- **Directory Structure**: Follow established patterns (mdBook's test organization)
- **Test Utilities**: Centralize common testing logic in `tests/common/mod.rs`
- **Feature Gating**: Use `#[cfg(feature = "...")]` for conditional test compilation
- **Test Isolation**: Each test should use `tempfile::TempDir` for clean state

#### 2. Rust Testing Best Practices
- **Async Testing**: `#[tokio::test]` for async functions, avoid blocking operations
- **Error Testing**: Test both success and failure cases with proper error messages
- **Feature Combinations**: Test with `--no-default-features` and `--all-features`
- **Target-Specific Tests**: Use `#[cfg(target_arch = "wasm32")]` for WASM tests

#### 3. Test Utility Design
- **Helper Classes**: Create reusable test structures like `TestBook`
- **Factory Functions**: `create_simple_book()`, `create_complex_book()` for scenarios
- **Custom Macros**: `assert_contains!()`, `assert_not_contains!()` for readable assertions
- **Builder Patterns**: Fluent interfaces for test setup (`.with_config()`, `.build()`)

#### 4. Integration Testing Strategies
- **Real File Operations**: Never mock file system - use temporary directories
- **Complete Workflows**: Test entire build processes, not just units
- **Configuration Scenarios**: Test different config formats and combinations
- **Asset Handling**: Verify static assets are copied correctly

#### 5. End-to-End Testing Approaches  
- **CLI Testing**: Test actual binary execution with `Command::new()`
- **Output Validation**: Check both stdout/stderr and generated files
- **Error Scenarios**: Test invalid arguments, missing files, permission errors
- **Cross-Platform**: Ensure path handling works on Windows and Unix

#### 6. WASM Testing Considerations
- **Target Compilation**: Tests must work with `wasm32-unknown-unknown` target
- **Feature Parity**: WASM and native builds should have identical functionality
- **Browser Compatibility**: Use `wasm-bindgen-test` for browser-specific tests
- **Performance**: WASM may have different performance characteristics

### Test Organization Insights

#### Test Asset Management
- **Realistic Data**: Use comprehensive test books with nested structures
- **Multiple Formats**: Test Markdown, GFM, MDX with real content
- **Configuration Examples**: TOML, JSON configs with various settings
- **Expected Outputs**: Keep fixtures for regression testing

#### Test Execution Patterns
- **Parallel Execution**: Tests should not interfere with each other
- **Resource Cleanup**: Always clean up temporary files and processes
- **Test Speed**: Fast unit tests, slower integration tests
- **CI Optimization**: Group tests by execution time and dependencies

### Configuration Testing Learnings

#### Layered Configuration Testing
- **Precedence Testing**: Environment vars > CLI args > config files > defaults
- **Validation Testing**: Invalid configs should fail gracefully
- **Format Support**: TOML, JSON, YAML all need validation
- **Default Behavior**: Empty configs should use sensible defaults

#### Common Configuration Issues
- **Default Values**: Ensure default functions return expected values
- **Struct Fields**: All config fields need proper `#[serde(default)]` attributes
- **Validation Logic**: Complex validation rules need thorough testing
- **Error Messages**: Config errors should be user-friendly

### Testing Markdown Processing

#### Format-Specific Testing
- **Basic Markdown**: Standard CommonMark compliance
- **GFM Extensions**: Tables, strikethrough, task lists, autolinks
- **MDX Support**: JSX-like syntax in markdown files
- **Frontmatter**: YAML/TOML metadata extraction and processing

#### Content Edge Cases
- **Empty Files**: Handle gracefully without errors
- **Invalid Markdown**: Don't crash on malformed input
- **Large Files**: Performance testing with substantial content
- **Unicode Content**: Proper encoding and character handling

### Development Workflow Improvements

#### Test-Driven Development
- **Write Tests First**: Define expected behavior before implementation
- **Red-Green-Refactor**: Failing test → implementation → cleanup
- **Coverage Goals**: Aim for >80% unit test coverage
- **Integration Focus**: Critical paths need integration tests

#### Continuous Integration Optimization
- **Test Grouping**: Unit → Integration → E2E execution order
- **Parallel Execution**: Run independent test suites concurrently
- **Failure Analysis**: Clear error messages and debug information
- **Performance Tracking**: Monitor test execution times

### Common Testing Pitfalls Avoided

#### Test Implementation Issues
- **Shared State**: Tests must be independent and isolated
- **Hardcoded Paths**: Use relative paths and temporary directories
- **Feature Dependencies**: Tests should work with various feature combinations
- **Platform Assumptions**: Don't assume Unix-only or Windows-only behavior

#### Assertion Best Practices
- **Specific Assertions**: Test exact expected behavior, not just "no error"
- **Readable Messages**: Custom assertion macros improve debugging
- **Multiple Assertions**: Test different aspects of the same functionality
- **Error Case Testing**: Verify error conditions produce correct errors

### Performance Testing Insights
- **Benchmarking Framework**: Use `criterion` for statistical analysis
- **Real-World Data**: Test with realistic book sizes and content
- **Memory Profiling**: Monitor memory usage during operations
- **Regression Detection**: Track performance over time

### Previous Learning: Pagefind Search Implementation

#### Search Integration Architecture
- **Static-first Design**: Pagefind generates static search bundles
- **Chunked Indexes**: Efficient loading of search data
- **WASM Compatibility**: WebAssembly support with proper CSP
- **Performance Targets**: <2s indexing for 1000 pages

#### Frontend Integration Patterns  
- **Progressive Enhancement**: Works with and without JavaScript
- **Keyboard UX**: Standard shortcuts and accessibility
- **URL Integration**: Shareable search URLs
- **Local Storage**: User preferences and search history

### Development Documentation
- **Living Documentation**: `@memory.md`, `@scratchpad.md`, `@lessons-learned.md`
- **Knowledge Retention**: Capture insights during development
- **Team Onboarding**: Document patterns and practices
- **Historical Context**: Preserve decision rationale