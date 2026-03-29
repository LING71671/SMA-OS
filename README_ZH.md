# SMA-OS v1.2.0

[中文](./README.md) | [English](./README_ZH.md)

---

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

- [Deployment Guide](docs/ops/DEPLOYMENT_ZH.md)
- [Release Notes](docs/ops/RELEASE_NOTES_ZH.md)
- [AI Developer Guide](docs/dev/AI_DEVELOPER_GUIDE.md)
- [Agent Development Guide](docs/dev/AGENTS.md)
- [Security Audit](docs/security/SECURITY_AUDIT.md)
- [Task Management API](orchestration/manager/AGENTS.md)

<details>
<summary>More Documentation</summary>

- [Contributing Guide](docs/contributing/CONTRIBUTING_ZH.md)
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

Contributions are welcome! Please read the [Contributing Guide](docs/contributing/CONTRIBUTING_ZH.md) for details.

## 📄 License

This project is licensed under Apache-2.0.

## 🙏 Acknowledgments

- [Aya](https://github.com/aya-rs/aya) - eBPF development framework
- [Tokio](https://tokio.rs/) - Rust async runtime
- [React Flow](https://reactflow.dev/) - DAG visualization

---

Made with ❤️ by LING71671
