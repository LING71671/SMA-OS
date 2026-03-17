# SMA-OS v1.1.0

> 一个基于事件溯源和 eBPF 的 AI 智能体调度系统。

[![CI](https://github.com/LING71671/SMA-OS/actions/workflows/ci.yml/badge.svg)](https://github.com/LING71671/SMA-OS/actions/workflows/ci.yml)
[![Security Audit](https://github.com/LING71671/SMA-OS/actions/workflows/security.yml/badge.svg)](https://github.com/LING71671/SMA-OS/actions/workflows/security.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/LING71671/SMA-OS)](https://github.com/LING71671/SMA-OS/releases)

## 📖 简介

SMA-OS (Stateful Machine/Memory Agent Operating System) 是一个面向 AI 智能体集群的调度系统。

### 核心能力

1. **事件溯源状态管理** - 基于 Redis/PostgreSQL 的状态持久化，支持状态回放和快照恢复
2. **eBPF 网络安全** - 内核态包过滤，实现纳秒级延迟的安全防护
3. **DAG 任务编排** - 拓扑排序的分布式任务执行，支持依赖管理和并行调度
4. **意图理解** - 基于 AI 大模型的自然语言理解，将用户指令转换为结构化操作

### 设计目标

- 为 AI 智能体提供可靠的调度基础设施
- 通过事件溯源保证状态的可追溯性
- 利用 eBPF 实现高性能的安全隔离
- 支持 DAG 工作流的自动化编排

## 🏗️ 核心架构

```
┌─────────────────────────────────────────────────────────────────┐
│                   Observability UI (Next.js)                    │
│                Real-time DAG Visualization                      │
├─────────────────────────────────────────────────────────────────┤
│                     Control Plane (Rust)                        │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐      │
│  │ State Engine│  Identity   │  Teardown   │   eBPF      │      │
│  │(Event Source)│   (IAM)    │  (Cleanup)  │  Gateway    │      │
│  └─────────────┴─────────────┴─────────────┴─────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│                    Orchestration (Go)                           │
│  ┌─────────────┬─────────────┬─────────────┐                    │
│  │   Manager   │  Scheduler  │  Evaluator  │                    │
│  │  (DAG Exec) │ (Worker Pool)│(Validation) │                    │
│  └─────────────┴─────────────┴─────────────┘                    │
├─────────────────────────────────────────────────────────────────┤
│                     Memory Bus (Go)                             │
│  ┌─────────────────────┬─────────────────────┐                  │
│  │     Ingestion       │      Vector-KV      │                  │
│  │   (AI Intent)       │  (Vector + KV Store)│                  │
│  └─────────────────────┴─────────────────────┘                  │
├─────────────────────────────────────────────────────────────────┤
│                  Execution Layer (Rust)                         │
│  ┌─────────────────────┬─────────────────────┐                  │
│  │   Sandbox Daemon    │    Stateful REPL    │                  │
│  │  (Firecracker VM)   │  (Persistent Term)  │                  │
│  └─────────────────────┴─────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 组件列表

| 模块 | 语言 | 功能 |
|------|------|------|
| `control-plane/state-engine` | Rust | 事件溯源状态内核 |
| `control-plane/identity` | Rust | 身份认证管理 |
| `control-plane/teardown-ctrl` | Rust | 级联清理控制器 |
| `control-plane/fractal-gateway-ebpf` | Rust (eBPF) | XDP 包过滤 |
| `orchestration/manager` | Go | DAG 拓扑执行引擎 |
| `orchestration/scheduler` | Go | Worker 调度器 |
| `orchestration/evaluator` | Go | 输出验证器 |
| `memory-bus/ingestion` | Go | AI 意图提取 |
| `memory-bus/vector-kv` | Go | 向量+KV 存储 |
| `observability-ui/web-dashboard` | TypeScript | DAG 可视化 |

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
./build-ebpf.sh
```

### 4. 运行服务

```bash
./start-services.sh
```

## 🧪 测试

```bash
# Go 测试
cd memory-bus && go test -v ./...
cd ../orchestration && go test -v ./...

# Rust 测试
docker run --rm -v "$(pwd):/workspace" -w /workspace rust:latest \
  bash -c "cd control-plane && cargo test --release"
```

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

- [部署指南](docs/DEPLOYMENT.md)
- [更新日志](RELEASE_NOTES.md)
- [AI 开发指引](AI_DEVELOPER_GUIDE.md)
- [安全审计](docs/security/SECURITY_AUDIT.md)

<details>
<summary>更多文档</summary>

- [代码审查问题](docs/research/CODE_REVIEW_ISSUES.md)
- [分布式系统研究](docs/research/distributed-systems-reconnection-task-allocation.md)
</details>

## 🤝 贡献

欢迎参与贡献！请阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 了解贡献流程。

## 📄 许可证

本项目采用 Apache-2.0 许可证。

## 🙏 致谢

- [Aya](https://github.com/aya-rs/aya) - eBPF 开发框架
- [Tokio](https://tokio.rs/) - Rust 异步运行时
- [React Flow](https://reactflow.dev/) - DAG 可视化

---

Made with ❤️ by LING71671
