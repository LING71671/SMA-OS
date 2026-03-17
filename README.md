# SMA-OS v1.1.0

> 一个基于事件溯源和 eBPF 的 AI 智能体调度系统。

[![CI](https://github.com/LING71671/SMA-OS/actions/workflows/ci.yml/badge.svg)](https://github.com/LING71671/SMA-OS/actions/workflows/ci.yml)
[![Security Audit](https://github.com/LING71671/SMA-OS/actions/workflows/security.yml/badge.svg)](https://github.com/LING71671/SMA-OS/actions/workflows/security.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/LING71671/SMA-OS)](https://github.com/LING71671/SMA-OS/releases)

## 📖 简介

SMA-OS (Stateful Machine/Memory Agent Operating System) v1.1.0 提供以下核心能力：

1. **事件溯源状态管理** - 基于 Redis/PostgreSQL 的状态持久化
2. **eBPF 网络过滤** - 内核态包过滤，低延迟安全防护
3. **DAG 任务编排** - 拓扑排序的分布式任务执行
4. **意图提取** - 基于 DeepSeek 的自然语言理解

## 🏗️ 核心架构

```
┌─────────────────────────────────────────────────────────────────┐
│                     Observability UI (Next.js)                   │
│              Real-time DAG Visualization & Metrics               │
├─────────────────────────────────────────────────────────────────┤
│                     Control Plane (Rust)                         │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐      │
│  │ State Engine│  Identity   │  Teardown   │   eBPF      │      │
│  │ (Event Sourcing) │ (IAM)  │   (Cleanup) │  Gateway    │      │
│  └─────────────┴─────────────┴─────────────┴─────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│                    Orchestration (Go)                            │
│  ┌─────────────┬─────────────┬─────────────┐                    │
│  │   Manager   │  Scheduler  │  Evaluator  │                    │
│  │  (DAG Exec) │ (Worker Pool)│(Validation) │                    │
│  └─────────────┴─────────────┴─────────────┘                    │
├─────────────────────────────────────────────────────────────────┤
│                     Memory Bus (Go)                              │
│  ┌─────────────────────┬─────────────────────┐                  │
│  │     Ingestion       │      Vector-KV      │                  │
│  │  (DeepSeek Intent)  │  (Vector + KV Store)│                  │
│  └─────────────────────┴─────────────────────┘                  │
├─────────────────────────────────────────────────────────────────┤
│                  Execution Layer (Rust)                          │
│  ┌─────────────────────┬─────────────────────┐                  │
│  │   Sandbox Daemon    │    Stateful REPL    │                  │
│  │  (Firecracker VM)   │  (Persistent Term)  │                  │
│  └─────────────────────┴─────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 组件列表

| 模块 | 语言 | 功能 | 状态 |
|------|------|------|------|
| `control-plane/state-engine` | Rust | 事件溯源状态内核 | ✅ |
| `control-plane/identity` | Rust | 身份认证管理 | ✅ |
| `control-plane/teardown-ctrl` | Rust | 级联清理控制器 | ✅ |
| `control-plane/fractal-gateway-ebpf` | Rust (eBPF) | XDP 包过滤 | ✅ |
| `orchestration/manager` | Go | DAG 拓扑执行引擎 | ✅ |
| `orchestration/scheduler` | Go | Worker 调度器 | ✅ |
| `orchestration/evaluator` | Go | 输出验证器 | ✅ |
| `memory-bus/ingestion` | Go | SLM 意图提取 | ✅ |
| `memory-bus/vector-kv` | Go | 向量+KV 存储 | ✅ |
| `observability-ui/web-dashboard` | TypeScript | 实时 DAG 可视化 | ✅ |

## 🚀 快速启动

### 环境要求

- **Docker Desktop** 20.10+ (with WSL2 on Windows)
- **Go** 1.21+
- **Rust** 1.75+
- **Node.js** 20+

### 1. 克隆仓库

```bash
git clone https://github.com/LING71671/SMA-OS.git
cd SMA-OS
```

### 2. 启动基础设施

```bash
# 复制环境配置
cp .env.example .env

# 编辑 .env 设置密码
# POSTGRES_PASSWORD=your_password
# CLICKHOUSE_PASSWORD=your_password

# 启动所有基础设施服务
docker-compose up -d
```

### 3. 构建服务

#### Go 服务 (跨平台)
```bash
cd memory-bus && go build -o bin/ingestion ./ingestion
cd orchestration && go build -o bin/manager ./manager
```

#### Rust 服务 (需要 Docker/Linux)
```bash
docker run --rm -v "$(pwd):/workspace" -w /workspace rust:latest \
  bash -c "apt-get update && apt-get install -y protobuf-compiler && \
  cd control-plane && cargo build --release"
```

#### eBPF 程序 (需要 Linux 内核 4.19+)
```bash
./build-ebpf.sh
```

### 4. 运行服务

```bash
# 使用启动脚本
./start-services.sh

# 或手动启动
./memory-bus/bin/ingestion &
./orchestration/bin/manager &
```

## 🧪 测试

```bash
# Go 测试
cd memory-bus && go test -v ./...
cd orchestration && go test -v ./...

# Rust 测试 (需要 Docker)
docker run --rm -v "$(pwd):/workspace" -w /workspace rust:latest \
  bash -c "cd control-plane && cargo test --release"
```

## 📊 性能指标

| 指标 | 目标 |
|------|------|
| P99 延迟 | < 10ms |
| 并发智能体 | 1000+ |
| 事件吞吐量 | 100K/sec |

## 🔒 安全特性

- **eBPF 沙箱**: 内核态网络过滤，纳秒级响应
- **身份认证**: 基于 PostgreSQL 的 IAM 系统
- **审计日志**: 所有操作可追溯
- **密钥管理**: 环境变量，无硬编码

## 📚 文档

- [部署指南](docs/DEPLOYMENT.md)
- [API 文档](docs/api.md)
- [架构设计](docs/architecture.md)
- [更新日志](RELEASE_NOTES.md)
- [AI 开发指引](AI_DEVELOPER_GUIDE.md)

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 📄 许可证

本项目采用 Apache-2.0 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [Aya](https://github.com/aya-rs/aya) - eBPF 开发框架
- [Tokio](https://tokio.rs/) - Rust 异步运行时
- [React Flow](https://reactflow.dev/) - DAG 可视化
- [DeepSeek](https://deepseek.com/) - LLM API

---

**[⬆ 返回顶部](#sma-os-v20)**

Made with ❤️ by LING71671
