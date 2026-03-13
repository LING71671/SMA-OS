# SMA-OS 失败重连和任务分配功能实现计划

## TL;DR

> **目标**: 为 SMA-OS 编排层添加任务失败重试和 Worker 健康检查机制
> 
> **核心交付物**: 
> - DAG Manager 支持失败重试（指数退避）和超时控制
> - Worker Scheduler 支持心跳健康检查和故障转移
> 
> **估算工作量**: Medium (~2-3 天)  
> **并行度**: 高 - 2 个功能可部分并行  
> **关键路径**: Task 1 → Task 6 → Task 10 → Final Verification

---

## Context

### 当前架构
SMA-OS 编排层由三个核心模块组成：

```
orchestration/
├── manager/     # DAG 执行器 - 拓扑排序并发执行
├── scheduler/   # Worker 调度器 - 亲和性调度
└── evaluator/    # 评估器 - 输出验证
```

### 发现的问题

1. **DAG Manager 缺失失败处理** (`manager/main.go:130-143`)
   ```go
   // ⚠️ 当前 dispatchWorker 只有成功路径
   func (dm *DAGManager) dispatchWorker(task *TaskNode, ...) {
       task.Status = Running
       time.Sleep(500 * time.Millisecond)  // 模拟执行
       task.Status = Completed  // ← 只有成功
       done <- task.ID
   }
   ```

2. **Worker 无健康检查** (`scheduler/main.go`)
   ```go
   type WorkerNode struct {
       ID       string
       Type     WorkerType
       NodeHost string
       Available bool  // ← 静态标志，无动态健康检查
   }
   ```

### 参考实现
- `control-plane/state-engine/src/failover.rs` - 健康检查和故障转移
- `execution-layer/sandbox-daemon/src/pool.rs` - VM 健康检查

---

## Work Objectives

### 目标 1: DAG Manager 任务失败重试机制
**实现位置**: `orchestration/manager/main.go`

**具体交付物**:
- [ ] FailureConfig 结构（最大重试次数、超时、退避策略）
- [ ] TaskResult 结构（包含错误信息）
- [ ] 重试逻辑（指数退避）
- [ ] 超时控制（context.WithTimeout）
- [ ] 失败事件传播
- [ ] 单元测试覆盖

### 目标 2: Worker 健康检查和故障转移
**实现位置**: `orchestration/scheduler/main.go`

**具体交付物**:
- [ ] WorkerHealth 结构（健康状态、最后心跳时间）
- [ ] 心跳机制（goroutine + ticker）
- [ ] 健康检查逻辑（连续失败阈值）
- [ ] 故障 Worker 自动剔除
- [ ] 任务重新分配逻辑
- [ ] 单元测试覆盖

### Definition of Done
- [ ] 所有新代码通过 `go test`
- [ ] 所有新代码通过 `go vet`
- [ ] Agent-Executed QA Scenarios 全部通过
- [ ] 代码审查通过（无种族条件、无资源泄漏）

### Must Have (核心需求)
- [ ] DAG 任务失败时自动重试（3 次）
- [ ] 重试使用指数退避（100ms → 200ms → 400ms）
- [ ] Worker 心跳每 10 秒一次
- [ ] 连续 3 次心跳失败标记为故障
- [ ] 故障 Worker 自动从可用列表移除

### Must NOT Have (明确排除)
- [ ] 不使用外部库（纯标准库实现）
- [ ] 不修改现有 API 签名（向后兼容）
- [ ] 不实现死信队列（Phase 2）
- [ ] 不实现断路器模式（Phase 2）

---

## Verification Strategy

### 测试决策
- **基础设施存在**: YES（Go 测试框架已配置）
- **自动化测试**: YES（Tests-after 模式）
- **框架**: 标准库 `testing`

### QA Policy
每个任务必须包含 Agent-Executed QA Scenarios：
- **Frontend/UI**: N/A（后端服务）
- **CLI/TUI**: 使用 `go test -v` 验证
- **API/Backend**: 使用 `go test` + 模拟依赖
- **Library/Module**: 使用单元测试 + 覆盖率检查

**证据保存**: `.sisyphus/evidence/task-{N}-{scenario-slug}.txt`

---

## Execution Strategy

### 并行执行波次

