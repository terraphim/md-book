#!/bin/bash
set -euo pipefail

# Benchmark Script for md-book
# Runs performance benchmarks and generates reports

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "\n${BLUE}📊 $1${NC}"
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

# Create benchmark data directory
mkdir -p benchmark_data

# Cleanup function
cleanup() {
    echo "Cleaning up benchmark artifacts..."
    rm -rf bench_test_input bench_test_output
}

trap cleanup EXIT

print_header "MD-BOOK PERFORMANCE BENCHMARKS"

# Check if criterion is available
if ! grep -q "criterion" Cargo.toml; then
    print_error "Criterion benchmarks not configured in Cargo.toml"
    exit 1
fi

# Generate test data of various sizes
print_header "Generating Test Data"

create_benchmark_content() {
    local size=$1
    local name=$2
    local dir="bench_test_input_${name}"
    
    rm -rf "$dir"
    mkdir -p "$dir/docs"
    
    echo "Creating $name dataset ($size pages)..."
    
    # Create main index
    cat > "$dir/index.md" << EOF
# Benchmark Test Documentation ($name)

This is a $name-scale test for performance benchmarking.

## Overview
Testing build performance with $size pages of content.

## Search Terms
Performance, benchmark, testing, documentation, build, search, index.
EOF

    # Create book.toml
    cat > "$dir/book.toml" << EOF
[book]
title = "Benchmark Test ($name)"
authors = ["Benchmark Test"]
description = "Performance testing dataset"

[markdown]
format = "gfm"
EOF

    # Generate multiple pages
    for i in $(seq 1 $size); do
        local page_num=$(printf "%04d" $i)
        cat > "$dir/docs/page_${page_num}.md" << EOF
# Page $i

This is page number $i of $size in the $name benchmark dataset.

## Content Section $i

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor 
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis 
nostrud exercitation ullamco laboris.

### Subsection $i.1

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore 
eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident.

### Subsection $i.2

Sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut 
perspiciatis unde omnis iste natus error sit voluptatem accusantium.

## Code Example $i

\`\`\`rust
fn benchmark_function_$i() {
    let data = vec![1, 2, 3, 4, 5];
    let result: i32 = data.iter().sum();
    println!("Result for page $i: {}", result);
}
\`\`\`

## Search Keywords for Page $i

Keywords: performance, speed, benchmark, test$i, page$i, content$i, search, 
indexing, build, generation, static, site, documentation, markdown.

Links to other pages: [Page $(( (i % size) + 1 ))](page_$(printf "%04d" $(( (i % size) + 1 ))).md)

EOF
    done
    
    print_status "$name dataset created ($size pages)"
}

# Create different sized datasets
create_benchmark_content 10 "small"
create_benchmark_content 50 "medium"
create_benchmark_content 200 "large"

# Run Criterion benchmarks
print_header "Running Criterion Benchmarks"

echo "Running pagefind benchmarks..."
if cargo bench --bench pagefind_bench -- --output-format json --quiet > benchmark_data/criterion_results.json 2>/dev/null; then
    print_status "Criterion benchmarks completed"
else
    print_warning "Criterion benchmarks failed or not available, running manual benchmarks"
fi

# Manual performance testing
print_header "Manual Performance Tests"

benchmark_build() {
    local size=$1
    local name=$2
    local input_dir="bench_test_input_${name}"
    local output_dir="bench_test_output_${name}"
    
    rm -rf "$output_dir"
    
    echo "Benchmarking $name build ($size pages)..."
    
    # Time the build process
    local start_time=$(date +%s.%N)
    
    if cargo run --release -- -i "$input_dir" -o "$output_dir" > /dev/null 2>&1; then
        local end_time=$(date +%s.%N)
        local duration=$(echo "$end_time - $start_time" | bc -l)
        
        # Get output size
        local output_size=$(du -sh "$output_dir" | cut -f1)
        local file_count=$(find "$output_dir" -type f | wc -l)
        
        echo "  Build time: ${duration}s"
        echo "  Output size: $output_size"
        echo "  Files generated: $file_count"
        
        # Save results
        cat >> benchmark_data/build_results.txt << EOF
$name,$size,$duration,$output_size,$file_count
EOF
        
        print_status "$name build completed in ${duration}s"
        return 0
    else
        print_error "$name build failed"
        return 1
    fi
}

# Initialize results file
echo "dataset,pages,build_time_s,output_size,file_count" > benchmark_data/build_results.txt

# Run build benchmarks for different sizes
benchmark_build 10 "small"
benchmark_build 50 "medium"
benchmark_build 200 "large"

# Memory usage benchmark
print_header "Memory Usage Analysis"

if command -v time &> /dev/null; then
    echo "Running memory usage test on large dataset..."
    /usr/bin/time -l cargo run --release -- -i bench_test_input_large -o bench_test_output_large_mem 2> benchmark_data/memory_usage.txt || true
    
    if [ -f benchmark_data/memory_usage.txt ]; then
        max_memory=$(grep "maximum resident set size" benchmark_data/memory_usage.txt | awk '{print $1}' | head -1)
        if [ ! -z "$max_memory" ]; then
            max_memory_mb=$((max_memory / 1024 / 1024))
            print_status "Peak memory usage: ${max_memory_mb}MB"
            echo "peak_memory_bytes,$max_memory" >> benchmark_data/build_results.txt
        fi
    fi
else
    print_warning "GNU time not available for memory benchmarking"
fi

# Search performance
print_header "Search Performance"

if command -v pagefind &> /dev/null; then
    echo "Testing search index generation time..."
    
    for size_name in small medium large; do
        output_dir="bench_test_output_${size_name}"
        if [ -d "$output_dir" ]; then
            index_start=$(date +%s.%N)
            pagefind --site "$output_dir" > /dev/null 2>&1 || true
            index_end=$(date +%s.%N)
            index_duration=$(echo "$index_end - $index_start" | bc -l)
            
            echo "  $size_name search indexing: ${index_duration}s"
            echo "${size_name}_search_index,$index_duration" >> benchmark_data/search_results.txt
        fi
    done
    
    print_status "Search indexing benchmarks completed"
else
    print_warning "Pagefind not available for search benchmarks"
fi

# Generate report
print_header "Benchmark Report Generation"

cat > benchmark_data/report.md << 'EOF'
# md-book Performance Benchmark Report

Generated on: $(date)

## Build Performance

| Dataset | Pages | Build Time (s) | Output Size | Files Generated |
|---------|-------|---------------|-------------|-----------------|
EOF

# Add build results to report
tail -n +2 benchmark_data/build_results.txt | while IFS=',' read -r dataset pages time size files; do
    echo "| $dataset | $pages | $time | $size | $files |" >> benchmark_data/report.md
done

cat >> benchmark_data/report.md << 'EOF'

## Performance Analysis

### Build Speed
- Small datasets (10 pages): Suitable for development iteration
- Medium datasets (50 pages): Typical documentation sites
- Large datasets (200+ pages): Enterprise documentation

### Memory Usage
Peak memory usage varies with content complexity and search indexing.

### Search Indexing
Search index generation time scales approximately linearly with content size.

## Recommendations

1. **Development**: Use watch mode for faster iteration
2. **CI/CD**: Consider incremental builds for large sites
3. **Deployment**: Pre-build search indexes for faster serving
4. **Optimization**: 
   - Minimize image sizes
   - Use efficient markdown structure
   - Consider content splitting for very large sites

EOF

print_status "Benchmark report generated: benchmark_data/report.md"

# Display summary
print_header "Benchmark Summary"

if [ -f benchmark_data/build_results.txt ]; then
    echo "Build Performance Results:"
    echo "========================="
    cat benchmark_data/build_results.txt | column -t -s','
fi

echo ""
print_status "Benchmarking complete!"
echo ""
echo "Results saved in:"
echo "  📊 benchmark_data/build_results.txt"
echo "  📊 benchmark_data/report.md"
echo "  📊 benchmark_data/criterion_results.json (if available)"
echo ""
echo "To view detailed report:"
echo "  cat benchmark_data/report.md"