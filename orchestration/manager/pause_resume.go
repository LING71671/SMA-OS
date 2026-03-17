package main

import (
	"fmt"
	"sync"
)

// taskCancelRegistry holds per-task cancel functions for pause signaling
type taskCancelRegistry struct {
	mu      sync.Mutex
	cancels map[string]func()
}

var cancelRegistry = &taskCancelRegistry{
	cancels: make(map[string]func()),
}

func (r *taskCancelRegistry) register(taskID string, cancel func()) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.cancels[taskID] = cancel
}

func (r *taskCancelRegistry) cancel(taskID string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if cancel, ok := r.cancels[taskID]; ok {
		cancel()
		delete(r.cancels, taskID)
	}
}

func (r *taskCancelRegistry) remove(taskID string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	delete(r.cancels, taskID)
}

// PauseTask pauses a running task by saving a checkpoint and canceling its context.
func (dm *DAGManager) PauseTask(taskID string) error {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	task := dm.Nodes[taskID]
	if task == nil {
		return fmt.Errorf("task %s not found", taskID)
	}
	if task.Status != Running {
		return fmt.Errorf("task %s not running (status: %s)", taskID, task.Status)
	}

	// Save checkpoint before changing state
	if err := dm.saveCheckpoint(taskID); err != nil {
		return fmt.Errorf("saveCheckpoint: %w", err)
	}

	task.Status = Paused

	// Signal the worker goroutine to stop
	cancelRegistry.cancel(taskID)

	return nil
}

// ResumeTask resumes a paused task from its last checkpoint.
func (dm *DAGManager) ResumeTask(taskID string, readyQueue chan<- *TaskNode) error {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	task := dm.Nodes[taskID]
	if task == nil {
		return fmt.Errorf("task %s not found", taskID)
	}
	if task.Status != Paused {
		return fmt.Errorf("task %s not paused (status: %s)", taskID, task.Status)
	}

	if err := dm.restoreFromCheckpoint(taskID); err != nil {
		return fmt.Errorf("restoreFromCheckpoint: %w", err)
	}

	task.Status = Running
	task.Scheduled = false

	// Re-enqueue for execution
	readyQueue <- task

	return nil
}
