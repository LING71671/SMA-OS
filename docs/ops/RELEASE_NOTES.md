# SMA-OS Release Notes

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
GET  /api/v1/tasks/{id}/progress    # Get task progress
POST /api/v1/tasks/{id}/pause       # Pause running task
POST /api/v1/tasks/{id}/resume      # Resume paused task
GET  /api/v1/dags/analysis          # Full dependency analysis
GET  /api/v1/dags/critical-path     # Critical path only
GET  /api/v1/dags/parallelism       # Parallelism info
GET  /api/v1/tasks/{id}/impact      # Failure impact scope
```

---

## v1.1.0 - eBPF Infrastructure

## 🚀 Highlights

This release includes major improvements to the eBPF infrastructure, test coverage, and build system.

## 📦 Components

### Control Plane (Rust)
- **state-engine**: Event sourcing state kernel with Redis/PostgreSQL
- **teardown-ctrl**: Cascading cleanup controller
- **identity**: Identity and access management module
- **fractal-gateway**: eBPF security gateway (userspace)
- **fractal-gateway-ebpf**: XDP packet filtering (eBPF)

### Orchestration (Go)
- **manager**: DAG topological execution engine
- **scheduler**: Worker dispatch with warm pool
- **evaluator**: Output validation and rollback

### Memory Bus (Go)
- **ingestion**: SLM-powered intent extraction with AI LLM
- **vector-kv**: Vector + KV storage with compression

### Observability UI (TypeScript)
- **web-dashboard**: Real-time DAG visualization with Next.js

## 🔧 What's Changed

### eBPF Infrastructure
- Completely rewrote eBPF build system with xtask
- Separated eBPF code from userspace code
- Added proper `#![no_std]` eBPF program
- Fixed bpf-linker integration

### Testing
- Fixed Go ingestion test API compatibility
- Rewrote Rust identity integration tests
- All tests now passing (23 Rust + 14 Go tests)

### Build System
- Added Dockerfile for eBPF compilation
- Added Dockerfile for services runtime
- Created build scripts for automation

## 📥 Downloads

| Platform | Component | Binary |
|----------|-----------|--------|
| Linux x64 | state-engine | `state-engine` |
| Linux x64 | teardown-ctrl | `teardown-ctrl` |
| eBPF | fractal-gateway | `fractal-gateway-ebpf.o` |
| Cross-platform | ingestion | `ingestion` |
| Cross-platform | manager | `manager` |

## 🛡️ Security

- eBPF sandbox for network filtering
- Identity-based access control
- Audit logging for all operations

## 📋 Requirements

- **eBPF**: Linux kernel 4.19+ with BTF support
- **Rust services**: Docker or Linux environment
- **Go services**: Windows/Linux/macOS
- **Infrastructure**: Docker Compose

## 🔗 Quick Start

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
