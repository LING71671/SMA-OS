# Vector-KV Store Module Guide

**Location**: `memory-bus/vector-kv/`  
**Domain**: Hybrid vector + key-value storage with compression  
**Language**: Go  
**Score**: 10/25 (storage layer, distinct data domain)

## Overview

Hybrid storage layer managing vector embeddings and structured key-value data. Handles async context compression (10:1 ratio) and provides low-latency queries bypassing LLM layer.

## Structure

```
vector-kv/
├── main.go              # HybridDBManagerProxy with compaction
├── go.mod              # Uses sma-os/memory-bus module
└── main_test.go        # (if exists)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Context compression | `main.go:22-27` | Async HNSW clustering |
| Direct reads | `main.go:29-33` | Low-latency bypass |
| Compression cycle | `main.go:57-58` | 15s ticker (6hr in prod) |
| Storage ratio | `main.go:26` | 10:1 compression target |

## Conventions (This Module)

### Hybrid Storage Pattern
```go
type HybridDBManagerProxy struct {
    // Backend: FoundationDB + Weaviate + Redis hot cache
    // Currently stubbed - implement actual connections
}
```

### Async Compression
```go
// Triggered periodically, non-blocking
func (m *HybridDBManagerProxy) CompactContexts() {
    log.Println("[VectorKV] Triggering Async HNSW Clustering...")
    // FDB → embed → Weaviate → ClickHouse archive
}
```

### Low-Latency Read
```go
// Returns JSON directly, bypasses LLM
func (m *HybridDBManagerProxy) ReadWithCache(tenantID, version string) string {
    // Completely bypasses LLM
    return `{"cached_payload": "true", "latency": "<1ms"}`
}
```

## Anti-Patterns (This Module)

### Forbidden
```go
// NEVER: Block on compression in read path
// WRONG: Calling CompactContexts() inside ReadWithCache()

// ALWAYS: Async background compression
generic <-ticker.C:
    manager.CompactContexts()  // Background only
```

### Synchronous Operations
```go
// WRONG: Blocking main thread
CompactContexts()  // May take minutes

// CORRECT: Goroutine for heavy operations
go manager.CompactContexts()
```

## Unique Styles

### Ticker-Based Maintenance
```go
// Production: 6 hours
// Demo: 15 seconds
ticker := time.NewTicker(15 * time.Second)
defer ticker.Stop()

for {
    select {
    case <-ticker.C:
        manager.CompactContexts()
    }
}
```

### Backend Abstraction
```go
// Stub for actual DB connections
type HybridDBManagerProxy struct {
    // Future: FDB connection
    // Future: Weaviate client
    // Future: Redis pool
}
```

## Commands

```bash
# Build
cd memory-bus/vector-kv && go build -o bin/vector-kv .

# Run
go run main.go
```

## Dependencies

| Package | Purpose |
|---------|---------|
| context | Cancellation handling |
| time | Ticker for maintenance |
| os/signal | Graceful shutdown |

## Notes

- **Currently stubbed**: No actual DB connections
- **Intended backends**: FoundationDB, Weaviate, ClickHouse
- **Compression**: HNSW clustering + context merging
- **Cache**: Redis hot cache for frequent queries
- **Read latency**: Target <1ms (bypasses LLM)
