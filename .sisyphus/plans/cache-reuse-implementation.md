# SMA-OS 缓存复用实施计划

## TL;DR

> **目标**: 在 SMA-OS 中实现多级缓存复用机制，解决 memory-bus 模块的缓存缺失问题，提升系统性能并降低 LLM API 成本。
>
> **核心问题**: 
> - `memory-bus/ingestion` 无缓存，每次请求直接调用 DeepSeek API
> - 缺少缓存雪崩防护（singleflight）
> - Rust模块缺少本地内存缓存层（L1）
>
> **预期收益**:
> - 延迟: 500ms → 1ms (Redis) → 100μs (本地)
> - 成本: 降低 85%+ (基于85%缓存命中率)
> - 稳定性: 消除缓存雪崩风险
>
> **技术栈**:
> - Go: `go-redis/v9` + `golang.org/x/sync/singleflight` + `github.com/dgraph-io/ristretto`
> - Rust: `moka` + `redis`
>
> **执行方式**: 3个Wave，支持并行执行

---

## Context

### 原始需求
探索 SMA-OS 中缓存复用的必要性，并实施改进方案。

### 研究发现

#### 1. 现有缓存实现（良好）
**`control-plane/state-engine` (Rust)**:
- ✅ Redis-first + PostgreSQL fallback
- ✅ Pipeline批量写入
- ✅ 选择性回填策略（防止缓存抖动）
- ✅ 24h TTL
- ✅ Prometheus监控指标

```rust
// engine.rs: 选择性回填策略
if !events.is_empty() && to_version.is_none() {
    let mut pipe = redis::pipe();
    pipe.del(&redis_key);
    for event in &events {
        pipe.zadd(&redis_key, event.version as f64, event_json);
    }
    pipe.expire(&redis_key, 86400).ignore()
        .query_async(&mut conn).await?;
}
```

#### 2. 关键缺失（需改进）

**`memory-bus/ingestion` (Go)**:
- ❌ **无缓存层**，每次请求直接调用 DeepSeek API
- ❌ **无singleflight**，存在缓存穿透/雪崩风险
- ❌ **无本地内存缓存**

**`memory-bus/vector-kv` (Go)**:
- ❌ stub实现，计划使用 Redis hot cache 但未实现

#### 3. 性能对比

| 指标 | 当前（无缓存） | 目标（有缓存） | 提升 |
|------|--------------|--------------|------|
| API调用延迟 | ~500ms | ~1ms (Redis) | **99.8%↓** |
| LLM API成本 | 100% | 15% (85%命中) | **85%↓** |
| 并发处理 | 受API限流 | 10,000+ req/s | **100x** |
| 缓存命中率 | 0% | 85-95% | - |

### 技术选型决策

| 场景 | 选择 | 理由 |
|------|------|------|
| Go本地缓存 | `ristretto` | Dgraph出品，高吞吐，低GC开销 |
| Go请求去重 | `singleflight` | 官方库，解决缓存穿透 |
| Go Redis | `go-redis/v9` | 标准实现，支持Pipeline |
| Rust本地缓存 | `moka` | crates.io在用，TinyLFU算法 |

---

## Work Objectives

### 核心目标
在 SMA-OS 中实现三级缓存架构：
1. **L1**: 本地内存缓存 (ristretto/moka) - 微秒级
2. **L2**: Redis分布式缓存 - 毫秒级
3. **L3**: PostgreSQL/API - 持久化

### 具体交付物
1. `memory-bus/ingestion` Redis缓存层 + singleflight
2. `memory-bus/ingestion` 本地内存缓存 (ristretto)
3. `control-plane/state-engine` moka本地缓存集成
4. 缓存命中率Prometheus监控
5. 缓存雪崩防护测试

### 完成标准
- [ ] 所有单元测试通过 (`go test`, `cargo test`)
- [ ] 缓存命中率 > 85% (基于测试数据)
- [ ] API调用次数减少 > 80% (基于测试)
- [ ] 无缓存穿透/雪崩问题 (通过singleflight验证)
- [ ] Prometheus指标正常采集

### 明确范围
**包含**:
- memory-bus/ingestion 缓存实现
- memory-bus/ingestion singleflight集成
- 基础Prometheus监控

