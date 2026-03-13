package main

import (
	"testing"
	"time"
)

// TestDAGManager_AddTask tests adding tasks to the DAG
func TestDAGManager_AddTask(t *testing.T) {
	manager := NewDAGManager(DefaultFailureConfig())

	// Test adding a single task
	task := TaskNode{
		ID:           "T1",
		ActionName:   "Test Action",
		Dependencies: []string{},
		Status:       Pending,
	}

	manager.AddTask(task)

	// Verify task was added
	if len(manager.Nodes) != 1 {
		t.Errorf("Expected 1 node, got %d", len(manager.Nodes))
	}

	if manager.Nodes["T1"] == nil {
		t.Error("Expected T1 to exist in nodes")
	}

	if manager.Nodes["T1"].ActionName != "Test Action" {
		t.Errorf("Expected ActionName 'Test Action', got '%s'", manager.Nodes["T1"].ActionName)
	}
}

// TestDAGManager_AddTaskWithDependencies tests adding tasks with dependencies
func TestDAGManager_AddTaskWithDependencies(t *testing.T) {
	manager := NewDAGManager(DefaultFailureConfig())

	// Add parent task
	parent := TaskNode{
		ID:           "T1",
		ActionName:   "Parent",
		Dependencies: []string{},
		Status:       Pending,
	}
	manager.AddTask(parent)

	// Add child task with dependency
	child := TaskNode{
		ID:           "T2",
		ActionName:   "Child",
		Dependencies: []string{"T1"},
		Status:       Pending,
	}
	manager.AddTask(child)

	// Verify both tasks exist
	if len(manager.Nodes) != 2 {
		t.Errorf("Expected 2 nodes, got %d", len(manager.Nodes))
	}

	// Verify in-degree is correct
	if manager.inDegree["T1"] != 0 {
		t.Errorf("Expected T1 in-degree 0, got %d", manager.inDegree["T1"])
	}

	if manager.inDegree["T2"] != 1 {
		t.Errorf("Expected T2 in-degree 1, got %d", manager.inDegree["T2"])
	}
}

// TestDAGManager_Execute tests the DAG execution
func TestDAGManager_Execute(t *testing.T) {
	manager := NewDAGManager(DefaultFailureConfig())

	// Create a simple DAG: T1 -> T2
	t1 := TaskNode{
		ID:           "T1",
		ActionName:   "First Task",
		Dependencies: []string{},
		Status:       Pending,
	}

	t2 := TaskNode{
		ID:           "T2",
		ActionName:   "Second Task",
		Dependencies: []string{"T1"},
		Status:       Pending,
	}

	manager.AddTask(t1)
	manager.AddTask(t2)

	// Execute the DAG
	err := manager.Execute()
	if err != nil {
		t.Errorf("Execute returned error: %v", err)
	}

	// Give some time for execution
	time.Sleep(3 * time.Second)

	// Verify tasks completed
	if manager.Nodes["T1"].Status != Completed {
		t.Errorf("Expected T1 to be Completed, got %s", manager.Nodes["T1"].Status)
	}

	if manager.Nodes["T2"].Status != Completed {
		t.Errorf("Expected T2 to be Completed, got %s", manager.Nodes["T2"].Status)
	}
}

// TestDAGManager_EmptyDAG tests execution of empty DAG
func TestDAGManager_EmptyDAG(t *testing.T) {
	manager := NewDAGManager(DefaultFailureConfig())

	// Execute empty DAG should not panic
	err := manager.Execute()
	if err != nil {
		t.Errorf("Execute on empty DAG returned error: %v", err)
	}
}

// TestTaskStatusTransitions tests valid status transitions
func TestTaskStatusTransitions(t *testing.T) {
	task := TaskNode{
		ID:           "T1",
		ActionName:   "Test",
		Dependencies: []string{},
		Status:       Pending,
	}

	// Initial status should be Pending
	if task.Status != Pending {
		t.Errorf("Expected initial status Pending, got %s", task.Status)
	}

	// Transition to Running
	task.Status = Running
	if task.Status != Running {
		t.Errorf("Expected status Running, got %s", task.Status)
	}

	// Transition to Completed
	task.Status = Completed
	if task.Status != Completed {
		t.Errorf("Expected status Completed, got %s", task.Status)
	}
}

// TestDAGManager_ConcurrentAccess tests thread safety
func TestDAGManager_ConcurrentAccess(t *testing.T) {
	manager := NewDAGManager(DefaultFailureConfig())

	// Add tasks concurrently
	done := make(chan bool, 10)

	for i := 0; i < 10; i++ {
		go func(id int) {
			task := TaskNode{
				ID:           string(rune('A' + id)),
				ActionName:   "Concurrent Task",
				Dependencies: []string{},
				Status:       Pending,
			}
			manager.AddTask(task)
			done <- true
		}(i)
	}

	// Wait for all goroutines
	for i := 0; i < 10; i++ {
		<-done
	}

	// Verify all tasks were added
	if len(manager.Nodes) != 10 {
		t.Errorf("Expected 10 nodes, got %d", len(manager.Nodes))
	}
}
