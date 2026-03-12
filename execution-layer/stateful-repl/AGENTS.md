# Stateful REPL Module Guide

**Location**: `execution-layer/stateful-repl/`  
**Domain**: Persistent terminal sessions with eBPF protection  
**Language**: Rust  
**Score**: 8/25 (supporting service, simpler domain)

## Overview

Provides persistent terminal/REPL sessions bound directly to MicroVM network namespaces. Implements secure terminal proxy with eBPF side-channel protection for memory isolation.

## Structure

```
stateful-repl/
├── src/
│   └── main.rs          # TCP listener + connection handler
├── Cargo.toml          # Dependencies: tokio, anyhow, tracing
└── main_test.rs        # (if exists)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| TCP listener | `main.rs:12-14` | Binds to random local port |
| Connection handler | `main.rs:32-52` | Accepts and handles connections |
| Signal handling | `main.rs:17-26` | tokio::select! for graceful shutdown |
| eBPF reference | `main.rs:15` | Side-channel protection mentioned |

## Conventions (This Module)

### Async TCP Server
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    info!("[REPL] Secure Terminal Proxy listening on port {}", port);
}
```

### Connection Spawning
```rust
async fn accept_connections(listener: TcpListener) -> Result<()> {
    loop {
        let (mut socket, peer) = listener.accept().await?;
        tokio::spawn(async move {
            // Handle connection in spawned task
        });
    }
}
```

### Graceful Shutdown
```rust
tokio::select! {
    _ = tokio::signal::ctrl_c() => {
        info!("Received termination signal.");
    }
    res = accept_connections(listener) => {
        if let Err(e) = res { warn!("Connection failed: {}", e); }
    }
}
```

## Anti-Patterns (This Module)

### Forbidden
```rust
// NEVER: Blocking operations in async context
socket.read(&mut buf).await?;  // OK - async
socket.read(&mut buf)?;        // WRONG - blocking

// NEVER: Not handling EOF
match socket.read(&mut buf).await {
    Ok(0) => break,  // Always check for EOF
    Ok(n) => { /* process */ }
    Err(_) => break,
}
```

### Error Handling
```rust
// WRONG: Ignoring write errors
let _ = socket.write_all(&buf[..n]).await;

// CORRECT: Check result (even if echo is non-critical)
if let Err(e) = socket.write_all(&buf[..n]).await {
    warn!("Write failed: {}", e);
}
```

### Buffer Management
```rust
// WRONG: Unbounded buffer
let mut buf = vec![0; 1024 * 1024];  // Too large

// CORRECT: Reasonable fixed buffer
let mut buf = [0; 1024];  // Stack allocated, fixed size
```

## Unique Styles

### tokio::select! Pattern
```rust
tokio::select! {
    _ = tokio::signal::ctrl_c() => { /* shutdown */ }
    res = accept_connections(listener) => { /* handle result */ }
}
```

### Connection Per-Task
```rust
tokio::spawn(async move {
    let mut buf = [0; 1024];
    loop {
        match socket.read(&mut buf).await {
            // Echo logic
        }
    }
    info!("Connection {} closed.", peer);
});
```

### Security Logging
```rust
info!("[REPL] eBPF side-channel protection mechanism strictly enforcing memory boundaries.");
```

## Commands

```bash
# Build
cd execution-layer/stateful-repl && cargo build

# Run
cargo run

# Test connection
telnet localhost <port>  # Port logged on startup
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.34 | Async runtime + TCP |
| anyhow | 1.0 | Error handling |
| tracing | 0.1 | Logging |
| tracing-subscriber | 0.3 | Log formatting |

## Notes

- **Port allocation**: Binds to port 0 (random available)
- **Port logging**: Reports actual port on startup
- **Echo mode**: Currently echoes back received data
- **Buffer size**: 1024 bytes per connection
- **MicroVM binding**: Should integrate with namespace management
- **eBPF protection**: Mentioned but not implemented (see fractal-gateway)
