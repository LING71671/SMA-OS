# SMA-OS Agent Development Guide

**Generated**: 2026-03-10  
**Commit**: See `git log -1`  
**Branch**: main  

## Hierarchy

This root AGENTS.md covers project-wide conventions. Module-specific guides:

### Control Plane (Rust)
- [`control-plane/state-engine/AGENTS.md`](control-plane/state-engine/AGENTS.md) - Event sourcing state kernel
- [`control-plane/fractal-gateway/AGENTS.md`](control-plane/fractal-gateway/AGENTS.md) - eBPF security gateway
- [`control-plane/fractal-gateway-ebpf/AGENTS.md`](control-plane/fractal-gateway-ebpf/AGENTS.md) - XDP packet filtering 🆕
- [`control-plane/formal-verifier/AGENTS.md`](control-plane/formal-verifier/AGENTS.md) - TLA+ formal verification 🆕
- [`control-plane/teardown-ctrl/AGENTS.md`](control-plane/teardown-ctrl/AGENTS.md) - Cascading cleanup controller 🆕

### Orchestration (Go)
- [`orchestration/manager/AGENTS.md`](orchestration/manager/AGENTS.md) - DAG topological execution
- [`orchestration/scheduler/AGENTS.md`](orchestration/scheduler/AGENTS.md) - Worker dispatch and warm pool
- [`orchestration/evaluator/AGENTS.md`](orchestration/evaluator/AGENTS.md) - Output validation and rollback

### Memory Bus (Go)
- [`memory-bus/ingestion/AGENTS.md`](memory-bus/ingestion/AGENTS.md) - SLM-powered intent extraction
- [`memory-bus/vector-kv/AGENTS.md`](memory-bus/vector-kv/AGENTS.md) - Vector + KV storage with compression

### Execution Layer (Rust)
- [`execution-layer/sandbox-daemon/AGENTS.md`](execution-layer/sandbox-daemon/AGENTS.md) - Firecracker MicroVM lifecycle
- [`execution-layer/stateful-repl/AGENTS.md`](execution-layer/stateful-repl/AGENTS.md) - Persistent terminal sessions

### Observability (TypeScript)
- [`observability-ui/web-dashboard/AGENTS.md`](observability-ui/web-dashboard/AGENTS.md) - Next.js observability UI

---

## Quick Start

### Prerequisites
- **Go**: 1.25+ 
- **Rust**: 1.75+
- **Node.js**: 20+
- **Docker Desktop** with WSL2 (Windows)

### Build Commands

```bash
# Rust (Control Plane & Execution Layer)
cd control-plane && cargo build --release
cd control-plane/state-engine && cargo build

# Go (Memory Bus & Orchestration)
cd memory-bus && go build -o bin/ingestion ./ingestion
cd orchestration && go build -o bin/manager ./manager

# Frontend (Observability UI)
cd observability-ui/web-dashboard && npm install && npm run build
```

### Test Commands

```bash
# Run all tests
.\test-all.ps1          # Windows PowerShell
./test-all.sh           # Linux/macOS

# Rust tests
cd control-plane && cargo test
cd control-plane/state-engine && cargo test -- --nocapture

# Go tests with coverage
cd memory-bus && go test -v -coverprofile=coverage.out ./...
go tool cover -html=coverage.out -o coverage.html

# Frontend tests
cd observability-ui/web-dashboard && npm run lint
```

### Lint Commands

```bash
# Rust
cargo clippy --all-targets --all-features -- -D warnings

# Go
golangci-lint run ./...

# Frontend
npm run lint
```

## Code Style Guidelines

### General Principles
1. **Zero-compromise security**: Never bypass eBPF sandbox or IAM policies
2. **Event sourcing**: All state changes must be append-only with version tracking
3. **Performance + isolation**: Balance nanosecond performance with strict isolation
4. **Bilingual docs**: Chinese for explanations, English for code/comments

### Rust Code Style

#### Imports
```rust
// Standard library first
use std::sync::Arc;
use thiserror::Error;

// External crates
use redis::AsyncCommands;
use sqlx::{PgPool, Row};

// Internal modules
use crate::models::{Snapshot, StateEvent};
```

#### Error Handling
- Use `thiserror` for custom error types
- Never use `.unwrap()` or `.expect()` in production code
- Prefer `Result<T, EngineError>` with explicit error variants
- Use `?` operator for error propagation

