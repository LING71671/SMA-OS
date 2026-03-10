#!/bin/bash
# Run all benchmarks
# This script executes all benchmark scenarios

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BENCHMARKS_DIR="$PROJECT_ROOT/benchmarks"

echo "=== SMA-OS Benchmarks ==="
echo "Project Root: $PROJECT_ROOT"

# Build benchmarks
echo "Building Rust benchmarks..."
cd "$BENCHMARKS_DIR/rust"
cargo build --release

# Run Rust benchmarks
echo "Running Rust benchmarks..."
cargo bench -- --save-baseline baseline

# Run Go benchmarks
echo "Running Go benchmarks..."
cd "$BENCHMARKS_DIR/go"
go test -bench=. -benchmem -benchtime=30s

# Generate report
echo "Generating benchmark report..."
cd "$BENCHMARKS_DIR"
./scripts/generate-report.sh

echo "Benchmarks completed!"
