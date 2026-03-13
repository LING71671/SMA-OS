package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"math"
	"sync"
	"sync/atomic"
	"time"
)

// FailureConfig defines retry and timeout configuration
type FailureConfig struct {
	MaxRetries         int
	RetryDelay         time.Duration
	Timeout            time.Duration
	CancelOnParentFail bool
}

// TaskResult represents the result of task execution
type TaskResult struct {
	TaskID    string
	Status    TaskStatus
	Error     error
	StartTime time.Time
	EndTime   time.Time
	RetryCnt  int
}

// DefaultFailureConfig returns default configuration
func DefaultFailureConfig() FailureConfig {
	return FailureConfig{
		MaxRetries:         3,
		RetryDelay:         100 * time.Millisecond,
		Timeout:            30 * time.Second,
		CancelOnParentFail: true,
	}
}

type TaskStatus string

const (
	Pending   TaskStatus = "PENDING"
	Running   TaskStatus = "RUNNING"
	Completed TaskStatus = "COMPLETED"
	Failed    TaskStatus = "FAILED"
)

// TaskNode represents a single sub-task in the Cognitive DAG
type TaskNode struct {
	ID           string
	ActionName   string
	Dependencies []string
	Status       TaskStatus
	Payload      string
}

// Orchestrator Manager responsible for Topological Execution
type DAGManager struct {
	Nodes         map[string]*TaskNode
	inDegree      map[string]int
	mu            sync.Mutex
	FailureConfig FailureConfig
}

func NewDAGManager(config FailureConfig) *DAGManager {
	return &DAGManager{
		Nodes:         make(map[string]*TaskNode),
		inDegree:      make(map[string]int),
		FailureConfig: config,
	}
}

// AddTask computes indegrees dynamically
func (dm *DAGManager) AddTask(t TaskNode) {
	dm.mu.Lock()
	defer dm.mu.Unlock()
	dm.Nodes[t.ID] = &t
	dm.inDegree[t.ID] = len(t.Dependencies)

	// Ensure dependencies exist in the graph (simplified)
	for _, dep := range t.Dependencies {
		if _, ok := dm.inDegree[dep]; !ok {
			dm.inDegree[dep] = 0 // Initialize to 0 if not present yet
		}
	}
}

// Execute performs topologically sorted concurrent worker dispatching
func (dm *DAGManager) Execute() error {
	log.Println("[Manager] Starting Topologically Sorted DAG Execution...")

	readyQueue := make(chan *TaskNode, len(dm.Nodes))
	completionChan := make(chan *TaskResult, len(dm.Nodes))
	dispatcherDone := make(chan struct{})
	var wg sync.WaitGroup

	// 1. Enqueue all initial nodes with 0 in-degree
	dm.mu.Lock()
	for id, degree := range dm.inDegree {
		if degree == 0 && dm.Nodes[id] != nil {
			readyQueue <- dm.Nodes[id]
		}
	}
	dm.mu.Unlock()

	totalTasks := len(dm.Nodes)
	var completedTasks int32 = 0

	// 2. Dispatch loop
	go func() {
		defer close(dispatcherDone)
		for {
			select {
			case task := <-readyQueue:
				wg.Add(1)
				// Simulating Worker assignment (e.g. gRPC to Firecracker MicroVM pool)
				go dm.dispatchWorker(task, completionChan, &wg)

			case res := <-completionChan:
				atomic.AddInt32(&completedTasks, 1)
				log.Printf("[Manager] Registered COMPLETION event for Task: %s", res.TaskID)

				// Calculate dependents and decrement their in-degrees
				dm.mu.Lock()
				for id, node := range dm.Nodes {
					if node.Status == Pending {
						// Check if completedID is in dependencies
						hasDep := false
						for _, d := range node.Dependencies {
							if d == res.TaskID {
								hasDep = true
								break
							}
						}
						if hasDep {
							dm.inDegree[id]--
							if dm.inDegree[id] == 0 {
								readyQueue <- node
							}
						}
					}
				}
				dm.mu.Unlock()

				if int(atomic.LoadInt32(&completedTasks)) == totalTasks {
					log.Println("[Manager] All tasks in DAG completed successfully!")
					return // End execution mapping
				}
			}
		}
	}()

	// Wait for dispatcher to finish (all tasks completed)
	<-dispatcherDone
	// Wait for all actual worker routines to finish
	wg.Wait()
	return nil
}

