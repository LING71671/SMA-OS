# SMA-OS 身份调度系统影响评估报告

**版本**: 1.0  
**日期**: 2026-03-14  
**评估范围**: SMA-OS全栈身份调度集成  
**状态**: 待评审

---

## 执行摘要

本报告评估在SMA-OS中实现**身份动态调度系统**的技术影响。基于对代码库的全面分析，发现以下关键结论：

| 维度 | 评估结果 | 风险等级 |
|------|----------|----------|
| **当前身份成熟度** | 部分实现（tenant_id + namespace） | 🟡 中等 |
| **技术债务** | 各层身份处理不一致 | 🔴 高 |
| **迁移复杂度** | 高（需修改15+核心文件） | 🔴 高 |
| **预计工期** | 6-8周（2-3个sprint） | 🟡 中等 |
| **向后兼容性** | 可通过feature flag维持 | 🟢 低 |

**核心发现**:
1. ✅ **State Engine已就绪** - tenant_id/namespace已完整实现
2. ⚠️ **Orchestration层缺失** - Task/Worker无身份概念
3. ❌ **Security层待集成** - eBPF策略未绑定身份
4. ⚠️ **Plugin系统部分就绪** - manifest有权限但无运行时身份

---

## 1. 现状分析

### 1.1 各层身份实现矩阵

| 层级 | 模块 | 当前身份支持 | 缺失能力 |
|------|------|--------------|----------|
| **Control Plane** | state-engine | ✅ tenant_id + namespace | 身份生命周期管理 |
| | fractal-gateway | ⚠️ agent_id | ❌ tenant绑定 |
| | teardown-ctrl | ✅ tenant_id | ❌ 身份级联清理 |
| **Orchestration** | manager | ❌ 无 | ❌ Task身份 |
| | scheduler | ❌ 无 | ❌ Worker授权 |
| | evaluator | ❌ 无 | ❌ 身份审计 |
| **Execution** | sandbox-daemon | ⚠️ vm_id | ❌ VM-身份绑定 |
| | stateful-repl | ❌ 无 | ❌ 会话身份 |
| **Memory Bus** | ingestion | ❌ 无 | ❌ 调用者追踪 |
| | vector-kv | ⚠️ tenant_id | ❌ 身份隔离 |
| **Plugin** | core | ⚠️ manifest权限 | ❌ 运行时身份检查 |

### 1.2 关键代码位置

```rust
// ✅ 已就绪：State Event含身份
// control-plane/state-engine/src/models.rs:8-12
pub struct StateEvent {
    pub event_id: Uuid,
    pub tenant_id: String,      // ✅ 已实现
    pub namespace: String,      // ✅ 已实现
    pub version: u64,
    pub payload: serde_json::Value,
}

// ❌ 缺失：Task无身份
// orchestration/manager/main.go:52-58
type TaskNode struct {
    ID string
    ActionName string
    Dependencies []string
    Status TaskStatus
    Payload string
    // ❌ 无OwnerID, PrincipalID, AuthorizationContext
}

// ⚠️ 部分：SecurityPolicy有agent_id但无tenant
// control-plane/fractal-gateway/src/security.rs:18-24
pub struct SecurityPolicy {
    pub agent_id: String,       // ⚠️ 有但无tenant绑定
    pub seccomp_profile: SeccompProfile,
    pub dynamic_quotas: DynamicQuotas,  // ✅ 动态配额已支持
}
```

---

## 2. 分层影响评估

### 2.1 Control Plane层影响

#### 2.1.1 State Engine (state-engine)

**当前状态**: ✅ **已就绪**

| 组件 | 现状 | 需变更 | 风险 |
|------|------|--------|------|
| `StateEvent` | tenant_id + namespace | 无需变更 | 🟢 无 |
| `Snapshot` | tenant_id + namespace | 无需变更 | 🟢 无 |
| Redis keys | `events:{tenant}:{namespace}` | 无需变更 | 🟢 无 |
| Rate limiter | per-tenant | 无需变更 | 🟢 无 |

**结论**: State Engine已完整支持身份隔离，**零变更**。

#### 2.1.2 Fractal Gateway (fractal-gateway)

