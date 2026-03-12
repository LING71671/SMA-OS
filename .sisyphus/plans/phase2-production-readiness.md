# Phase 2: Production Readiness 开发计划

> **计划生成时间**: 2026-03-10  
> **当前阶段**: Phase 2: Production Readiness (进行中)  
> **目标**: 达到生产就绪状态，支持 Firecracker MicroVM 和 eBPF 部署

---

## TL;DR

**核心目标**: 完成从"可用原型"到"生产就绪"的关键跨越

**主要交付物**:
- Firecracker MicroVM 完整集成（执行层沙箱）
- eBPF 探针部署和验证
- 混沌工程测试套件
- 性能基准测试（P99 < 10ms）
- 文档完整度 >90%

**预计工作量**: 中等规模（约 15-20 个任务）

**关键路径**: 
执行层沙箱 → Firecracker 集成 → eBPF 部署 → 混沌测试 → 性能优化

---

## 上下文

### 当前状态

根据对代码库的分析：

**已完成的 Phase 1 核心功能**:
- ✅ State Engine: 事件溯源核心，支持 Redis 热缓存 + PostgreSQL 冷存储
- ✅ Fractal Gateway: eBPF 安全网关框架
- ✅ Orchestration: DAG 调度和任务编排
- ✅ Memory Bus: LLM 回退的意图提取
- ✅ Observability UI: 实时 DAG 可视化

**待完成的 Phase 2 功能**:
- 🔲 Firecracker MicroVM 集成（执行层沙箱的核心）
- 🔲 eBPF 探针的实际部署和验证
- 🔲 混沌工程测试套件
- 🔲 性能基准测试和优化
- 🔲 文档完整性提升

### 技术栈决策

| 层次 | 技术选择 | 状态 |
|------|---------|------|
| 状态内核 | Rust + Redis + PostgreSQL | ✅ 完成 |
| 执行沙箱 | Firecracker + eBPF | 🔲 进行中 |
| 编排层 | Go + DAG 调度 | ✅ 完成 |
| 记忆总线 | Go + Weaviate/ClickHouse | ✅ 完成 |
| 可观测性 | Next.js + ReactFlow | ✅ 完成 |

---

## 工作目标

### 核心目标

实现 SMA-OS 从原型到生产的关键能力：

1. **物理隔离执行环境**: 基于 Firecracker MicroVM 的安全沙箱
2. **内核级安全监控**: eBPF 探针部署和实时反馈
3. **弹性验证**: 混沌工程测试套件
4. **性能保证**: P99 延迟 < 10ms
5. **文档完备**: 开发、部署、运维文档覆盖率 >90%

### 具体交付物

- [ ] `execution-layer/sandbox-daemon/` - Firecracker 生命周期管理
- [ ] `execution-layer/stateful-repl/` - 持久化终端
- [ ] `control-plane/fractal-gateway-ebpf/` - 可部署的 eBPF 探针
- [ ] `./chaos-tests/` - 混沌工程测试套件
- [ ] `./benchmarks/` - 性能基准测试
- [ ] 完善的文档覆盖率

### 必须包含 (IN Scope)

- Firecracker MicroVM 的完整集成（启动、停止、快照、恢复）
- eBPF 探针的编译、加载、数据采集
- 混沌测试（节点故障、网络分区、资源耗尽）
- 性能基准测试（延迟、吞吐量、资源利用率）
- 生产部署文档和运维手册

### 禁止包含 (OUT Scope)

- Phase 3 的水平扩展功能（1000+ 并发 Agent）
- Phase 4 的插件系统
- 多区域部署配置
- 企业级功能（SSO、审计日志等）

### 成功标准

1. **Firecracker 集成**: 能够动态创建/销毁 MicroVM 沙箱
2. **eBPF 探针**: 成功部署并采集系统调用数据
3. **混沌测试**: 通过所有预定义的故障注入测试
4. **性能指标**: P99 延迟 < 10ms（在基准测试中验证）
5. **文档覆盖**: 所有模块都有完整的 README 和使用示例

---

## 执行策略

### 并行执行波次

```
Wave 1 (基础层 - 立即开始，5 个任务并行):
├── Task 1: Firecracker 配置和依赖管理
├── Task 2: eBPF 探针编译和加载器
├── Task 3: 混沌测试框架搭建
├── Task 4: 性能测试基础设施
└── Task 5: 文档模板和结构

Wave 2 (核心实现 - 依赖 Wave 1，4 个任务并行):
├── Task 6: Firecracker MicroVM 生命周期管理
├── Task 7: eBPF 数据采集和上报
├── Task 8: 混沌测试场景实现
└── Task 9: 基准测试用例编写

Wave 3 (集成验证 - 依赖 Wave 2，3 个任务并行):
├── Task 10: 沙箱与编排层集成
├── Task 11: eBPF 与可观测性集成
└── Task 12: 混沌测试自动化

Wave 4 (优化完善 - 依赖 Wave 3，3 个任务并行):
├── Task 13: 性能优化和调优
├── Task 14: 文档完善和示例
└── Task 15: 生产部署脚本
```

