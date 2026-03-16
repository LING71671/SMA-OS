# SMA-OS Release v1.1.0

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
- **ingestion**: SLM-powered intent extraction with DeepSeek
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
./build-ebpf.sh

# Run services
./start-services.sh
```

---

**Full Changelog**: https://github.com/LING71671/SMA-OS/compare/v1.0.0...v1.1.0
