# Chaos Tests Module Guide

**Location**: `chaos-tests/`
**Domain**: Chaos engineering test framework for system resilience
**Language**: Rust
**Score**: 15/25 (complex testing framework, distinct domain)

## Overview

Automated chaos engineering framework for validating SMA-OS resilience and fault tolerance. Injects failures (node crashes, network partitions, resource exhaustion) and verifies automatic recovery.

## Structure

```
chaos-tests/
├── src/
│   ├── main.rs          # CLI entry point with clap
│   ├── framework.rs     # Core framework abstractions
│   ├── reporters/       # Test result reporting
│   └── scenarios/       # Failure scenarios (node-failure, network-partition, resource-exhaustion)
├── framework/          # Framework implementations
├── scenarios/          # Scenario configurations (YAML)
├── configs/           # Test configurations
├── scripts/           # CI/CD automation scripts
├── Cargo.toml         # Dependencies (bollard, clap, tokio)
└── README.md          # Detailed usage docs
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| CLI args | `main.rs` | clap derive macros for --scenario, --dry-run |
| Scenario runner | `framework.rs` | Abstract runner with health checks |
| Docker integration | `framework/` | Uses bollard for container management |
| Config loading | `scenarios/` | YAML scenario definitions |
| Report generation | `reporters/` | JSON/text output formats |

## Conventions (This Module)

### Scenario Definition
```rust
#[derive(Debug, Clone)]
struct Scenario {
    name: String,
    duration_secs: u64,
    failure_probability: f64,
    targets: Vec<String>,
}
```

### Dry-Run Mode
```rust
if dry_run {
    log::info!("[DryRun] Would kill container {}", container_id);
    return Ok(()); // No actual failure
}
```

### Health Check Pattern
```rust
async fn wait_for_recovery(&self, service: &str, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if self.check_health(service).await? {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(ChaosError::RecoveryTimeout)
}
```

## Anti-Patterns (This Module)

### Forbidden
```rust
// NEVER: Run chaos tests without dry-run first
chaos.run_scenario(scenario)?; // WRONG: May cause real damage

// ALWAYS: Dry run first
chaos.dry_run(scenario)?;
chaos.run_scenario(scenario)?; // CORRECT
```

### Error Propagation
```rust
// WRONG: Silent failure
if let Err(_) = chaos.run().await { return; }

// CORRECT: Log with context
match chaos.run().await {
    Ok(_) => log::info!("Scenario passed"),
    Err(e) => {
        log::error!("Scenario failed: {}", e);
        std::process::exit(1);
    }
}
```

### Resource Cleanup
```rust
// WRONG: No cleanup on panic
chaos.inject_failure().await?;
// Test ends here, resources leaked

// CORRECT: Always cleanup (use Drop or defer pattern)
struct ChaosRun { ... }
impl Drop for ChaosRun {
    fn drop(&mut self) { self.cleanup(); }
}
```

## Commands

```bash
# Build
cd chaos-tests && cargo build --release

# Run specific scenario
cargo run --release -- --scenario node-failure

# Dry run (no actual failures)
cargo run --release -- --scenario all --dry-run

# Run all scenarios
cargo run --release -- --scenario all

# JSON output
cargo run --release -- --scenario all --output json

# Lint
cargo clippy -- -D warnings
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| clap | CLI argument parsing |
| bollard | Docker API client |
| tokio | Async runtime |
| reqwest | Health check HTTP |
| rand | Random failure selection |
| serde_yaml | Config parsing |

## Notes

- **Requires Docker**: Needs docker.sock access
- **Root may be needed**: For network partition (tc command)
- **Isolation critical**: Never run in production without staging validation
- **Cleanup mandatory**: Always runs cleanup even on panic
- **Timeout handling**: Every scenario must have recovery timeout
- **Target services**: state-engine, fractal-gateway, sandbox-daemon
