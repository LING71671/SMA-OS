# SMA-OS v2.0

> 一个高性能、数学可证明绝对确定性、且具备强力沙箱物理隔离与认知编排的高端 AI 操作系统基座。

## 简介
SMA-OS (Stateful Machine/Memory Agent Operating System) v2.0 致力于为下一代百万级 AI 智能体集群提供：
1. **绝对确定的内核状态转移**（形式化验证保障）。
2. **极高密度的物理执行层**（基于 Firecracker 与 eBPF 探针）。
3. **视觉震撼的可观测性平面**（实时 DAG 拓扑、探针动画反馈、内存回放系统）。
4. **长短记忆读写分离数据总线**（LLM 与结构化数据高速通信）。

## 核心架构模块
1. **控制面与状态内核层 (Control Plane & State Kernel)** (Rust)
2. **认知编排与业务数据面 (Cognitive Orchestration & Data Plane)** (Go)
3. **读写分离的结构化记忆总线 (Decoupled Structured Memory Bus)** (Go/Rust)
4. **沙箱化物理执行层 (Sandboxed Execution Layer)** (Rust)
5. **统一可观测性平面 (Observability Plane)** (TypeScript / Next.js)

## 快速启动
本项目运行依赖于 Docker Desktop 及 WSL2 环境。
使用 PowerShell 自动化检查本机环境是否就绪：
```powershell
.\check-env.ps1
```

一键启动底层配套服务（PostgreSQL, ClickHouse, Redis, Weaviate, Jaeger, Prometheus等）：
```bash
docker-compose up -d
```

## AI 开发指引
对于进行接力开发或者自动修复指令的 AI Agent 协作者，请务必阅读本目录底下的 `AI_DEVELOPER_GUIDE.md` 获取技术架构纪要和全局系统约束。
