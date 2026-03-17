package main

import (
	"encoding/json"
	"fmt"
	"sync"
	"sync/atomic"
	"time"
)

// globalCheckpointStore is the default in-process checkpoint store
var globalCheckpointStore = &struct {
	mu      sync.Mutex
	data    map[string]*TaskCheckpoint
	version uint64
}{
	data: make(map[string]*TaskCheckpoint),
}

// saveCheckpoint persists a checkpoint for the given task (caller must hold dm.mu)
func (dm *DAGManager) saveCheckpoint(taskID string) error {
	task := dm.Nodes[taskID]
	if task == nil {
		return fmt.Errorf("task %s not found", taskID)
	}
	ver := atomic.AddUint64(&globalCheckpointStore.version, 1)
	cp, err := NewCheckpoint(ver, task)
	if err != nil {
		return err
	}
	globalCheckpointStore.mu.Lock()
	globalCheckpointStore.data[taskID] = cp
	globalCheckpointStore.mu.Unlock()
	return nil
}

// restoreFromCheckpoint loads the latest checkpoint and applies it to the task (caller must hold dm.mu)
func (dm *DAGManager) restoreFromCheckpoint(taskID string) error {
	globalCheckpointStore.mu.Lock()
	cp, ok := globalCheckpointStore.data[taskID]
	globalCheckpointStore.mu.Unlock()
	if !ok {
		return fmt.Errorf("no checkpoint found for task %s", taskID)
	}
	task := dm.Nodes[taskID]
	if task == nil {
		return fmt.Errorf("task %s not found", taskID)
	}
	return cp.RestoreTask(task)
}

// TaskCheckpoint represents a saved state of a task for pause/resume support
type TaskCheckpoint struct {
	Version   uint64         `json:"version"`
	StateData []byte         `json:"state_data"`
	Position  string         `json:"position"`
	CreatedAt time.Time      `json:"created_at"`
	Metadata  map[string]any `json:"metadata,omitempty"`
}

// taskStateSnapshot is the internal structure serialized into StateData
type taskStateSnapshot struct {
	TaskID     string     `json:"task_id"`
	Status     TaskStatus `json:"status"`
	Progress   float64    `json:"progress"`
	ActionName string     `json:"action_name"`
	Payload    string     `json:"payload"`
}

// Serialize encodes the checkpoint to JSON bytes
func (c *TaskCheckpoint) Serialize() ([]byte, error) {
	return json.Marshal(c)
}

// Deserialize decodes JSON bytes into a TaskCheckpoint
func Deserialize(data []byte) (*TaskCheckpoint, error) {
	var c TaskCheckpoint
	if err := json.Unmarshal(data, &c); err != nil {
		return nil, err
	}
	return &c, nil
}

// NewCheckpoint creates a checkpoint from a TaskNode
func NewCheckpoint(version uint64, task *TaskNode) (*TaskCheckpoint, error) {
	snap := taskStateSnapshot{
		TaskID:     task.ID,
		Status:     task.Status,
		Progress:   task.Progress,
		ActionName: task.ActionName,
		Payload:    task.Payload,
	}
	stateData, err := json.Marshal(snap)
	if err != nil {
		return nil, err
	}
	return &TaskCheckpoint{
		Version:   version,
		StateData: stateData,
		Position:  task.ActionName,
		CreatedAt: time.Now(),
	}, nil
}

// RestoreTask applies checkpoint state back onto a TaskNode
func (c *TaskCheckpoint) RestoreTask(task *TaskNode) error {
	var snap taskStateSnapshot
	if err := json.Unmarshal(c.StateData, &snap); err != nil {
		return err
	}
	task.Progress = snap.Progress
	task.Payload = snap.Payload
	return nil
}