**当前状态**: ⚠️ **需增强**

```rust
// 需修改: security.rs
pub struct SecurityPolicy {
    pub agent_id: String,
    pub tenant_id: String,              // 🆕 新增
    pub namespace: String,              // 🆕 新增
    pub identity_scope: IdentityScope, // 🆕 新增
    pub seccomp_profile: SeccompProfile,
    pub dynamic_quotas: DynamicQuotas,
}

// 需新增: identity-aware enforcement
pub async fn enforce_by_identity(
    &self,
    identity: &IdentityContext,
    syscall: Syscall,
) -> Result<(), SecurityError>;
```

| 变更项 | 影响范围 | 复杂度 | 风险 |
|--------|----------|--------|------|
| SecurityPolicy结构 | 安全策略全链路 | 中 | 🟡 eBPF map结构变更 |
| eBPF quota lookup | O(1) → O(1) (tenant+agent key) | 低 | 🟢 性能无影响 |
| 动态配额API | 新增tenant参数 | 低 | 🟢 向后兼容 |

**关键变更**:
- **文件**: `control-plane/fractal-gateway/src/security.rs`
- **行数**: +50-80行
- **API变化**: `update_quotas()` 新增 `tenant_id` 参数

#### 2.1.3 Teardown Controller (teardown-ctrl)

**当前状态**: ✅ **已就绪**

`TeardownTarget`已含tenant_id，只需增强级联清理逻辑：

```rust
// 现有: controller.rs:21-27
pub struct TeardownTarget {
    pub tenant_id: String,     // ✅ 已存在
    pub namespace: String,
    pub task_group_id: Uuid,
    pub force: bool,
}

// 需增强: 级联身份清理
pub async fn cascade_teardown_by_identity(
    &self,
    identity_id: &str,
) -> Result<TeardownReport>;
```

---

### 2.2 Orchestration层影响

#### 2.2.1 Manager (DAG执行)

**当前状态**: ❌ **缺失严重**

| 结构/函数 | 当前签名 | 需变更 | 破坏? |
|-----------|----------|--------|-------|
| `TaskNode` | 5个字段 | +4个身份字段 | ⚠️ 是 |
| `TaskResult` | 7个字段 | +2个执行者字段 | ⚠️ 是 |
| `AddTask()` | 无身份验证 | 需验证身份声明 | ⚠️ 是 |
| `Execute()` | 无身份传递 | 通道传递身份 | ⚠️ 是 |
| JSON格式 | 旧格式 | 新identity字段 | ⚠️ 是 |

**代码变更示例**:

```go
// orchestration/manager/main.go:52-58
// BEFORE:
type TaskNode struct {
    ID string
    ActionName string
    Dependencies []string
    Status TaskStatus
    Payload string
}

// AFTER:
type TaskNode struct {
    ID string
    ActionName string
    Dependencies []string
    Status TaskStatus
    Payload string
    // 🆕 身份字段
    OwnerID string
    PrincipalID string
    AuthorizationContext string
    IdentityClaims map[string]string
}

// API变更: DAG提交
// BEFORE: POST /api/v1/tasks (旧JSON)
// AFTER: POST /api/v1/tasks (含identity字段)
```

**DAG执行流修改**:

```
AddTask(task + identity)
    ↓
Validate identity claims
    ↓
Enqueue to readyQueue (with identity)
    ↓
dispatchWorker(task + identity)
    ↓
Pass identity to sandbox-daemon
    ↓
TaskResult includes ExecutedBy
```

**迁移策略**:
1. 阶段1: 新增字段（nullable），旧API仍支持
2. 阶段2: 双格式JSON解析
3. 阶段3: 强制身份验证

#### 2.2.2 Scheduler (Worker调度)

**当前状态**: ❌ **缺失严重**

| 组件 | 当前 | 需变更 | 复杂度 |
|------|------|--------|--------|
| `WorkerNode` | 5个字段 | +4个授权字段 | 中 |
| `AssignTask()` | 2参数 | +1 identity参数 | 高 |
| 调度逻辑 | 无身份检查 | 需授权验证 | 高 |
| 亲和性 | 基于previousHost | 增加身份亲和 | 中 |

**关键变更**:

