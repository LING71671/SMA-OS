# DAG Manager Module Guide

**Location**: `orchestration/manager/`  
**Domain**: Topological task execution and cognitive DAG orchestration  
**Language**: Go  
**Score**: 18/25 (core orchestration logic, distinct domain)

## Overview

Core orchestration layer implementing topological sorting for concurrent task execution. Manages the lifecycle of a cognitive execution graph (DAG) with concurrent worker dispatch and dependency resolution.

## Structure

```
manager/
├── main.go              # DAGManager implementation with Execute() method
├── go.mod              # Uses sma-os/orchestration module
└── main_test.go        # (if exists)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| DAG execution | `main.go:59-127` | Topological sort with channels and goroutines |
| Task registration | `main.go:44-56` | AddTask with in-degree computation |
| Worker dispatch | `main.go:129-142` | Simulated Firecracker warm pool assignment |
| Task status | `main.go:10-18` | PENDING → RUNNING → COMPLETED |

## Conventions (This Module)

### Concurrent Execution Pattern
```go
// Channel-based coordination
readyQueue := make(chan *TaskNode, len(dm.Nodes))
completionChan := make(chan string, len(dm.Nodes))
```

### Dependency Resolution
```go
// Decrement in-degree on completion
dm.inDegree[id]--
if dm.inDegree[id] == 0 {
    readyQueue <- node  // Ready for execution
}
```

### Goroutine Management
```go
// Always use WaitGroup for coordination
var wg sync.WaitGroup
defer wg.Done()  // In worker goroutine
wg.Wait()        // At DAG completion
```

## Anti-Patterns (This Module)

### Forbidden
```go
// NEVER: Direct state mutation without mutex
dm.inDegree[id]--  // WRONG: No lock

// ALWAYS: Use mutex for shared state
dm.mu.Lock()
dm.inDegree[id]--
dm.mu.Unlock()
```

### Channel Safety
```go
// WRONG: Channel could block on full
readyQueue <- task  // Blocks if channel full

// CORRECT: Buffered channel sized to total tasks
readyQueue := make(chan *TaskNode, len(dm.Nodes))
```

### Loop Termination
```go
// WRONG: Infinite loop without exit condition
for { /* never exits */ }

// CORRECT: Check completion count
if completedTasks == totalTasks {
    return
}
```

## Unique Styles

### Task Status Management
```go
// Centralized status transitions
type TaskStatus string
const (
    Pending    TaskStatus = "PENDING"
    Running    TaskStatus = "RUNNING"
    Completed  TaskStatus = "COMPLETED"
    Failed     TaskStatus = "FAILED"
)
```

### Logging Convention
```go
// Always prefix with component
log.Println("[Manager] Starting DAG execution...")
log.Printf("[Worker Scheduler] -> Dispatching Task [%s]...", task.ID)
log.Printf("[Worker Scheduler] <- Task [%s] completed", task.ID)
```

## Commands

```bash
# Build
cd orchestration/manager && go build -o bin/manager .

# Run
go run main.go

# Test (if test file exists)
go test -v
```

## Dependencies

| Package | Purpose |
|---------|---------|
| encoding/json | Task serialization |
| sync | WaitGroup and Mutex |
| log | Structured logging |

## Notes

- **Concurrency model**: Channels + goroutines + WaitGroup
- **Worker simulation**: Currently sleeps 500ms, should connect to Firecracker pool
- **In-memory only**: No persistence (relies on state-engine for that)
- **Demo mode**: Includes sample DAG JSON in main()
