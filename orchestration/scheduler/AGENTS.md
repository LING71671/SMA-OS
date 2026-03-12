# Worker Scheduler Module Guide

**Location**: `orchestration/scheduler/`  
**Domain**: Firecracker MicroVM warm pool and task assignment  
**Language**: Go  
**Score**: 12/25 (supporting service, distinct scheduling logic)

## Overview

Manages the warm pool of Firecracker MicroVMs and handles task-to-worker assignment with affinity awareness. Implements resident vs transient worker types for different workload patterns.

## Structure

```
scheduler/
├── main.go              # FractalClusterScheduler with warm pool
├── go.mod              # Uses sma-os/orchestration module
└── main_test.go        # (if exists)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Warm pool init | `main.go:45-53` | Pre-warms Firecracker instances |
| Task assignment | `main.go:56-76` | Affinity-aware scheduling |
| Worker types | `main.go:14-20` | RESIDENT vs TRANSIENT |
| Pool metrics | `main.go:105` | Periodic maintenance logging |

## Conventions (This Module)

### Worker Classification
```go
type WorkerType string
const (
    WorkerTypeResident  WorkerType = "RESIDENT"   // Long-running
    WorkerTypeTransient WorkerType = "TRANSIENT"  // Ephemeral
)
```

### Affinity Scheduling
```go
// 1. Try affinity match first
if previousHost != "" {
    // Search for available worker on same host
}

// 2. Fallback to warm pool pull
assignedID := "microvm-pool-" + string(rune(rand.Intn(100)))
```

### RWMutex Pattern
```go
type FractalClusterScheduler struct {
    mu sync.RWMutex  // Multiple readers, single writer
    // ...
}
```

## Anti-Patterns (This Module)

### Forbidden
```go
// NEVER: Assign without checking availability
w.Available = false  // WRONG: Without verifying current state

// ALWAYS: Check and atomically update
if w.Available && w.NodeHost == previousHost {
    w.Available = false
}
```

### Random Assignment
```go
// WRONG: Non-deterministic worker selection
assignedID := string(rune(rand.Intn(100)))  // Limited range

// CORRECT: Use proper random with bounds checking
assignedID := fmt.Sprintf("microvm-%04d", rand.Intn(10000))
```

### Context Cancellation
```go
// WRONG: No graceful shutdown
for { /* infinite loop */ }

// CORRECT: Signal handling with context
ctx, cancel := context.WithCancel(context.Background())
signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
```

## Unique Styles

### Pool Sizing
```go
// Configurable warm pool size
func NewScheduler(warmPool int) *FractalClusterScheduler {
    s := &FractalClusterScheduler{WarmPoolSize: warmPool}
    s.initWarmPool()
    return s
}
```

### Graceful Shutdown
```go
// Standard pattern across Go services
quit := make(chan os.Signal, 1)
signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
ctx, cancel := context.WithCancel(context.Background())
```

## Commands

```bash
# Build
cd orchestration/scheduler && go build -o bin/scheduler .

# Run
go run main.go
```

## Dependencies

| Package | Purpose |
|---------|---------|
| context | Cancellation propagation |
| os/signal | SIGINT/SIGTERM handling |
| sync | RWMutex for thread-safe access |
| math/rand | Worker selection |

## Notes

- **Warm pool**: Pre-initializes VMs for <5ms startup
- **Affinity**: Tries to schedule on same host as previous execution
- **Mock mode**: Currently simulates VM lifecycle
- **Pool size**: 50 by default, configurable