```go
// orchestration/scheduler/main.go:69-75
// BEFORE:
type WorkerNode struct {
    ID string
    Type WorkerType
    NodeHost string
    Available bool
    Health WorkerHealth
}

// AFTER:
type WorkerNode struct {
    ID string
    Type WorkerType
    NodeHost string
    Available bool
    Health WorkerHealth
    // 🆕 授权字段
    AuthorizedPrincipals []string
    SecurityLevel int
    AllowedTenants []string
}

// 调度函数签名变更
// BEFORE:
func (s *FractalClusterScheduler) AssignTask(
    taskID string,
    previousHost string,
) string

// AFTER:
func (s *FractalClusterScheduler) AssignTask(
    taskID string,
    identity *IdentityContext,  // 🆕 新增
    previousHost string,
) (string, error)
```

**调度算法增强**:

```go
// 新增: 身份授权验证
func (s *FractalClusterScheduler) validateIdentityForWorker(
    identity *IdentityContext,
    worker *WorkerNode,
) bool {
    // 检查principal白名单
    if len(worker.AuthorizedPrincipals) > 0 {
        if !contains(worker.AuthorizedPrincipals, identity.PrincipalID) {
            return false
        }
    }
    // 检查tenant白名单
    if len(worker.AllowedTenants) > 0 {
        if !contains(worker.AllowedTenants, identity.TenantID) {
            return false
        }
    }
    // 检查安全等级
    if identity.SecurityLevel < worker.SecurityLevel {
        return false
    }
    return true
}
```

#### 2.2.3 Evaluator (输出验证)

**当前状态**: ❌ **缺失**

**变更较小**:

```go
// orchestration/evaluator/main.go:13-18
// BEFORE:
type VersionedReject struct {
    TaskID string
    RejectedVersion uint64
    Reason string
    RollbackTo uint64
}

// AFTER:
type VersionedReject struct {
    TaskID string
    RejectedVersion uint64
    Reason string
    RollbackTo uint64
    PrincipalID string           // 🆕 决策身份
    RejectedIdentity string      // 🆕 产出者身份
    AuditTrail string           // 🆕 审计日志
}

// 函数签名变更
// BEFORE:
func AuditTaskResult(taskID string, version uint64, result string) *VersionedReject

// AFTER:
func AuditTaskResult(
    taskID string,
    identity *IdentityContext,  // 🆕 新增
    version uint64,
    result string,
) *VersionedReject
```

---

### 2.3 Execution Layer影响

#### 2.3.1 Sandbox Daemon (MicroVM)

**当前状态**: ⚠️ **需增强**

| 组件 | 当前 | 需变更 | 复杂度 |
|------|------|--------|--------|
| `FirecrackerVM` | vm_id | +identity_id + agent_id | 中 |
| `WarmedVm` | 基础字段 | +identity绑定 | 中 |
| `WarmPool` | 单一队列 | 身份感知队列 | 高 |
| `AssignMicroVM` | 无身份参数 | +identity参数 | 中 |

**代码变更**:

```rust
// execution-layer/sandbox-daemon/src/microvm.rs:88-96
// BEFORE:
pub struct FirecrackerVM {
    pub vm_id: String,
    pub socket_path: String,
    pub config: VmConfig,
    pub state: VmState,
}

// AFTER:
pub struct FirecrackerVM {
    pub vm_id: String,
    pub identity_id: Option<String>,    // 🆕 身份绑定
    pub agent_id: Option<String>,        // 🆕 Agent绑定
    pub socket_path: String,
    pub config: VmConfig,
    pub state: VmState,
    pub assigned_at: Option<DateTime<Utc>>, // 🆕 绑定时间
}

// Pool结构增强
// execution-layer/sandbox-daemon/src/pool.rs:104-116
pub struct WarmPool {
    config: PoolConfig,
    available: Arc<RwLock<VecDeque<WarmedVm>>>,
    // 🆕 身份感知队列
    identity_pools: Arc<RwLock<HashMap<String, VecDeque<WarmedVm>>>>,
    in_use: Arc<RwLock<Vec<WarmedVm>>>,
}

// gRPC API变更 (sma-proto/sandbox.proto:17-22)
// BEFORE:
message AssignRequest {
    string tenant_id = 1;
    string namespace = 2;
    string action_name = 3;
    string payload_json = 4;
}

// AFTER:
message AssignRequest {
    string tenant_id = 1;
    string namespace = 2;
    string action_name = 3;
    string payload_json = 4;
    IdentityContext identity = 5;  // 🆕 新增
}
```

