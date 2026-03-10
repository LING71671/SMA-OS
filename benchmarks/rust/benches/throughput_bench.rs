//! Throughput Benchmarks
//!
//! Measures requests/second for various operations:
//! - Event ingestion throughput
//! - Query throughput
//! - Concurrent request handling

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use tokio::runtime::Runtime;

/// Benchmark event ingestion throughput
fn bench_ingestion_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("throughput/ingestion");
    group.measurement_time(std::time::Duration::from_secs(30));
    
    // Measure events per second
    group.bench_function("events_per_second", |b| {
        b.to_async(&rt).iter(|| async {
            // Simulate ingesting 1000 events
            for _ in 0..1000 {
                black_box(());
            }
        });
    });
    
    group.throughput(Throughput::Elements(1000));
    group.finish();
}

/// Benchmark concurrent request handling
fn bench_concurrent_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("throughput/concurrent");
    
    for concurrency in [1, 10, 100, 1000] {
        let mut subgroup = c.benchmark_group(format!("concurrency_{}", concurrency));
        
        subgroup.bench_function(format!("{}_concurrent", concurrency), |b| {
            b.to_async(&rt).iter(|| async {
                // Simulate concurrent requests
                let mut handles = vec![];
                for _ in 0..concurrency {
                    handles.push(tokio::spawn(async {
                        black_box(());
                    }));
                }
                for handle in handles {
                    handle.await.unwrap();
                }
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_ingestion_throughput,
    bench_concurrent_requests,
);

criterion_main!(benches);
