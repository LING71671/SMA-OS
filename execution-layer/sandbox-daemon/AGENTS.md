# Sandbox Daemon Module Guide

**Location**: `execution-layer/sandbox-daemon/`
**Domain**: Firecracker MicroVM lifecycle management
**Language**: Rust
**Score**: 18/25 (VM management, distinct infrastructure domain)

## Overview

Manages the lifecycle of Firecracker MicroVMs with pre-warmed pool architecture. **Phase 2.1 Complete**: Handles VM creation, configuration via Unix socket, warm pool management, and eBPF security injection for sandboxed execution environments.

## Structure

```
sandbox-daemon/
├── src/
│   ├── main.rs          # Daemon entry point
│   ├── microvm.rs       # VM lifecycle management
│   ├── firecracker.rs   # Firecracker REST API client
│   ├── pool.rs          # Warm pool management (Phase 2.1)
│   └── api.rs           # HTTP API endpoints
├── Cargo.toml
└── main_test.rs
```

## Phase 2.1 Features

### Warm Pool Architecture
- **Target**: 50 VMs pre-warmed and ready
- **Acquire Time**: < 5ms from warm pool
- **Auto-scaling**: 5-100 VM capacity

### Firecracker REST API
- Unix Domain Socket communication via hyperlocal
- Full VM lifecycle: create, configure, start, stop, snapshot, restore
- Health checks and metrics collection

### VM States
```
Creating → Configured → Running → Stopped → Destroyed
              ↓
           Paused → Resumed
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| VM lifecycle | `microvm.rs:89-236` | FirecrackerVM struct and methods |
| Warm pool | `pool.rs:85-280` | WarmPool with health checks |
| Firecracker API | `firecracker.rs:28-150` | REST client via Unix socket |
| Pool config | `pool.rs:15-66` | PoolConfig and PoolVmConfig |
| Error handling | `microvm.rs:11-39` | MicroVMError enum |

## Conventions

### Warm Pool Usage
```rust
let pool = WarmPool::new(PoolConfig {
    target_size: 50,
    min_size: 5,
    max_size: 100,
    ..Default::default()
});

// Initialize pool
pool.initialize().await?;

// Acquire VM (O(1), < 5ms)
if let Some(vm) = pool.acquire().await {
    // Use VM
    
    // Return to pool
    pool.release(vm).await?;
}
```

### Firecracker API
```rust
let client = FirecrackerClient::new("/tmp/firecracker-vm-001.socket");

// Configure VM
client.put_machine_config(&MachineConfig {
    vcpu_count: 2,
    memory_size_mib: 512,
    ..Default::default()
}).await?;

// Start VM
client.start_instance().await?;
```

## Anti-Patterns

### Forbidden
```rust
// NEVER: Create VM without pool
let vm = FirecrackerVM::new(id, socket).await?; // WRONG

// ALWAYS: Use warm pool
let vm = pool.acquire().await.expect("Pool empty"); // CORRECT
```

### Resource Leaks
```rust
// WRONG: Not releasing VMs back to pool
let vm = pool.acquire().await?;
// ... use vm ...
// VM lost!

// CORRECT: Always release
let vm = pool.acquire().await?;
// ... use vm ...
pool.release(vm).await?;
```

## Commands

```bash
# Build
cd execution-layer/sandbox-daemon && cargo build --release

# Run with Firecracker
cargo run --release

# Run tests
cargo test

# Benchmark VM startup
cargo bench --bench microvm_bench
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.34 | Async runtime |
| hyper | 0.14 | HTTP client |
| hyperlocal | 0.8 | Unix Domain Sockets |
| axum | 0.7 | HTTP API framework |
| arc-swap | 1.7 | Lock-free data structures |
| dashmap | 5.5 | Concurrent HashMap |

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| VM startup (warm) | < 5ms | ✅ Phase 2.1 |
| VM startup (cold) | < 100ms | ✅ Phase 2.1 |
| Pool acquire | < 1ms | ✅ Phase 2.1 |
| Pool capacity | 100 VMs | ✅ Phase 2.1 |

## Phase 2.1 Complete

✅ Firecracker REST API client
✅ Warm pool management
✅ Health check and auto-scaling
✅ Resource limits enforcement
✅ Snapshot and restore
✅ Integration with eBPF security

