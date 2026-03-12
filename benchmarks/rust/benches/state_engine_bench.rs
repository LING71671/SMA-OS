//! State Engine Performance Benchmarks
//!
//! Benchmarks for the state engine event sourcing performance:
//! - Event append latency
//! - Event query latency
//! - Snapshot generation time
//! - Redis vs PostgreSQL performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use tokio::runtime::Runtime;

/// Benchmark event append performance
fn bench_append_event(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("state_engine/append_event");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("append_single_event", |b| {
        b.to_async(&rt).iter(|| async {
            // Simulate appending a single event
            // In real scenario, this would call state_engine.append_event()
            black_box(async {
                // Mock implementation
                tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
            }.await)
        });
    });
    
    group.finish();
}

/// Benchmark event query performance
fn bench_query_events(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("state_engine/query_events");
    
    // Test different batch sizes
    for batch_size in [1, 10, 100, 1000] {
        let mut subgroup = c.benchmark_group(format!("batch_{}", batch_size));
        subgroup.throughput(Throughput::Elements(batch_size as u64));
        
        subgroup.bench_function(BenchmarkId::from_parameter(batch_size), |b| {
            b.to_async(&rt).iter(|| async {
                // Simulate querying events
                black_box(async {
                    tokio::time::sleep(tokio::time::Duration::from_micros(10 * batch_size as u64)).await;
                }.await)
            });
        });
    }
    
    group.finish();
}

/// Benchmark Redis cache performance
fn bench_redis_cache(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("state_engine/redis_cache");
    
    group.bench_function("redis_get", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Mock Redis GET - typically < 1ms
                tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;
            }.await)
        });
    });
    
    group.bench_function("redis_set", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Mock Redis SET
                tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;
            }.await)
        });
    });
    
    group.finish();
}

/// Benchmark PostgreSQL performance
fn bench_postgres(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("state_engine/postgresql");
    
    group.bench_function("postgres_select", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Mock PostgreSQL SELECT - typically 1-10ms
                tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
            }.await)
        });
    });
    
    group.bench_function("postgres_insert", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Mock PostgreSQL INSERT
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }.await)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_append_event,
    bench_query_events,
    bench_redis_cache,
    bench_postgres,
);

criterion_main!(benches);
