package main

import (
	"testing"
)

func TestGetProgress_Atomic_Pending(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "A", Status: Pending, IsAtomic: true})

	p := dm.GetProgress("T1")
	if p.Progress != 0 {
		t.Errorf("expected 0, got %f", p.Progress)
	}
}

func TestGetProgress_Atomic_Running(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "A", Status: Running, IsAtomic: true})

	p := dm.GetProgress("T1")
	if p.Progress != 50 {
		t.Errorf("expected 50, got %f", p.Progress)
	}
}

func TestGetProgress_Atomic_Completed(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "A", Status: Completed, IsAtomic: true})

	p := dm.GetProgress("T1")
	if p.Progress != 100 {
		t.Errorf("expected 100, got %f", p.Progress)
	}
}

func TestGetProgress_WithSubTasks(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "P", ActionName: "Parent", Status: Running, SubTasks: []string{"C1", "C2", "C3", "C4"}})
	dm.AddTask(TaskNode{ID: "C1", ActionName: "Child1", Status: Completed})
	dm.AddTask(TaskNode{ID: "C2", ActionName: "Child2", Status: Completed})
	dm.AddTask(TaskNode{ID: "C3", ActionName: "Child3", Status: Pending})
	dm.AddTask(TaskNode{ID: "C4", ActionName: "Child4", Status: Pending})

	p := dm.GetProgress("P")
	if p.Progress != 50 {
		t.Errorf("expected 50, got %f", p.Progress)
	}
}

func TestGetProgress_AllSubTasksComplete(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "P", ActionName: "Parent", Status: Running, SubTasks: []string{"C1", "C2"}})
	dm.AddTask(TaskNode{ID: "C1", ActionName: "Child1", Status: Completed})
	dm.AddTask(TaskNode{ID: "C2", ActionName: "Child2", Status: Completed})

	p := dm.GetProgress("P")
	if p.Progress != 100 {
		t.Errorf("expected 100, got %f", p.Progress)
	}
}

func TestGetProgress_NotFound(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	p := dm.GetProgress("nonexistent")
	if p.Status != Failed {
		t.Errorf("expected Failed status for missing task")
	}
}

func TestGetProgress_Paused(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "A", Status: Paused, Progress: 42.5})

	p := dm.GetProgress("T1")
	if p.Progress != 42.5 {
		t.Errorf("expected 42.5, got %f", p.Progress)
	}
}

func TestGetProgressRecursive_SubTaskList(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "P", ActionName: "Parent", Status: Running, SubTasks: []string{"C1", "C2"}})
	dm.AddTask(TaskNode{ID: "C1", ActionName: "Child1", Status: Completed})
	dm.AddTask(TaskNode{ID: "C2", ActionName: "Child2", Status: Running})

	p := dm.GetProgress("P")
	if len(p.SubTasks) != 2 {
		t.Errorf("expected 2 subtasks, got %d", len(p.SubTasks))
	}
}

func TestProgressConcurrent(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	for i := 0; i < 20; i++ {
		id := string(rune('A' + i))
		dm.AddTask(TaskNode{ID: id, ActionName: "Task", Status: Pending})
	}

	done := make(chan bool, 20)
	for i := 0; i < 20; i++ {
		go func(idx int) {
			id := string(rune('A' + idx))
			dm.GetProgress(id)
			done <- true
		}(i)
	}
	for i := 0; i < 20; i++ {
		<-done
	}
}
