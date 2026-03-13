# SMA-OS 失败重连和任务分配机制深度分析报告

> 生成时间：2026-03-13  
> 搜索范围：全代码库（Rust/Go/TypeScript）  
> 搜索模式：MAXIMUM SEARCH EFFORT（多代理并行搜索）

---

## 📋 执行摘要

本报告通过 **5 个并行搜索代理** 和 **多次直接工具搜索**（Grep、AST-Grep、文件读取）对 SMA-OS 的失败重连和任务分配机制进行了全面探索。

**关键发现**：
- ✅ **任务分配**：已实现亲和性调度 + Warm Pool 机制
- ⚠️ **失败重连**：Redis/PostgreSQL 已实现，但 DAG 任务失败重试缺失
- ⚠️ **故障转移**：FailoverManager 已实现，但缺少显式断路器
- ❌ **重试策略**：当前仅简单线性退避，建议改为指数退避

---

## 🔍 第一部分：失败重连机制

### 1.1 Redis 集群重连与重试

**文件**：`control-plane/state-engine/src/cluster.rs`

#### 核心实现

```rust
/// 带重试的执行逻辑 (第 132-174 行)
pub async fn execute_with_retry<F, Fut, T>(
    &self,
    key: &str,
    mut operation: F,
) -> Result<T, RedisError>
where
    F: FnMut(MultiplexedConnection) -> Fut,
    Fut: std::future::Future<Output = Result<T, RedisError>>,
{
    let mut last_error = None;

    for attempt in 0..self.config.max_retries {
        match self.get_connection(key).await {
            Ok((node_addr, conn)) => {
                match operation(conn).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        warn!("[Cluster] Node {} operation failed (attempt {}/{}): {}", 
                              node_addr, attempt + 1, self.config.max_retries, e);
                        last_error = Some(e);
                        self.mark_node_unhealthy(&node_addr).await;  // 标记节点不健康
                    }
                }
            }
            Err(e) => { error!("[Cluster] Failed to get connection: {}", e); last_error = Some(e); }
        }
        // 线性退避: 100ms * attempt
        tokio::time::sleep(tokio::time::Duration::from_millis(100 * ((attempt + 1) as u64))).await;
    }
    Err(last_error.unwrap_or_else(|| RedisError::from((redis::ErrorKind::IoError, "All retry attempts failed"))))
}
```

**关键特性**：
| 特性 | 值 |
|------|------|
| 最大重试次数 | 3 次（可配置） |
| 退避算法 | 线性退避（100ms * attempt） |
| 健康检查 | 自动标记不健康节点 |
| 重连机制 | health_check() 自动重连健康节点 |

**建议改进**：
```rust
// 当前：线性退避
sleep(Duration::from_millis(100 * (attempt + 1))).await;

// 建议：指数退避 + 抖动
let base = Duration::from_millis(100);
let max = Duration::from_secs(5);
let delay = base * 2_u32.pow(attempt).min(50); // 避免溢出
let jitter = rand::random::<u64>() % 100;
sleep(delay.min(max) + Duration::from_millis(jitter)).await;
```

### 1.2 PostgreSQL 连接池管理

**文件**：`control-plane/state-engine/src/pool.rs`

```rust
/// PostgreSQL 连接池配置 (第 25-31 行)
async fn create(&self) -> Result<Self::Type, Self::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)           // 最大连接数
        .acquire_timeout(Duration::from_secs(5))  // 获取超时 5 秒
        .connect(&self.pg_url)
        .await
}

/// 连接回收检查 (第 33-48 行)
async fn recycle(&self, conn: &mut Self::Type, _metrics: &Metrics) -> deadpool::managed::RecycleResult<Self::Error> {
    if conn.is_closed() {
        warn!("[Pool] Connection pool is closed");
        return Err(deadpool::managed::RecycleError::Backend(
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Pool closed"))
        ));
    }
    Ok(())
}
```

**关键特性**：
- 使用 `deadpool` 管理连接池
- 自动回收已关闭的连接
- 5 秒获取超时

### 1.3 Redis/PostgreSQL 热-冷存储故障转移