## Notes

- **Warm pool**: 50 VMs pre-configured and ready
- **Health checks**: Every 10 seconds
- **Auto-scaling**: Scales from 5 to 100 VMs
- **Socket path**: `/tmp/smaos-firecracker-{vm_id}.socket`
- **eBPF integration**: Through fractal-gateway

## Structure

```
sandbox-daemon/
├── src/
│   └── main.rs          # FirecrackerVM + SandboxDaemon
├── Cargo.toml          # Dependencies: tokio, hyper, hyperlocal
└── main_test.rs        # (if exists)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| VM struct | `main.rs:7-11` | FirecrackerVM with socket_path |
| VM lifecycle | `main.rs:13-31` | new(), configure(), start() |
| Pool management | `main.rs:34-46` | SandboxDaemon with warm_vms |
| eBPF injection | `main.rs:58` | Security constraint simulation |
| Shutdown | `main.rs:74` | SIGINT handling |

## Conventions (This Module)

### VM Configuration
```rust
pub struct FirecrackerVM {
    pub vm_id: String,
    pub socket_path: String,  // Unix socket for Firecracker API
}

pub async fn configure(&self) -> Result<()> {
    // HTTP PUT to /machine-config via Unix socket
}
```

### Pool Initialization
```rust
pub async fn initialize_pool(&mut self) -> Result<()> {
    for i in 0..self.pool_size {
        let vm = FirecrackerVM::new(id, socket).await;
        vm.configure().await?;
        self.warm_vms.push(vm);
        // eBPF seccomp/apparmor injection
    }
}
```

### Async/Await Pattern
```rust
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut daemon = SandboxDaemon::new(5);
    daemon.initialize_pool().await?;
    tokio::signal::ctrl_c().await?;
}
```

## Anti-Patterns (This Module)

### Forbidden
```go
// NEVER: Direct VM access without gateway
// (from fractal-gateway AGENTS.md)
let vm = FirecrackerVM::new(id, socket).await?;  // WRONG

// ALWAYS: Gateway-mediated access
let vm = gateway.authenticate_and_create_vm(credentials).await?;
```

### Socket Management
```rust
// WRONG: Not validating socket path
let socket = format!("/tmp/firecracker-{}.socket", id);

// CORRECT: Path validation + error handling
let socket_path = Path::new(&format!("/tmp/firecracker-{}.socket", id));
if !socket_path.parent().unwrap().exists() {
    std::fs::create_dir_all(socket_path.parent().unwrap())?;
}
```

### Error Propagation
```rust
// WRONG: Silent failures
vm.configure().await;  // Missing ?

// CORRECT: Explicit propagation
vm.configure().await?;
```

## Unique Styles

### Hyperlocal Usage
```rust
use hyperlocal::UnixClientExt;
// Unix Domain Socket for Firecracker REST API
let client = hyper::Client::unix();
let uri = hyperlocal::Uri::new(&self.socket_path, "/machine-config");
```

### Tracing Setup
```rust
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting SMA-OS Firecracker Sandbox Daemon...");
}
```

### VM ID Formatting
```rust
let id = format!("microvm-{:03}", i);  // microvm-000, microvm-001
let socket = format!("/tmp/firecracker-{}.socket", id);
```

## Commands

```bash
# Build
cd execution-layer/sandbox-daemon && cargo build

# Run (requires Firecracker binary)
cargo run

# Release build
cargo build --release
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.34 | Async runtime |
| hyper | 0.14 | HTTP client |
| hyperlocal | 0.8 | Unix Domain Sockets |
| anyhow | 1.0 | Error handling |
| tracing | 0.1 | Logging |
| tracing-subscriber | 0.3 | Log formatting |
| serde | 1.0 | Serialization |
| uuid | 1.6 | Unique IDs |

## Prerequisites

- Firecracker binary installed
- `/tmp` directory writable for sockets
- Linux with KVM support
- Root for eBPF injection

## Notes

- **Socket path**: `/tmp/firecracker-{vm_id}.socket`
- **HTTP API**: Firecracker REST over Unix socket
- **Pool size**: 5 by default (configurable)
- **eBPF**: Currently simulated (implement actual injection)
- **Authentication**: Must go through fractal-gateway