### 依赖关系矩阵

| 任务 | 依赖 | 阻塞 |
|------|------|------|
| 1-5 | 无 | 6-15 |
| 6 | 1 | 10, 13 |
| 7 | 2 | 11, 13 |
| 8 | 3 | 12 |
| 9 | 4 | 13 |
| 10 | 6 | 15 |
| 11 | 7 | 15 |
| 12 | 8 | 15 |
| 13 | 9, 10, 11 | 14, 15 |
| 14 | 13 | - |
| 15 | 10, 11, 12, 13, 14 | - |

---

## TODOs

> 每个任务包含：具体实现步骤 + 测试要求 + QA 验证场景

- [ ] **1. Firecracker 配置和依赖管理**

  **What to do**:
  - 在 `execution-layer/Cargo.toml` 中添加 Firecracker 相关依赖
  - 创建 Firecracker 配置文件模板（VM 规格、镜像、网络）
  - 准备 minimal Linux 镜像（如 Alpine）用于 MicroVM
  - 配置 Firecracker API socket 路径和认证机制

  **Must NOT do**:
  - 不要实现 VM 生命周期管理（Task 6）
  - 不要修改 eBPF 相关代码

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `[]`
  - **Reason**: 配置和依赖管理是直接的文档/配置工作

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2-5)
  - **Blocks**: Task 6
  - **Blocked By**: None

  **References**:
  - Firecracker 官方文档：`https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md`
  - 现有配置：`execution-layer/Cargo.toml`
  - 参考模式：`control-plane/state-engine/Cargo.toml` 的依赖组织方式

  **Acceptance Criteria**:
  - [ ] `execution-layer/Cargo.toml` 包含 Firecracker 相关依赖
  - [ ] `execution-layer/configs/firecracker-config.json` 存在且格式正确
  - [ ] 准备最小 Linux 镜像（< 50MB）
  - [ ] 配置文档说明每个配置项的用途

  **QA Scenarios**:

  ```
  Scenario: 验证 Firecracker 配置可执行
  Tool: Bash
  Preconditions: Firecracker 二进制文件已安装
  Steps:
    1. firecracker --version
    2. 使用配置的 JSON 启动 Firecracker（dry-run 模式）
  Expected Result: Firecracker 版本输出，配置验证通过
  Evidence: .sisyphus/evidence/task-1-firecracker-version.txt
  ```

- [ ] **2. eBPF 探针编译和加载器**

  **What to do**:
  - 在 `control-plane/fractal-gateway-ebpf/` 中实现 eBPF 探针程序
  - 使用 `bpf` 或 `aya` crate 编写 eBPF 代码
  - 实现探针加载器（加载、卸载、状态查询）
  - 配置 eBPF maps 用于数据导出

  **Must NOT do**:
  - 不要实现数据采集逻辑（Task 7）
  - 不要修改 state-engine 代码

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `[]`
  - **Reason**: eBPF 开发需要专业知识和调试

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3-5)
  - **Blocks**: Task 7
  - **Blocked By**: None

  **References**:
  - Aya 项目文档：`https://aya-rs.dev/`
  - 现有代码：`control-plane/fractal-gateway/`
  - eBPF 模式参考：Linux bpf samples

  **Acceptance Criteria**:
  - [ ] eBPF 探针程序编译成功（`cargo build-bpf`）
  - [ ] 加载器可以加载/卸载探针
  - [ ] eBPF maps 正确初始化
  - [ ] 无编译错误或警告

  **QA Scenarios**:
  ```
  Scenario: eBPF 探针加载成功
  Tool: Bash
  Preconditions: 内核支持 eBPF，权限充足
  Steps:
    1. cargo build-bpf
    2. ./target/release/fractal-gateway-ebpf load
    3. bpftool prog list | grep fractal
  Expected Result: 探针出现在 eBPF 程序列表中
  Evidence: .sisyphus/evidence/task-2-ebpf-load.txt
  ```

- [ ] **3. 混沌测试框架搭建**

  **What to do**:
  - 创建 `./chaos-tests/` 目录结构
  - 实现混沌测试基础框架（故障注入、恢复验证）
  - 配置测试环境（Docker Compose 扩展）
  - 定义测试场景模板

  **Must NOT do**:
  - 不要实现具体故障场景（Task 8）
  - 不要修改生产代码

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `[]`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 8, 12
  - **Blocked By**: None

  **Acceptance Criteria**:
  - [ ] 混沌测试框架可运行
  - [ ] 至少一个示例场景
  - [ ] 测试报告生成功能

