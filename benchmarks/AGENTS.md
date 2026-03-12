# Benchmarks Module Guide

**Location**: `benchmarks/`
**Domain**: Performance benchmarking suite for SMA-OS
**Language**: Rust + Go
**Score**: 12/25 (testing infrastructure, distinct domain)

## Overview

Performance measurement suite for latency, throughput, and scalability validation. Uses Criterion for Rust benchmarks and Go's testing package for micro-benchmarks.

## Structure

```
benchmarks/
├── rust/
│   ├── benches/
│   │   ├── state_engine_bench.rs   # Event sourcing benchmarks
│   │   ├── latency_bench.rs        # P50/P95/P99 latency
│   │   ├── throughput_bench.rs   # Events/second
│   │   └── microvm_bench.rs       # VM lifecycle benchmarks
│   └── Cargo.toml                # Criterion dependency
├── go/
│   └── benchmark_test.go         # Go benchmarks
├── scenarios/
│   ├── latency.json              # Test scenarios
│   └── throughput.json
├── configs/
│   └── benchmark-config.yaml     # Configuration
└── scripts/
    └── run-all.sh               # CI automation
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| State engine bench | `rust/benches/state_engine_bench.rs` | append_event, query_events |
| Latency bench | `rust/benches/latency_bench.rs` | P99 measurements |
| VM bench | `rust/benches/microvm_bench.rs` | Firecracker lifecycle |
| Go benches | `go/benchmark_test.go` | DAG execution timing |
| Config | `configs/benchmark-config.yaml` | Warmup, duration, concurrency |

## Conventions (This Module)

### Criterion Benchmark
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_append_event(c: &mut Criterion) {
    c.bench_function("append_event", |b| {
        b.iter(|| {
            // Setup
            let engine = StateEngine::new();
            // Benchmark
            black_box(engine.append_event(event).await.unwrap());
        });
    });
}
```

### Go Benchmark
```go
func BenchmarkDAGExecution(b *testing.B) {
    manager := NewDAGManager()
    for i := 0; i < b.N; i++ {
        manager.Execute()
    }
}
```

### Performance Targets
```yaml
# benchmark-config.yaml
targets:
  p99_latency: 10ms
  throughput: 10000  # events/sec
  concurrency: 1000  # concurrent agents
```

## Anti-Patterns (This Module)

### Forbidden
```rust
// NEVER: Include setup in benchmark loop
b.iter(|| {
    let engine = StateEngine::new();  // WRONG: Setup in loop
    engine.append_event(event).await.unwrap();
});

// ALWAYS: Setup outside loop
let engine = StateEngine::new();
b.iter(|| {
    black_box(engine.append_event(event).await.unwrap());
});
```

### Measurement Bias
```rust
// WRONG: Cold start measurements
fn bench_first_event(c: &mut Criterion) {
    c.bench_function("first_event", |b| {
        b.iter(|| {
            let engine = StateEngine::new();  // Cold every iteration
            engine.append_event(event)
        });
    });
}

// CORRECT: Use proper warmup
criterion = Criterion::default()
    .warm_up_time(Duration::from_secs(5))  // Warm up first
    .measurement_time(Duration::from_secs(30));
```

## Commands

```bash
# Run all Rust benchmarks
cd benchmarks/rust && cargo bench

# Run specific benchmark
cargo bench --bench state_engine_bench

# Generate HTML report
open target/criterion/report/index.html

# Run Go benchmarks
cd benchmarks/go && go test -bench=.

# CI script
./scripts/run-all.sh
```

## Dependencies

| Crate/Package | Purpose |
|---------------|---------|
| criterion | Rust benchmark framework |
| criterion-perf | Performance counters |
| state-engine | Benchmark target |
| fractal-gateway | Benchmark target |

## Notes

- **Warmup critical**: Always include warmup iterations
- **Black box**: Use black_box to prevent optimization
- **Isolated runs**: Run on dedicated machines
- **Baseline comparison**: Compare with previous results
- **HTML reports**: Criterion generates detailed reports

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| P99 Latency | < 10ms | ⏳ TBD |
| Throughput | > 10k req/s | ⏳ TBD |
| Concurrency | > 1000 | ⏳ TBD |
| Memory | < 512MB | ⏳ TBD |

## Benchmark Types

| Benchmark | Target | Metrics |
|-----------|--------|---------|
| state_engine_bench | Event sourcing | Append/query latency |
| latency_bench | End-to-end | P50, P95, P99 |
| throughput_bench | Load test | Events/sec |
| microvm_bench | Firecracker | VM lifecycle |

## CI Integration

```yaml
# .github/workflows/benchmarks.yml
- name: Run benchmarks
  run: |
    cd benchmarks
    cargo bench -- --save-baseline main
- name: Compare
  run: ./scripts/compare.sh
```
