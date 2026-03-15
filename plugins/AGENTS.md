# Plugin System Guide

**Location**: `plugins/`
**Domain**: Dynamic plugin architecture and marketplace
**Language**: Rust
**Score**: 16/25 (multi-crate workspace, distinct extension domain)

## Overview

Extensible plugin system supporting custom executors, middleware, and extensions. Provides foundation for dynamically loaded plugins with sandboxed execution and centralized marketplace for discovery and distribution.

## Structure

```
plugins/
├── core/
│   ├── src/
│   │   ├── lib.rs          # Plugin trait and core abstractions
│   │   ├── registry.rs       # Plugin registration and lifecycle
│   │   ├── loader.rs         # Dynamic loading (dlopen/libloading)
│   │   ├── executor.rs       # Plugin execution context
│   │   ├── manifest.rs       # Plugin metadata and capabilities
│   │   └── sandbox.rs        # Sandboxed execution environment
│   └── Cargo.toml          # async-trait, libloading, serde
├── marketplace/
│   ├── src/
│   │   ├── lib.rs          # Marketplace client API
│   │   ├── store.rs          # Local plugin storage
│   │   ├── registry_client.rs # Remote registry communication
│   │   ├── installer.rs      # Download and install logic
│   │   └── validator.rs      # Package integrity verification
│   └── Cargo.toml          # reqwest, sha2, tar
└── executors/              # Built-in executor implementations
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Plugin trait | `core/src/lib.rs:40-80` | `Plugin` async trait with init/shutdown/health |
| Registry | `core/src/registry.rs` | PluginRegistration, lifecycle management |
| Loader | `core/src/loader.rs` | Dynamic library loading with safety checks |
| Manifest | `core/src/manifest.rs` | PluginManifest, capabilities, metadata |
| Sandbox | `core/src/sandbox.rs` | Resource limits, syscall filtering |
| Store | `marketplace/src/store.rs` | Local package management |
| Installer | `marketplace/src/installer.rs` | Download, verify, install flow |
| Validator | `marketplace/src/validator.rs` | SHA256, signature verification |

## Conventions (This Module)

### Plugin Trait Implementation
```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    async fn init(&mut self, config: PluginConfig) -> Result<(), PluginError>;
    async fn shutdown(&mut self) -> Result<(), PluginError>;
    async fn health(&self) -> PluginHealth;
    fn capabilities(&self) -> Vec<PluginCapability>;
}
```

### Dynamic Loading Safety
- **Version compatibility**: Check ABI version before loading
- **Symbol verification**: Validate required exports exist
- **Sandbox first**: All plugins run in restricted environment

### Manifest Schema
```rust
pub struct PluginManifest {
    pub name: String,           // MAX 100 chars
    pub version: String,        // SemVer
    pub author: String,
    pub capabilities: Vec<PluginCapability>,
    pub permissions: Vec<Permission>,
    pub entry_point: String,    // Dynamic lib path
}
```

## Anti-Patterns (This Module)

### Forbidden
```rust
// NEVER: Load unsigned plugins
let plugin = loader.load(path)?; // WRONG: No signature check

// ALWAYS: Verify before loading
validator.verify_package(&path, &signature)?;
let plugin = loader.load(path)?;
```

### Symbol Safety
```rust
// WRONG: Blind symbol resolution
let func: Symbol<fn()> = lib.get(b"init")?;

// CORRECT: Type-safe with validation
let init: Symbol<PluginInitFn> = lib.get(b"sma_plugin_init")?;
assert!(registry.validate_symbol(&init));
```

### Resource Limits
```rust
// WRONG: Unlimited plugin execution
plugin.execute(input).await?;

// CORRECT: Enforce sandbox limits
sandbox.with_limits(|| async {
    plugin.execute(input).await
}).await?;
```

## Unique Styles

### Async Plugin Methods
```rust
#[async_trait]
impl Plugin for MyExecutor {
    async fn init(&mut self, config: PluginConfig) -> Result<(), PluginError> {
        // Async initialization
    }
}
```

### Capability Declaration
```rust
fn capabilities(&self) -> Vec<PluginCapability> {
    vec![
        PluginCapability::Executor,
        PluginCapability::Middleware,
    ]
}
```

## Commands

```bash
# Build all plugins
cd plugins/core && cargo build
cd plugins/marketplace && cargo build

# Test
cargo test --workspace

# Run with example plugin
cargo run --example hello_plugin
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| async-trait | Async trait support |
| libloading | Dynamic library loading |
| serde | Manifest serialization |
| sha2 | Package verification |
| tar | Archive extraction |
| reqwest | Registry HTTP client |

## Security Notes

- **Never load plugins from untrusted sources without verification**
- **All plugins run in sandbox with restricted syscalls**
- **Memory isolation via cgroups (Linux) or job objects (Windows)**
- **Network access requires explicit capability grant**
