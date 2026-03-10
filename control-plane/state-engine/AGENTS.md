# State Engine Module Guide

**Location**: `control-plane/state-engine/`  
**Domain**: Event sourcing state kernel with Redis + PostgreSQL persistence  
**Language**: Rust  
**Score**: 18/25 (high complexity, distinct domain)

## Overview

Core state management layer implementing event sourcing pattern with hot-cold storage separation. Handles all state transitions for SMA-OS agents with mathematical guarantees.

## Structure

```
state-engine/
├── src/
│   ├── engine.rs      # StateEngine impl (append_event, get_events, snapshots)
│   ├── models.rs     # StateEvent, Snapshot structs
│   ├── lib.rs        # Public exports
│   └── main.rs       # Binary entry point
├── migrations/        # SQLx database migrations
│   └── 20231010000000_create_hot_events.sql
├── Cargo.toml        # Dependencies (tokio, redis, sqlx, thiserror)
└── src/bin/          # Test binaries (e.g., test_query.rs)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Append events | `engine.rs:42-68` | Redis write → PostgreSQL persist → snapshot trigger |
| Query events | `engine.rs:100-196` | Redis cache → PostgreSQL fallback → cache refill |
| Snapshot generation | `engine.rs:71-94` | Every 1000 events, compress to snapshots table |
| Error types | `engine.rs:6-14` | `EngineError` enum (Redis, Postgres, Serialization) |
| Data models | `models.rs` | `StateEvent`, `Snapshot` with serde |

## Conventions (This Module)

### Error Handling
- **Always** use `Result<T, EngineError>` - never `.unwrap()` or `.expect()`
- Use `thiserror` for automatic trait implementations
- Propagate with `?` operator, handle at boundaries

### Cache Strategy
- **Redis-first**: Always try Redis before PostgreSQL
- **Selective refill**: Only refill cache for full-range queries (`to_version.is_none()`)
- **Batch operations**: Use Redis pipeline for multiple writes
- **TTL**: 24 hours (`REDIS_CACHE_TTL_SECS = 86400`)

### Event Sourcing
- **Append-only**: Events are immutable, versioned
- **Conflict resolution**: `ON CONFLICT (tenant_id, namespace, version) DO NOTHING`
- **Snapshot trigger**: Every 1000 events, auto-generate snapshot

## Anti-Patterns (This Module)

### Forbidden
```rust
// NEVER do this:
let event = events[0];  // No bounds check
let conn = redis_client.get_async_connection().await.unwrap();  // No error handling

// ALWAYS do this:
let event = events.first().ok_or(EngineError::Empty)?;
let mut conn = redis_client.get_async_connection().await?;
```

### Cache Invalidation
```rust
// WRONG: Deletes entire cache for partial queries
if !events.is_empty() {
    conn.del(&redis_key).await?;  // Deletes ALL cached events
    // ...
}

// CORRECT: Only refill for full-range queries
if !events.is_empty() && to_version.is_none() {
    let mut pipe = redis::pipe();
    pipe.del(&redis_key);
    // ... batch refill
}
```

### JSON Parsing
```rust
// WRONG: Fails on corrupted cache
let event: StateEvent = serde_json::from_str(&json)?;

// CORRECT: Skip corrupted entries
let events: Vec<StateEvent> = event_strings
    .iter()
    .filter_map(|s| serde_json::from_str::<StateEvent>(s).ok())
    .collect();
```

## Unique Styles

### Import Order
```rust
// 1. Module imports
use crate::models::{Snapshot, StateEvent};

// 2. External crates
use redis::AsyncCommands;
use sqlx::Row;
use thiserror::Error;

// 3. Standard library (implicit)
```

### SQL Queries
```rust
// Use sqlx::query! with explicit type handling
let rows = sqlx::query(sql)
    .bind(tenant_id)
    .bind(namespace)
    .bind(version as i64)
    .fetch_all(&self.pg_pool)
    .await?;

// Extract with explicit type annotation
let version = row.get::<i64, _>("version") as u64;
```

### Redis Pipeline
```rust
let mut pipe = redis::pipe();
pipe.del(&redis_key);
for event in &events {
    let event_json = serde_json::to_string(event)?;
    pipe.zadd(&redis_key, event.version as f64, event_json);
}
pipe.expire(&redis_key, REDIS_CACHE_TTL_SECS)
    .ignore()
    .query_async::<_, ()>(&mut conn)
    .await?;
```

## Commands

```bash
# Build
cd control-plane/state-engine && cargo build

# Test
cargo test -- --nocapture

# Run binary
cargo run --bin state-engine

# Run test binary (requires Redis + PostgreSQL)
cargo run --bin test_query

# Lint
cargo clippy --all-targets --all-features -- -D warnings
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.34 | Async runtime |
| redis | 0.23 | Redis client with tokio-comp |
| sqlx | 0.7 | PostgreSQL with runtime-tokio-native-tls |
| thiserror | 1.0 | Error handling |
| serde | 1.0 | Serialization |
| uuid | 1.6 | Unique identifiers |
| tracing | 0.1 | Logging |
| chrono | 0.4 | Timestamps |

## Notes

- **Workspace member**: Part of `control-plane` Cargo workspace
- **Migration tooling**: Uses `sqlx::migrate!` macro for database migrations
- **Test isolation**: Tests should use `testcontainers` for isolated DB instances
- **Performance target**: Redis queries < 1ms, PostgreSQL < 10ms, snapshot generation < 100ms
