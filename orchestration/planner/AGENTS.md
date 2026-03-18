# orchestration/planner

AI-driven task decomposition module. Converts a `ParsedIntent` into a `[]TaskNode` DAG ready for submission to `orchestration/manager`.

## Architecture

```
ParsedIntent
    │
    ▼
LLMDecomposer.Decompose()   →   []DecomposedTask
    │
    ▼
Bridge.DecomposedTaskToTaskNode()   →   []TaskNode
    │
    ▼
ConflictDetector.Detect()   →   []Conflict (warnings)
    │
    ▼
DAGManager.AddTasksFromIntent()   →   execution
```

## Key Files

| File | Purpose |
|------|---------|
| `types.go` | Shared types: `DecomposedTask`, `DecompositionRequest`, `DecompositionResult`, `TaskNode`, `ParsedIntent` |
| `decomposer.go` | `TaskDecomposer` interface + `Conflict`/`ConflictType`/`Severity` types |
| `llm_decomposer.go` | `LLMDecomposer` — calls LLM, parses JSON response, validates DAG |
| `bridge.go` | `Bridge` — converts `[]DecomposedTask` → `[]TaskNode` |
| `conflict.go` | `ConflictDetector` — resource, goal, and cycle conflict detection |

## Usage

```go
import "sma-os/orchestration/planner"

// 1. Create decomposer (inject any LLMClient implementation)
decomposer := planner.NewLLMDecomposer(llmManager, 5, 20)

// 2. Decompose intent
result, err := decomposer.Decompose(ctx, planner.DecompositionRequest{
    Intent: planner.ParsedIntent{
        Action:     "create_vm",
        Target:     "pool-A",
        Parameters: "cpu=2,ram=4G",
    },
})

// 3. Convert and submit to manager
nodes := planner.DecomposedTaskToTaskNode(result.Tasks)
err = dagManager.AddTasksFromIntent(nodes)
```

## HTTP API

`POST /api/v1/tasks/decompose` (handled in `orchestration/manager/api.go`)

Request:
```json
{
  "intent": { "action": "create_vm", "target": "pool-A", "parameters": "cpu=2" },
  "max_depth": 5,
  "max_sub_tasks": 20
}
```

Response:
```json
{
  "root_task_id": "T1",
  "tasks": [...],
  "duration": "1.2ms"
}
```

## Constraints

- Max decomposition depth: 5 (configurable)
- Max sub-tasks: 20 (configurable)
- Request timeout: 60s
- No cyclic dependencies allowed (validated before submission)
- Does not persist decomposition results

## Testing

```bash
go test ./orchestration/planner/...
go test -coverprofile=coverage.out ./orchestration/planner/... && go tool cover -func=coverage.out
```

Current coverage: **95%**
