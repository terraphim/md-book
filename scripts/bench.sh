#!/bin/bash
set -e

# Benchmark script for md-book performance testing
# This script runs benchmarks and compares results with previous runs
#
# Usage: bench.sh [--quick]
# --quick: Run only fast benchmarks suitable for CI

# Parse command line arguments
QUICK_MODE=false
for arg in "$@"; do
    case $arg in
        --quick)
            QUICK_MODE=true
            shift
            ;;
        *)
            # Unknown option
            ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BENCHMARK_DIR="$PROJECT_DIR/benchmark-results"
CURRENT_RESULTS="$BENCHMARK_DIR/current.json"
PREVIOUS_RESULTS="$BENCHMARK_DIR/previous.json"

# Trap handler to ensure cleanup and JSON creation on exit
cleanup() {
    local exit_code=$?
    
    # If current.json doesn't exist or is incomplete, create minimal structure
    if [[ ! -f "$CURRENT_RESULTS" ]] || ! grep -q "}" "$CURRENT_RESULTS" 2>/dev/null; then
        echo "Creating fallback JSON result..."
        cat > "$CURRENT_RESULTS" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "status": "interrupted",
  "benchmarks": []
}
EOF
    fi
    
    # Clean up temporary files
    rm -f "$CURRENT_RESULTS.raw" "$CURRENT_RESULTS.log"
    
    exit $exit_code
}

trap cleanup EXIT INT TERM

echo "=== MD-Book Performance Benchmarks ==="

# Create benchmark results directory
mkdir -p "$BENCHMARK_DIR"

# Save previous results if they exist
if [[ -f "$CURRENT_RESULTS" ]]; then
    mv "$CURRENT_RESULTS" "$PREVIOUS_RESULTS"
fi

echo "Running benchmarks..."

# Run benchmarks and capture output
cd "$PROJECT_DIR"

# Run benchmarks and save both text and structured output
if [[ "$QUICK_MODE" == "true" ]]; then
    echo "Running quick benchmarks for CI..."
    # Run only fast benchmarks with reduced sample size
    if cargo bench --bench pagefind_bench -- "pagefind_init" --warm-up-time 1 --measurement-time 3 --sample-size 10 2>&1 | tee "$CURRENT_RESULTS.raw"; then
        benchmark_success=true
    else
        benchmark_success=false
    fi
else
    echo "Running full benchmarks..."
    if cargo bench --message-format=json 2>/dev/null | tee "$CURRENT_RESULTS.raw"; then
        benchmark_success=true
    else
        benchmark_success=false
    fi
fi

if [[ "$benchmark_success" == "true" ]]; then
    # Try to extract meaningful data from criterion output
    echo "Processing benchmark results..."
    
    # Create a structured JSON result
    cat > "$CURRENT_RESULTS" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "benchmarks": [
EOF
    
    # Parse criterion output for benchmark results  
    if grep -q "time:" "$CURRENT_RESULTS.raw" 2>/dev/null; then
        # Process each line and build JSON entries
        while IFS= read -r line; do
            # Use sed to extract benchmark name and median timing values
            parsed=$(echo "$line" | sed -n 's/^\([^[:space:]]*\)[[:space:]]*time:[[:space:]]*\[\([0-9.]*\)[[:space:]]*\([a-z]*\)[[:space:]]*\([0-9.]*\)[[:space:]]*\([a-z]*\)[[:space:]]*\([0-9.]*\)[[:space:]]*\([a-z]*\)\].*/\1|\4|\5/p')
            
            if [[ -n "$parsed" ]]; then
                IFS='|' read -r name time_val time_unit <<< "$parsed"
                
                # Convert to nanoseconds for consistency
                case "$time_unit" in
                    "ns") multiplier=1 ;;
                    "µs"|"us") multiplier=1000 ;;
                    "ms") multiplier=1000000 ;;
                    "s") multiplier=1000000000 ;;
                    *) multiplier=1 ;;
                esac
                
                # Use awk for floating point multiplication (more reliable than bc)
                time_ns=$(awk "BEGIN {printf \"%.0f\", $time_val * $multiplier}")
                
                echo "    {" >> "$CURRENT_RESULTS"
                echo "      \"benchmark_name\": \"$name\"," >> "$CURRENT_RESULTS"
                echo "      \"mean\": {" >> "$CURRENT_RESULTS"
                echo "        \"estimate\": $time_ns" >> "$CURRENT_RESULTS"
                echo "      }," >> "$CURRENT_RESULTS"
                echo "      \"unit\": \"ns\"" >> "$CURRENT_RESULTS"
                echo "    }," >> "$CURRENT_RESULTS"
            fi
        done < <(grep "time:" "$CURRENT_RESULTS.raw")
        
        # Remove trailing comma and close JSON
        if [[ "$(uname)" == "Darwin" ]]; then
            # macOS sed requires different syntax
            sed -i '' '$ s/,$//' "$CURRENT_RESULTS"
        else
            # Linux sed
            sed -i '$ s/,$//' "$CURRENT_RESULTS"
        fi
    else
        # No benchmark results found, create empty structure  
        echo "    {}" >> "$CURRENT_RESULTS"
    fi
    
    echo "  ]" >> "$CURRENT_RESULTS"
    echo "}" >> "$CURRENT_RESULTS"
    
    # Clean up intermediate file
    rm -f "$CURRENT_RESULTS.raw"
    
else
    # Fallback: run with basic output
    echo "Running benchmarks with basic output..."
    cargo bench 2>&1 | tee "$CURRENT_RESULTS.log"
    
    # Create minimal JSON structure with log content
    cat > "$CURRENT_RESULTS" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "status": "completed",
  "log_output": $(cat "$CURRENT_RESULTS.log" | jq -Rs . 2>/dev/null || echo "\"Benchmark completed\""),
  "benchmarks": []
}
EOF
    
    rm -f "$CURRENT_RESULTS.log"
fi

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