# Teardown Controller Module Guide

**Location**: `control-plane/teardown-ctrl/`
**Domain**: Cascading cleanup controller for graceful resource termination
**Language**: Rust
**Score**: 10/25 (infrastructure service, Kubernetes integration)

## Overview

Manages graceful teardown and garbage collection of SMA-OS resources. Ensures clean termination of MicroVMs, namespaces, and cgroup resources when tasks complete or security breaches occur.

## Structure

```
teardown-ctrl/
├── src/
│   ├── main.rs       # Daemon entry point
│   └── controller.rs   # CascadingTeardownController implementation
├── Cargo.toml        # Dependencies (kube, k8s-openapi)
└── main_test.rs      # (if exists)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Controller init | `controller.rs` | K8s client initialization |
| Teardown execution | `controller.rs` | ExecuteTeardown method |
| Target definition | `controller.rs` | TeardownTarget struct |
| Force mode | `controller.rs` | SIGKILL vs graceful SIGTERM |
| Signal handling | `main.rs:27` | Graceful shutdown |

## Conventions (This Module)

### Target Definition
```rust
pub struct TeardownTarget {
    pub tenant_id: String,
    pub namespace: String,
    pub task_group_id: Uuid,
    pub force: bool,  // true = SIGKILL, false = SIGTERM
}
```

### Controller Pattern
```rust
pub struct CascadingTeardownController {
    k8s_client: Client,  // kube-rs client
}

impl CascadingTeardownController {
    pub async fn new() -> Result<Self> {
        let config = Config::infer().await?;
        let client = Client::try_from(config)?;
        Ok(Self { k8s_client: client })
    }
}
```

### Graceful Degradation
```rust
pub async fn execute_teardown(&self, target: TeardownTarget) -> Result<()> {
    // 1. Stop new task scheduling
    // 2. Drain running tasks
    // 3. Delete namespace
    // 4. Clean up cgroups
    // 5. Archive logs
}
```

## Anti-Patterns (This Module)

### Forbidden
```rust
// NEVER: Force kill without attempting graceful shutdown
if target.force {
    self.kill_immediately(&target)?;  // WRONG: Skip graceful
}

// ALWAYS: Try graceful first
if !target.force {
    match self.graceful_shutdown(&target).await {
        Ok(_) => return Ok(()),
        Err(_) => self.force_kill(&target).await?,
    }
}
```

### Resource Leaks
```rust
// WRONG: Not cleaning up on error
self.delete_pods(&target).await?;
// If namespace delete fails, pods leaked

// CORRECT: Cleanup in reverse dependency order
self.stop_scheduling(&target).await?;
self.drain_tasks(&target).await?;
self.delete_pods(&target).await?;
self.delete_namespace(&target).await?;
self.cleanup_cgroups(&target).await?;
```

## Commands

```bash
# Build
cd control-plane/teardown-ctrl && cargo build --release

# Run (requires K8s cluster)
cargo run --release

# Test with dummy target
cargo test

# Lint
cargo clippy -- -D warnings
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| kube | Kubernetes client |
| k8s-openapi | K8s API types |
| tokio | Async runtime |
| uuid | Task group IDs |
| tracing | Structured logging |

## Notes

- **K8s required**: Needs active cluster or kubeconfig
- **Force mode**: SIGKILL immediate termination
- **Graceful mode**: SIGTERM with drain timeout
- **Cascading**: Cleans up dependencies in order
- **Currently stubbed**: Controller logic commented out

## Teardown Sequence

```
1. Stop new scheduling
2. Drain running tasks (graceful only)
3. Delete pods
4. Delete namespace
5. Clean up cgroups
6. Archive logs
```

## Target Types

| Field | Type | Description |
|-------|------|-------------|
| tenant_id | String | Tenant identifier |
| namespace | String | K8s namespace |
| task_group_id | Uuid | Task group to terminate |
| force | bool | Immediate vs graceful |

## Integration

- **Triggered by**: Evaluator (security breach), Manager (task completion)
- **Calls**: sandbox-daemon (VM teardown), fractal-gateway (cleanup rules)
- **Monitored by**: Observability UI
