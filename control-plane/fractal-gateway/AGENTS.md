# Fractal Gateway Module Guide

**Location**: `control-plane/fractal-gateway/`  
**Domain**: eBPF-based resource isolation and authentication  
**Language**: Rust  
**Score**: 12/25 (distinct security domain)

## Overview

Security gateway implementing eBPF-based network filtering and resource authentication. Acts as the first line of defense for SMA-OS execution layer.

## Structure

```
fractal-gateway/
├── src/
│   ├── main.rs       # Gateway daemon entry point
│   └── lib.rs        # Auth utilities, eBPF loader
├── Cargo.toml       # Dependencies (aya, tokio, aya-log)
└── ../fractal-gateway-ebpf/  # Separate eBPF program (compiled via xtask)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| eBPF loading | `lib.rs` | Runtime loading of `fractal-gateway-ebpf` |
| Auth gateway | `main.rs` | Resource isolation, IAM policy enforcement |
| Network filtering | eBPF program | Intercept traffic to/from sandboxed VMs |

## Conventions (This Module)

### Security First
- **Zero trust**: All requests authenticated, even from internal services
- **Least privilege**: Grant minimal permissions per task
- **Audit trail**: Log all auth decisions with trace IDs

### eBPF Integration
```rust
// eBPF program loaded at runtime, not linked as library
// Use aya for safe eBPF management
let ebpf = aya::Ebpf::load()?;
let prog: &mut Kprobe = aya::programs::KProbe::try_from(ebpf.get_mut("kprobe"))?;
```

## Anti-Patterns (This Module)

### Forbidden
```rust
// NEVER bypass eBPF sandbox for "convenience"
// NEVER hardcode credentials or API keys
// NEVER skip auth checks even for internal requests

// WRONG: Direct VM access without gateway
let vm = FirecrackerVM::new(id, socket).await?;

// CORRECT: Gateway-mediated access
let vm = gateway.authenticate_and_create_vm(credentials).await?;
```

### Error Handling
```rust
// WRONG: Silent failures
if auth_result.is_err() {
    return;  // No logging, no audit trail
}

// CORRECT: Explicit logging
match auth_result {
    Ok(allowed) => Ok(allowed),
    Err(e) => {
        tracing::warn!("Auth failed: {}", e);
        Err(e)
    }
}
```

## Unique Styles

### Import Order
```rust
// 1. External crates (aya, tokio)
use aya::{Ebpf, programs::KProbe};
use tokio::sync::RwLock;

// 2. Standard library
use std::sync::Arc;

// 3. Internal modules
use crate::auth::Policy;
```

## Commands

```bash
# Build
cd control-plane/fractal-gateway && cargo build

# Run gateway
cargo run --bin fractal-gateway

# Lint
cargo clippy -- -D warnings

# Build eBPF program (separate)
cd ../fractal-gateway-ebpf && cargo xtask build-ebpf
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| aya | * | eBPF framework |
| aya-log | * | eBPF logging |
| tokio | 1.34 | Async runtime (rt, rt-multi-thread, net, signal) |
| env_logger | 0.10 | Logging |
| anyhow | 1.0 | Error handling |

## Notes

- **Workspace member**: Part of `control-plane` Cargo workspace
- **eBPF compilation**: Requires separate `xtask` build step
- **Runtime loading**: eBPF program loaded dynamically, not statically linked
- **Security boundary**: All execution layer traffic passes through this gateway
