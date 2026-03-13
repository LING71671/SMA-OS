# Draft: SMA-OS Completion Plan

## Project Analysis

### Current State
**Completed Modules (Production Ready):**
- state-engine: Event sourcing with Redis/PostgreSQL, cluster, failover
- sandbox-daemon: Firecracker MicroVM lifecycle, warm pool <5ms
- manager: DAG topological execution with channels/goroutines
- scheduler: Warm pool scheduling
- evaluator: Output validation
- ingestion: DeepSeek API + regex fallback
- web-dashboard: Next.js + ReactFlow + Framer Motion
- teardown-ctrl: Cascading cleanup
- security-audit: Audit reports
- benchmarks: Performance testing
- chaos-tests framework: Core framework (3 scenarios stubbed)

**Stub/Partial Modules:**
- fractal-gateway: Only `add(2,2)` stub - needs eBPF auth gateway
- vector-kv: Mock responses only - needs FoundationDB/Weaviate/Redis
- chaos scenarios: 2 TODOs (resource_exhaustion, network_partition)
- formal-verifier: TLA+ specs only - needs Rust implementation

**Missing Infrastructure:**
- test-all.ps1 / test-all.sh scripts
- CI/CD pipeline (GitHub Actions)
- Frontend tests (Jest/React Testing Library)

### Technical Requirements

#### fractal-gateway (Rust)
- Load eBPF programs from fractal-gateway-ebpf
- IAM policy enforcement
- Resource authentication
- Network filtering integration
- Zero-trust security model

#### vector-kv (Go)
- FoundationDB connection for structured KV
- Weaviate client for vector embeddings
- Redis hot cache integration
- Async HNSW clustering (10:1 compression)
- <1ms read latency target

#### chaos-tests scenarios (Rust)
- Resource exhaustion: CPU/memory/disk pressure
- Network partition: tc command for latency/partitions

#### CI/CD (GitHub Actions)
- Rust build & test matrix
- Go build & test
- TypeScript build & lint
- Docker image builds
- Chaos tests in staging

## Requirements Confirmed
- Implement all stub modules to production quality
- Follow existing AGENTS.md conventions
- Maintain zero-compromise security
- All tests must pass
- Documentation must be accurate