```
Wave 1 (Start Immediately - Foundation):
├── Task 1: DAG Manager FailureConfig 和 TaskResult 结构 [quick]
├── Task 2: Worker Health 结构定义 [quick]
└── Task 3: 重构现有代码（提取公共逻辑）[quick]

Wave 2 (After Wave 1 - Core Logic):
├── Task 4: DAG 重试逻辑实现 [unspecified-high]
├── Task 5: Worker 心跳机制 [unspecified-high]
├── Task 6: Worker 健康检查和故障检测 [unspecified-high]
└── Task 7: Worker 任务重新分配逻辑 [unspecified-high]

Wave 3 (After Wave 2 - Integration):
├── Task 8: DAG Manager 与 state-engine 集成 [unspecified-high]
├── Task 9: Scheduler 事件广播 [unspecified-high]
└── Task 10: 端到端测试和修复 [unspecified-high]

Wave 4 (After Wave 3 - Verification):
├── Task F1: 单元测试覆盖检查 [quick]
├── Task F2: 并发安全审查 (race detector) [unspecified-high]
├── Task F3: 性能基准测试 [unspecified-high]
└── Task F4: 文档更新 [quick]

Critical Path: Task 1 → Task 4 → Task 8 → Task 10 → F1-F4
Parallel Speedup: ~40% faster than sequential
Max Concurrent: 4 (Wave 2)
```

### 依赖矩阵

| Task | Depends On | Blocks | Agent Category |
|------|-----------|---------|---------------|
| 1 | - | 4, 8 | quick |
| 2 | - | 5, 6, 7 | quick |
| 3 | - | 4, 5, 6, 7 | quick |
| 4 | 1, 3 | 8, 10 | unspecified-high |
| 5 | 2, 3 | 6, 7 | unspecified-high |
| 6 | 2, 5 | 7, 9 | unspecified-high |
| 7 | 2, 5, 6 | 9, 10 | unspecified-high |
| 8 | 1, 4 | 10 | unspecified-high |
| 9 | 5, 6, 7 | 10 | unspecified-high |
| 10 | 4, 7, 8, 9 | F1-F4 | unspecified-high |
| F1 | 10 | - | quick |
| F2 | 10 | - | unspecified-high |
| F3 | 10 | - | unspecified-high |
| F4 | 10 | - | quick |

---

## TODOs

### Wave 1: Foundation (Start Immediately)

- [ ] **Task 1: DAG Manager - FailureConfig 和 TaskResult 结构定义**

  **What to do**:
  - 在 `orchestration/manager/main.go` 添加 FailureConfig 和 TaskResult 结构
  - 修改 DAGManager 结构添加 FailureConfig 字段
  - 修改 completionChan 类型从 `chan string` 到 `chan *TaskResult`
  
  **Must NOT do**:
  - 不修改 dispatchWorker 函数逻辑（在 Task 4 实现）
  - 不添加具体重试逻辑
  
  **Recommended Agent Profile**:
  - **Category**: quick
  - **Skills**: []
  - Reason: 纯结构定义，无复杂逻辑
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 4, Task 8
  - **Blocked By**: None
  
  **References**:
  - `orchestration/manager/main.go:10-34` - 现有结构定义
  - `control-plane/state-engine/src/failover.rs:36-57` - FailoverConfig 参考
  
  **Acceptance Criteria**:
  - [ ] `go build` 成功
  - [ ] 新结构可以通过编译
  
  **QA Scenarios**:
  - Tool: Bash
  - Steps:
    1. cd orchestration/manager && go build
    2. echo $?
  - Expected Result: 0
  - Evidence: .sisyphus/evidence/task-1-build-success.txt
  
  **Commit**: YES
  - Message: `feat(manager): add FailureConfig and TaskResult structs`
  - Files: `orchestration/manager/main.go`

---

- [ ] **Task 2: Worker Scheduler - WorkerHealth 结构定义**

  **What to do**:
  - 在 `orchestration/scheduler/main.go` 添加 WorkerHealth 结构
  - 扩展 WorkerNode 结构添加 Health 字段
  
  **Must NOT do**:
  - 不添加心跳逻辑（在 Task 5 实现）
  - 不修改 AssignTask 函数
  
  **Recommended Agent Profile**:
  - **Category**: quick
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 5, Task 6, Task 7
  - **Blocked By**: None
  
  **References**:
  - `orchestration/scheduler/main.go:14-27` - 现有结构
  
  **Acceptance Criteria**:
  - [ ] `go build` 成功
  - [ ] WorkerHealth 结构可以通过编译
  
  **QA Scenarios**:
  - Tool: Bash
  - Steps:
    1. cd orchestration/scheduler && go build
    2. echo $?
  - Expected Result: 0
  - Evidence: .sisyphus/evidence/task-2-build-success.txt
  
  **Commit**: YES
  - Message: `feat(scheduler): add WorkerHealth structures`
  - Files: `orchestration/scheduler/main.go`