**不包含**:
- vector-kv 完整实现（仅添加Redis客户端stub）
- 缓存预热机制（Phase 2）
- 多级缓存链式框架（Phase 2）

---

## Verification Strategy

### 测试决策
- **基础设施**: Go已有test框架，Rust使用cargo test
- **测试策略**: TDD for核心逻辑，集成测试for缓存行为
- **Agent QA**: 每个任务包含Playwright/Bash验证场景

### QA策略
- **单元测试**: 每个缓存操作单独测试
- **集成测试**: 模拟缓存穿透/雪崩场景
- **性能测试**: 测量缓存命中率和延迟
- **监控验证**: Prometheus指标正确采集

---

## Execution Strategy

### Wave 1 (基础依赖 - 可立即并行启动)
```
Wave 1 (Foundation - 3 tasks):
├── Task 1: 添加go-redis依赖和配置 [quick]
├── Task 2: 添加ristretto依赖 [quick]
└── Task 3: 添加moka依赖 (Rust) [quick]
```

### Wave 2 (核心实现 - 可并行)
```
Wave 2 (Core Implementation - 6 tasks):
├── Task 4: memory-bus Redis客户端封装 [quick]
├── Task 5: singleflight请求去重机制 [quick]
├── Task 6: ingestion缓存层实现 [unspecified-high]
├── Task 7: ristretto本地缓存集成 [unspecified-high]
├── Task 8: Rust moka缓存集成 [unspecified-high]
└── Task 9: Prometheus监控指标 [quick]
```

### Wave 3 (验证与测试)
```
Wave 3 (Verification - 4 tasks):
├── Task 10: 缓存命中率测试 [unspecified-high]
├── Task 11: 缓存雪崩防护测试 [deep]
├── Task 12: 性能基准测试 [unspecified-high]
└── Task 13: 集成验证 [unspecified-high]
```

### Wave FINAL (独立审查)
```
Wave FINAL (Review - 4 tasks):
├── Task F1: Plan compliance audit [oracle]
├── Task F2: Code quality review [unspecified-high]
├── Task F3: Real manual QA [unspecified-high]
└── Task F4: Scope fidelity check [deep]
```

---

## TODOs

### Wave 1: Foundation Dependencies

- [ ] **1. 添加go-redis依赖和基础配置**

  **What to do**:
  1. 在 `memory-bus/ingestion/go.mod` 添加 `github.com/redis/go-redis/v9`
  2. 创建 `memory-bus/ingestion/internal/cache/redis.go`:
     - Redis客户端封装结构体
     - Get/Set/Delete方法
     - 带context的超时处理
  3. 从环境变量读取 `REDIS_URL`
  4. 添加连接池配置 (PoolSize: 10, MinIdleConns: 5)

  **Must NOT do**:
  - 不要修改现有业务逻辑，仅添加封装层
  - 不要使用RediSearch等高级功能

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - **Reason**: 标准库集成，无复杂逻辑

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 4, Task 6
  - **Blocked By**: None

  **References**:
  - `control-plane/state-engine/src/cluster.rs:86-91` - Redis连接模式参考
  - `control-plane/state-engine/Cargo.toml:8` - Redis版本参考 (Go使用v9对应Rust 0.23)
  - https://github.com/redis/go-redis/blob/master/options.go - 连接池配置

  **Acceptance Criteria**:
  - [ ] `go mod tidy` 成功，无依赖冲突
  - [ ] `go build ./...` 成功编译
  - [ ] Redis客户端能连接到本地Redis (redis-cli ping 返回 PONG)

  **QA Scenarios**:
  ```
  Scenario: Redis连接测试
  Tool: Bash
  Preconditions: Redis运行在localhost:6379
  Steps:
    1. cd memory-bus/ingestion && go test -run TestRedisConnection -v
    2. 断言输出包含 "Redis connected successfully"
  Expected Result: 测试通过，无错误
  Evidence: .sisyphus/evidence/task-1-redis-connection.txt
  ```

  **Commit**:
  - Message: `feat(memory-bus): add go-redis dependency and client wrapper`
  - Files: `memory-bus/ingestion/go.mod`, `memory-bus/ingestion/internal/cache/redis.go`

---