**文件**：`control-plane/state-engine/src/engine.rs` (第 147-165 行)

```rust
/// 查询事件时先尝试 Redis，失败则回退到 PostgreSQL
pub async fn get_events(...) -> Result<Vec<StateEvent>, EngineError> {
    let mut conn = self.redis_client.get_async_connection().await?;
    
    // Try Redis first
    let event_strings: Vec<String> = conn
        .zrangebyscore(&redis_key, &min_score, &max_score)
        .await
        .unwrap_or_default();  // 失败时返回空向量，不 panic
    
    if !event_strings.is_empty() {
        // 解析事件，跳过损坏的条目
        let events: Vec<StateEvent> = event_strings
            .iter()
            .filter_map(|s| serde_json::from_str::<StateEvent>(s).ok())  // 跳过损坏的
            .collect();
        if !events.is_empty() { return Ok(sorted_events); }
    }
    
    // Fallback to PostgreSQL (第 167-224 行)
    // ... PostgreSQL 查询逻辑
}
```

**故障转移策略**：
1. **Redis-first**：优先从 Redis 缓存读取
2. **优雅降级**：Redis 失败时自动回退到 PostgreSQL
3. **跳过损坏**：解析失败时跳过而非失败
4. **自动回填**：全量查询时自动重新填充 Redis 缓存

---

## 🔍 第二部分：任务分配机制

### 2.1 任务分配策略

**文件**：`orchestration/scheduler/main.go` (第 56-76 行)

```go
/// 亲和性调度算法
func (s *FractalClusterScheduler) AssignTask(taskID string, previousHost string) string {
    s.mu.Lock()
    defer s.mu.Unlock()

    // 1. 优先尝试亲和性匹配
    if previousHost != "" {
        for id, w := range s.Workers {
            if w.Available && w.NodeHost == previousHost {
                w.Available = false
                log.Printf("[Scheduler] Affinity Hit: Task %s assigned to existing host %s via Worker %s", 
                           taskID, previousHost, id)
                return id
            }
        }
    }

    // 2. 回退到 Warm Pool
    assignedHost := "host-alpha-x1"
    assignedID := "microvm-pool-" + string(rune(rand.Intn(100)))
    log.Printf("[Scheduler] Affinity Miss: Task %s waking transient worker %s on %s (<5ms).", 
               taskID, assignedID, assignedHost)
    return assignedID
}
```

**调度策略分析**：

| 策略 | 实现状态 | 描述 |
|------|---------|------|
| **亲和性调度** | ✅ 已实现 | 优先分配到之前执行的主机，最大化缓存命中率 |
| **Round-Robin** | ❌ 未实现 | - |
| **一致性哈希** | ❌ 未实现 | - |
| **负载均衡** | ❌ 未实现 | - |

### 2.2 DAG 拓扑排序执行

**文件**：`orchestration/manager/main.go` (第 59-127 行)

```go
/// Kahn 算法实现拓扑排序
func (dm *DAGManager) Execute() error {
    readyQueue := make(chan *TaskNode, len(dm.Nodes))
    completionChan := make(chan string, len(dm.Nodes))
    
    // 1. 入度为 0 的节点入队
    for id, degree := range dm.inDegree {
        if degree == 0 && dm.Nodes[id] != nil {
            readyQueue <- dm.Nodes[id]
        }
    }
    
    // 2. 调度循环
    go func() {
        for {
            select {
            case task := <-readyQueue:
                wg.Add(1)
                go dm.dispatchWorker(task, completionChan, &wg)  // 并发执行
                
            case completedID := <-completionChan:
                completedTasks++
                // 递减依赖节点的入度
                for id, node := range dm.Nodes {
                    if hasDep {  // 如果依赖当前完成的任务
                        dm.inDegree[id]--
                        if dm.inDegree[id] == 0 { readyQueue <- node }
                    }
                }
                if completedTasks == totalTasks { return }
            }
        }
    }()
    
    <-dispatcherDone
    wg.Wait()
    return nil
}
```

