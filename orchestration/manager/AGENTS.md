# DAG Manager Module Guide

**Location**: `orchestration/manager/`
**Domain**: Topological task execution, cognitive DAG orchestration, task management, and dependency analysis
**Language**: Go
**Coverage**: 87.2%
**Score**: 24/25 (comprehensive orchestration with task management and analysis)

## Overview

Core orchestration layer implementing topological sorting for concurrent task execution. Manages the lifecycle of a cognitive execution graph (DAG) with concurrent worker dispatch, dependency resolution, task management (pause/resume/progress), and comprehensive dependency analysis.

## Structure

```
manager/
├── main.go                  # DAGManager core (TaskStatus, TaskNode, Execute)
├── api.go                   # HTTP handlers (progress, pause, resume, analysis)
├── checkpoint.go            # TaskCheckpoint serialization/restore
├── progress.go              # Progress calculation and aggregation
├── pause_resume.go          # PauseTask/ResumeTask with cancel registry
├── dependency_analysis.go   # Cycle detection, critical path, parallelism
├── main_test.go             # Core DAG tests
├── api_test.go              # API handler tests
├── checkpoint_test.go       # Checkpoint tests
├── progress_test.go         # Progress tests
├── pause_resume_test.go     # Pause/resume tests
├── dependency_analysis_test.go # Analysis tests
└── go.mod                   # Module dependencies
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| DAG execution | `main.go:99-185` | Topological sort with channels and goroutines |
| Task registration | `main.go:85-100` | AddTask with in-degree computation |
| Worker dispatch | `main.go:129-142` | Simulated Firecracker warm pool assignment |
| **TaskStatus enum** | `main.go:42-51` | PENDING, RUNNING, PAUSED, RESUMED, COMPLETED, FAILED |
| **TaskNode extensions** | `main.go:54-65` | ParentID, SubTasks, Progress, IsAtomic |
| **Progress calculation** | `progress.go` | GetProgress, subtask aggregation |
| **Pause/Resume** | `pause_resume.go` | Checkpoint save/restore, cancel registry |
| **Cycle detection** | `dependency_analysis.go:44-90` | DFS algorithm |
| **Critical path** | `dependency_analysis.go:92-150` | Dynamic programming |
| **Parallelism** | `dependency_analysis.go:152-200` | Layer calculation |
| **Impact analysis** | `dependency_analysis.go:202-250` | Recursive dependency traversal |
| **API handlers** | `api.go` | REST endpoints for all features |

## New Types

### TaskStatus (Extended)
```go
// main.go:42-51
const (
    Pending   TaskStatus = "PENDING"
    Running   TaskStatus = "RUNNING"
    Paused    TaskStatus = "PAUSED"   // NEW in v1.2.0
    Resumed   TaskStatus = "RESUMED"  // NEW in v1.2.0
    Completed TaskStatus = "COMPLETED"
    Failed    TaskStatus = "FAILED"
)
```

### TaskNode (Extended)
```go
// main.go:54-65
type TaskNode struct {
    ID           string
    ActionName   string
    Dependencies []string
    Status       TaskStatus
    Payload      string
    Scheduled    bool
    // NEW in v1.2.0:
    ParentID     *string   // Parent task ID for hierarchy
    SubTasks     []string  // Child task IDs
    Progress     float64   // 0-100 progress percentage
    IsAtomic     bool      // True if cannot be decomposed
}
```

### TaskCheckpoint
```go
// checkpoint.go
type TaskCheckpoint struct {
    Version     uint64          // State version number
    StateData   []byte          // Serialized task state
    Position    string          // Execution position marker
    CreatedAt   time.Time       // Creation timestamp
    Metadata    map[string]any  // Optional metadata
}
```

### DependencyAnalysis
```go
// dependency_analysis.go
type DependencyAnalysis struct {
    HasCycle           bool              // Cycle detected
    CyclePath          []string          // Cycle path (if any)
    CriticalPath       []string          // Longest dependency chain
    CriticalPathLength int               // Critical path length
    ParallelismMax     int               // Maximum parallel tasks
    DependencyDepth    int               // Max dependency depth
    DependencyMatrix   map[string][]string // Complete dependency map
    ImpactMap          map[string][]string // Failure impact scope
    Graph              *DependencyGraph  // Serializable graph
    GeneratedAt        time.Time
}
```

## API Endpoints

| Endpoint | Method | Handler | Description |
|----------|--------|---------|-------------|
| `/api/v1/tasks/{id}/progress` | GET | `HandleGetProgress` | Get task progress (0-100%) |
| `/api/v1/tasks/{id}/pause` | POST | `HandlePauseTask` | Pause running task |
| `/api/v1/tasks/{id}/resume` | POST | `HandleResumeTask` | Resume paused task |
| `/api/v1/dags/analysis` | GET | `HandleDependencyAnalysis` | Full dependency analysis |
| `/api/v1/dags/critical-path` | GET | `HandleCriticalPath` | Critical path only |
| `/api/v1/dags/parallelism` | GET | `HandleParallelism` | Parallelism info |
| `/api/v1/tasks/{id}/impact` | GET | `HandleTaskImpact` | Failure impact scope |

## Conventions (This Module)

### Concurrent Execution Pattern
```go
// Channel-based coordination
readyQueue := make(chan *TaskNode, len(dm.Nodes))
completionChan := make(chan *TaskResult, len(dm.Nodes))
```

### Dependency Resolution
```go
// Decrement in-degree on completion
dm.mu.Lock()
dm.inDegree[id]--
if dm.inDegree[id] == 0 {
    readyQueue <- node // Ready for execution
}
dm.mu.Unlock()
```

### Pause/Resume Pattern
```go
// Register cancel function for task
cancelRegistry.register(taskID, cancel)

