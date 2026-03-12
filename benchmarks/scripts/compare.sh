#!/bin/bash
# Compare benchmark results with baseline
# This script compares current results with the baseline

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARKS_DIR="$(dirname "$SCRIPT_DIR")"
BASELINE_FILE="$BENCHMARKS_DIR/baseline.json"
CURRENT_FILE="$BENCHMARKS_DIR/current.json"

echo "=== Benchmark Comparison ==="

if [ ! -f "$BASELINE_FILE" ]; then
    echo "No baseline found. Creating baseline from current run..."
    cp "$CURRENT_FILE" "$BASELINE_FILE"
    exit 0
fi

echo "Comparing with baseline..."

# Simple comparison (can be enhanced with actual benchmark data)
if [ -f "$CURRENT_FILE" ]; then
    echo "Current vs Baseline:"
    echo "  Baseline: $(cat $BASELINE_FILE)"
    echo "  Current:  $(cat $CURRENT_FILE)"
else
    echo "No current benchmark data found. Run benchmarks first."
fi

echo "Comparison complete!"