- [ ] **2. 添加ristretto本地缓存依赖**

  **What to do**:
  1. 在 `memory-bus/ingestion/go.mod` 添加 `github.com/dgraph-io/ristretto v0.1.1`
  2. 创建 `memory-bus/ingestion/internal/cache/local.go`:
     - ristretto缓存封装
     - Get/SetWithTTL方法
     - 配置: NumCounters: 10000, MaxCost: 100MB
  3. 添加Close方法用于资源释放

  **Must NOT do**:
  - 不要使用过期版本 (v0.1.0有已知bug)
  - 不要设置无限内存 (必须设置MaxCost)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 7
  - **Blocked By**: None

  **References**:
  - https://github.com/dgraph-io/ristretto#config - 配置参考
  - https://github.com/eko/gocache/tree/master/store/ristretto - gocache ristretto store实现

  **Acceptance Criteria**:
  - [ ] `go mod tidy` 成功
  - [ ] `go build ./...` 成功
  - [ ] 本地缓存Set/Get工作正常

  **QA Scenarios**:
  ```
  Scenario: 本地缓存基本操作
  Tool: Bash
  Steps:
    1. cd memory-bus/ingestion && go test -run TestLocalCache -v
    2. 测试SetWithTTL和Get操作
  Expected Result: 10000次操作无错误，命中率>0%
  Evidence: .sisyphus/evidence/task-2-local-cache.txt
  ```

  **Commit**:
  - Message: `feat(memory-bus): add ristretto local cache`
  - Files: `memory-bus/ingestion/internal/cache/local.go`

---

- [ ] **3. 添加singleflight请求去重机制**

  **What to do**:
  1. 在 `memory-bus/ingestion/go.mod` 确认 `golang.org/x/sync` 已存在
  2. 创建 `memory-bus/ingestion/internal/cache/dedup.go`:
     - 封装 `singleflight.Group`
     - Do(key, fn) 方法
     - 泛型支持 (Go 1.18+)
  3. 添加context支持，支持超时取消
  4. 添加metrics: 统计去重次数

  **Must NOT do**:
  - 不要自己实现去重逻辑 (容易出错)
  - 不要在singleflight内做复杂操作

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 5, Task 6
  - **Blocked By**: None

  **References**:
  - https://pkg.go.dev/golang.org/x/sync/singleflight - 官方文档
  - https://medium.com/@augustus281/singleflight-in-go - 最佳实践
  - `control-plane/state-engine/src/metrics.rs:86-90` - 监控指标模式

  **Acceptance Criteria**:
  - [ ] 并发10个相同请求，只执行1次
  - [ ] 支持context取消

  **QA Scenarios**:
  ```
  Scenario: 请求去重验证
  Tool: Bash
  Steps:
    1. cd memory-bus/ingestion && go test -run TestSingleflightDedup -v
    2. 启动10个goroutine同时请求相同key
    3. 断言底层函数只被调用1次
  Expected Result: 10个请求返回相同结果，底层调用次数=1
  Evidence: .sisyphus/evidence/task-3-singleflight.txt
  ```

  **Commit**:
  - Message: `feat(memory-bus): add singleflight request deduplication`
  - Files: `memory-bus/ingestion/internal/cache/dedup.go`

---

### Wave 2: Core Implementation

- [ ] **4. 实现memory-bus多级缓存管理器**

  **What to do**:
  1. 创建 `memory-bus/ingestion/internal/cache/manager.go`:
     - CacheManager结构体，包含L1(ristretto)和L2(Redis)
     - Get方法：先查L1，miss则查L2，miss则调用loader
     - Set方法：同时写入L1和L2
     - Delete方法：同时删除L1和L2
  2. 实现缓存键生成函数：`cacheKey(input string) string`
  3. 添加TTL配置：L1默认5分钟，L2默认1小时
  4. 集成singleflight到Get方法

  **Must NOT do**:
  - 不要实现复杂的缓存更新策略 (如Write-Through)
  - 不要添加分布式锁

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
  - **Reason**: 核心逻辑，需要仔细处理多级缓存一致性

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 6
  - **Blocked By**: Task 1, Task 2, Task 3

  **References**:
  - `memory-bus/ingestion/main.go` - 查看现有processInput函数
  - https://github.com/eko/gocache/tree/master/chain - 链式缓存模式

  **Acceptance Criteria**:
  - [ ] L1 hit时不访问L2
  - [ ] L1 miss时自动回填
  - [ ] 集成singleflight防止重复loader调用

  **QA Scenarios**:
  ```
  Scenario: 多级缓存命中测试
  Tool: Bash
  Steps:
    1. 首次请求：L1 miss, L2 miss, 调用loader → 回填L1+L2
    2. 第二次请求（相同key）：L1 hit → 不访问L2
    3. 第三次请求（L1过期）：L2 hit → 回填L1
  Expected Result: 三次请求都返回正确结果，loader只调用1次
  Evidence: .sisyphus/evidence/task-4-cache-manager.txt
  ```

  **Commit**:
  - Message: `feat(memory-bus): implement multi-level cache manager`
  - Files: `memory-bus/ingestion/internal/cache/manager.go`, `*_test.go`

