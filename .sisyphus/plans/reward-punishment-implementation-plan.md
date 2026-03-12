# SMA-OS 奖惩机制实施计划

## ✅ 架构兼容性评估结论

**现有架构完全支持奖惩机制，无需重构。**

### 兼容性矩阵

| 奖惩组件 | 现有技术基础 | 兼容性 | 实施方式 |
|---------|-------------|--------|---------|
| **Reward Calculator** | Evaluator (Critic) 已有审计框架 | ✅ 100% | 扩展现有模块 |
| **Reward/Punishment Events** | Event Sourcing (State Engine) | ✅ 100% | 添加新 Event 类型 |
| **Punishment Enforcer** | Fractal Gateway (硬编码网关) | ✅ 100% | 扩展权限控制 |
| **Merit Ledger** | Vector-KV + ClickHouse | ✅ 100% | 扩展查询接口 |
| **Self-Reflection** | Worker 节点架构 | ✅ 100% | 添加提示词更新 |
| **Promotion/Demotion** | Scheduler (亲和性调度) | ✅ 100% | 扩展调度权重 |

---

## 🎯 增量添加方案（推荐）

### Phase 1: 核心奖惩计算（4-6周）

#### 1.1 Evaluator 扩展（2周）

**目标**：将简单的 "准奏/驳回" 扩展为完整的六科给事中考核体系

**当前代码**（`orchestration/evaluator/main.go`）:
```go
type VersionedReject struct {
    TaskID string
    RejectedVersion uint64
    Reason string
    RollbackTo uint64
}

func (e *EvaluatorAgent) AuditTaskResult(...) *VersionedReject {
    if result == "invalid_schema" { return &VersionedReject{...} }
    return nil
}
```

**需要扩展为**:
```go
type EvaluationResult struct {
    TaskID string
    // 奖惩分数
    RewardScore float64        // 0-1，军功
    PunishmentScore float64    // 0-1，过失
    // 考核维度
    CompletionRate float64     // 任务完成度（战功）
    ConstraintCompliance float64  // 约束遵守度（军纪）
    TimeEfficiency float64     // 时间效率（粮草消耗）
    TokenEfficiency float64    // Token效率（资源消耗）
    QualityScore float64       // 质量分数（战利品质）
    // 元数据
    Timestamp int64
    AgentID string
    TaskType string
    Version uint64
}

func (e *EvaluatorAgent) EvaluateTask(...) *EvaluationResult {
    // 1. 多维度评分计算
    // 2. 归一化处理
    // 3. 生成结构化结果
}
```

**关键工作项**:
- [ ] 创建 `EvaluationResult` 结构体
- [ ] 实现多维度评分算法
- [ ] 集成外部验证器（用户反馈、LLM Critic）
- [ ] 添加测试覆盖

#### 1.2 State Engine 扩展（2周）

**目标**：添加 Reward/Punishment Event 类型

**在 `control-plane/state-engine/src/models.rs` 添加**:
```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventType {
    StateTransition,
    ToolCall,
    ContextChange,
    // 新增
    RewardGranted,      // 军功授予
    PunishmentApplied,  // 过失记录
    Promotion,          // 升官
    Demotion,           // 降级
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RewardEvent {
    pub agent_id: String,
    pub task_id: String,
    pub reward_score: f64,
    pub reason: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PunishmentEvent {
    pub agent_id: String,
    pub task_id: String,
    pub punishment_score: f64,
    pub reason: String,
    pub timestamp: i64,
}
```

**关键工作项**:
- [ ] 定义新 Event 类型
- [ ] 更新 Event Sourcing 逻辑
- [ ] 更新 Snapshot 生成逻辑
- [ ] 添加 Redis/PostgreSQL 序列化支持

#### 1.3 Fractal Gateway 扩展（2周）

**目标**：实现 O(1) 硬编码的资源配额调整

