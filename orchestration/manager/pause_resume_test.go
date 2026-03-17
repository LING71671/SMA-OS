package main

import (
	"testing"
)

func TestPauseTask_Success(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "Test", Status: Running, Progress: 50.0})

	if err := dm.PauseTask("T1"); err != nil {
		t.Fatalf("PauseTask: %v", err)
	}
	if dm.Nodes["T1"].Status != Paused {
		t.Errorf("expected Paused, got %s", dm.Nodes["T1"].Status)
	}
}

func TestPauseTask_NotRunning(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "Test", Status: Pending})

	err := dm.PauseTask("T1")
	if err == nil {
		t.Error("expected error when pausing non-running task")
	}
}

func TestPauseTask_NotFound(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	err := dm.PauseTask("nonexistent")
	if err == nil {
		t.Error("expected error for missing task")
	}
}

func TestResumeTask_Success(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "Test", Status: Running, Progress: 50.0})

	// Pause first
	if err := dm.PauseTask("T1"); err != nil {
		t.Fatalf("PauseTask: %v", err)
	}

	readyQueue := make(chan *TaskNode, 1)
	if err := dm.ResumeTask("T1", readyQueue); err != nil {
		t.Fatalf("ResumeTask: %v", err)
	}

	if dm.Nodes["T1"].Status != Running {
		t.Errorf("expected Running after resume, got %s", dm.Nodes["T1"].Status)
	}

	select {
	case task := <-readyQueue:
		if task.ID != "T1" {
			t.Errorf("expected T1 in ready queue, got %s", task.ID)
		}
	default:
		t.Error("expected task to be re-enqueued")
	}
}

func TestResumeTask_NotPaused(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "Test", Status: Running})

	readyQueue := make(chan *TaskNode, 1)
	err := dm.ResumeTask("T1", readyQueue)
	if err == nil {
		t.Error("expected error when resuming non-paused task")
	}
}

func TestPauseResume_ProgressPreserved(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "Test", Status: Running, Progress: 73.5})

	if err := dm.PauseTask("T1"); err != nil {
		t.Fatalf("PauseTask: %v", err)
	}

	readyQueue := make(chan *TaskNode, 1)
	if err := dm.ResumeTask("T1", readyQueue); err != nil {
		t.Fatalf("ResumeTask: %v", err)
	}

	if dm.Nodes["T1"].Progress != 73.5 {
		t.Errorf("expected progress 73.5 preserved, got %f", dm.Nodes["T1"].Progress)
	}
}