```rust
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("PostgreSQL error: {0}")]
    Postgres(#[from] sqlx::Error),
}
```

#### Naming Conventions
- **Structs/Enums**: PascalCase (`StateEngine`, `StateEvent`)
- **Functions/Methods**: snake_case (`append_event`, `get_latest_snapshot`)
- **Constants**: UPPER_SNAKE_CASE (`REDIS_CACHE_TTL_SECS`)
- **Files**: snake_case (`engine.rs`, `state_event.rs`)

#### Comments
- Use English for all comments and documentation
- Include doc comments (`///`) for public APIs
- Explain *why*, not just *what*

### Go Code Style

#### Imports
```go
// Standard library first
import (
    "encoding/json"
    "log"
    "sync"
)

// External packages
import (
    "github.com/google/uuid"
)

// Internal packages
import (
    "sma-os/memory-bus/models"
)
```

#### Error Handling
- Always check errors explicitly
- Use descriptive error messages with context
- Never ignore errors with `_`

```go
if err != nil {
    log.Printf("[Component] Failed to process: %v", err)
    return err
}
```

#### Naming Conventions
- **Structs/Types**: PascalCase (`TaskNode`, `DAGManager`)
- **Functions**: PascalCase for export, camelCase for private
- **Constants**: ALL_CAPS with underscores
- **Files**: snake_case (`ingestion_test.go`)

### TypeScript/React Style

#### Imports
```typescript
// React and Next.js first
import { useState, useCallback } from "react";
import { motion } from "framer-motion";

// Third-party libraries
import ReactFlow from "reactflow";
import "reactflow/dist/style.css";

// Internal components
import { DagNode } from "@/components/DagNode";
```

#### Component Structure
```typescript
"use client";

export interface DagViewerProps {
  initialNodes: Node[];
  initialEdges: Edge[];
}

export default function DagViewer({ initialNodes }: DagViewerProps) {
  // Hooks first
  const [nodes, setNodes] = useNodesState(initialNodes);
  
  // Event handlers
  const onNodeClick = useCallback((node: Node) => {
    // Handler logic
  }, []);
  
  // Render
  return <div>{/* JSX */}</div>;
}
```

#### Naming Conventions
- **Components**: PascalCase (`DagViewer`, `StateNode`)
- **Functions/Variables**: camelCase (`onNodeClick`, `isLoading`)
- **Constants**: UPPER_SNAKE_CASE (`MAX_RETRY_COUNT`)
- **CSS**: kebab-case (`.dag-viewer`, `.state-node`)

## Architecture Overview

### Directory Structure
```
SMA-OS/
├── control-plane/          # Rust: State kernel, eBPF, formal verification
│   ├── state-engine/      # Event sourcing with Redis/PostgreSQL
│   ├── fractal-gateway/   # Resource isolation and auth
│   └── teardown-ctrl/     # Cascading cleanup controller
├── orchestration/          # Go: DAG orchestration and scheduling
│   ├── manager/          # Topological task execution
│   ├── scheduler/        # Worker dispatch
│   └── evaluator/        # Output validation
├── execution-layer/        # Rust: Firecracker MicroVM management
│   ├── sandbox-daemon/   # VM lifecycle management
│   └── stateful-repl/    # Persistent terminals
├── memory-bus/            # Go: Structured memory with LLM fallback
│   ├── ingestion/        # SLM-powered intent extraction
│   └── vector-kv/        # Vector + KV storage
└── observability-ui/      # Next.js: Real-time DAG visualization
```

### Key Patterns

#### Event Sourcing (Rust)
```rust
// All state changes are events appended to a log
pub async fn append_event(&self, event: StateEvent) -> Result<(), EngineError> {
    // 1. Write to Redis for fast recovery
    // 2. Persist to PostgreSQL for durability
    // 3. Trigger snapshot every 1000 events
}
```

#### DAG Execution (Go)
```go
// Topological sort with concurrent worker dispatch
func (dm *DAGManager) Execute() error {
    // 1. Compute in-degrees
    // 2. Enqueue zero in-degree nodes
    // 3. Dispatch workers concurrently
    // 4. Decrement in-degrees on completion
}
```

## Testing Guidelines