**在 `control-plane/fractal-gateway/` 添加**:
```rust
// 奖惩执行器
pub struct PunishmentEnforcer {
    ebpf_probe: Arc<EbpfProbe>,
    quota_cache: DashMap<String, ResourceQuota>,
}

impl PunishmentEnforcer {
    // O(1) 硬编码执行
    pub fn apply_reward(&self, agent_id: &str, reward_score: f64) {
        // 纳秒级生效：
        // 1. 增加资源配额
        // 2. 提升调度优先级
        // 3. 扩大权限额度
    }
    
    pub fn apply_punishment(&self, agent_id: &str, punishment_score: f64) {
        // O(1) 硬编码：
        // 1. 降低资源配额
        // 2. 强制限流
        // 3. 收紧递归深度
    }
}
```

**关键工作项**:
- [ ] 创建 PunishmentEnforcer 模块
- [ ] 集成 eBPF 探针
- [ ] 实现 O(1) 配额调整
- [ ] 添加安全边界校验

---

### Phase 2: 奖惩持久化与查询（2-4周）

#### 2.1 Memory Bus 扩展（2周）

**目标**：实现 Merit Ledger（军功簿）的存储与查询

**在 `memory-bus/vector-kv/` 添加**:
```go
// Merit Ledger 查询接口
type MeritLedger struct {
    db *HybridDBManager
}

func (ml *MeritLedger) RecordReward(event RewardEvent) error {
    // 写入 Weaviate + FoundationDB
    // 打上 Version + Tenant + TraceID + Score 标签
}

func (ml *MeritLedger) GetAgentMerit(agentID string) (*MeritSummary, error) {
    // 查询累计奖惩分数
    // 本地热缓存 < 1ms
}

func (ml *MeritLedger) QueryHighRewardPaths(minScore float64) ([]TaskPath, error) {
    // 按 RewardScore 阈值查询历史高奖励路径
}
```

**关键工作项**:
- [ ] 扩展 HybridDBManager 支持奖惩事件
- [ ] 实现标签体系（Version + Tenant + TraceID + Score）
- [ ] 添加本地热缓存
- [ ] 实现异步 GC 压缩逻辑

#### 2.2 State Engine 崩溃恢复（1周）

**目标**：状态 Hydration 时自动恢复奖惩记录

**在 `control-plane/state-engine/src/engine.rs` 添加**:
```rust
impl StateEngine {
    pub async fn hydrate_with_merit(&self, agent_id: &str) -> Result<AgentState> {
        // 1. 从 Snapshot 恢复基础状态
        // 2. 从 Merit Ledger 恢复累计奖惩分数
        // 3. 确保进化状态不丢失
    }
}
```

**关键工作项**:
- [ ] 扩展状态恢复逻辑
- [ ] 集成 Merit Ledger 查询
- [ ] 添加恢复测试

---

### Phase 3: 进化闭环（2-4周）

#### 3.1 Worker 节点扩展（2周）

**目标**：常驻 Worker 接收自反思诏令

**在 Execution Layer 添加**:
```go
// SelfReflection 模块
type SelfReflection struct {
    agentID string
    cumulativeReward float64
    promptTemplate string
}

func (sr *SelfReflection) UpdatePrompt(evaluation EvaluationResult) {
    // 根据奖惩分数动态更新提示词
    if evaluation.RewardScore > 0.8 {
        sr.promptTemplate = fmt.Sprintf(
            "你累计军功 %.2f，战功显赫，当继续保持以下策略...",
            sr.cumulativeReward,
        )
    } else if evaluation.PunishmentScore > 0.7 {
        sr.promptTemplate = fmt.Sprintf(
            "你累计过失 %.2f，军纪涣散，当引以为戒，改进如下...",
            evaluation.PunishmentScore,
        )
    }
}
```

**关键工作项**:
- [ ] 创建 SelfReflection 模块
- [ ] 集成提示词更新逻辑
- [ ] 添加测试

#### 3.2 Scheduler 扩展（2周）

**目标**：优先复用历史高奖励路径