- [ ] **4. 性能测试基础设施**

  **What to do**:
  - 创建 `./benchmarks/` 目录
  - 配置性能测试工具（如 `criterion` for Rust, `go test -bench`）
  - 定义性能指标和基线
  - 设置持续性能监控

  **Acceptance Criteria**:
  - [ ] 基准测试可执行
  - [ ] 生成 P99 延迟报告
  - [ ] 性能趋势图表

- [ ] **5. 文档模板和结构**

  **What to do**:
  - 创建文档模板（API 参考、部署指南、故障排查）
  - 定义文档结构（参考 AGENTS.md 的层次）
  - 配置文档生成工具（如 `mdbook`）

  **Acceptance Criteria**:
  - [ ] 文档模板完整
  - [ ] 所有模块都有 README
  - [ ] 文档覆盖率检查脚本

---

## 最终验证波

> 所有实现任务完成后，并行执行以下验证：

- [ ] **F1. Firecracker 集成验证**
  - 启动 MicroVM 并运行简单命令
  - 验证资源隔离（CPU、内存、网络）
  - 快照和恢复功能

- [ ] **F2. eBPF 探针验证**
  - 探针成功加载
  - 数据采集正常
  - 与可观测性 UI 集成

- [ ] **F3. 混沌测试验证**
  - 所有混沌场景通过
  - 系统可恢复性验证
  - 故障报告完整

- [ ] **F4. 性能基准验证**
  - P99 延迟 < 10ms
  - 吞吐量达标
  - 资源使用合理

- [ ] **F5. 文档完整性验证**
  - 所有模块文档完整
  - 示例可运行
  - 部署指南清晰

---

## 提交策略

| 提交点 | 类型 | 描述 |
|--------|------|------|
| `feat(firecracker): config and deps` | type: feat | Task 1 完成后 |
| `feat(ebpf): probe and loader` | type: feat | Task 2 完成后 |
| `test(chaos): framework setup` | type: test | Task 3 完成后 |
| `test(bench): infrastructure` | type: test | Task 4 完成后 |
| `docs: templates and structure` | type: docs | Task 5 完成后 |
| `feat(sandbox): lifecycle management` | type: feat | Task 6 完成后 |
| `feat(ebpf): data collection` | type: feat | Task 7 完成后 |
| `test(chaos): scenarios` | type: test | Task 8 完成后 |
| `test(bench): use cases` | type: test | Task 9 完成后 |
| `feat(integration): sandbox-orchestration` | type: feat | Task 10 完成后 |
| `feat(integration): ebpf-observability` | type: feat | Task 11 完成后 |
| `test(chaos): automation` | type: test | Task 12 完成后 |
| `perf: optimization and tuning` | type: perf | Task 13 完成后 |
| `docs: completion` | type: docs | Task 14 完成后 |
| `ci: production deployment` | type: ci | Task 15 完成后 |

---

## 成功标准

### 验证命令

```bash
# 1. Firecracker 集成验证
cd execution-layer && cargo run --bin sandbox-daemon
# Expected: MicroVM 成功启动并可交互

# 2. eBPF 探针验证
cd control-plane/fractal-gateway-ebpf && cargo build-bpf
./target/release/fractal-gateway-ebpf load
bpftool prog list | grep fractal
# Expected: 探针出现在列表中

# 3. 混沌测试
./chaos-tests/run-all.sh
# Expected: 所有场景通过

# 4. 性能基准
cd benchmarks && cargo bench
# Expected: P99 < 10ms

# 5. 文档覆盖
./scripts/check-docs.sh
# Expected: 覆盖率 > 90%
```

### 最终检查清单

- [ ] Firecracker MicroVM 集成完成
- [ ] eBPF 探针部署成功
- [ ] 混沌测试套件完整
- [ ] 性能基准 P99 < 10ms
- [ ] 文档覆盖率 > 90%
- [ ] 所有测试通过
- [ ] 生产部署脚本就绪

---

## 风险与注意事项

### 已知风险

1. **Firecracker 依赖复杂**: 需要特定内核模块和配置
2. **eBPF 兼容性**: 不同 Linux 内核版本可能有差异
3. **性能调优时间**: 可能需要多次迭代才能达到 P99 < 10ms

### 缓解策略

- 使用 Docker 容器提供一致的开发环境
- 提供 eBPF 的备用方案（如 seccomp）
- 早期进行性能基准测试，避免后期大幅修改

---

## 下一步

1. 确认本计划范围和目标
2. 运行 `/start-work phase2-production-readiness` 开始执行
3. 按照 Wave 顺序逐步完成任务