**安全风险**:
- VM可能伪造身份 → 需签名验证
- 跨租户数据泄漏 → eBPF强制隔离
- 配额规避 → 全局身份配额

#### 2.3.2 Stateful REPL

**当前状态**: ❌ **缺失**

```rust
// execution-layer/stateful-repl/src/main.rs
// 需新增:
pub struct ReplSession {
    pub session_id: String,
    pub vm_id: String,
    pub identity_id: Option<String>,    // 🆕 身份绑定
    pub created_at: Instant,
}
```

---

### 2.4 Memory Bus层影响

#### 2.4.1 Ingestion (意图提取)

**当前状态**: ❌ **缺失**

| 组件 | 当前 | 需变更 | 复杂度 |
|------|------|--------|--------|
| `ProcessInput()` | 2参数 | +identity参数 | 中 |
| `ParsedIntent` | 5字段 | +身份来源 | 低 |
| 缓存key | input hash | identity+input hash | 中 |

**变更**:

```go
// memory-bus/ingestion/main.go:133
// BEFORE:
func ProcessInput(
    userInput string,
    cacheManager *cache.CacheManager,
) (*ParsedIntent, error)

// AFTER:
func ProcessInput(
    userInput string,
    identity IdentityContext,        // 🆕 新增
    cacheManager *cache.CacheManager,
) (*ParsedIntent, error)

// 缓存key变更
// BEFORE: intent:{sha256(input)}
// AFTER:  intent:{identity}:{sha256(identity+input)}
```

**迁移挑战**: 缓存失效策略（需双key过渡）

#### 2.4.2 Vector KV

**当前状态**: ⚠️ **部分就绪**

已含tenant_id，需增强身份隔离。

---

### 2.5 Plugin系统影响

**当前状态**: ⚠️ **部分就绪**

| 组件 | 当前 | 需变更 | 复杂度 |
|------|------|--------|--------|
| `PluginManifest` | 静态权限 | +身份范围 | 高 |
| `PluginPermission` | resource+actions | +identity_scope | 高 |
| `PluginConfig` | tenant+namespace | +IdentityContext | 中 |
| `PluginSandbox` | ResourceLimits | +IdentityBinding | 中 |
| `Capability检查` | 按类型 | +身份过滤 | 高 |

**代码变更**:

```rust
// plugins/core/src/manifest.rs:208-213
// BEFORE:
pub struct PluginPermission {
    pub resource: String,
    pub actions: Vec<String>,
}

// AFTER:
pub struct PluginPermission {
    pub resource: String,
    pub actions: Vec<String>,
    pub identity_scope: IdentityScope,  // 🆕 身份范围
}

pub enum IdentityScope {
    All,                    // 任何人
    Tenant(String),         // 特定tenant
    Identity(Vec<String>),  // 特定身份
    Role(String),           // 角色
}

// plugins/core/src/sandbox.rs:62-68
// BEFORE:
pub struct PluginSandbox {
    limits: ResourceLimits,
    sandbox_dir: PathBuf,
}

// AFTER:
pub struct PluginSandbox {
    limits: ResourceLimits,
    sandbox_dir: PathBuf,
    identity_binding: Option<IdentityBinding>, // 🆕 身份绑定
}

// 目录结构变更
// BEFORE: /sandbox/{plugin_id}/
// AFTER:  /sandbox/{tenant}/{identity}/{session}/
```

---

## 3. 协议层影响

### 3.1 gRPC协议变更

| 服务 | 消息 | 变更 | 破坏? |
|------|------|------|-------|
| `SandboxManager` | `AssignRequest` | +IdentityContext | ⚠️ 是 |
| `SandboxManager` | `AssignResponse` | +身份相关元数据 | 🟢 否 |
| | `TeardownRequest` | +identity字段 | 🟢 否 |