**在 `orchestration/scheduler/main.go` 添加**:
```go
type MeritAwareScheduler struct {
    baseScheduler *Scheduler
    meritLedger *MeritLedger
}

func (mas *MeritAwareScheduler) Schedule(task Task) (Worker, error) {
    // 1. 查询历史高奖励路径
    highRewardPaths := mas.meritLedger.QueryHighRewardPaths(0.8)
    
    // 2. 优先调度至高奖励 Worker
    // 3. 调整亲和性权重
}
```

**关键工作项**:
- [ ] 扩展 Scheduler 支持 Merit-aware 调度
- [ ] 实现高奖励路径优先逻辑
- [ ] 更新亲和性权重算法

---

## 📊 需要修改的模块清单

### 必改模块（核心）

| 模块 | 文件 | 改动类型 | 工作量 | 优先级 |
|------|------|---------|--------|--------|
| **evaluator** | `main.go` | 扩展 | 中 | P0 |
| **state-engine** | `models.rs` | 扩展 | 小 | P0 |
| **state-engine** | `engine.rs` | 扩展 | 中 | P0 |
| **fractal-gateway** | 新增 `punishment.rs` | 新增 | 中 | P0 |

### 扩展模块（持久化）

| 模块 | 文件 | 改动类型 | 工作量 | 优先级 |
|------|------|---------|--------|--------|
| **vector-kv** | 新增 `merit_ledger.go` | 新增 | 中 | P1 |
| **state-engine** | `engine.rs` (hydrate) | 扩展 | 小 | P1 |

### 增强模块（进化闭环）

| 模块 | 文件 | 改动类型 | 工作量 | 优先级 |
|------|------|---------|--------|--------|
| **sandbox-daemon** | 新增 `reflection.rs` | 新增 | 中 | P2 |
| **scheduler** | `main.go` | 扩展 | 中 | P2 |

### 文档更新

| 文件 | 改动内容 | 工作量 |
|------|---------|--------|
| `AGENTS.md` (evaluator) | 添加 Reward/Punishment 章节 | 小 |
| `AGENTS.md` (state-engine) | 添加 Event 类型说明 | 小 |
| `AGENTS.md` (fractal-gateway) | 添加 PunishmentEnforcer 说明 | 小 |
| `AGENTS.md` (新增) | Merit Ledger 模块文档 | 小 |

---

## ⏱️ 实施时间线

```
Week 1-2:  Phase 1.1 - Evaluator 扩展
Week 3-4:  Phase 1.2 - State Engine 扩展
Week 5-6:  Phase 1.3 - Fractal Gateway 扩展
Week 7-8:  Phase 2.1 - Memory Bus 扩展
Week 9:    Phase 2.2 - 崩溃恢复
Week 10-11: Phase 3.1 - Worker 扩展
Week 12-13: Phase 3.2 - Scheduler 扩展
Week 14:   集成测试 & 文档更新
```

**总计：14周（约3.5个月）**

---

## 🎯 关键里程碑

- **M1 (Week 6)**: 核心奖惩计算与执行完成
- **M2 (Week 9)**: 奖惩持久化与恢复完成
- **M3 (Week 13)**: 进化闭环完成
- **M4 (Week 14)**: 集成测试通过，文档完整

---

## 📝 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| eBPF 集成复杂度 | 中 | 高 | 提前进行 PoC 验证 |
| 性能目标未达成 | 低 | 高 | 持续基准测试 |
| 数据迁移 | 低 | 中 | 保持向后兼容 |

---

## ✅ 决策确认

**问题**：是否需要 `/init-deep` 重新生成 AGENTS.md？

**答案**：**不需要**

**理由**：
1. 现有架构完全支撑
2. 只需增量添加/扩展模块
3. 保持现有 AGENTS.md，仅添加奖惩相关章节

**建议**：
- 实施完奖惩机制后，再统一更新 AGENTS.md
- 或者边实施边更新对应模块的 AGENTS.md

---

**计划创建完成！** 🎉

是否需要我开始实施 Phase 1.1（Evaluator 扩展）？ 🚀