**执行流程**：
```
PENDING → RUNNING → COMPLETED
            ↓
         FAILED  (⚠️ 定义但未实现处理)
```

### 2.3 Worker 池管理

**文件**：`execution-layer/sandbox-daemon/src/pool.rs`

```rust
/// VM 获取时健康检查 (第 165-186 行)
pub async fn acquire(&self) -> Option<WarmedVm> {
    let mut available = self.available.write().await;
    
    // 查找健康 VM
    while let Some(vm) = available.pop_front() {
        if vm.is_healthy().await {  // 健康检查
            let mut in_use = self.in_use.write().await;
            in_use.push(vm);
            *self.total_assigned.write().await += 1;
            return in_use.last().cloned();
        } else {
            warn!("[Pool] VM {} is unhealthy, discarding", vm.vm_id);
        }
    }
    
    info!("[Pool] Pool empty, returning None (need on-demand creation)");
    None
}

/// 后台健康检查任务 (第 234-271 行)
pub async fn start_health_check(&self) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(interval_secs));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let mut healthy_vms = VecDeque::new();
                    let mut unhealthy_count = 0usize;
                    
                    for vm in available_guard.iter() {
                        if vm.is_healthy().await { healthy_vms.push_back(vm.clone()); }
                        else { unhealthy_count += 1; }
                    }
                    
                    if unhealthy_count > 0 {
                        warn!("[Pool] Health check: {} unhealthy VMs removed", unhealthy_count);
                        *available.write().await = healthy_vms;
                    }
                }
                _ = shutdown.notified() => { break; }
            }
        }
    });
}
```

**Warm Pool 特性**：
| 特性 | 值 |
|------|------|
| 默认大小 | 50 个 VM |
| 容量范围 | 5 - 100 个 VM |
| 健康检查间隔 | 10 秒 |
| 获取时间 | < 5ms |

---

## 🔍 第三部分：故障恢复与转移

### 3.1 FailoverManager 故障转移

**文件**：`control-plane/state-engine/src/failover.rs`

```rust
/// 故障转移配置 (第 36-57 行)
pub struct FailoverConfig {
    pub check_interval: Duration,      // 健康检查间隔，默认 10 秒
    pub failure_threshold: u32,          // 故障阈值，默认 3 次
    pub recovery_time: Duration,         // 恢复时间，默认 30 秒
    pub auto_failover: bool,             // 自动故障转移启用
}

/// 健康状态 (第 12-23 行)
pub enum HealthStatus { Healthy, Degraded, Unhealthy }

/// 更新健康状态并触发故障转移 (第 114-154 行)
pub async fn update_health(&self, check: HealthCheck) {
    match check.status {
        HealthStatus::Healthy => {
            if old_status != HealthStatus::Healthy {
                info!("[Failover] Component {} recovered to healthy", component);
                let _ = self.event_tx.send(FailoverEvent::RecoveryCompleted { component: component.clone() });
            }
            counts.insert(component.clone(), 0);  // 重置故障计数
        }
        HealthStatus::Degraded | HealthStatus::Unhealthy => {
            let count = counts.entry(component.clone()).or_insert(0);
            *count += 1;
            
            if *count >= self.config.failure_threshold {
                warn!("[Failover] Component {} failed threshold {}/{}", component, count, self.config.failure_threshold);
                if self.config.auto_failover && old_status.is_healthy() {
                    self.trigger_failover(&component, &check.message).await;
                }
            }
        }
    }
}

/// 触发自动故障转移 (第 156-187 行)
async fn trigger_failover(&self, component: &str, reason: &str) {
    error!("[Failover] Triggering failover for component {}: {}", component, reason);
    
    let _ = self.event_tx.send(FailoverEvent::ComponentFailed { 
        component: component.to_string(), 
        reason: reason.to_string() 
    });
    
    // 生产环境中会：
    // 1. 更新 DNS/负载均衡器
    // 2. 提升副本为主节点
    // 3. 通知编排层
    // 4. 更新服务网格
    
    let _ = self.event_tx.send(FailoverEvent::FailoverStarted {
        from: component.to_string(),
        to: format!("{}-replica", component),
    });
    
    tokio::time::sleep(Duration::from_secs(2)).await;  // 模拟故障转移延迟
    
    let _ = self.event_tx.send(FailoverEvent::FailoverCompleted {
        component: component.to_string(),
    });
}
```

