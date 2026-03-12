# Evaluator/Critic Agent Module Guide

**Location**: `orchestration/evaluator/`  
**Domain**: Output validation and versioned rollback decisions  
**Language**: Go  
**Score**: 10/25 (supporting service, validation logic)

## Overview

Independent verification layer for cognitive task outputs. Implements critic pattern with versioned rejections, enabling rollback to previous valid state when hallucinations or schema mismatches are detected.

## Structure

```
evaluator/
├── main.go              # EvaluatorAgent with AuditTaskResult
├── go.mod              # Uses sma-os/orchestration module
└── main_test.go        # (if exists)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Audit logic | `main.go:29-43` | VersionedReject decision making |
| Rejection struct | `main.go:13-18` | Rollback instructions |
| Critic validation | `main.go:32` | Schema mismatch detection |
| Version tracking | `main.go:38` | RollbackTo version field |

## Conventions (This Module)

### Versioned Rejection Pattern
```go
type VersionedReject struct {
    TaskID          string
    RejectedVersion uint64  // Current invalid version
    Reason          string  // e.g., "schema_mismatch"
    RollbackTo      uint64  // Previous valid version
}
```

### Validation Result
```go
// Returns nil if valid, VersionedReject if invalid
func (e *EvaluatorAgent) AuditTaskResult(...) *VersionedReject {
    if result == "invalid_schema" {
        return &VersionedReject{...}  // Reject
    }
    return nil  // Accept
}
```

### Critic Logging
```go
// Explicit alignment status
log.Printf("[Critic] Rejecting Task %s at Version %d...", taskID, version)
log.Printf("[Critic] Task %s Version %d alignment OK.", taskID, version)
```

## Anti-Patterns (This Module)

### Forbidden
```go
// NEVER: Silent rejection
if result == "invalid" {
    return nil  // WRONG: Hiding rejection
}

// ALWAYS: Explicit rejection with reason
return &VersionedReject{
    Reason: "explicit_validation_failure",
}
```

### Version Management
```go
// WRONG: Rollback to same version
RollbackTo: version  // Pointless

// CORRECT: Rollback to previous valid
RollbackTo: version - 1
```

### Nil Handling
```go
// WRONG: Not checking nil return
rejectCmd := evaluator.AuditTaskResult(...)
// rejectCmd.TaskID  // PANIC: nil pointer

// CORRECT: Explicit nil check
if rejectCmd != nil {
    log.Printf("Rollback task %s", rejectCmd.TaskID)
}
```

## Unique Styles

### Simulated LLM Critic
```go
// Placeholder for actual LLM critic layer
// In reality this would invoke LLM critic layers or rule-validators
func (e *EvaluatorAgent) AuditTaskResult(...) *VersionedReject {
    // Simulated check
    if result == "invalid_schema" { ... }
}
```

### Graceful Loop
```go
// Standard across Go services
ticker := time.NewTicker(10 * time.Second)
for {
    select {
    case <-ticker.C:
        log.Println("[Critic] Waiting for tasks...")
    case <-ctx.Done():
        return
    }
}
```

## Commands

```bash
# Build
cd orchestration/evaluator && go build -o bin/evaluator .

# Run
go run main.go
```

## Dependencies

| Package | Purpose |
|---------|---------|
| context | Cancellation |
| os/signal | Graceful shutdown |
| time | Audit ticker |

## Notes

- **Placeholder logic**: Currently uses hardcoded "invalid_schema" check
- **Version control**: Integrates with state-engine for rollback
- **Critic layer**: Intended to use LLM for output validation
- **No persistence**: Stateless validation (state in state-engine)
