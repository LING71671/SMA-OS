package main

import (
	"strings"
	"testing"
)

// TestSchedulerInitialization tests scheduler creation
func TestSchedulerInitialization(t *testing.T) {
	scheduler := NewScheduler(10)

	if scheduler == nil {
		t.Fatal("Scheduler should not be nil")
	}

	if scheduler.WarmPoolSize != 10 {
		t.Errorf("Expected WarmPoolSize 10, got %d", scheduler.WarmPoolSize)
	}

	// initWarmPool 会在 NewScheduler 中创建 WarmPoolSize 个 worker
	if len(scheduler.Workers) != 10 {
		t.Errorf("Expected 10 workers after warm pool init, got %d", len(scheduler.Workers))
	}
}

// TestAssignTask_Basic tests basic task assignment
func TestAssignTask_Basic(t *testing.T) {
	scheduler := NewScheduler(5)

	taskID := "test-task-1"
	workerID := scheduler.AssignTask(taskID, "")

	if workerID == "" {
		t.Error("Expected non-empty worker ID")
	}
}

// TestAssignTask_Affinity tests affinity-based assignment
func TestAssignTask_Affinity(t *testing.T) {
	scheduler := NewScheduler(5)

	// Add a worker with specific host
	worker := &WorkerNode{
		ID:        "worker-1",
		Type:      WorkerTypeResident,
		NodeHost:  "host-alpha",
		Available: true,
	}
	scheduler.Workers[worker.ID] = worker

	// Try to assign task with affinity to host-alpha
	assignedWorker := scheduler.AssignTask("task-with-affinity", "host-alpha")

	// Should return a worker (either the affinity match or fallback)
	if assignedWorker == "" {
		t.Error("Expected assigned worker ID")
	}
}

// TestWorkerTypes tests different worker types
func TestWorkerTypes(t *testing.T) {
	resident := WorkerTypeResident
	transient := WorkerTypeTransient

	if resident != "RESIDENT" {
		t.Errorf("Expected RESIDENT, got %s", resident)
	}

	if transient != "TRANSIENT" {
		t.Errorf("Expected TRANSIENT, got %s", transient)
	}
}

// TestWorkerNodeStructure tests WorkerNode structure
func TestWorkerNodeStructure(t *testing.T) {
	worker := WorkerNode{
		ID:        "test-worker",
		Type:      WorkerTypeResident,
		NodeHost:  "test-host",
		Available: true,
	}

	if worker.ID != "test-worker" {
		t.Errorf("Expected ID 'test-worker', got '%s'", worker.ID)
	}

	if worker.Type != WorkerTypeResident {
		t.Errorf("Expected Type RESIDENT, got %s", worker.Type)
	}

	if worker.NodeHost != "test-host" {
		t.Errorf("Expected NodeHost 'test-host', got '%s'", worker.NodeHost)
	}

	if !worker.Available {
		t.Error("Expected worker to be available")
	}
}

// TestSchedulerConcurrency tests concurrent task assignments
func TestSchedulerConcurrency(t *testing.T) {
	scheduler := NewScheduler(10)

	// Assign multiple tasks concurrently
	done := make(chan bool, 10)

	for i := 0; i < 10; i++ {
		go func(id int) {
			taskID := strings.Join([]string{"task", string(rune('A' + id))}, "-")
			_ = scheduler.AssignTask(taskID, "")
			done <- true
		}(i)
	}

	// Wait for all assignments
	for i := 0; i < 10; i++ {
		<-done
	}
}

// TestWarmPoolInitialization tests warm pool creation
func TestWarmPoolInitialization(t *testing.T) {
	warmPoolSize := 5
	scheduler := NewScheduler(warmPoolSize)

	// Scheduler should be initialized
	if scheduler == nil {
		t.Error("Scheduler should be initialized")
	}

	if scheduler.WarmPoolSize != warmPoolSize {
		t.Errorf("Expected warm pool size %d, got %d", warmPoolSize, scheduler.WarmPoolSize)
	}
}

// TestAssignTask_NoPreviousHost tests assignment without previous host
func TestAssignTask_NoPreviousHost(t *testing.T) {
	scheduler := NewScheduler(5)

	// Assign task without previous host (empty string)
	workerID := scheduler.AssignTask("new-task", "")

	if workerID == "" {
		t.Error("Expected worker ID to be assigned")
	}
}
