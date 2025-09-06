# Test Scripts

This directory contains helper scripts for testing and development of md-book.

## Scripts Overview

### `test-setup.sh`
**Purpose**: Prepares the test environment and installs required tools.

**Usage**:
```bash
./scripts/test-setup.sh
```

**What it does**:
- Checks for required tools (cargo, pagefind, npm)
- Installs missing tools (cargo-tarpaulin, wasm-pack, cargo-deny)
- Creates test content and directories
- Runs initial code quality checks
- Displays available test commands

**Prerequisites**: 
- Rust toolchain
- Node.js (for pagefind installation)
- Internet connection for tool installation

---

### `run-tests.sh`
**Purpose**: Comprehensive test runner that executes all test suites.

**Usage**:
```bash
./scripts/run-tests.sh
```

**Test Suites Included**:
- **Pre-flight Checks**: Verify environment and project structure
- **Code Quality**: Formatting, Clippy, security audits
- **Unit Tests**: Library and binary tests
- **Integration Tests**: Cross-component testing
- **End-to-End Tests**: Full pipeline testing
- **WASM Tests**: WebAssembly compilation and functionality
- **Frontend Tests**: JavaScript/CSS functionality
- **Performance Tests**: Benchmark compilation
- **Documentation Tests**: Doc generation and doc tests
- **Cross-compilation Tests**: Multi-platform builds

**Output**: Colored status messages with final summary and success rate.

---

### `test-search.sh`
**Purpose**: Dedicated testing of Pagefind search functionality.

**Usage**:
```bash
./scripts/test-search.sh
```

**Tests Performed**:
- Creates comprehensive test content with searchable terms
- Builds test site with search integration
- Verifies search index generation
- Checks search component integration
- Tests search asset inclusion
- Validates search query functionality
- Analyzes content indexing

**Test Content**: Auto-generates documentation with:
- Installation guides
- Configuration documentation
- API references
- Tutorials and guides
- Various search terms and keywords

**Prerequisites**: `pagefind` CLI tool must be installed

---

### `benchmark.sh`
**Purpose**: Performance benchmarking and analysis.

**Usage**:
```bash
./scripts/benchmark.sh
```

**Benchmarks Include**:
- **Build Performance**: Tests with small (10), medium (50), and large (200) page datasets
- **Memory Usage**: Peak memory consumption during builds
- **Search Indexing**: Time to generate search indexes
- **Criterion Integration**: Automated benchmark suite integration

**Outputs**:
- `benchmark_data/build_results.txt`: CSV data of build performance
- `benchmark_data/report.md`: Comprehensive performance report
- `benchmark_data/memory_usage.txt`: Memory analysis (if available)
- `benchmark_data/criterion_results.json`: Criterion benchmark results

**Generated Data**: Creates test datasets of various sizes for consistent benchmarking.

---

## Quick Start

1. **First-time setup**:
   ```bash
   ./scripts/test-setup.sh
   ```

2. **Run all tests**:
   ```bash
   ./scripts/run-tests.sh
   ```

3. **Test search specifically**:
   ```bash
   ./scripts/test-search.sh
   ```

4. **Performance analysis**:
   ```bash
   ./scripts/benchmark.sh
   ```

## CI/CD Integration

These scripts are designed to work both locally and in CI environments:

- **GitHub Actions**: Scripts handle missing tools gracefully
- **Local Development**: Full tool installation and setup
- **Cross-platform**: Works on Linux, macOS, and Windows (with appropriate shell)

## Troubleshooting

### Common Issues

**Script not executable**:
```bash
chmod +x scripts/*.sh
```

**Tool not found errors**:
- Run `test-setup.sh` first to install required tools
- Check that Node.js is installed for pagefind
- Verify Rust toolchain is properly installed

**Test failures**:
- Check that you're in the project root directory
- Ensure all dependencies are installed via `test-setup.sh`
- Review test output for specific error messages

### Tool Requirements

| Tool | Purpose | Installation |
|------|---------|--------------|
| `cargo` | Rust package manager | Rust toolchain |
| `pagefind` | Search indexing | `npm install -g pagefind` |
| `cargo-tarpaulin` | Coverage analysis | `cargo install cargo-tarpaulin` |
| `wasm-pack` | WASM testing | `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf \| sh` |
| `cargo-deny` | Security auditing | `cargo install cargo-deny` |
| `node`/`npm` | Frontend testing | Node.js installation |

## Development Workflow

Recommended workflow for contributors:

1. **Initial setup**: `./scripts/test-setup.sh`
2. **Development cycle**:
   - Make changes to code
   - Run `./scripts/run-tests.sh` to verify
   - Test search specifically with `./scripts/test-search.sh`
3. **Performance testing**: `./scripts/benchmark.sh` for performance-critical changes
4. **Pre-commit**: Ensure all tests pass before committing

## Script Maintenance

When modifying these scripts:

- Maintain backward compatibility
- Add appropriate error handling
- Update this README with changes
- Test scripts on different platforms
- Keep tool version requirements up to date