**故障转移流程**：
```
Health Check → Degraded/Unhealthy → Count++ → Threshold Reached → Failover Triggered
                     ↓                      ↓                        ↓
               Auto-recover              Recovery              Promote Replica
```

---

## 🔍 第四部分：缺失机制与建议

### 4.1 DAG 任务失败重试（缺失）

**当前代码**：`orchestration/manager/main.go` (第 130-143 行)

```go
/// ⚠️ 只有成功路径，无失败处理
func (dm *DAGManager) dispatchWorker(task *TaskNode, done chan<- string, wg *sync.WaitGroup) {
    defer wg.Done()
    task.Status = Running
    log.Printf("[Worker Scheduler] -> Dispatching Task [%s]...", task.ID)
    time.Sleep(500 * time.Millisecond)  // 模拟执行
    task.Status = Completed  // ← 只有成功
    log.Printf("[Worker Scheduler] <- Task [%s] completed successfully.", task.ID)
    done <- task.ID
}
```

**建议改进**：
```go
/// 建议：添加重试逻辑和失败处理
type FailureConfig struct {
    MaxRetries     int
    RetryDelay     time.Duration
    Timeout        time.Duration
    CancelOnFail   bool
}

func (dm *DAGManager) dispatchWorker(task *TaskNode, done chan<- *TaskResult, wg *sync.WaitGroup) {
    defer wg.Done()
    task.Status = Running
    
    for attempt := 0; attempt <= dm.FailureConfig.MaxRetries; attempt++ {
        ctx, cancel := context.WithTimeout(context.Background(), dm.FailureConfig.Timeout)
        defer cancel()
        
        err := dm.executeWithContext(ctx, task)
        if err == nil {
            task.Status = Completed
            done <- &TaskResult{TaskID: task.ID, Status: Completed, Error: nil}
            return
        }
        
        if attempt < dm.FailureConfig.MaxRetries {
            log.Printf("[Worker] Task %s failed (attempt %d/%d): %v", task.ID, attempt+1, dm.FailureConfig.MaxRetries, err)
            time.Sleep(dm.FailureConfig.RetryDelay * time.Duration(attempt+1))  // 退避
        }
    }
    
    task.Status = Failed
    done <- &TaskResult{TaskID: task.ID, Status: Failed, Error: err}
}
```

### 4.2 断路器模式（缺失）

**当前状态**：代码库中未发现显式断路器实现。

**建议实现**：
```rust
// 新增：control-plane/state-engine/src/circuit_breaker.rs
pub enum CircuitState { Closed, Open, HalfOpen }

pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    pub async fn call<F, Fut, T>(&mut self, operation: F) -> Result<T, CircuitError>
    where F: FnOnce() -> Fut, Fut: Future<Output = Result<T, E>> {
        match self.state {
            CircuitState::Open => {
                if self.last_failure_time.unwrap().elapsed() > self.timeout {
                    self.state = CircuitState::HalfOpen;
                } else {
                    return Err(CircuitError::Open);
                }
            }
            _ => {}
        }
        
        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(_) => {
                self.on_failure();
                Err(CircuitError::Failure)
            }
        }
    }
}
```

### 4.3 指数退避算法改进

**当前实现**（cluster.rs 第 167 行）：
```rust
// 线性退避
tokio::time::sleep(tokio::time::Duration::from_millis(100 * ((attempt + 1) as u64))).await;
```

