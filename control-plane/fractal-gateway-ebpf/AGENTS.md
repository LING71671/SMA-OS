# Fractal Gateway eBPF Module Guide

**Location**: `control-plane/fractal-gateway-ebpf/`
**Domain**: Nanosecond-level XDP packet filtering using eBPF
**Language**: Rust (eBPF)
**Score**: 16/25 (low-level kernel programming, security-critical)

## Overview

Kernel-level network filtering using eBPF XDP (eXpress Data Path). Provides O(1) packet filtering by dropping malicious traffic before it reaches userspace. Powers the SMA-OS security boundary.

## Structure

```
fractal-gateway-ebpf/
├── src/
│   ├── main.rs        # XDP program loader
│   ├── lib.rs         # eBPF program definitions and user API
│   ├── quota.rs       # Resource quota enforcement (LSM hooks)
│   ├── cgroup.rs      # Cgroup resource tracking
│   ├── collector.rs   # Metrics collection
│   └── aggregator.rs  # Packet statistics aggregation
├── build.rs           # eBPF build script (uses aya-build)
├── Cargo.toml         # Dependencies (aya-ebpf, network-types)
├── README.md          # Detailed usage
└── USAGE.md           # API examples
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| XDP filter | `lib.rs` | Packet drop logic at line rate |
| IP blocking | `lib.rs` | BLOCKED_IPS map operations |
| Quota enforcement | `quota.rs` | LSM hooks for resource limits |
| Cgroup tracking | `cgroup.rs` | Per-cgroup resource accounting |
| Metrics | `aggregator.rs` | Prometheus-compatible metrics |
| Build | `build.rs` | eBPF compilation via aya-build |

## Conventions (This Module)

### eBPF Program Structure
```rust
#[aya_ebpf::macros::xdp]
pub fn fractal_gateway(ctx: XdpContext) -> u32 {
    match try_fractal_gateway(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp::XDP_ABORTED,
    }
}
```

### Map Definition
```rust
#[aya_ebpf::macros::map]
static BLOCKED_IPS: HashMap<u32, u8> = HashMap::with_max_entries(1024, 0);
```

### Safe Wrapper Pattern
```rust
// lib.rs - userspace wrapper
pub struct FractalGatewayEbpf {
    bpf: Ebpf,
}

impl FractalGatewayEbpf {
    pub fn load() -> Result<Self> {
        let bpf = Ebpf::load()?;
        Ok(Self { bpf })
    }
}
```

## Anti-Patterns (This Module)

### Forbidden
```rust
// NEVER: Unbounded maps (kernel verifier rejects)
#[map]
static BLOCKED_IPS: HashMap<u32, u8> = HashMap::with_max_entries(1000000, 0); // WRONG

// ALWAYS: Reasonable limits
static BLOCKED_IPS: HashMap<u32, u8> = HashMap::with_max_entries(1024, 0); // CORRECT
```

### Unsafe in eBPF
```rust
// WRONG: Direct pointer access (verifier may reject)
let ip = unsafe { *(ctx.data() as *const u32) };

// CORRECT: Use aya helpers
let ip = u32::from_be(ctx.load::<u32>(offset)?);
```

### Error in XDP
```rust
// WRONG: Panic in eBPF (kills kernel path)
panic!("invalid packet"); // NEVER

// CORRECT: Return XDP action
if packet_invalid {
    return xdp::XDP_DROP;
}
```

## Commands

```bash
# Build eBPF (requires xtask)
cargo xtask build-ebpf --release

# Build userspace loader
cargo build --release

# Run (requires root)
sudo ./target/release/fractal-gateway-ebpf --interface eth0

# Block IP
cargo run --release -- block-ip 192.168.1.100

# Run tests (requires root for integration)
cargo test
sudo cargo test --test integration

# Lint (includes eBPF-specific rules)
cargo clippy -- -D warnings
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| aya-ebpf | eBPF runtime and macros |
| aya-log-ebpf | eBPF logging |
| network-types | Network protocol types |
| aya-build | Build script integration |
| prometheus-client | Metrics export |

## Notes

- **Kernel 4.19+**: Requires eBPF/XDP support
- **Root required**: Loading eBPF needs CAP_BPF or root
- **Interface dependency**: Must attach to physical interface
- **Verifier limits**: eBPF code must pass kernel verifier
- **Map pinning**: Maps persist across reloads
- **Performance**: <100ns per packet overhead
- **Security**: All blocked IPs logged for audit

## Architecture

```
User Space (Rust + aya)
    ↓
eBPF Runtime (kernel)
    ↓
XDP Program (fractal_gateway)
    ↓
Network Interface (eth0)
```

## XDP Actions

| Action | Description |
|--------|-------------|
| XDP_DROP | Drop packet (blocked IP) |
| XDP_PASS | Allow to userspace |
| XDP_TX | Bounce back to interface |
| XDP_REDIRECT | Redirect to another interface |
| XDP_ABORTED | Error, drop and log |