---

- [ ] **Task 3: 重构 - 提取公共错误处理逻辑**

  **What to do**:
  - 在 `orchestration/utils/` 创建新的错误处理工具
  - 提取指数退避计算函数
  
  **Must NOT do**:
  - 不修改 manager 或 scheduler 的逻辑
  
  **Recommended Agent Profile**:
  - **Category**: quick
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 4, Task 5, Task 6, Task 7
  - **Blocked By**: None
  
  **References**:
  - `control-plane/state-engine/src/cluster.rs:167` - 当前线性退避实现
  
  **Acceptance Criteria**:
  - [ ] utils 包可以编译
  - [ ] CalculateBackoff 函数可以被其他包引用
  
  **QA Scenarios**:
  - Tool: Bash
  - Steps:
    1. cd orchestration/utils && go test -run TestBackoff
    2. echo $?
  - Expected Result: 0
  - Evidence: .sisyphus/evidence/task-3-backoff-test.txt
  
  **Commit**: YES
  - Message: `refactor(orchestration): add backoff utility function`
  - Files: `orchestration/utils/backoff.go`

---

### Wave 2: Core Logic (After Wave 1)

- [ ] **Task 4: DAG Manager - 实现重试逻辑**

  **What to do**:
  - 重构 dispatchWorker 函数，添加重试循环
  - 使用指数退避策略
  - 添加超时控制
  - 失败时标记任务状态为 Failed
  
  **Must NOT do**:
  - 不修改 Execute() 函数的主循环
  
  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 8, Task 10
  - **Blocked By**: Task 1, Task 3
  
  **References**:
  - `orchestration/manager/main.go:130-143` - 当前 dispatchWorker
  - `control-plane/state-engine/src/cluster.rs:132-174` - execute_with_retry 模式
  
  **Acceptance Criteria**:
  - [ ] dispatchWorker 支持失败重试
  - [ ] 使用指数退避（100ms -> 200ms -> 400ms）
  - [ ] 任务超时后标记为 Failed
  
  **QA Scenarios**:
  - Tool: Bash (go test)
  - Steps:
    1. cd orchestration/manager
    2. go test -v -run TestDispatchWorkerRetry
  - Expected Result: PASS
  - Evidence: .sisyphus/evidence/task-4-retry-test.txt
  
  **Commit**: YES
  - Message: `feat(manager): implement retry logic with exponential backoff`
  - Files: `orchestration/manager/main.go`, `orchestration/manager/main_test.go`

---

- [ ] **Task 5: Worker Scheduler - 实现心跳机制**

  **What to do**:
  - 添加心跳接收 goroutine
  - 维护 Worker 最后心跳时间
  - 启动定期健康检查 ticker
  
  **Must NOT do**:
  - 不实现故障判定逻辑（在 Task 6 实现）
  
  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 6, Task 7
  - **Blocked By**: Task 2, Task 3
  
  **References**:
  - `execution-layer/sandbox-daemon/src/pool.rs:234-271` - VM 健康检查实现
  
  **Acceptance Criteria**:
  - [ ] Worker 可以发送心跳
  - [ ] Scheduler 可以接收心跳
  - [ ] 最后心跳时间被正确更新
  
  **QA Scenarios**:
  - Tool: Bash (go test)
  - Steps:
    1. cd orchestration/scheduler
    2. go test -v -run TestWorkerHeartbeat
  - Expected Result: PASS
  - Evidence: .sisyphus/evidence/task-5-heartbeat-test.txt
  
  **Commit**: YES
  - Message: `feat(scheduler): implement worker heartbeat mechanism`
  - Files: `orchestration/scheduler/main.go`, `orchestration/scheduler/scheduler_test.go`

---

- [ ] **Task 6: Worker Scheduler - 健康检查和故障检测**

  **What to do**:
  - 实现健康检查逻辑（连续 N 次心跳失败判定为故障）
  - 自动标记故障 Worker
  - 从可用列表移除故障 Worker
  
  **Must NOT do**:
  - 不实现任务重新分配（在 Task 7 实现）
  
  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 7, Task 9
  - **Blocked By**: Task 2, Task 5
  
  **References**:
  - `control-plane/state-engine/src/failover.rs:114-154` - update_health 逻辑
  - `execution-layer/sandbox-daemon/src/pool.rs:165-186` - VM 健康检查
  
  **Acceptance Criteria**:
  - [ ] 连续 3 次心跳失败标记为 Unhealthy
  - [ ] Unhealthy Worker 自动从可用列表移除
  
  **QA Scenarios**:
  - Tool: Bash (go test)
  - Steps:
    1. cd orchestration/scheduler
    2. go test -v -run TestWorkerHealthCheck
  - Expected Result: PASS
  - Evidence: .sisyphus/evidence/task-6-health-test.txt
  
  **Commit**: YES
  - Message: `feat(scheduler): implement worker health check and failure detection`
  - Files: `orchestration/scheduler/main.go`

