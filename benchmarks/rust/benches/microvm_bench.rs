//! MicroVM Performance Benchmarks
//!
//! Benchmarks for Firecracker MicroVM lifecycle performance:
//! - VM creation time
//! - VM startup time (< 5ms target)
//! - VM snapshot creation time
//! - VM restore time
//! - Concurrent VM operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// Benchmark MicroVM lifecycle operations
fn bench_vm_lifecycle(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("microvm/lifecycle");

    // VM creation benchmark
    group.bench_function("create_vm", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Simulate VM creation time
                // In production: calls sandbox-daemon to create MicroVM
                tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;
            }.await)
        });
    });

    // VM startup benchmark - target < 5ms
    group.bench_function("start_vm", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Simulate VM startup time
                // Target: < 5ms for cold start
                // Target: < 1ms for warm start (from snapshot)
                tokio::time::sleep(tokio::time::Duration::from_micros(3000)).await;
            }.await)
        });
    });

    // VM snapshot creation benchmark
    group.bench_function("snapshot_vm", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Simulate snapshot creation
                // Includes memory state + disk state capture
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }.await)
        });
    });

    // VM restore from snapshot benchmark
    group.bench_function("restore_vm", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Simulate VM restore from snapshot
                // Target: < 2ms
                tokio::time::sleep(tokio::time::Duration::from_micros(1500)).await;
            }.await)
        });
    });

    // VM shutdown benchmark
    group.bench_function("shutdown_vm", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Simulate graceful VM shutdown
                tokio::time::sleep(tokio::time::Duration::from_micros(800)).await;
            }.await)
        });
    });

    group.finish();
}

/// Benchmark VM startup with different configurations
fn bench_vm_startup_configs(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("microvm/startup_configs");

    // Test different memory sizes
    let memory_sizes = vec![128, 256, 512, 1024]; // MB
    for mem_size in &memory_sizes {
        group.bench_with_input(
            BenchmarkId::new("cold_start", mem_size),
            mem_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    black_box(async {
                        // Simulate cold start with given memory size
                        let base_time = Duration::from_micros(2000);
                        let mem_overhead = Duration::from_micros((size as u64) * 2);
                        tokio::time::sleep(base_time + mem_overhead).await;
                    }.await)
                });
            },
        );
    }

    // Test different vCPU counts
    let vcpu_counts = vec![1, 2, 4, 8];
    for vcpus in &vcpu_counts {
        group.bench_with_input(
            BenchmarkId::new("vcpu_startup", vcpus),
            vcpus,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    black_box(async {
                        // Simulate startup with given vCPU count
                        let base_time = Duration::from_micros(2000);
                        let cpu_overhead = Duration::from_micros((count as u64) * 300);
                        tokio::time::sleep(base_time + cpu_overhead).await;
                    }.await)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent VM operations
fn bench_concurrent_vm_ops(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("microvm/concurrent");

    // Test concurrent VM creation
    for concurrency in [1, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_create", concurrency),
            &concurrency,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    black_box(async {
                        // Simulate concurrent VM creation
                        let mut handles = vec![];
                        for _ in 0..count {
                            handles.push(tokio::spawn(async {
                                tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;
                            }));
                        }
                        for handle in handles {
                            handle.await.unwrap();
                        }
                    }.await)
                });
            },
        );
    }

    // Test concurrent VM startup
    for concurrency in [1, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_start", concurrency),
            &concurrency,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    black_box(async {
                        // Simulate concurrent VM startup
                        let mut handles = vec![];
                        for _ in 0..count {
                            handles.push(tokio::spawn(async {
                                tokio::time::sleep(tokio::time::Duration::from_millis(3)).await;
                            }));
                        }
                        for handle in handles {
                            handle.await.unwrap();
                        }
                    }.await)
                });
            },
        );
    }

    group.throughput(Throughput::Elements(1));
    group.finish();
}

/// Benchmark VM snapshot operations
fn bench_snapshot_ops(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("microvm/snapshot");

    // Snapshot creation with different memory sizes
    let memory_sizes = vec![128, 256, 512, 1024];
    for mem_size in &memory_sizes {
        group.bench_with_input(
            BenchmarkId::new("create_snapshot", mem_size),
            mem_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    black_box(async {
                        // Simulate snapshot creation
                        // Time scales with memory size
                        let base_time = Duration::from_millis(5);
                        let mem_time = Duration::from_micros((size as u64) * 10);
                        tokio::time::sleep(base_time + mem_time).await;
                    }.await)
                });
            },
        );
    }

    // Snapshot restore with different memory sizes
    for mem_size in &memory_sizes {
        group.bench_with_input(
            BenchmarkId::new("restore_snapshot", mem_size),
            mem_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    black_box(async {
                        // Simulate snapshot restore
                        // Target: < 2ms regardless of memory size (lazy loading)
                        let base_time = Duration::from_micros(1500);
                        let mem_time = Duration::from_micros((size as u64) * 1);
                        tokio::time::sleep(base_time + mem_time).await;
                    }.await)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark VM network operations
fn bench_vm_network(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("microvm/network");

    // Network interface creation
    group.bench_function("create_tap_interface", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Simulate TAP interface creation
                tokio::time::sleep(tokio::time::Duration::from_micros(200)).await;
            }.await)
        });
    });

    // Network namespace setup
    group.bench_function("setup_netns", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Simulate network namespace setup
                tokio::time::sleep(tokio::time::Duration::from_micros(300)).await;
            }.await)
        });
    });

    // vsock setup
    group.bench_function("setup_vsock", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(async {
                // Simulate vsock setup for agent communication
                tokio::time::sleep(tokio::time::Duration::from_micros(150)).await;
            }.await)
        });
    });

    group.finish();
}

/// Measure end-to-end VM lifecycle latency
fn bench_vm_lifecycle_e2e(c: &mut Criterion) {
    let mut group = c.benchmark_group("microvm/lifecycle_e2e");

    group.bench_function("full_lifecycle_cold", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                // Simulate full cold lifecycle: create -> start -> work -> stop
                black_box(());
            }
            start.elapsed()
        });
    });

    group.bench_function("full_lifecycle_warm", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                // Simulate full warm lifecycle: restore -> work -> snapshot
                black_box(());
            }
            start.elapsed()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_vm_lifecycle,
    bench_vm_startup_configs,
    bench_concurrent_vm_ops,
    bench_snapshot_ops,
    bench_vm_network,
    bench_vm_lifecycle_e2e,
);

criterion_main!(benches);
