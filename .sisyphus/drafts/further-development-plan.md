# SMA-OS 进一步开发计划

**创建日期**: 2026-03-15
**版本**: 1.0
**状态**: 草案

---

## 当前项目状态摘要

### 完成度概览
- **总体进度**: 82% (18/22 模块已实现)
- **Phase 1-3**: 100% 完成（核心基础设施、生产就绪、规模与可靠性）
- **Phase 4**: 85% 完成（生态系统）

### 代码提交情况
- **总提交数**: 55
- **未推送提交**: 1 个（Identity 调度相关）
- **主要贡献者**: LING71671 (45), Una (7), dependabot (2), Sisyphus-Junior (1)

### 技术债务识别
- **TODO/FIXME 数量**: 15 处（Rust 代码）
- **关键未完成模块**: 
  - `formal-verifier`: 仅 TLA+ 规范，无实现
  - `plugins/executors`: 空目录
  - `execution-layer/images`: 空目录

---

## 开发优先级排序

### 🔴 P0 - 紧急（生产安全）

#### 1. 身份调度系统实现
**优先级理由**: 已有完整影响评估报告，是当前最大的架构缺口

**范围**: 根据 `docs/identity-scheduling-impact-assessment.md`
- Orchestration 层身份支持（Task/Worker）
- Execution 层 VM-身份绑定
- Plugin 系统运行时身份检查
- Memory Bus 调用者追踪

**预计工期**: 6-8 周
**风险**: 高（需修改 35-50 个文件）

#### 2. Chaos Tests 实现补全
**问题**: 两个关键场景仅有 TODO 占位符

**具体任务**:
```rust
// chaos-tests/src/scenarios/resource_exhaustion.rs:18
// TODO: Implement resource exhaustion logic

// chaos-tests/src/scenarios/network_partition.rs:18
// TODO: Implement actual network partition logic
```

**预计工期**: 1 周

### 🟡 P1 - 重要（功能完善）

#### 3. Formal Verifier 实现层
**当前状态**: 仅 TLA+ 规范文件，无 Rust 实现

**需求**:
- TLA+ 模型解析器
- 状态空间探索引擎
- 不变量检查器
- 与 State Engine 集成

**预计工期**: 3-4 周

#### 4. Plugin Executors 模块
**当前状态**: 空目录

**需求**:
- 自定义执行器接口定义
- WASM 执行器实现
- 容器执行器实现
- 执行器生命周期管理

**预计工期**: 2-3 周

#### 5. Identity Audit 持久化
**问题**: 审计日志存储未实现

```rust
// control-plane/identity/src/audit.rs:181-182
// TODO: Write to PostgreSQL
// TODO: Batch write to ClickHouse for analytics
```

**预计工期**: 1 周

### 🟢 P2 - 改进（质量提升）

#### 6. 代码质量清理
- 消除所有 TODO/FIXME（15 处）
- 添加缺失的单元测试
- 改进错误处理模式

#### 7. 文档完善
- API 文档自动化生成
- 架构决策记录 (ADR)
- 运维手册

#### 8. 性能基准测试扩展
- 覆盖更多关键路径
- 建立性能回归检测

---

## 详细实施计划

### Sprint 1: 紧急修复 + 身份调度 Phase 0-1 (Week 1-2)

**目标**: 修复紧急问题，启动身份调度基础设施

#### Wave 1: 紧急修复 (并行)
1. **Chaos Tests 补全** (`quick`)
   - 实现 resource_exhaustion 场景
   - 实现 network_partition 场景
   - 添加集成测试

2. **Identity Audit 持久化** (`quick`)
   - PostgreSQL 写入实现
   - ClickHouse 批量写入
   - 测试覆盖

#### Wave 2: 身份调度基础 (串行)
3. **IdentityContext 共享定义** (`quick`)
   - Rust: `control-plane/identity/src/types.rs` 扩展
   - Go: `orchestration/types/identity.go` 新建
   - Protobuf: `sma-proto/identity.proto` 新建

4. **Feature Flag 系统** (`unspecified-low`)
   - 身份调度开关
   - 配置管理
   - 运行时切换

---

### Sprint 2-3: 身份调度 Phase 2 (Week 3-4)

**目标**: Orchestration 层身份集成

#### Wave 1: Manager 层 (高复杂度)
5. **TaskNode 身份字段** (`deep`)
   - 结构体扩展
   - 双格式 JSON 解析
   - 向后兼容测试

6. **DAG 执行流身份传递** (`deep`)
   - 通道消息增强
   - 身份上下文传播
   - 错误处理

