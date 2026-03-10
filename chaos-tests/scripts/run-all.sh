#!/bin/bash
# Run all chaos tests
# This script executes all chaos test scenarios

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== SMA-OS Chaos Tests ==="
echo "Project Root: $PROJECT_ROOT"

# Build the chaos tests
echo "Building chaos tests..."
cd "$PROJECT_ROOT/chaos-tests"
cargo build --release

# Run all scenarios
echo "Running all chaos scenarios..."
cargo run --release -- --scenario all --config configs/chaos-config.yaml

echo "Chaos tests completed!"
