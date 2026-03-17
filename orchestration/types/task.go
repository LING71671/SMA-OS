package types

// TaskStatus represents the lifecycle state of a task
type TaskStatus string

const (
	TaskStatusPending   TaskStatus = "PENDING"
	TaskStatusRunning   TaskStatus = "RUNNING"
	TaskStatusCompleted TaskStatus = "COMPLETED"
	TaskStatusFailed    TaskStatus = "FAILED"
	TaskStatusPaused    TaskStatus = "PAUSED"
	TaskStatusResumed   TaskStatus = "RESUMED"
)

// TaskNode represents a single task in the Cognitive DAG
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

// TaskResult represents the result of task execution
type TaskResult struct {
	TaskID     string
	Status     TaskStatus
	Error      error
	StartTime  interface{} // time.Time — avoid import cycle
	EndTime    interface{}
	RetryCnt   int
	Checkpoint []byte // serialized checkpoint if paused
}