#### Wave 2: Scheduler 层 (高复杂度)
7. **WorkerNode 授权字段** (`deep`)
   - 结构体扩展
   - 授权白名单
   - 安全等级

8. **身份感知调度算法** (`ultrabrain`)
   - 身份过滤逻辑
   - 亲和性计算增强
   - 竞态条件处理

---

### Sprint 4: 身份调度 Phase 3 (Week 5)

**目标**: Execution 层身份绑定

#### Wave 1: Sandbox Daemon
9. **VM-身份绑定** (`deep`)
   - FirecrackerVM 结构增强
   - 身份令牌验证
   - 绑定生命周期

10. **WarmPool 身份队列** (`deep`)
    - 身份感知池化
    - 快速分配优化
    - 清理策略

---

### Sprint 5: 身份调度 Phase 4-5 (Week 6)

**目标**: Memory Bus + Plugin 系统

#### Wave 1: Memory Bus
11. **Ingestion 身份参数** (`quick`)
    - ProcessInput 签名变更
    - 缓存 key 策略
    - 双 key 过渡

#### Wave 2: Plugin System
12. **Plugin 权限增强** (`deep`)
    - IdentityScope 枚举
    - 运行时检查
    - 沙箱绑定

---

### Sprint 6: Formal Verifier + Executors (Week 7-8)

**目标**: 补全缺失模块

#### Wave 1: Formal Verifier
13. **TLA+ 解析器** (`artistry`)
    - 规范文件解析
    - 模型构建

14. **状态探索引擎** (`ultrabrain`)
    - 状态空间遍历
    - 不变量检查

#### Wave 2: Plugin Executors
15. **执行器接口** (`deep`)
    - trait 定义
    - 生命周期 hooks

16. **WASM 执行器** (`deep`)
    - wasmtime 集成
    - 资源限制

---

## 技术决策待确认

### 决策 1: 身份令牌格式
| 选项 | 优势 | 劣势 |
|------|------|------|
| JWT | 标准化、工具支持好 | 载荷大小固定 |
| Macaroons | 可衰减、灵活 | 实现复杂 |
| 自定义签名 | 性能最优 | 无标准工具 |

**建议**: JWT（开发效率）+ 缓存（性能）

### 决策 2: Formal Verifier 实现方式
| 选项 | 优势 | 劣势 |
|------|------|------|
| TLC 集成 | 成熟、覆盖广 | 外部依赖重 |
| 自研 Rust | 性能好、可控 | 工作量大 |
| 混合模式 | 平衡 | 架构复杂 |

**建议**: 混合模式（TLC 模型检查 + Rust 快速路径）

### 决策 3: 执行器隔离级别
| 选项 | 安全性 | 性能 |
|------|--------|------|
| 进程级 | 高 | 中 |
| 容器级 | 最高 | 低 |
| WASM 沙箱 | 中 | 高 |

**建议**: 分级策略（敏感操作容器，轻量操作 WASM）

---

## 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 身份调度破坏现有功能 | 高 | 严重 | Feature Flag + 渐进发布 |
| Formal Verifier 工期超预期 | 中 | 中 | MVP 优先，迭代增强 |
| 性能回退 | 中 | 中 | 基准测试监控 |
| 安全漏洞引入 | 低 | 严重 | 安全审计 + 渗透测试 |

---

## 成功标准

### Sprint 1-2 完成标准
- [ ] Chaos tests 全部通过
- [ ] Identity audit 持久化可用
- [ ] Feature Flag 系统运行

### Sprint 3-4 完成标准
- [ ] Task 提交携带身份
- [ ] Worker 分配验证身份
- [ ] VM 绑定身份

### Sprint 5-6 完成标准
- [ ] Plugin 权限检查生效
- [ ] Formal Vericator 基础可用
- [ ] Executors 模块非空

---

## 资源需求

### 开发人力
- Rust 开发: 2 人（Control Plane, Execution Layer）
- Go 开发: 1 人（Orchestration, Memory Bus）
- TypeScript: 1 人（UI 集成）
- 测试/QA: 1 人

### 基础设施
- CI/CD 流水线增强
- 性能测试环境
- 安全扫描工具

---

## 下一步行动

1. **立即**: 推送未提交代码，清理 git status
2. **本周**: 启动 Sprint 1 Wave 1（Chaos Tests + Audit）
3. **下周**: 确认技术决策，开始身份调度 Phase 0

---

**计划状态**: 待评审
**下一步**: 运行 `/start-work` 开始执行