// On pause: save checkpoint, cancel context
func (dm *DAGManager) PauseTask(taskID string) error {
    dm.saveCheckpoint(taskID)
    cancelRegistry.cancel(taskID)
    task.Status = Paused
    return nil
}

// On resume: restore from checkpoint, re-enqueue
func (dm *DAGManager) ResumeTask(taskID string) error {
    dm.restoreFromCheckpoint(taskID)
    task.Status = Running
    dm.readyQueue <- task
    return nil
}
```

### Progress Calculation
```go
// Atomic task progress
switch task.Status {
case Pending: return 0
case Running: return 50
case Completed, Failed: return 100
case Paused: return task.Progress // Preserve on pause
}

// Parent task progress = average of subtasks
if len(task.SubTasks) > 0 {
    completed := countCompleted(task.SubTasks)
    return float64(completed) / float64(len(task.SubTasks)) * 100
}
```

## Anti-Patterns (This Module)

### Forbidden
```go
// NEVER: Direct state mutation without mutex
dm.inDegree[id]-- // WRONG: No lock

// ALWAYS: Use mutex for shared state
dm.mu.Lock()
dm.inDegree[id]--
dm.mu.Unlock()
```

### Channel Safety
```go
// WRONG: Channel could block on full
readyQueue <- task // Blocks if channel full

// CORRECT: Buffered channel sized to total tasks
readyQueue := make(chan *TaskNode, len(dm.Nodes))
```

### Pause Without Checkpoint
```go
// WRONG: Pause without saving state
task.Status = Paused // Data loss on resume

// CORRECT: Always checkpoint before pause
dm.saveCheckpoint(taskID)
task.Status = Paused
```

## Commands

```bash
# Build
cd orchestration/manager && go build -o bin/manager .

# Run tests
go test -v -cover ./...

# Run E2E tests
cd tests/e2e && go test -v .

# Run benchmarks
cd tests/benchmark && go test -bench=. -benchmem .
```

## Dependencies

| Package | Purpose |
|---------|---------|
| encoding/json | Task/Checkpoint serialization |
| sync | WaitGroup, Mutex, atomic |
| context | Cancellation for pause |
| log | Structured logging |
| net/http | REST API handlers |

## Performance Notes

- **Progress calculation**: O(1) for atomic, O(n) for n subtasks
- **Cycle detection**: O(V + E) DFS traversal
- **Critical path**: O(V + E) topological sort + DP
- **Checkpoint save**: ~800ns (JSON serialization)
- **Memory overhead**: Minimal (no extra allocations for progress)

## Notes

- **Concurrency model**: Channels + goroutines + WaitGroup
- **Pause safety**: Checkpoint-based state preservation
- **Dependency analysis**: All algorithms are deterministic
- **API design**: RESTful, JSON responses
- **Test coverage**: 87.2% (target: >80%)
