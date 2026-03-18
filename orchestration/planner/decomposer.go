package planner

import "context"

// ConflictType classifies the kind of conflict between tasks.
type ConflictType string

const (
	ResourceConflict ConflictType = "RESOURCE_CONFLICT"
	GoalConflict     ConflictType = "GOAL_CONFLICT"
	CycleConflict    ConflictType = "CYCLE_CONFLICT"
)

// Severity indicates how critical a conflict is.
type Severity string

const (
	Warning  Severity = "WARNING"
	Critical Severity = "CRITICAL"
)

// Conflict represents a detected conflict between tasks.
type Conflict struct {
	Type     ConflictType `json:"type"`
	TaskIDs  []string     `json:"task_ids"`
	Message  string       `json:"message"`
	Severity Severity     `json:"severity"`
}

// TaskDecomposer is the core interface for AI-driven task decomposition.
type TaskDecomposer interface {
	// Decompose breaks a ParsedIntent into an ordered list of DecomposedTasks.
	Decompose(ctx context.Context, req DecompositionRequest) (*DecompositionResult, error)

	// ValidateDecomposition checks the result for cycles and structural issues.
	ValidateDecomposition(result *DecompositionResult) error
}