**兼容性策略**:
```protobuf
// 使用optional保持向后兼容
message AssignRequest {
    string tenant_id = 1;
    string namespace = 2;
    string action_name = 3;
    string payload_json = 4;
    IdentityContext identity = 5;  // 🆕 optional
}
```

---

## 4. 迁移复杂度汇总

### 4.1 按模块复杂度

| 层级 | 模块 | 复杂度 | 工期 | 风险 |
|------|------|--------|------|------|
| **Control Plane** | state-engine | 🟢 低 | 0周 | 无变更 |
| | fractal-gateway | 🟡 中 | 1周 | eBPF map结构 |
| | teardown-ctrl | 🟢 低 | 0.5周 | 增强现有 |
| **Orchestration** | manager | 🔴 高 | 2-3周 | DAG核心变更 |
| | scheduler | 🔴 高 | 2周 | 调度算法 |
| | evaluator | 🟡 中 | 1周 | 影响较小 |
| **Execution** | sandbox-daemon | 🟡 中 | 1.5周 | Pool重构 |
| | stateful-repl | 🟡 中 | 1周 | 新增身份 |
| **Memory Bus** | ingestion | 🟡 中 | 1周 | 缓存key变更 |
| | vector-kv | 🟢 低 | 0.5周 | 增强隔离 |
| **Plugin** | core | 🔴 高 | 2周 | 权限模型 |
| | registry | 🟡 中 | 1周 | 查询API |
| **总计** | | **🔴 高** | **6-8周** | |

### 4.2 代码变更统计

| 文件类型 | 预计变更文件数 | 新增代码行 | 修改代码行 |
|----------|----------------|------------|------------|
| Rust文件 (.rs) | 15-20 | 800-1200 | 400-600 |
| Go文件 (.go) | 8-12 | 600-900 | 300-500 |
| Protobuf (.proto) | 2-3 | 100-150 | 50-80 |
| 测试文件 | 10-15 | 400-600 | 200-300 |
| **总计** | **35-50** | **1900-2850** | **950-1480** |

---

## 5. 风险评估与缓解

### 5.1 高风险项

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| **DAG并发身份丢失** | 高 | 严重 | 所有通道传递身份副本 |
| **Worker分配竞态** | 中 | 严重 | auth检查+标记原子操作 |
| **VM伪造身份** | 中 | 高 | 签名验证身份token |
| **缓存key不兼容** | 高 | 中 | 双key策略+渐进迁移 |
| **性能下降** | 中 | 中 | 授权列表缓存 |

### 5.2 向后兼容策略

```go
// 策略1: Feature Flag
if identitySchedulingEnabled {
    result, err = scheduler.AssignTask(taskID, identity, host)
} else {
    result = scheduler.AssignTaskLegacy(taskID, host)
}

// 策略2: 可选字段
type TaskNode struct {
    // ...现有字段
    OwnerID string `json:"owner_id,omitempty"`  // optional
}

// 策略3: 双格式JSON
func parseTaskJSON(data []byte) (*TaskNode, error) {
    // 尝试新格式
    if err := json.Unmarshal(data, &newFormat); err == nil {
        return newFormat, nil
    }
    // 回退旧格式
    return parseLegacyFormat(data)
}
```

---

## 6. 推荐实施路线图

### Phase 0: 基础设施 (Week 1)
- [ ] 定义`IdentityContext`结构（共享库）
- [ ] 实现身份验证中间件
- [ ] 搭建feature flag系统
- [ ] 设计审计日志schema

### Phase 1: Control Plane增强 (Week 1-2)
- [ ] 更新`SecurityPolicy`含tenant_id
- [ ] 增强eBPF quota lookup (tenant+agent key)
- [ ] 测试fractal-gateway身份集成

### Phase 2: Orchestration核心 (Week 2-4)
- [ ] 新增`IdentityContext`到TaskNode/WorkerNode
- [ ] 实现身份授权验证逻辑
- [ ] 更新DAG执行流传递身份
- [ ] 修改调度算法支持身份过滤
- [ ] 双格式JSON解析（向后兼容）