---

- [ ] **5. 在ingestion主流程集成缓存层**

  **What to do**:
  1. 修改 `memory-bus/ingestion/main.go`:
     - 在processInput函数中添加缓存查询
     - 使用CacheManager.Get代替直接API调用
     - 在API调用成功后回填缓存
  2. 添加配置项：CACHE_ENABLED (bool)
  3. 添加 graceful shutdown：关闭缓存连接
  4. 更新main函数初始化CacheManager

  **Must NOT do**:
  - 不要修改API调用参数
  - 不要添加业务逻辑修改

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 10, Task 11, Task 12
  - **Blocked By**: Task 4

  **References**:
  - `memory-bus/ingestion/AGENTS.md` - 模块指南
  - `memory-bus/ingestion/main.go:current` - 现有流程

  **Acceptance Criteria**:
  - [ ] 流程：检查缓存 → miss则调用API → 回填缓存
  - [ ] 支持CACHE_ENABLED开关
  - [ ] 优雅关闭时清理资源

  **QA Scenarios**:
  ```
  Scenario: 端到端缓存流程
  Tool: Bash
  Preconditions: Redis运行，DeepSeek API stub可用
  Steps:
    1. 发送请求input="test query"
    2. 断言：首次调用API，结果写入缓存
    3. 再次发送相同请求
    4. 断言：从缓存返回，未调用API
  Expected Result: 第二次请求延迟<10ms，API调用次数=1
  Evidence: .sisyphus/evidence/task-5-integration.txt
  ```

  **Commit**:
  - Message: `feat(memory-bus): integrate cache layer into ingestion flow`
  - Files: `memory-bus/ingestion/main.go`

---

- [ ] **6. 添加Prometheus缓存监控指标**

  **What to do**:
  1. 在 `memory-bus/ingestion` 添加 Prometheus client库：
     - `github.com/prometheus/client_golang/prometheus`
  2. 创建 `internal/metrics/cache_metrics.go`:
     - `cache_hits_total` CounterVec (按tier: l1/l2)
     - `cache_misses_total` CounterVec
     - `cache_hit_ratio` GaugeVec
     - `dedup_prevented_total` Counter (singleflight统计)
     - `api_calls_total` Counter
  3. 在CacheManager中埋点
  4. 添加 /metrics HTTP endpoint

  **Must NOT do**:
  - 不要添加业务指标（只监控缓存）
  - 不要修改现有metrics（Rust模块已有）

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 10
  - **Blocked By**: Task 5

  **References**:
  - `control-plane/state-engine/src/metrics.rs:86-90` - cache_hit_ratio参考
  - https://prometheus.io/docs/guides/go-application/ - Go Prometheus指南

  **Acceptance Criteria**:
  - [ ] curl http://localhost:8080/metrics 返回缓存指标
  - [ ] 指标包含hits/misses/hit_ratio

  **QA Scenarios**:
  ```
  Scenario: 监控指标验证
  Tool: Bash
  Steps:
    1. 启动服务
    2. curl http://localhost:8080/metrics
    3. grep "cache_hits_total" 验证存在
    4. 发送100个请求（50个不同key）
    5. 再次curl，验证hit_ratio约为0.5
  Expected Result: 指标正确，hit_ratio≈50%
  Evidence: .sisyphus/evidence/task-6-metrics.txt
  ```

  **Commit**:
  - Message: `feat(memory-bus): add Prometheus cache metrics`
  - Files: `memory-bus/ingestion/internal/metrics/*.go`

