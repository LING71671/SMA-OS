# SMA-OS Benchmarks / SMA-OS 性能基准测试

[**English**](#english) | [**中文**](#中文)

---

<a name="中文"></a>
## 中文

SMA-OS 模块的性能基准测试套件。

## 概述

本基准测试套件测量：
- **延迟**: P50、P95、P99 响应时间
- **吞吐量**: 请求/秒、事件/秒
- **资源使用**: CPU、内存、磁盘 I/O
- **可扩展性**: 增加负载下的性能

## 快速开始

### 运行所有基准测试

```bash
cd benchmarks
./scripts/run-all.sh
```

### 运行特定基准测试

```bash
# Rust 基准测试
cd benchmarks/rust
cargo bench

# Go 基准测试
cd benchmarks/go
go test -bench=.

# 特定基准测试
cargo bench --bench state_engine_bench
```

## 基准测试

### 状态引擎

- `append_event`: 事件追加延迟和吞吐量
- `query_events`: 事件查询性能（批量大小：1、10、100、1000）
- `redis_cache`: Redis GET/SET 性能
- `postgresql`: PostgreSQL 查询性能

### 分形网关

- `packet_filter`: XDP 包过滤吞吐量
- `ip_blocking`: IP 封禁/解封延迟

### 编排

- `dag_execution`: DAG 执行时间
- `task_scheduling`: 任务调度延迟

## 性能目标

| 指标 | 目标 | 当前 | 状态 |
|------|------|------|------|
| P99 延迟 | < 10ms | TBD | ⏳ |
| 吞吐量 | > 10k req/s | TBD | ⏳ |
| 并发数 | > 1000 | TBD | ⏳ |
| 内存 | < 512MB | TBD | ⏳ |

## 输出

### 控制台输出

```
Running benchmarks...
state_engine/append_event 1.2ms (P99: 2.3ms)
state_engine/query_events 0.8ms (P99: 1.5ms)
fractal_gateway/filter 0.1ms (P99: 0.2ms)
```

### HTML 报告

Criterion 生成详细的 HTML 报告：

```bash
open target/criterion/report/index.html
```

### JSON 输出

```bash
cargo bench -- --save-baseline baseline
cat target/criterion/baseline.json
```

## 配置

编辑 `configs/benchmark-config.yaml` 以自定义：

- 预热迭代次数
- 测量持续时间
- 负载大小
- 并发级别
- 性能目标

## 持续基准测试

### CI 集成

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

### 趋势分析

基准测试自动与基线比较以检测回归：

```bash
# 与基线比较当前运行
./scripts/compare.sh

# 生成趋势报告
./scripts/generate-report.sh
```

## 解读结果

### 延迟基准测试

- **P50**: 中位延迟（第 50 百分位）
- **P95**: 95% 的请求快于此值
- **P99**: 99% 的请求快于此值（我们的目标）

### 吞吐量基准测试

- **req/s**: 每秒请求数
- **events/s**: 每秒事件数
- **bytes/s**: 数据吞吐量

### 资源使用

- **CPU**: 使用的 CPU 百分比
- **Memory**: RAM 消耗
- **I/O**: 磁盘和网络 I/O

## 故障排查

### "基准测试结果波动较大"

运行更多迭代：
```bash
cargo bench --measurement-time 60
```

### "内存不足"

在配置中减少并发或负载大小。

### "基准测试超时"

在 `configs/benchmark-config.yaml` 中增加超时。

## 最佳实践

1. **预热**: 测量前始终运行预热迭代
2. **多次运行**: 多次运行基准测试以确保一致性
3. **隔离环境**: 在专用机器上运行基准测试
4. **与基线比较**: 始终与之前的结果比较
5. **监控资源**: 测试期间关注 CPU、内存和 I/O

## 后续步骤

- 任务 9: 实现特定基准测试用例
- 任务 13: 基于结果进行性能优化

## 参考资料

- [Criterion 文档](https://bheisler.github.io/criterion.rs/book/)
- [Go 基准测试](https://pkg.go.dev/testing#hdr-Benchmarks)
- [性能测试指南](https://github.com/perf-tooling/perf-tooling)

---
---

<a name="english"></a>
## English

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
state_engine/append_event 1.2ms (P99: 2.3ms)
state_engine/query_events 0.8ms (P99: 1.5ms)
fractal_gateway/filter 0.1ms (P99: 0.2ms)
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