### Rust Tests
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_append_and_replay() {
        // Use testcontainers for isolated DB tests
        // Test event append + state recovery
    }
}
```

### Go Tests
```go
func TestProcessInput_ValidInput(t *testing.T) {
    // Mock external APIs with httptest.Server
    // Test both success and error paths
}
```

## Common Issues

### Redis Connection Fails
- Check `REDIS_URL` environment variable
- Ensure Docker container is running: `docker ps | grep redis`

### PostgreSQL Migration Errors
- Verify `DATABASE_URL` is correct
- Run migrations manually: `sqlx migrate run --database-url <url>`

### Cargo Dependency Conflicts
- Clear cache: `cargo clean && cargo update`
- Check for conflicting tokio features

### Go Module Issues
- Clear module cache: `go clean -modcache && go mod tidy`

## Deployment

### Local Development
```bash
# Start all dependencies
docker-compose up -d postgres redis clickhouse weaviate jaeger prometheus

# Run services
cargo run --bin state-engine
go run ./memory-bus/ingestion
npm run dev --prefix observability-ui/web-dashboard
```

### Production
- Use Kubernetes Helm charts for Enterprise mode
- Configure resource limits and network policies
- Enable OpenTelemetry tracing to Jaeger

## Security Notes

1. **Never commit `.env` files** - Use `.env.example` as template
2. **API keys in environment variables only** - DeepSeek, database URLs, etc.
3. **eBPF sandbox is mandatory** - No bypassing for "convenience"
4. **Audit all external dependencies** - Use `cargo audit` and `npm audit`

## Getting Help

- **Architecture questions**: See `AI_DEVELOPER_GUIDE.md`
- **API documentation**: Check inline doc comments
- **Debugging**: Enable tracing with `RUST_LOG=debug`
- **Performance**: Check Jaeger traces for latency analysis

---

## Module Index

| Module | Purpose | Language | Complexity |
|--------|---------|----------|------------|
| `control-plane/state-engine` | Event sourcing with Redis/PostgreSQL | Rust | High |
| `control-plane/fractal-gateway` | Resource isolation and auth | Rust | Medium |
| `control-plane/fractal-gateway-ebpf` | XDP packet filtering | Rust | High |
| `control-plane/formal-verifier` | TLA+ formal verification | TLA+ | High |
| `control-plane/teardown-ctrl` | Cascading cleanup controller | Rust | Medium |
| `orchestration/manager` | Topological task execution | Go | Medium |
| `orchestration/scheduler` | Worker dispatch | Go | Low |
| `orchestration/evaluator` | Output validation | Go | Low |
| `memory-bus/ingestion` | SLM-powered intent extraction | Go | Medium |
| `memory-bus/vector-kv` | Vector + KV storage | Go | Low |
| `execution-layer/sandbox-daemon` | VM lifecycle management | Rust | Medium |
| `execution-layer/stateful-repl` | Persistent terminals | Rust | Low |
| `observability-ui/web-dashboard` | Real-time DAG visualization | TypeScript | Medium |
| `chaos-tests` | Chaos engineering framework | Rust | High |
| `benchmarks` | Performance benchmarking suite | Rust/Go | Medium |
| `sma-proto` | gRPC Protocol definitions | Protobuf | Low |

## Final Goals

### Phase 1: Core Infrastructure ✅
- [x] Event sourcing state engine with Redis/PostgreSQL
- [x] eBPF security gateway
- [x] DAG orchestration layer
- [x] Observability UI with real-time DAG visualization

### Phase 2: Production Readiness ✅
- [x] Firecracker MicroVM integration
- [x] eBPF probe deployment
- [x] Chaos engineering tests
- [x] Performance benchmarks (P99 latency < 10ms)
- [x] Documentation completeness (>90%)

### Phase 3: Scale & Reliability ✅
- [x] Horizontal scaling (1000+ concurrent agents)
- [x] Redis cluster with failover
- [x] Connection pooling & rate limiting
- [x] Multi-region deployment
- [x] Automated failover and recovery
- [x] Advanced monitoring and alerting
- [x] Security audit and penetration testing

### Phase 4: Ecosystem ✅
- [x] Plugin architecture for custom executors
- [x] Marketplace for pre-built agent templates
- [x] Community-driven module registry
- [x] Enterprise support and SLA

## Roadmap Timeline

| Quarter | Focus | Milestone |
|---------|-------|-----------|
| Q2 2026 | Core completion | Firecracker + eBPF production ready |
| Q3 2026 | Performance | 1000+ agents, P99 < 10ms |
| Q4 2026 | Enterprise | Multi-region, HA, security audit |
| Q1 2027 | Ecosystem | Plugin system, marketplace launch |
