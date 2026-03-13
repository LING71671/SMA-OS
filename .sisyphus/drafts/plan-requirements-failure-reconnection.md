# SMA-OS 失败重连和任务分配实现计划 - 需求确认草稿

## 确认的功能范围

用户已选择实现以下 P0 优先级功能：

### 功能 1: DAG Manager 任务失败重试机制
**位置**: `orchestration/manager/main.go`
**当前问题**:
- dispatchWorker (第 130-143 行) 只有成功路径
- Failed 状态定义 (第 17 行) 但未处理
- 无超时控制
- 无重试策略

**需求**:
- [ ] 添加任务失败重试机制（指数退避）
- [ ] 添加任务超时控制（context.WithTimeout）
- [ ] 添加失败状态传播
- [ ] 配置化重试参数（最大重试次数、退避策略）
- [ ] 与 state-engine 集成（记录失败事件）

**参考实现**:
- `control-plane/state-engine/src/failover.rs` - 阈值和事件机制
- `control-plane/state-engine/src/cluster.rs` - execute_with_retry 模式

### 功能 2: Worker 健康检查和故障转移
**位置**: `orchestration/scheduler/main.go`
**当前问题**:
- WorkerNode 结构有 Available 字段，但无健康检查
- 无心跳机制
- 无故障 Worker 自动剔除
- 无任务重新分配

**需求**:
- [ ] 添加 Worker 心跳机制
- [ ] 添加健康状态跟踪（Healthy/Degraded/Unhealthy）
- [ ] 添加故障 Worker 自动剔除
- [ ] 添加任务重新分配逻辑
- [ ] 与 FailoverManager 类似的事件广播

**参考实现**:
- `execution-layer/sandbox-daemon/src/pool.rs` - VM 健康检查 (第 165-186 行)
- `control-plane/state-engine/src/failover.rs` - 健康检查和故障转移

## 技术决策

### 语言
- Go 1.25+ (与现有 orchestration 模块一致)

### 依赖
- 使用标准库 `context` 处理超时
- 使用 `time` 处理退避算法
- 使用 `sync` 处理并发安全
- 可选：`github.com/cenkalti/backoff/v4` 提供高级退避策略

### 架构原则
1. 向后兼容：不破坏现有 API
2. 渐进式：先实现基础功能，再优化
3. 可配置：所有参数可配置，有合理默认值
4. 可观测：添加 metrics 和 logging

## 验收标准（草拟）

### 功能 1 验收标准
- [ ] dispatchWorker 在失败时触发重试（最多 3 次）
- [ ] 重试间隔使用指数退避（100ms → 200ms → 400ms）
- [ ] 任务超时后自动标记为 Failed
- [ ] 失败事件记录到 state-engine
- [ ] 可通过配置禁用重试

### 功能 2 验收标准
- [ ] Worker 每 10 秒发送心跳
- [ ] 连续 3 次心跳失败标记为 Unhealthy
- [ ] Unhealthy Worker 自动从可用列表移除
- [ ] 分配给不健康 Worker 的任务自动重新分配
- [ ] 故障事件广播到订阅者

## 风险识别

1. **并发安全**: DAGManager 和 Scheduler 都有 mutex，需要确保新代码线程安全
2. **死锁风险**: 健康检查和任务分配可能竞争锁
3. **无限重试风暴**: 需要最大重试次数和死信队列
4. **脑裂问题**: Worker 网络分区时可能误判为故障

## 测试策略

1. **单元测试**: 每个新函数单独测试
2. **集成测试**: 模拟 Worker 故障场景
3. **混沌测试**: 使用 chaos-tests 框架
4. **性能测试**: 确保健康检查不成为瓶颈

---

*草稿创建时间: 2026-03-13*
*下一步: 生成完整工作计划到 .sisyphus/plans/*