---

- [ ] **7. 在control-plane/state-engine添加moka本地缓存**

  **What to do**:
  1. 在 `control-plane/state-engine/Cargo.toml` 添加：
     - `moka = { version = "0.12", features = ["future"] }`
  2. 创建 `src/cache/local.rs`:
     - LocalCache结构体，包装moka::future::Cache
     - 实现get_with方法（原子插入）
     - 配置：容量10000，TTL 5分钟
  3. 集成到StateEngine：
     - 在查询Redis前，先查本地缓存
     - 使用get_with防止重复计算
  4. 更新engine.rs添加local_cache字段

  **Must NOT do**:
  - 不要替换Redis（仅作为L1补充）
  - 不要修改现有缓存键格式

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
  - **Reason**: Rust代码，需要精确的类型处理

  **Parallelization**:
  - **Can Run In Parallel**: YES (与Task 4-6并行)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 11
  - **Blocked By**: None

  **References**:
  - https://docs.rs/moka/latest/moka/future/struct.Cache.html - moka文档
  - https://github.com/moka-rs/moka#example - 使用示例
  - `control-plane/state-engine/src/engine.rs:138-165` - 查询流程

  **Acceptance Criteria**:
  - [ ] `cargo build` 成功
  - [ ] 本地缓存命中时返回微秒级延迟

  **QA Scenarios**:
  ```
  Scenario: moka本地缓存集成
  Tool: Bash
  Steps:
    1. cd control-plane/state-engine && cargo test test_local_cache -v
    2. 测试get_with原子插入
    3. 验证并发安全
  Expected Result: 100并发请求，只查询Redis 1次
  Evidence: .sisyphus/evidence/task-7-moka.txt
  ```

  **Commit**:
  - Message: `feat(state-engine): add moka local cache layer`
  - Files: `control-plane/state-engine/src/cache/local.rs`, `Cargo.toml`

---

### Wave 3: Verification & Testing

- [ ] **8. 实现缓存命中率测试**

  **What to do**:
  1. 创建 `memory-bus/ingestion/test/cache_hit_test.go`:
     - 模拟10000次请求，80%重复key
     - 测量命中率是否>75%
     - 测量平均延迟<5ms
  2. 创建负载测试：
     - 100并发goroutine
     - 持续10秒
     - 统计总请求数、命中数、miss数
  3. 测试不同TTL配置的影响
  4. 输出测试报告到 `.sisyphus/evidence/`

  **Must NOT do**:
  - 不要使用mock（用真实Redis）
  - 不要测试API本身（只测试缓存层）

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3
  - **Blocks**: Task F1-F4
  - **Blocked By**: Task 5, Task 6

  **References**:
  - https://github.com/eko/gocache/blob/master/test/cache_test.go - gocache测试

  **Acceptance Criteria**:
  - [ ] 命中率 > 75% (80%重复key场景)
  - [ ] 平均延迟 < 5ms
  - [ ] 无race condition

  **QA Scenarios**:
  ```
  Scenario: 缓存命中率基准测试
  Tool: Bash
  Preconditions: Redis运行，API stub返回固定值
  Steps:
    1. cd memory-bus/ingestion && go test -run TestCacheHitRate -v
    2. 模拟10000请求，80%重复
    3. 收集hits/misses/total
    4. 计算命中率
  Expected Result: hit_rate > 0.75, latency_p99 < 5ms
  Evidence: .sisyphus/evidence/task-8-hit-rate.txt
  ```

  **Commit**:
  - Message: `test(memory-bus): add cache hit rate benchmarks`
  - Files: `memory-bus/ingestion/test/cache_hit_test.go`

---

