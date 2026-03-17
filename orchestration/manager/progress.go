package main

import (
	"time"
)

// TaskProgress represents the current progress of a task
type TaskProgress struct {
	TaskID       string            `json:"task_id"`
	Status       TaskStatus        `json:"status"`
	Progress     float64           `json:"progress"` // 0-100
	SubTasks     []SubTaskProgress `json:"sub_tasks"`
	StartTime    time.Time         `json:"start_time"`
	UpdatedAt    time.Time         `json:"updated_at"`
	EstimatedEnd *time.Time        `json:"estimated_end,omitempty"`
}

// SubTaskProgress represents progress of a child task
type SubTaskProgress struct {
	TaskID   string     `json:"task_id"`
	Status   TaskStatus `json:"status"`
	Progress float64    `json:"progress"`
}

// GetProgress calculates the progress of a task by its ID
func (dm *DAGManager) GetProgress(taskID string) TaskProgress {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	task := dm.Nodes[taskID]
	if task == nil {
		return TaskProgress{TaskID: taskID, Status: Failed, Progress: 0, UpdatedAt: time.Now()}
	}

	progress := dm.calcProgress(task)

	var subProgresses []SubTaskProgress
	for _, subID := range task.SubTasks {
		if sub := dm.Nodes[subID]; sub != nil {
			subProgresses = append(subProgresses, SubTaskProgress{
				TaskID:   sub.ID,
				Status:   sub.Status,
				Progress: dm.calcProgress(sub),
			})
		}
	}

	return TaskProgress{
		TaskID:    taskID,
		Status:    task.Status,
		Progress:  progress,
		SubTasks:  subProgresses,
		UpdatedAt: time.Now(),
	}
}

// calcProgress computes progress percentage for a single node (must hold dm.mu)
func (dm *DAGManager) calcProgress(task *TaskNode) float64 {
	if len(task.SubTasks) > 0 {
		completed := 0
		for _, subID := range task.SubTasks {
			if sub := dm.Nodes[subID]; sub != nil {
				if sub.Status == Completed || sub.Status == Failed {
					completed++
				}
			}
		}
		return float64(completed) / float64(len(task.SubTasks)) * 100
	}
	switch task.Status {
	case Pending:
		return 0
	case Running:
		return 50
	case Completed:
		return 100
	case Failed:
		return 100
	case Paused:
		return task.Progress
	case Resumed:
		return task.Progress
	}
	return 0
}