---

- [ ] **Task 7: Worker Scheduler - 任务重新分配逻辑**

  **What to do**:
  - 修改 AssignTask 函数，检查 Worker 健康状态
  - 如果首选 Worker 不健康，自动选择备用 Worker
  - 维护故障 Worker 的任务重分配队列
  
  **Must NOT do**:
  - 不修改亲和性调度逻辑（在保持现有行为的基础上添加健康检查）
  
  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: []
  
  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9, Task 10
  - **Blocked By**: Task 2, Task 5, Task 6
  
  **References**:
  - `orchestration/scheduler/main.go:56-76` - 当前 AssignTask
  
  **Acceptance Criteria**:
  - [ ] AssignTask 跳过不健康 Worker
  - [ ] 故障 Worker 的任务被重新分配
  
  **QA Scenarios**:
  - Tool: Bash (go test)
  - Steps:
    1. cd orchestration/scheduler
    2. go test -v -run TestTaskReassignment
  - Expected Result: PASS
  - Evidence: .sisyphus/evidence/task-7-reassign-test.txt
  
  **Commit**: YES
  - Message: `feat(scheduler): implement task reassignment for unhealthy workers`
  - Files: `orchestration/scheduler/main.go`

---

## Final Verification Wave

### F1: 单元测试覆盖检查
- **Agent**: quick
- **任务**: 运行 `go test -cover` 确保覆盖率 > 80%
- **验收**: `coverage: 80.5% of statements`

### F2: 并发安全审查
- **Agent**: unspecified-high
- **任务**: 运行 `go test -race` 检测种族条件
- **验收**: `PASS` 无种族条件警告

### F3: 性能基准测试
- **Agent**: unspecified-high
- **任务**: 对比重试机制对 DAG 执行性能的影响
- **验收**: P99 延迟增加 < 10%

### F4: 文档更新
- **Agent**: quick
- **任务**: 更新 orchestration/manager/AGENTS.md 和 scheduler/AGENTS.md
- **验收**: 文档包含新的失败处理章节

---

## Commit Strategy

### 提交格式
```
type(scope): description

Body explaining what and why
```

### 提交计划

| Commit | Scope | Message | Files |
|--------|-------|---------|-------|
| 1 | manager | feat(manager): add FailureConfig and TaskResult structs | manager/main.go |
| 2 | scheduler | feat(scheduler): add WorkerHealth and heartbeat structures | scheduler/main.go |
| 3 | manager | feat(manager): implement retry logic with exponential backoff | manager/main.go |
| 4 | scheduler | feat(scheduler): implement worker health check and failover | scheduler/main.go |
| 5 | manager | test(manager): add unit tests for retry mechanism | manager/main_test.go |
| 6 | scheduler | test(scheduler): add unit tests for health check | scheduler/scheduler_test.go |
| 7 | orchestration | docs(orchestration): update AGENTS.md with failure handling | */AGENTS.md |

---

## Success Criteria

### 验证命令
```bash
# Build verification
cd orchestration/manager && go build -o bin/manager .
cd orchestration/scheduler && go build -o bin/scheduler .

# Test verification
cd orchestration/manager && go test -v -race -cover
cd orchestration/scheduler && go test -v -race -cover

# Lint verification
cd orchestration && go vet ./...
```

### 预期输出
```
PASS
coverage: 80.5% of statements
ok      sma-os/orchestration/manager    0.234s

PASS
coverage: 75.2% of statements
ok      sma-os/orchestration/scheduler  0.189s
```

### Final Checklist
- [ ] DAG Manager 任务失败时自动重试（3 次）
- [ ] 重试使用指数退避
- [ ] Worker 心跳每 10 秒一次
- [ ] 故障 Worker 自动剔除
- [ ] 所有测试通过
- [ ] 无种族条件
- [ ] 文档已更新

---

*Plan Version: 1.0*
*Created: 2026-03-13*
*Ready for: /start-work*
