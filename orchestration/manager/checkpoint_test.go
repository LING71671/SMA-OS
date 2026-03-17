package main

import (
	"testing"
)

func TestCheckpointSerialize(t *testing.T) {
	task := &TaskNode{ID: "T1", ActionName: "Test", Status: Running, Progress: 42.0, Payload: "data"}
	cp, err := NewCheckpoint(1, task)
	if err != nil {
		t.Fatalf("NewCheckpoint: %v", err)
	}

	data, err := cp.Serialize()
	if err != nil {
		t.Fatalf("Serialize: %v", err)
	}
	if len(data) == 0 {
		t.Error("expected non-empty serialized data")
	}

	cp2, err := Deserialize(data)
	if err != nil {
		t.Fatalf("Deserialize: %v", err)
	}
	if cp2.Version != cp.Version {
		t.Errorf("version mismatch: %d != %d", cp2.Version, cp.Version)
	}
	if cp2.Position != cp.Position {
		t.Errorf("position mismatch: %s != %s", cp2.Position, cp.Position)
	}
}

func TestCheckpointRestoreTask(t *testing.T) {
	original := &TaskNode{ID: "T1", ActionName: "Test", Status: Running, Progress: 75.0, Payload: "payload"}
	cp, err := NewCheckpoint(1, original)
	if err != nil {
		t.Fatalf("NewCheckpoint: %v", err)
	}

	restored := &TaskNode{ID: "T1", ActionName: "Test", Status: Paused}
	if err := cp.RestoreTask(restored); err != nil {
		t.Fatalf("RestoreTask: %v", err)
	}
	if restored.Progress != 75.0 {
		t.Errorf("expected progress 75.0, got %f", restored.Progress)
	}
	if restored.Payload != "payload" {
		t.Errorf("expected payload 'payload', got %s", restored.Payload)
	}
}

func TestSaveAndRestoreCheckpoint(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "Test", Status: Running, Progress: 60.0})

	dm.mu.Lock()
	if err := dm.saveCheckpoint("T1"); err != nil {
		dm.mu.Unlock()
		t.Fatalf("saveCheckpoint: %v", err)
	}
	dm.Nodes["T1"].Progress = 0 // simulate reset
	if err := dm.restoreFromCheckpoint("T1"); err != nil {
		dm.mu.Unlock()
		t.Fatalf("restoreFromCheckpoint: %v", err)
	}
	dm.mu.Unlock()

	if dm.Nodes["T1"].Progress != 60.0 {
		t.Errorf("expected progress 60.0 after restore, got %f", dm.Nodes["T1"].Progress)
	}
}

func TestRestoreCheckpoint_NotFound(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "NO_CHECKPOINT_TASK", ActionName: "Test", Status: Running})

	dm.mu.Lock()
	err := dm.restoreFromCheckpoint("NO_CHECKPOINT_TASK")
	dm.mu.Unlock()

	if err == nil {
		t.Error("expected error for missing checkpoint")
	}
}