func (dm *DAGManager) dispatchWorker(task *TaskNode, done chan<- *TaskResult, wg *sync.WaitGroup) {
	defer wg.Done()
	task.Status = Running
	start := time.Now()
	var lastErr error
	retryCount := 0

	// Retry loop
	for attempt := 0; attempt <= dm.FailureConfig.MaxRetries; attempt++ {
		if attempt > 0 {
			// Calculate backoff delay
			delay := time.Duration(float64(dm.FailureConfig.RetryDelay) * math.Pow(2, float64(attempt-1)))
			if delay > 5*time.Second {
				delay = 5 * time.Second // Cap at 5s
			}
			log.Printf("[Worker] Task %s retry %d/%d after %v", task.ID, attempt, dm.FailureConfig.MaxRetries, delay)
			time.Sleep(delay)
			retryCount++
		}

		// Create timeout context
		ctx, cancel := context.WithTimeout(context.Background(), dm.FailureConfig.Timeout)

		// Execute task with timeout
		resultChan := make(chan error, 1)
		go func() {
			resultChan <- dm.executeTask(task)
		}()

		select {
		case err := <-resultChan:
			cancel()
			if err == nil {
				// Success
				task.Status = Completed
				done <- &TaskResult{
					TaskID:    task.ID,
					Status:    Completed,
					Error:     nil,
					StartTime: start,
					EndTime:   time.Now(),
					RetryCnt:  retryCount,
				}
				log.Printf("[Worker] Task %s completed (retries: %d)", task.ID, retryCount)
				return
			}
			lastErr = err
			log.Printf("[Worker] Task %s failed (attempt %d/%d): %v", task.ID, attempt+1, dm.FailureConfig.MaxRetries+1, err)
		case <-ctx.Done():
			cancel()
			lastErr = fmt.Errorf("task timeout after %v", dm.FailureConfig.Timeout)
			log.Printf("[Worker] Task %s timeout (attempt %d/%d)", task.ID, attempt+1, dm.FailureConfig.MaxRetries+1)
		}
	}

	// All retries exhausted
	task.Status = Failed
	done <- &TaskResult{
		TaskID:    task.ID,
		Status:    Failed,
		Error:     lastErr,
		StartTime: start,
		EndTime:   time.Now(),
		RetryCnt:  retryCount,
	}
	log.Printf("[Worker] Task %s failed after %d retries: %v", task.ID, retryCount, lastErr)
}

// executeTask performs actual task execution
func (dm *DAGManager) executeTask(task *TaskNode) error {
	log.Printf("[Worker Scheduler] -> Executing Task [%s] (%s)...", task.ID, task.ActionName)
	time.Sleep(500 * time.Millisecond)
	// Success
	return nil
}

func main() {
	log.Println("Initializing SMA-OS Cognitive Data Plane: DAG Orchestrator v2.0")

	// Simulating a parsed Cognitive Execution Graph (DAG)
	dagJSON := `[
		{"ID": "T1", "ActionName": "Extract User Intent", "Dependencies": []},
		{"ID": "T2", "ActionName": "Query Vector DB", "Dependencies": ["T1"]},
		{"ID": "T3", "ActionName": "Verify RBAC Scope", "Dependencies": ["T1"]},
		{"ID": "T4", "ActionName": "Generate Multi-Step Output", "Dependencies": ["T2", "T3"]}
	]`

	var tasks []TaskNode
	if err := json.Unmarshal([]byte(dagJSON), &tasks); err != nil {
		log.Fatalf("JSON parse error: %v", err)
	}

	manager := NewDAGManager(DefaultFailureConfig())
	for _, t := range tasks {
		t.Status = Pending
		manager.AddTask(t)
	}

	// Begin High-Performance execution
	manager.Execute()
}
