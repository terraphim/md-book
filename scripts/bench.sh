#!/bin/bash
set -e

# Benchmark script for md-book performance testing
# This script runs benchmarks and compares results with previous runs

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BENCHMARK_DIR="$PROJECT_DIR/benchmark-results"
CURRENT_RESULTS="$BENCHMARK_DIR/current.json"
PREVIOUS_RESULTS="$BENCHMARK_DIR/previous.json"

echo "=== MD-Book Performance Benchmarks ==="

# Create benchmark results directory
mkdir -p "$BENCHMARK_DIR"

# Save previous results if they exist
if [[ -f "$CURRENT_RESULTS" ]]; then
    mv "$CURRENT_RESULTS" "$PREVIOUS_RESULTS"
fi

echo "Running benchmarks..."

# Run benchmarks with JSON output
cd "$PROJECT_DIR"
cargo bench -- --output-format json | tee "$CURRENT_RESULTS"

# Parse and compare results if we have previous data
if [[ -f "$PREVIOUS_RESULTS" ]] && command -v jq >/dev/null 2>&1; then
    echo -e "\n=== Performance Comparison ==="
    
    # Extract benchmark results and compare
    current_times=$(jq -r '.results[]? | select(.benchmark_name) | "\(.benchmark_name):\(.mean.estimate)"' "$CURRENT_RESULTS" 2>/dev/null || echo "")
    previous_times=$(jq -r '.results[]? | select(.benchmark_name) | "\(.benchmark_name):\(.mean.estimate)"' "$PREVIOUS_RESULTS" 2>/dev/null || echo "")
    
    if [[ -n "$current_times" && -n "$previous_times" ]]; then
        echo "Benchmark\tCurrent (ns)\tPrevious (ns)\tChange (%)"
        echo "--------------------------------------------------------"
        
        # Create associative arrays for comparison
        declare -A current_map previous_map
        
        while IFS=: read -r name time; do
            current_map["$name"]="$time"
        done <<< "$current_times"
        
        while IFS=: read -r name time; do
            previous_map["$name"]="$time"
        done <<< "$previous_times"
        
        # Compare results
        regression_detected=false
        for bench_name in "${!current_map[@]}"; do
            current="${current_map[$bench_name]}"
            previous="${previous_map[$bench_name]:-}"
            
            if [[ -n "$previous" ]]; then
                # Calculate percentage change
                change=$(echo "scale=2; (($current - $previous) / $previous) * 100" | bc -l 2>/dev/null || echo "0")
                
                # Format output
                printf "%-30s\t%-12.0f\t%-12.0f\t%+.2f%%\n" "$bench_name" "$current" "$previous" "$change"
                
                # Check for significant regression (>10% slower)
                if (( $(echo "$change > 10.0" | bc -l) )); then
                    regression_detected=true
                    echo "⚠️  Significant regression detected in $bench_name"
                fi
            else
                printf "%-30s\t%-12.0f\t%-12s\t%s\n" "$bench_name" "$current" "N/A" "NEW"
            fi
        done
        
        if $regression_detected; then
            echo -e "\n❌ Performance regressions detected!"
            exit 1
        else
            echo -e "\n✅ No significant performance regressions detected"
        fi
    else
        echo "Unable to parse benchmark results for comparison"
    fi
else
    echo "No previous results found for comparison, or jq not available"
fi

echo -e "\nBenchmark results saved to: $CURRENT_RESULTS"