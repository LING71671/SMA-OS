# SMA-OS Benchmarks

Performance benchmarking suite for SMA-OS modules.

## Overview

This benchmark suite measures:
- **Latency**: P50, P95, P99 response times
- **Throughput**: Requests/second, events/second
- **Resource Usage**: CPU, memory, disk I/O
- **Scalability**: Performance under increasing load

## Quick Start

### Run All Benchmarks

```bash
cd benchmarks
./scripts/run-all.sh
```

### Run Specific Benchmarks

```bash
# Rust benchmarks
cd benchmarks/rust
cargo bench

# Go benchmarks
cd benchmarks/go
go test -bench=.

# Specific benchmark
cargo bench --bench state_engine_bench
```

## Benchmarks

### State Engine

- `append_event`: Event append latency and throughput
- `query_events`: Event query performance (batch sizes: 1, 10, 100, 1000)
- `redis_cache`: Redis GET/SET performance
- `postgresql`: PostgreSQL query performance

### Fractal Gateway

- `packet_filter`: XDP packet filtering throughput
- `ip_blocking`: IP blocking/unblocking latency

### Orchestration

- `dag_execution`: DAG execution time
- `task_scheduling`: Task scheduling latency

## Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| P99 Latency | < 10ms | TBD | ⏳ |
| Throughput | > 10k req/s | TBD | ⏳ |
| Concurrency | > 1000 | TBD | ⏳ |
| Memory | < 512MB | TBD | ⏳ |

## Output

### Console Output

```
Running benchmarks...
state_engine/append_event     1.2ms (P99: 2.3ms)
state_engine/query_events     0.8ms (P99: 1.5ms)
fractal_gateway/filter        0.1ms (P99: 0.2ms)
```

### HTML Report

Criterion generates detailed HTML reports:

```bash
open target/criterion/report/index.html
```

### JSON Output

```bash
cargo bench -- --save-baseline baseline
cat target/criterion/baseline.json
```

## Configuration

Edit `configs/benchmark-config.yaml` to customize:

- Warmup iterations
- Measurement duration
- Payload sizes
- Concurrency levels
- Performance targets

## Continuous Benchmarking

### CI Integration

```yaml
# .github/workflows/benchmarks.yml
name: Benchmarks
on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run benchmarks
        run: |
          cd benchmarks
          cargo bench -- --save-baseline main
      
      - name: Compare with baseline
        run: |
          cd benchmarks
          ./scripts/compare.sh
```

### Trend Analysis

Benchmarks are automatically compared with the baseline to detect regressions:

```bash
# Compare current run with baseline
./scripts/compare.sh

# Generate trend report
./scripts/generate-report.sh
```

## Interpreting Results

### Latency Benchmarks

- **P50**: Median latency (50th percentile)
- **P95**: 95% of requests faster than this
- **P99**: 99% of requests faster than this (our target)

### Throughput Benchmarks

- **req/s**: Requests per second
- **events/s**: Events per second
- **bytes/s**: Data throughput

### Resource Usage

- **CPU**: Percentage of CPU used
- **Memory**: RAM consumption
- **I/O**: Disk and network I/O

## Troubleshooting

### "Benchmark results vary widely"

Run more iterations:
```bash
cargo bench --measurement-time 60
```

### "Out of memory"

Reduce concurrency or payload size in config.

### "Benchmark timeout"

Increase timeout in `configs/benchmark-config.yaml`.

## Best Practices

1. **Warm up**: Always run warmup iterations before measuring
2. **Multiple runs**: Run benchmarks multiple times for consistency
3. **Isolated environment**: Run benchmarks on dedicated machines
4. **Compare with baseline**: Always compare with previous results
5. **Monitor resources**: Watch CPU, memory, and I/O during tests

## Next Steps

- Task 9: Implement specific benchmark use cases
- Task 13: Performance optimization based on results

## References

- [Criterion Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Go Benchmarking](https://pkg.go.dev/testing#hdr-Benchmarks)
- [Performance Testing Guide](https://github.com/perf-tooling/perf-tooling)
