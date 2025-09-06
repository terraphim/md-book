#!/bin/bash
set -euo pipefail

# Comprehensive Test Runner for md-book
# Runs all test suites with proper reporting

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to print colored output
print_header() {
    echo -e "\n${BLUE}🚀 $1${NC}"
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

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    local continue_on_error="${3:-false}"
    
    echo "Running: $test_name"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command" > /dev/null 2>&1; then
        print_status "$test_name passed"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        print_error "$test_name failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        
        if [ "$continue_on_error" = "false" ]; then
            echo "Running command with output for debugging:"
            eval "$test_command"
            exit 1
        fi
        return 1
    fi
}

# Function to show final results
show_results() {
    echo ""
    echo "================================================================"
    echo -e "${BLUE}📊 TEST RESULTS SUMMARY${NC}"
    echo "================================================================"
    echo "Total Tests: $TOTAL_TESTS"
    echo -e "Passed:      ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed:      ${RED}$FAILED_TESTS${NC}"
    echo "Success Rate: $((PASSED_TESTS * 100 / TOTAL_TESTS))%"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "\n${GREEN}🎉 All tests passed!${NC}"
        return 0
    else
        echo -e "\n${RED}💥 Some tests failed!${NC}"
        return 1
    fi
}

# Cleanup function
cleanup() {
    echo "Cleaning up test artifacts..."
    rm -rf test_output test_temp
    find . -name "*.orig" -delete 2>/dev/null || true
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

print_header "MD-BOOK COMPREHENSIVE TEST SUITE"

# Pre-flight checks
print_header "Pre-flight Checks"
run_test "Cargo available" "command -v cargo"
run_test "Project structure" "test -f Cargo.toml && test -d src"

# Code quality checks
print_header "Code Quality Checks"
run_test "Code formatting" "cargo fmt --check"
run_test "Clippy lints" "cargo clippy --all-targets --all-features -- -D warnings"

# Security and dependency checks
print_header "Security & Dependency Checks"
if command -v cargo-deny &> /dev/null; then
    run_test "Security audit" "cargo deny check" true
else
    print_warning "cargo-deny not found, skipping security audit"
fi

if command -v cargo-audit &> /dev/null; then
    run_test "Vulnerability scan" "cargo audit" true
else
    print_warning "cargo-audit not found, skipping vulnerability scan"
fi

# Unit tests
print_header "Unit Tests"
run_test "Library unit tests" "cargo test --lib"
run_test "Binary unit tests" "cargo test --bin md-book"

# Integration tests
print_header "Integration Tests"
run_test "Integration tests" "cargo test --test integration" true

# End-to-end tests
print_header "End-to-End Tests"
run_test "Build pipeline tests" "cargo test --test build_test"
run_test "Server functionality tests" "cargo test --test server_test" true

# WASM tests (if wasm-pack is available)
print_header "WebAssembly Tests"
if command -v wasm-pack &> /dev/null; then
    run_test "WASM compilation" "cargo build --target wasm32-unknown-unknown --lib"
    run_test "WASM tests" "wasm-pack test --headless --chrome" true
else
    print_warning "wasm-pack not found, skipping WASM tests"
fi

# Frontend tests (if npm is available and package.json exists)
print_header "Frontend Tests"
if [ -f "package.json" ] && command -v npm &> /dev/null; then
    run_test "Frontend dependencies" "npm ci"
    run_test "Frontend tests" "npm test" true
else
    print_warning "Frontend testing environment not available"
fi

# Performance tests
print_header "Performance Tests"
run_test "Benchmark compilation" "cargo bench --no-run"

# Documentation tests
print_header "Documentation Tests"
run_test "Documentation build" "cargo doc --no-deps --document-private-items"
run_test "Documentation tests" "cargo test --doc"

# Functional tests with real content
print_header "Functional Tests"
mkdir -p test_temp/input test_temp/output

# Create test content
cat > test_temp/input/index.md << 'EOF'
# Functional Test
Testing basic functionality.
Search terms: installation, configuration, API.
EOF

run_test "Basic build functionality" "cargo run -- -i test_temp/input -o test_temp/output"
run_test "Output files generated" "test -f test_temp/output/index.html"

if command -v pagefind &> /dev/null; then
    run_test "Search index created" "test -d test_temp/output/pagefind"
else
    print_warning "Pagefind not available, skipping search index test"
fi

# Cross-compilation tests (if targets are available)
print_header "Cross-Compilation Tests"
if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    run_test "WASM target build" "cargo build --target wasm32-unknown-unknown --lib" true
fi

if rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    run_test "Windows target build" "cargo build --target x86_64-pc-windows-gnu" true
fi

# Show final results
show_results