**建议实现**：
```rust
use rand::Rng;

/// 指数退避 + 抖动
fn calculate_backoff(attempt: u32, base_ms: u64, max_ms: u64) -> Duration {
    let exp = base_ms * 2_u64.pow(attempt.min(6));  // 最大 64x
    let capped = exp.min(max_ms);
    let jitter = rand::thread_rng().gen_range(0..capped / 2);  // 25% 抖动
    Duration::from_millis(capped + jitter)
}

// 使用
for attempt in 0..max_retries {
    match operation().await {
        Ok(result) => return Ok(result),
        Err(e) if attempt < max_retries - 1 => {
            let delay = calculate_backoff(attempt, 100, 5000);
            tokio::time::sleep(delay).await;
        }
        Err(e) => return Err(e),
    }
}
```

---

## 📊 总结矩阵

| 组件 | 重试机制 | 退避算法 | 断路器 | 故障转移 | 健康检查 |
|------|---------|---------|-------|---------|---------|
| **Redis 集群** | ✅ 3 次 | ⚠️ 线性 | ❌ 无 | ✅ 自动 | ✅ 有 |
| **PostgreSQL** | ✅ 连接池 | ✅ 5s 超时 | ⚠️ 回收 | ❌ 无 | ⚠️ 回收 |
| **FailoverManager** | ✅ 阈值 | ❌ 无 | ❌ 无 | ✅ 自动 | ✅ 有 |
| **VM Warm Pool** | ⚠️ 继续剩余 | ❌ 无 | ❌ 无 | ⚠️ 健康 | ✅ 有 |
| **DAG Manager** | ❌ 无 | ❌ 无 | ❌ 无 | ❌ 无 | ❌ 无 |
| **Scheduler** | ❌ 无 | ❌ 无 | ❌ 无 | ❌ 无 | ❌ 无 |

**图例**：✅ 已实现 | ⚠️ 部分实现 | ❌ 缺失

---

## 🎯 改进建议优先级

### P0（紧急）
1. **DAG Manager 重试机制**：当前任务失败无处理，需添加重试和超时
2. **Scheduler Worker 健康检查**：无 Worker 故障检测机制

### P1（重要）
3. **指数退避算法**：替代当前线性退避
4. **断路器模式**：防止级联故障
5. **死信队列**：超过重试次数的任务特殊处理

### P2（建议）
6. **任务超时上下文**：为每个任务添加 context.WithTimeout
7. **亲和性故障转移**：Scheduler 中 Worker 故障时迁移到备用 Worker
8. **Metrics 导出**：Prometheus 指标导出

---

## 📁 相关文件索引

### 失败重连
- `control-plane/state-engine/src/cluster.rs` — Redis 集群重试
- `control-plane/state-engine/src/pool.rs` — PostgreSQL 连接池
- `control-plane/state-engine/src/engine.rs` — 热-冷存储故障转移
- `control-plane/state-engine/src/failover.rs` — FailoverManager

### 任务分配
- `orchestration/scheduler/main.go` — Worker 调度器（亲和性调度）
- `orchestration/manager/main.go` — DAG 执行器（拓扑排序）
- `orchestration/evaluator/main.go` — 输出验证器（版本化回滚）

### 执行层
- `execution-layer/sandbox-daemon/src/pool.rs` — VM Warm Pool 管理
- `execution-layer/sandbox-daemon/src/firecracker.rs` — Firecracker API 客户端

### 测试
- `chaos-tests/src/framework.rs` — 混沌工程框架
- `chaos-tests/src/scenarios/node_failure.rs` — 节点故障场景

---

## 🔗 外部参考

根据 Librarian 代理的研究，以下是分布式系统中常用的最佳实践库：

### Rust 生态
- **`backoff`** — 指数退避算法库
- **`tokio-retry`** — 异步重试策略
- **`failsafe`** — 断路器实现

### Go 生态
- **Temporal** — 工作流编排（含内置重试）
- **Argo Workflows** — Kubernetes 原生工作流
- **gRPC 重试拦截器**

### Firecracker
- 官方文档推荐的故障恢复策略
- MicroVM 快照恢复机制

---

## 📝 备注

- 本报告基于 2026-03-13 的代码库状态
- 部分代码可能处于原型阶段（POC），尚未达到生产级别
- 建议结合 chaos-tests 验证故障恢复能力

---

*报告生成完成 | 搜索代理：5 个并行任务 | 文件读取：15+ 个关键文件*