### Phase 3: Execution Layer (Week 4-5)
- [ ] 修改`FirecrackerVM`含身份字段
- [ ] 重构WarmPool支持身份队列
- [ ] 更新gRPC协议（AssignRequest）
- [ ] 实现VM-身份绑定逻辑
- [ ] 集成fractal-gateway安全策略

### Phase 4: Memory Bus (Week 5-6)
- [ ] 更新`ProcessInput`含身份参数
- [ ] 实现双key缓存策略
- [ ] 增强Vector KV身份隔离

### Phase 5: Plugin系统 (Week 6-7)
- [ ] 扩展`PluginPermission`含identity_scope
- [ ] 实现运行时身份检查
- [ ] 修改Sandbox绑定身份
- [ ] 更新Capability查询API

### Phase 6: 验证与发布 (Week 7-8)
- [ ] 集成测试（全链路身份流）
- [ ] 性能基准测试
- [ ] 安全审计
- [ ] Feature flag渐进发布

---

## 7. 关键决策点

### 决策1: 身份标识格式

| 选项 | 优势 | 劣势 | 推荐 |
|------|------|------|------|
| UUID | 简单 | 无语义 | 🟡 |
| **Type+Key** (AutoGen) | 语义丰富 | 稍复杂 | ✅ |
| SPIFFE | 标准互操作 | 依赖重 | 🟡 |

**建议**: 采用`Type+Key`模式：
```rust
struct IdentityId {
    type_: IdentityType,  // System/Manager/Worker/Service
    key: String,          // 实例标识
}
```

### 决策2: 权限模型

| 模型 | 适用场景 | 复杂度 | 推荐 |
|------|----------|--------|------|
| **RBAC** | 角色分组 | 中 | ✅ |
| ABAC | 属性细控 | 高 | 🟡 |
| ACL | 直接授权 | 低 | 🟡 |

**建议**: RBAC为主，ABAC扩展点

### 决策3: 审计粒度

| 粒度 | 开销 | 推荐 |
|------|------|------|
| 无审计 | 低 | ❌ |
| **操作级** | 中 | ✅ |
| API级 | 高 | 🟡 |
| 系统调用级 | 极高 | ❌ |

---

## 8. 成功标准

### 8.1 功能验收

- [ ] Task提交必须携带身份上下文
- [ ] Worker分配必须验证身份授权
- [ ] VM创建必须绑定身份
- [ ] Plugin执行必须检查身份权限
- [ ] 所有操作必须记录身份审计日志
- [ ] 跨租户访问必须被阻止

### 8.2 性能指标

| 指标 | 目标 | 当前基线 |
|------|------|----------|
| 身份验证延迟 | <1ms | N/A |
| Worker分配延迟 | <5ms | <5ms |
| VM启动延迟 | <100ms (冷) | <100ms |
| 审计日志吞吐量 | >10k ops/s | N/A |

### 8.3 兼容性

- [ ] 旧API请求仍可工作（向后兼容）
- [ ] 新旧Worker可共存
- [ ] 零停机迁移

---

## 9. 附录

### 9.1 关键文件清单

**必须修改**:
1. `control-plane/fractal-gateway/src/security.rs`
2. `orchestration/manager/main.go`
3. `orchestration/scheduler/main.go`
4. `execution-layer/sandbox-daemon/src/microvm.rs`
5. `execution-layer/sandbox-daemon/src/pool.rs`
6. `sma-proto/sandbox.proto`
7. `memory-bus/ingestion/main.go`
8. `plugins/core/src/manifest.rs`
9. `plugins/core/src/sandbox.rs`
10. `plugins/core/src/registry.rs`

**建议新增**:
1. `control-plane/identity/src/lib.rs` (新模块)
2. `orchestration/types/identity.go` (共享类型)
3. `docs/identity-scheduling-guide.md` (开发文档)

### 9.2 相关Issue

- Issue #1: Implement identity context in TaskNode
- Issue #2: Add identity-aware worker scheduling
- Issue #3: Bind VM to identity in sandbox-daemon
- Issue #4: Support identity-scoped plugin permissions
- Issue #5: Cache key migration for identity isolation

---

**报告编制**: SMA-OS Architecture Team  
**评审状态**: 待技术评审  
**下次更新**: 根据实施进度