- [ ] **9. 实现缓存雪崩防护测试**

  **What to do**:
  1. 创建 `memory-bus/ingestion/test/stampede_test.go`:
     - 模拟缓存同时失效场景
     - 100并发请求相同key
     - 验证singleflight只执行1次loader
  2. 测试metric: dedup_prevented_total
  3. 测试极端场景：
     - 缓存过期瞬间的并发请求
     - Redis连接中断后的降级
  4. 对比测试：有/无singleflight的差异

  **Must NOT do**:
  - 不要只测试单线程场景
  - 不要忽略Redis失败场景

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3
  - **Blocks**: Task F1-F4
  - **Blocked By**: Task 5

  **References**:
  - https://medium.com/@augustus281/singleflight-in-go - stampede场景

  **Acceptance Criteria**:
  - [ ] 100并发相同key，loader调用次数=1
  - [ ] 无singleflight时，loader调用次数=100

  **QA Scenarios**:
  ```
  Scenario: 缓存雪崩防护验证
  Tool: Bash
  Steps:
    1. 设置缓存TTL=100ms
    2. 启动100个goroutine，同时请求相同key
    3. 等待缓存过期
    4. 再次同时请求
    5. 统计loader调用次数
  Expected Result: 有singleflight时调用次数=1，无singleflight时=100
  Evidence: .sisyphus/evidence/task-9-stampede.txt
  ```

  **Commit**:
  - Message: `test(memory-bus): add cache stampede protection tests`
  - Files: `memory-bus/ingestion/test/stampede_test.go`

---

- [ ] **10. 性能对比测试**

  **What to do**:
  1. 创建基准测试文件：
     - `test/bench_no_cache_test.go` - 无缓存基线
     - `test/bench_with_cache_test.go` - 有缓存
  2. 测试场景：
     - 冷启动（首次请求）
     - 热缓存（命中）
     - 混合负载（80%命中）
  3. 测量指标：
     - 延迟：p50, p95, p99
     - 吞吐量：req/sec
     - API调用次数
  4. 生成对比报告

  **Must NOT do**:
  - 不要只测平均延迟
  - 不要忽略p99延迟

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3
  - **Blocks**: Task F1-F4
  - **Blocked By**: Task 5

  **References**:
  - https://dave.cheney.net/2013/06/30/how-to-write-benchmarks-in-go - Go benchmark

  **Acceptance Criteria**:
  - [ ] 缓存命中时延迟 < 1ms (p99)
  - [ ] 缓存miss时延迟 < 600ms (API + 缓存写入)
  - [ ] 吞吐量提升 > 50x

  **QA Scenarios**:
  ```
  Scenario: 性能基准测试
  Tool: Bash
  Steps:
    1. go test -bench=BenchmarkNoCache -benchmem
    2. go test -bench=BenchmarkWithCache -benchmem
    3. 对比ops/sec和内存分配
  Expected Result: 
    - WithCache ops/sec > 10000
    - NoCache ops/sec ~ 2 (500ms每次)
  Evidence: .sisyphus/evidence/task-10-bench.txt
  ```

  **Commit**:
  - Message: `test(memory-bus): add performance benchmark tests`
  - Files: `memory-bus/ingestion/test/bench_*_test.go`

---

- [ ] **11. 集成验证测试**

  **What to do**:
  1. 创建 `test/integration_test.go`:
     - 启动完整memory-bus服务
     - 测试全流程：请求→缓存→API→回填
     - 测试错误处理：Redis失败、API失败
     - 测试优雅关闭
  2. 使用testcontainers启动依赖服务
  3. 验证Prometheus指标正常
  4. 测试跨模块集成（如有其他模块依赖）

  **Must NOT do**:
  - 不要mock外部服务（用testcontainers）
  - 不要跳过错误场景

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3
  - **Blocks**: Task F1-F4
  - **Blocked By**: Task 5, Task 6

  **Acceptance Criteria**:
  - [ ] 全流程测试通过
  - [ ] 错误场景处理正确
  - [ ] 指标采集正常

  **QA Scenarios**:
  ```
  Scenario: 端到端集成测试
  Tool: Bash
  Preconditions: Docker运行
  Steps:
    1. docker-compose up -d redis
    2. cd memory-bus/ingestion && go test -run TestIntegration -v
    3. 验证：请求→缓存→API→回填→指标
  Expected Result: 所有场景通过
  Evidence: .sisyphus/evidence/task-11-integration.txt
  ```

  **Commit**:
  - Message: `test(memory-bus): add integration tests`
  - Files: `memory-bus/ingestion/test/integration_test.go`

---

## Final Verification Wave

