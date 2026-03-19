# SMA-OS v1.2.0

[**English**](#english) | [**中文**](#中文)

---

<a name="中文"></a>
## 中文

> 一个基于事件溯源和 eBPF 的 AI 智能体调度系统。

[![CI](https://github.com/LING71671/SMA-OS/actions/workflows/ci.yml/badge.svg)](https://github.com/LING71671/SMA-OS/actions/workflows/ci.yml)
[![Security Audit](https://github.com/LING71671/SMA-OS/actions/workflows/security.yml/badge.svg)](https://github.com/LING71671/SMA-OS/actions/workflows/security.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/LING71671/SMA-OS)](https://github.com/LING71671/SMA-OS/releases)
[![Coverage](https://img.shields.io/badge/coverage-87.2%25-green.svg)](orchestration/manager)

## 📖 简介

SMA-OS (Stateful Machine/Memory Agent Operating System) 是一个面向 AI 智能体集群的调度系统。

### 核心能力

1. **事件溯源状态管理** - 基于 Redis/PostgreSQL 的状态持久化，支持状态回放和快照恢复
2. **eBPF 网络安全** - 内核态包过滤，实现纳秒级延迟的安全防护
3. **DAG 任务编排** - 拓扑排序的分布式任务执行，支持依赖管理和并行调度
4. **意图理解** - 基于 AI 大模型的自然语言理解，将用户指令转换为结构化操作
5. **🆕 任务管理** - 任务拆解、进度查询、暂停/恢复、子任务聚合
6. **🆕 依赖分析** - 循环依赖检测、关键路径分析、并行度计算、依赖可视化

### 设计目标

- 为 AI 智能体提供可靠的调度基础设施
- 通过事件溯源保证状态的可追溯性
- 利用 eBPF 实现高性能的安全隔离
- 支持 DAG 工作流的自动化编排

## 🏗️ 核心架构

```
┌─────────────────────────────────────────────────────────────────┐
│ Observability UI (Next.js)                                      │
│ Real-time DAG Visualization                                     │
├─────────────────────────────────────────────────────────────────┤
│ Control Plane (Rust)                                            │
│ ┌─────────────┬─────────────┬─────────────┬─────────────┐       │
│ │ State Engine│ Identity    │ Teardown    │ eBPF        │       │
│ │(Event Source)│ (IAM)      │ (Cleanup)   │ Gateway     │       │
│ └─────────────┴─────────────┴─────────────┴─────────────┘       │
├─────────────────────────────────────────────────────────────────┤
│ Orchestration (Go)                                              │
│ ┌─────────────┬─────────────┬─────────────┐                     │
│ │ Manager     │ Scheduler   │ Evaluator   │                     │
│ │ (DAG Exec)  │ (Worker Pool)│(Validation) │                     │
│ └─────────────┴─────────────┴─────────────┘                     │
├─────────────────────────────────────────────────────────────────┤
│ Memory Bus (Go)                                                 │
│ ┌─────────────────────┬─────────────────────┐                   │
│ │ Ingestion           │ Vector-KV           │                   │
│ │ (AI Intent)         │ (Vector + KV Store)│                   │
│ └─────────────────────┴─────────────────────┘                   │
├─────────────────────────────────────────────────────────────────┤
│ Execution Layer (Rust)                                          │
│ ┌─────────────────────┬─────────────────────┐                   │
│ │ Sandbox Daemon      │ Stateful REPL       │                   │
│ │ (Firecracker VM)    │ (Persistent Term)   │                   │
│ └─────────────────────┴─────────────────────┘                   │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 组件列表

| 模块 | 语言 | 功能 |
|------|------|------|
| `control-plane/state-engine` | Rust | 事件溯源状态内核 |
| `control-plane/identity` | Rust | 身份认证管理 |
| `control-plane/teardown-ctrl` | Rust | 级联清理控制器 |
| `control-plane/fractal-gateway-ebpf` | Rust (eBPF) | XDP 包过滤 |
| `orchestration/manager` | Go | DAG 拓扑执行引擎 + 任务管理 + 依赖分析 |
| `orchestration/scheduler` | Go | Worker 调度器 |
| `orchestration/evaluator` | Go | 输出验证器 |
| `orchestration/types` | Go | 共享类型定义 |
| `memory-bus/ingestion` | Go | AI 意图提取 |
| `memory-bus/vector-kv` | Go | 向量+KV 存储 |
| `observability-ui/web-dashboard` | TypeScript | DAG 可视化 + 进度追踪 + 依赖图 |

## 🚀 快速启动

### 环境要求

- Docker Desktop 20.10+
- Go 1.21+
- Rust 1.75+
- Node.js 20+

### 1. 克隆仓库

```bash
git clone https://github.com/LING71671/SMA-OS.git
cd SMA-OS
```

### 2. 启动基础设施

```bash
cp .env.example .env
docker-compose up -d
```

### 3. 构建服务

```bash
# Go 服务
cd memory-bus && go build -o bin/ingestion ./ingestion
cd ../orchestration && go build -o bin/manager ./manager

# Rust 服务 (需要 Docker/Linux)
docker run --rm -v "$(pwd):/workspace" -w /workspace rust:latest \
  bash -c "apt-get update && apt-get install -y protobuf-compiler && \
  cd control-plane && cargo build --release"

# eBPF 程序 (需要 Linux 内核 4.19+)
./scripts/build-ebpf.sh
```

### 4. 运行服务

```bash
./scripts/start-services.sh
```

## 🧪 测试

```bash
# Go 测试
cd memory-bus && go test -v ./...
cd ../orchestration && go test -v ./... -cover

# E2E 测试
cd tests/e2e && go test -v .

# 性能基准测试
cd tests/benchmark && go test -bench=. -benchmem .

# Rust 测试
docker run --rm -v "$(pwd):/workspace" -w /workspace rust:latest \
  bash -c "cd control-plane && cargo test --release"
```

### 测试覆盖率

| 模块 | 覆盖率 | 状态 |
|------|--------|------|
| orchestration/manager | 87.2% | ✅ 超过目标 (80%) |
| E2E 测试 | 8 tests | ✅ 全部通过 |
| 性能基准 | 7 benchmarks | ✅ 全部达标 |

## 📊 性能目标

| 指标 | 目标值 |
|------|--------|
| P99 延迟 | < 10ms |
| 并发智能体 | 1000+ |
| 事件吞吐量 | 100K/sec |

## 🔒 安全特性

- **eBPF 沙箱**: 内核态网络过滤
- **身份认证**: PostgreSQL IAM 系统
- **审计日志**: 操作可追溯
- **密钥管理**: 环境变量配置

## 📚 文档

- [部署指南](docs/ops/DEPLOYMENT.md)
- [更新日志](docs/ops/RELEASE_NOTES.md)
- [AI 开发指引](docs/dev/AI_DEVELOPER_GUIDE.md)
- [Agent 开发指南](docs/dev/AGENTS.md)
- [安全审计](docs/security/SECURITY_AUDIT.md)
- [任务管理 API](orchestration/manager/AGENTS.md)

<details>
<summary>更多文档</summary>

- [贡献指南](docs/contributing/CONTRIBUTING.md)
- [代码审查问题](docs/research/CODE_REVIEW_ISSUES.md)
- [分布式系统研究](docs/research/distributed-systems-reconnection-task-allocation.md)
</details>

## 🔌 API 端点

### 任务管理 API

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/v1/tasks/{id}/progress` | GET | 获取任务进度 (0-100%) |
| `/api/v1/tasks/{id}/pause` | POST | 暂停运行中的任务 |
| `/api/v1/tasks/{id}/resume` | POST | 恢复已暂停的任务 |

### 依赖分析 API

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/v1/dags/analysis` | GET | 完整依赖分析 |
| `/api/v1/dags/critical-path` | GET | 关键路径分析 |
| `/api/v1/dags/parallelism` | GET | 并行度分析 |
| `/api/v1/tasks/{id}/impact` | GET | 任务失败影响范围 |

## 🤝 贡献

欢迎参与贡献！请阅读 [贡献指南](docs/contributing/CONTRIBUTING.md) 了解贡献流程。

## 📄 许可证

本项目采用 Apache-2.0 许可证。

## 🙏 致谢

- [Aya](https://github.com/aya-rs/aya) - eBPF 开发框架
- [Tokio](https://tokio.rs/) - Rust 异步运行时
- [React Flow](https://reactflow.dev/) - DAG 可视化

---

Made with ❤️ by LING71671

---
---

<a name="english"></a>
## English

> An Event-Sourcing and eBPF-based AI Agent Scheduling System.

[![CI](https://github.com/LING71671/SMA-OS/actions/workflows/ci.yml/badge.svg)](https://github.com/LING71671/SMA-OS/actions/workflows/ci.yml)
[![Security Audit](https://github.com/LING71671/SMA-OS/actions/workflows/security.yml/badge.svg)](https://github.com/LING71671/SMA-OS/actions/workflows/security.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/LING71671/SMA-OS)](https://github.com/LING71671/SMA-OS/releases)
[![Coverage](https://img.shields.io/badge/coverage-87.2%25-green.svg)](orchestration/manager)

## 📖 Introduction

SMA-OS (Stateful Machine/Memory Agent Operating System) is a scheduling system designed for AI agent clusters.

### Core Capabilities

1. **Event-Sourcing State Management** - Redis/PostgreSQL-based state persistence with replay and snapshot recovery
2. **eBPF Network Security** - Kernel-level packet filtering with nanosecond latency
3. **DAG Task Orchestration** - Topologically-sorted distributed task execution with dependency management
4. **Intent Understanding** - AI LLM-based natural language understanding for structured operations
5. **🆕 Task Management** - Task decomposition, progress tracking, pause/resume, subtask aggregation
6. **🆕 Dependency Analysis** - Cycle detection, critical path analysis, parallelism calculation, dependency visualization

### Design Goals

- Provide reliable scheduling infrastructure for AI agents
- Ensure state traceability through event sourcing
- Achieve high-performance security isolation with eBPF
- Support automated DAG workflow orchestration

## 🏗️ Core Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Observability UI (Next.js)                                      │
│ Real-time DAG Visualization                                     │
├─────────────────────────────────────────────────────────────────┤
│ Control Plane (Rust)                                            │
│ ┌─────────────┬─────────────┬─────────────┬─────────────┐       │
│ │ State Engine│ Identity    │ Teardown    │ eBPF        │       │
│ │(Event Source)│ (IAM)      │ (Cleanup)   │ Gateway     │       │
│ └─────────────┴─────────────┴─────────────┴─────────────┘       │
├─────────────────────────────────────────────────────────────────┤
│ Orchestration (Go)                                              │
│ ┌─────────────┬─────────────┬─────────────┐                     │
│ │ Manager     │ Scheduler   │ Evaluator   │                     │
│ │ (DAG Exec)  │ (Worker Pool)│(Validation) │                     │
│ └─────────────┴─────────────┴─────────────┘                     │
├─────────────────────────────────────────────────────────────────┤
│ Memory Bus (Go)                                                 │
│ ┌─────────────────────┬─────────────────────┐                   │
│ │ Ingestion           │ Vector-KV           │                   │
│ │ (AI Intent)         │ (Vector + KV Store)│                   │
│ └─────────────────────┴─────────────────────┘                   │
├─────────────────────────────────────────────────────────────────┤
│ Execution Layer (Rust)                                          │
│ ┌─────────────────────┬─────────────────────┐                   │
│ │ Sandbox Daemon      │ Stateful REPL       │                   │
│ │ (Firecracker VM)    │ (Persistent Term)   │                   │
│ └─────────────────────┴─────────────────────┘                   │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 Components

| Module | Language | Function |
|--------|----------|----------|
| `control-plane/state-engine` | Rust | Event sourcing state kernel |
| `control-plane/identity` | Rust | Identity and access management |
| `control-plane/teardown-ctrl` | Rust | Cascading cleanup controller |
| `control-plane/fractal-gateway-ebpf` | Rust (eBPF) | XDP packet filtering |
| `orchestration/manager` | Go | DAG execution engine + Task management + Dependency analysis |
| `orchestration/scheduler` | Go | Worker scheduler |
| `orchestration/evaluator` | Go | Output validator |
| `orchestration/types` | Go | Shared type definitions |
| `memory-bus/ingestion` | Go | AI intent extraction |
| `memory-bus/vector-kv` | Go | Vector + KV storage |
| `observability-ui/web-dashboard` | TypeScript | DAG visualization + Progress tracking + Dependency graph |

## 🚀 Quick Start

### Requirements

- Docker Desktop 20.10+
- Go 1.21+
- Rust 1.75+
- Node.js 20+

### 1. Clone Repository

```bash
git clone https://github.com/LING71671/SMA-OS.git
cd SMA-OS
```

### 2. Start Infrastructure

```bash
cp .env.example .env
docker-compose up -d
```

### 3. Build Services

```bash
# Go services
cd memory-bus && go build -o bin/ingestion ./ingestion
cd ../orchestration && go build -o bin/manager ./manager

# Rust services (requires Docker/Linux)
docker run --rm -v "$(pwd):/workspace" -w /workspace rust:latest \
  bash -c "apt-get update && apt-get install -y protobuf-compiler && \
  cd control-plane && cargo build --release"

# eBPF programs (requires Linux kernel 4.19+)
./scripts/build-ebpf.sh
```

### 4. Run Services

```bash
./scripts/start-services.sh
```

## 🧪 Testing

```bash
# Go tests
cd memory-bus && go test -v ./...
cd ../orchestration && go test -v ./... -cover

# E2E tests
cd tests/e2e && go test -v .

# Benchmarks
cd tests/benchmark && go test -bench=. -benchmem .

# Rust tests
docker run --rm -v "$(pwd):/workspace" -w /workspace rust:latest \
  bash -c "cd control-plane && cargo test --release"
```

### Test Coverage

| Module | Coverage | Status |
|--------|----------|--------|
| orchestration/manager | 87.2% | ✅ Exceeds target (80%) |
| E2E Tests | 8 tests | ✅ All passed |
| Benchmarks | 7 benchmarks | ✅ All met |

## 📊 Performance Targets

| Metric | Target |
|--------|--------|
| P99 Latency | < 10ms |
| Concurrent Agents | 1000+ |
| Event Throughput | 100K/sec |

## 🔒 Security Features

- **eBPF Sandbox**: Kernel-level network filtering
- **Identity Authentication**: PostgreSQL IAM system
- **Audit Logging**: Traceable operations
- **Secret Management**: Environment variable configuration

## 📚 Documentation

- [Deployment Guide](docs/ops/DEPLOYMENT.md)
- [Release Notes](docs/ops/RELEASE_NOTES.md)
- [AI Developer Guide](docs/dev/AI_DEVELOPER_GUIDE.md)
- [Agent Development Guide](docs/dev/AGENTS.md)
- [Security Audit](docs/security/SECURITY_AUDIT.md)
- [Task Management API](orchestration/manager/AGENTS.md)

<details>
<summary>More Documentation</summary>

- [Contributing Guide](docs/contributing/CONTRIBUTING.md)
- [Code Review Issues](docs/research/CODE_REVIEW_ISSUES.md)
- [Distributed Systems Research](docs/research/distributed-systems-reconnection-task-allocation.md)
</details>

## 🔌 API Endpoints

### Task Management API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/tasks/{id}/progress` | GET | Get task progress (0-100%) |
| `/api/v1/tasks/{id}/pause` | POST | Pause running task |
| `/api/v1/tasks/{id}/resume` | POST | Resume paused task |

### Dependency Analysis API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/dags/analysis` | GET | Full dependency analysis |
| `/api/v1/dags/critical-path` | GET | Critical path analysis |
| `/api/v1/dags/parallelism` | GET | Parallelism analysis |
| `/api/v1/tasks/{id}/impact` | GET | Task failure impact scope |

## 🤝 Contributing

Contributions are welcome! Please read the [Contributing Guide](docs/contributing/CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under Apache-2.0.

## 🙏 Acknowledgments

- [Aya](https://github.com/aya-rs/aya) - eBPF development framework
- [Tokio](https://tokio.rs/) - Rust async runtime
- [React Flow](https://reactflow.dev/) - DAG visualization

---

Made with ❤️ by LING71671
