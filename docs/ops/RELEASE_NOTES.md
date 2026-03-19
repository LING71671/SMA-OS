# SMA-OS Release Notes / SMA-OS 更新日志

[**English**](#english) | [**中文**](#中文)

---

<a name="中文"></a>
## 中文

## v1.2.0 - 任务管理与依赖分析

### 🚀 亮点

本版本为编排层添加了全面的任务管理和依赖分析能力。

### ✨ 新功能

#### 任务管理

| 功能 | 描述 |
|------|------|
| **任务拆解** | 支持父/子关系的层级任务结构 |
| **进度追踪** | 实时进度计算 (0-100%) 及子任务聚合 |
| **暂停/恢复** | 基于检查点的任务暂停，保留状态 |
| **扩展任务状态** | 新增 `PAUSED` 和 `RESUMED` 状态 |

#### 依赖分析

| 功能 | 描述 |
|------|------|
| **依赖矩阵** | 完整的任务依赖映射 |
| **循环检测** | 基于 DFS 的循环依赖检测 |
| **关键路径** | 最长依赖链分析 |
| **并行度分析** | 最大并发任务计算 |
| **影响分析** | 失败传播范围映射 |

#### UI 组件

| 组件 | 用途 |
|------|------|
| `ProgressBar.tsx` | 带百分比显示的可视化进度指示器 |
| `TaskControls.tsx` | 基于状态显示的暂停/恢复按钮 |
| `TaskTree.tsx` | 层级任务结构可视化 |
| `DependencyGraph.tsx` | 使用 ReactFlow 的交互式依赖图 |
| `CriticalPathHighlight.tsx` | 关键路径覆盖可视化 |

### 📊 性能基准

| 基准测试 | 结果 | 目标 | 状态 |
|----------|------|------|------|
| ProgressCalc_Atomic | 2.08 ns/op | < 1ms | ✅ 优秀 |
| ProgressCalc_SubTasks (100) | 1,447 ns/op | < 1ms | ✅ 通过 |
| CheckpointSave | 796 ns/op | < 50ms | ✅ 优秀 |
| CheckpointSerialize | 1,233 ns/op | < 50ms | ✅ 通过 |
| CheckpointDeserialize | 3,138 ns/op | < 50ms | ✅ 通过 |
| DependencyDepth (50 nodes) | 9,259 ns/op | < 100ms | ✅ 通过 |
| ProgressReport_Async | 476 ns/op | < 1ms | ✅ 通过 |

### 🧪 测试覆盖率

| 模块 | 覆盖率 | 测试数 |
|------|--------|--------|
| orchestration/manager | **87.2%** | 47 个单元测试 |
| E2E 测试 | 100% 通过 | 8 个测试 |
| 基准测试 | 100% 通过 | 7 个基准 |

### 📁 新增文件

```
orchestration/manager/
├── api.go                    # HTTP 处理器
├── checkpoint.go             # 检查点保存/恢复
├── progress.go               # 进度计算
├── pause_resume.go           # 暂停/恢复逻辑
├── dependency_analysis.go    # 依赖分析算法
├── api_test.go               # API 测试
├── checkpoint_test.go        # 检查点测试
├── progress_test.go          # 进度测试
├── pause_resume_test.go      # 暂停/恢复测试
└── dependency_analysis_test.go # 分析测试

orchestration/types/
└── task.go                   # 共享类型定义

observability-ui/web-dashboard/src/app/components/
├── ProgressBar.tsx           # 进度指示器
├── TaskControls.tsx          # 暂停/恢复按钮
├── TaskTree.tsx              # 任务层级
├── DependencyGraph.tsx       # 依赖可视化
└── CriticalPathHighlight.tsx # 关键路径覆盖

tests/
├── e2e/pause_resume_test.go  # E2E 测试 (8 个测试)
└── benchmark/progress_bench_test.go # 基准测试 (7 个)
```

### 🔌 新增 API 端点

```
GET /api/v1/tasks/{id}/progress   # 获取任务进度
POST /api/v1/tasks/{id}/pause     # 暂停运行中的任务
POST /api/v1/tasks/{id}/resume    # 恢复已暂停的任务
GET /api/v1/dags/analysis         # 完整依赖分析
GET /api/v1/dags/critical-path    # 仅关键路径
GET /api/v1/dags/parallelism      # 并行度信息
GET /api/v1/tasks/{id}/impact     # 失败影响范围
```

---

## v1.1.0 - eBPF 基础设施

### 🚀 亮点

本版本包含 eBPF 基础设施、测试覆盖率和构建系统的重大改进。

### 📦 组件

#### Control Plane (Rust)
- **state-engine**: 支持 Redis/PostgreSQL 的事件溯源状态内核
- **teardown-ctrl**: 级联清理控制器
- **identity**: 身份和访问管理模块
- **fractal-gateway**: eBPF 安全网关（用户空间）
- **fractal-gateway-ebpf**: XDP 包过滤（eBPF）

#### Orchestration (Go)
- **manager**: DAG 拓扑执行引擎
- **scheduler**: 带热池的 Worker 调度器
- **evaluator**: 输出验证和回滚

#### Memory Bus (Go)
- **ingestion**: AI 大模型驱动的 SLM 意图提取
- **vector-kv**: 带压缩的向量 + KV 存储

#### Observability UI (TypeScript)
- **web-dashboard**: 使用 Next.js 的实时 DAG 可视化

### 🔧 变更内容

#### eBPF 基础设施
- 使用 xtask 完全重写 eBPF 构建系统
- 分离 eBPF 代码和用户空间代码
- 添加正确的 `#![no_std]` eBPF 程序
- 修复 bpf-linker 集成

#### 测试
- 修复 Go ingestion 测试 API 兼容性
- 重写 Rust identity 集成测试
- 所有测试现在通过（23 个 Rust + 14 个 Go 测试）

#### 构建系统
- 添加 eBPF 编译的 Dockerfile
- 添加服务运行时的 Dockerfile
- 创建自动化构建脚本

### 📥 下载

| 平台 | 组件 | 二进制 |
|------|------|--------|
| Linux x64 | state-engine | `state-engine` |
| Linux x64 | teardown-ctrl | `teardown-ctrl` |
| eBPF | fractal-gateway | `fractal-gateway-ebpf.o` |
| 跨平台 | ingestion | `ingestion` |
| 跨平台 | manager | `manager` |

### 🛡️ 安全

- 用于网络过滤的 eBPF 沙箱
- 基于身份的访问控制
- 所有操作的审计日志

### 📋 系统要求

- **eBPF**: 支持 BTF 的 Linux 内核 4.19+
- **Rust 服务**: Docker 或 Linux 环境
- **Go 服务**: Windows/Linux/macOS
- **基础设施**: Docker Compose

### 🔗 快速开始

```bash
# 启动基础设施
docker-compose up -d

# 构建（在 Docker 中用于 eBPF）
./scripts/build-ebpf.sh

# 运行服务
./scripts/start-services.sh
```

---

**完整变更日志**: https://github.com/LING71671/SMA-OS/compare/v1.0.0...v1.1.0

---
---

<a name="english"></a>
## English

## v1.2.0 - Task Management & Dependency Analysis

### 🚀 Highlights

This release adds comprehensive task management and dependency analysis capabilities to the orchestration layer.

### ✨ New Features

#### Task Management

| Feature | Description |
|---------|-------------|
| **Task Decomposition** | Hierarchical task structure with parent/child relationships |
| **Progress Tracking** | Real-time progress calculation (0-100%) with subtask aggregation |
| **Pause/Resume** | Checkpoint-based task suspension with state preservation |
| **Extended TaskStatus** | New `PAUSED` and `RESUMED` states |

#### Dependency Analysis

| Feature | Description |
|---------|-------------|
| **Dependency Matrix** | Complete task dependency mapping |
| **Cycle Detection** | DFS-based circular dependency detection |
| **Critical Path** | Longest dependency chain analysis |
| **Parallelism Analysis** | Maximum concurrent task calculation |
| **Impact Analysis** | Failure propagation scope mapping |

#### UI Components

| Component | Purpose |
|-----------|---------|
| `ProgressBar.tsx` | Visual progress indicator with percentage display |
| `TaskControls.tsx` | Pause/resume buttons with status-based visibility |
| `TaskTree.tsx` | Hierarchical task structure visualization |
| `DependencyGraph.tsx` | Interactive dependency graph with ReactFlow |
| `CriticalPathHighlight.tsx` | Critical path overlay visualization |

### 📊 Performance Benchmarks

| Benchmark | Result | Target | Status |
|-----------|--------|--------|--------|
| ProgressCalc_Atomic | 2.08 ns/op | < 1ms | ✅ Excellent |
| ProgressCalc_SubTasks (100) | 1,447 ns/op | < 1ms | ✅ Pass |
| CheckpointSave | 796 ns/op | < 50ms | ✅ Excellent |
| CheckpointSerialize | 1,233 ns/op | < 50ms | ✅ Pass |
| CheckpointDeserialize | 3,138 ns/op | < 50ms | ✅ Pass |
| DependencyDepth (50 nodes) | 9,259 ns/op | < 100ms | ✅ Pass |
| ProgressReport_Async | 476 ns/op | < 1ms | ✅ Pass |

### 🧪 Test Coverage

| Module | Coverage | Tests |
|--------|----------|-------|
| orchestration/manager | **87.2%** | 47 unit tests |
| E2E Tests | 100% pass | 8 tests |
| Benchmarks | 100% pass | 7 benchmarks |

### 📁 New Files

```
orchestration/manager/
├── api.go                    # HTTP handlers
├── checkpoint.go             # Checkpoint save/restore
├── progress.go               # Progress calculation
├── pause_resume.go           # Pause/resume logic
├── dependency_analysis.go    # Dependency analysis algorithms
├── api_test.go               # API tests
├── checkpoint_test.go        # Checkpoint tests
├── progress_test.go          # Progress tests
├── pause_resume_test.go      # Pause/resume tests
└── dependency_analysis_test.go # Analysis tests

orchestration/types/
└── task.go                   # Shared type definitions

observability-ui/web-dashboard/src/app/components/
├── ProgressBar.tsx           # Progress indicator
├── TaskControls.tsx          # Pause/resume buttons
├── TaskTree.tsx              # Task hierarchy
├── DependencyGraph.tsx       # Dependency visualization
└── CriticalPathHighlight.tsx # Critical path overlay

tests/
├── e2e/pause_resume_test.go  # E2E tests (8 tests)
└── benchmark/progress_bench_test.go # Benchmarks (7)
```

### 🔌 New API Endpoints

```
GET /api/v1/tasks/{id}/progress   # Get task progress
POST /api/v1/tasks/{id}/pause     # Pause running task
POST /api/v1/tasks/{id}/resume    # Resume paused task
GET /api/v1/dags/analysis         # Full dependency analysis
GET /api/v1/dags/critical-path    # Critical path only
GET /api/v1/dags/parallelism      # Parallelism info
GET /api/v1/tasks/{id}/impact     # Failure impact scope
```

---

## v1.1.0 - eBPF Infrastructure

### 🚀 Highlights

This release includes major improvements to the eBPF infrastructure, test coverage, and build system.

### 📦 Components

#### Control Plane (Rust)
- **state-engine**: Event sourcing state kernel with Redis/PostgreSQL
- **teardown-ctrl**: Cascading cleanup controller
- **identity**: Identity and access management module
- **fractal-gateway**: eBPF security gateway (userspace)
- **fractal-gateway-ebpf**: XDP packet filtering (eBPF)

#### Orchestration (Go)
- **manager**: DAG topological execution engine
- **scheduler**: Worker dispatch with warm pool
- **evaluator**: Output validation and rollback

#### Memory Bus (Go)
- **ingestion**: SLM-powered intent extraction with AI LLM
- **vector-kv**: Vector + KV storage with compression

#### Observability UI (TypeScript)
- **web-dashboard**: Real-time DAG visualization with Next.js

### 🔧 What's Changed

#### eBPF Infrastructure
- Completely rewrote eBPF build system with xtask
- Separated eBPF code from userspace code
- Added proper `#![no_std]` eBPF program
- Fixed bpf-linker integration

#### Testing
- Fixed Go ingestion test API compatibility
- Rewrote Rust identity integration tests
- All tests now passing (23 Rust + 14 Go tests)

#### Build System
- Added Dockerfile for eBPF compilation
- Added Dockerfile for services runtime
- Created build scripts for automation

### 📥 Downloads

| Platform | Component | Binary |
|----------|-----------|--------|
| Linux x64 | state-engine | `state-engine` |
| Linux x64 | teardown-ctrl | `teardown-ctrl` |
| eBPF | fractal-gateway | `fractal-gateway-ebpf.o` |
| Cross-platform | ingestion | `ingestion` |
| Cross-platform | manager | `manager` |

### 🛡️ Security

- eBPF sandbox for network filtering
- Identity-based access control
- Audit logging for all operations

### 📋 Requirements

- **eBPF**: Linux kernel 4.19+ with BTF support
- **Rust services**: Docker or Linux environment
- **Go services**: Windows/Linux/macOS
- **Infrastructure**: Docker Compose

### 🔗 Quick Start

```bash
# Start infrastructure
docker-compose up -d

# Build (in Docker for eBPF)
./scripts/build-ebpf.sh

# Run services
./scripts/start-services.sh
```

---

**Full Changelog**: https://github.com/LING71671/SMA-OS/compare/v1.0.0...v1.1.0
