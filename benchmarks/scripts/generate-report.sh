#!/bin/bash
# Generate benchmark report
# This script generates a comprehensive benchmark report

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARKS_DIR="$(dirname "$SCRIPT_DIR")"
REPORT_DIR="$BENCHMARKS_DIR/reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$REPORT_DIR"

echo "Generating benchmark report..."

# Generate HTML report from criterion
if [ -d "$BENCHMARKS_DIR/rust/target/criterion" ]; then
    echo "Generating HTML report..."
    cp -r "$BENCHMARKS_DIR/rust/target/criterion" "$REPORT_DIR/criterion_$TIMESTAMP"
fi

# Generate summary
cat > "$REPORT_DIR/summary_$TIMESTAMP.md" << EOF
# SMA-OS Benchmark Report

Generated: $(date)

## Summary

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| P99 Latency | TBD | < 10ms | TBD |
| Throughput | TBD | > 10k req/s | TBD |
| Concurrency | TBD | > 1000 | TBD |

## Details

See criterion report for detailed analysis.

## Trends

Compare with previous runs to identify regressions.
EOF

echo "Report generated: $REPORT_DIR/summary_$TIMESTAMP.md"
