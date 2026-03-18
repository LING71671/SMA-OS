package planner

import "time"

// TaskStatus mirrors the manager's TaskStatus for cross-package use.
type TaskStatus string

const (
	Pending   TaskStatus = "PENDING"
	Running   TaskStatus = "RUNNING"
	Completed TaskStatus = "COMPLETED"
	Failed    TaskStatus = "FAILED"
	Paused    TaskStatus = "PAUSED"
	Resumed   TaskStatus = "RESUMED"
)

// ParsedIntent mirrors memory-bus/ingestion's ParsedIntent.
type ParsedIntent struct {
	Action     string  `json:"action"`
	Target     string  `json:"target"`
	Parameters string  `json:"parameters"`
	Confidence float64 `json:"confidence"`
	Source     string  `json:"source"`
}

// TaskNode mirrors orchestration/manager's TaskNode for DAG submission.
type TaskNode struct {
	ID           string
	ActionName   string
	Dependencies []string
	Status       TaskStatus
	Payload      string
	Scheduled    bool
	ParentID     *string
	SubTasks     []string
	Progress     float64
	IsAtomic     bool
}

// DecomposedTask represents a single task produced by AI decomposition.
type DecomposedTask struct {
	ID            string        `json:"id"`
	ActionName    string        `json:"action_name"`
	Description   string        `json:"description"`
	Dependencies  []string      `json:"dependencies"`
	AgentID       string        `json:"agent_id,omitempty"`
	EstimatedTime time.Duration `json:"estimated_time,omitempty"`
	Priority      int           `json:"priority"`
}

// DecompositionRequest is the input to the decomposer.
type DecompositionRequest struct {
	Intent      ParsedIntent        `json:"intent"`
	MaxDepth    int                 `json:"max_depth"`    // default 5
	MaxSubTasks int                 `json:"max_sub_tasks"` // default 20
	Options     DecompositionOptions `json:"options"`
}

// DecompositionOptions controls decomposition behaviour.
type DecompositionOptions struct {
	IncludeAgentAssignment bool `json:"include_agent_assignment"`
	IncludeTimeEstimate    bool `json:"include_time_estimate"`
	SyncMode               bool `json:"sync_mode"`
}

// DecompositionResult is the output of the decomposer.
type DecompositionResult struct {
	Tasks      []DecomposedTask `json:"tasks"`
	RootTaskID string           `json:"root_task_id"`
	Duration   time.Duration    `json:"duration"`
	Metadata   map[string]any   `json:"metadata,omitempty"`
}