- [ ] **F1. Plan Compliance Audit**

  **What to verify**:
  1. 所有TODO任务已完成
  2. 代码覆盖率 > 80%
  3. 无编译错误 (go build, cargo build)
  4. 所有测试通过 (go test, cargo test)
  5. Prometheus指标正确导出
  6. 文档已更新

  **Agent**: `oracle`
  **Skills**: []

  **Evidence**:
  - [ ] 覆盖率报告: `.sisyphus/evidence/coverage.html`
  - [ ] 构建日志: `.sisyphus/evidence/build.log`
  - [ ] 测试报告: `.sisyphus/evidence/test-report.txt`

---

- [ ] **F2. Code Quality Review**

  **What to verify**:
  1. `go vet ./...` 无警告
  2. `cargo clippy` 无警告
  3. `gofmt -l .` 无未格式化文件
  4. `rustfmt --check` 通过
  5. 无 `TODO` / `FIXME` 注释
  6. 无 `println` / `console.log` (应使用tracing/log)

  **Agent**: `unspecified-high`
  **Skills**: []

  **Evidence**:
  - [ ] Lint报告: `.sisyphus/evidence/lint.txt`

---

- [ ] **F3. Real Manual QA**

  **What to verify**:
  1. 实际启动服务
  2. 发送真实请求
  3. 验证缓存行为（首次miss，二次hit）
  4. 验证Prometheus指标
  5. 验证singleflight（并发请求）
  6. 截图/录屏证据

  **Agent**: `unspecified-high` + `playwright`
  **Skills**: [`playwright`]

  **Evidence**:
  - [ ] QA报告: `.sisyphus/evidence/manual-qa.txt`
  - [ ] 截图: `.sisyphus/evidence/qa-screenshots/`

---

- [ ] **F4. Scope Fidelity Check**

  **What to verify**:
  1. 所有Must Have已实现
  2. 所有Must NOT Have未引入
  3. 无范围蔓延（只改了计划的文件）
  4. 无无关文件修改
  5. 提交历史符合计划

  **Agent**: `deep`
  **Skills**: []

---

## Commit Strategy

### Commit Message Format
```
<type>(<scope>): <subject>

<body>

Refs: <task-number>
```

**Types**: feat, fix, test, docs, refactor, perf

**Examples**:
- `feat(memory-bus): add go-redis dependency and client wrapper`
- `feat(memory-bus): implement multi-level cache manager`
- `test(memory-bus): add cache hit rate benchmarks`

### Commit Grouping
1. **Phase 1** (Tasks 1-3): 依赖添加 - 单个commit
2. **Phase 2** (Tasks 4-6): 核心实现 - 每个task一个commit
3. **Phase 3** (Tasks 7-10): 测试 - 每个task一个commit
4. **Final** (F1-F4): Review - 无代码commit

---

## Success Criteria

### Verification Commands
```bash
# Build
cd memory-bus/ingestion && go build ./...
cd control-plane/state-engine && cargo build

# Test
cd memory-bus/ingestion && go test -v ./...
cd control-plane/state-engine && cargo test

# Lint
cd memory-bus/ingestion && go vet ./...
cd control-plane/state-engine && cargo clippy

# Metrics check
curl http://localhost:8080/metrics | grep cache
```

### Final Checklist
- [ ] 所有TODO任务已完成
- [ ] 缓存命中率 > 75%
- [ ] API调用减少 > 80%
- [ ] 无缓存穿透/雪崩问题
- [ ] Prometheus指标正常
- [ ] 代码覆盖率 > 80%
- [ ] 所有lint通过
- [ ] 文档已更新

---

## Additional Notes

### Known Limitations
1. vector-kv 模块只添加基础Redis客户端stub（非完整实现）
2. 缓存预热机制在Phase 2实现
3. 分布式锁不在本次范围

### Future Work
- [ ] 实现缓存预热机制（启动时预加载热点数据）
- [ ] 添加缓存更新通知（Redis pub/sub）
- [ ] 实现分布式缓存一致性（当多个实例时）
- [ ] 添加缓存可视化Dashboard

### Dependencies
- go-redis/v9
- ristretto v0.1.1
- golang.org/x/sync/singleflight
- prometheus/client_golang
- moka 0.12

---

**Plan saved to**: `.sisyphus/plans/cache-reuse-implementation.md`

**To start execution, run**: `/start-work cache-reuse-implementation`



