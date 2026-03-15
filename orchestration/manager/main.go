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
	Scheduled    bool // Prevents duplicate enqueuing when multiple dependencies complete
}

// Orchestrator Manager responsible for Topological Execution
type DAGManager struct {
	Nodes         map[string]*TaskNode
	inDegree      map[string]int
	dependents    map[string][]string // Adjacency list: taskID -> list of dependent task IDs
	mu            sync.Mutex
	FailureConfig FailureConfig
}

func NewDAGManager(config FailureConfig) *DAGManager {
	return &DAGManager{
		Nodes:         make(map[string]*TaskNode),
		inDegree:      make(map[string]int),
		dependents:    make(map[string][]string),
		FailureConfig: config,
	}
}

// AddTask computes indegrees dynamically and builds adjacency list
func (dm *DAGManager) AddTask(t TaskNode) {
	dm.mu.Lock()
	defer dm.mu.Unlock()
	dm.Nodes[t.ID] = &t
	dm.inDegree[t.ID] = len(t.Dependencies)

	// Build adjacency list for O(1) dependent lookup
	for _, dep := range t.Dependencies {
		dm.dependents[dep] = append(dm.dependents[dep], t.ID)
	}

	// Ensure dependencies exist in the graph (simplified)
	for _, dep := range t.Dependencies {
		if _, ok := dm.inDegree[dep]; !ok {
			dm.inDegree[dep] = 0 // Initialize to 0 if not present yet
		}
	}
}

// Execute performs topologically sorted concurrent worker dispatching
// with failure propagation: if a task fails, all its dependents are also marked Failed.
func (dm *DAGManager) Execute() error {
	totalTasks := len(dm.Nodes)
	if totalTasks == 0 {
		log.Println("[Manager] Empty DAG, nothing to execute.")
		return nil
	}

	log.Println("[Manager] Starting Topologically Sorted DAG Execution...")

	readyQueue := make(chan *TaskNode, totalTasks)
	completionChan := make(chan *TaskResult, totalTasks)
	var wg sync.WaitGroup
	var completedTasks int32 = 0

	// 1. Enqueue all initial nodes with 0 in-degree
	dm.mu.Lock()
	for id, degree := range dm.inDegree {
		if degree == 0 && dm.Nodes[id] != nil {
			readyQueue <- dm.Nodes[id]
		}
	}
	dm.mu.Unlock()

	// 全局超时防护，避免任何情况下的永久阻塞
	globalTimeout := time.Duration(totalTasks) * (dm.FailureConfig.Timeout + dm.FailureConfig.RetryDelay*time.Duration(dm.FailureConfig.MaxRetries+1))
	if globalTimeout < 30*time.Second {
		globalTimeout = 30 * time.Second
	}
	timer := time.NewTimer(globalTimeout)
	defer timer.Stop()

	// 2. Dispatch loop
	dispatcherDone := make(chan error, 1)
	go func() {
		for {
			select {
			case task := <-readyQueue:
				wg.Add(1)
				go dm.dispatchWorker(task, completionChan, &wg)

			case res := <-completionChan:
				atomic.AddInt32(&completedTasks, 1)
				log.Printf("[Manager] Registered %s event for Task: %s", res.Status, res.TaskID)

				dm.mu.Lock()
				if res.Status == Failed && dm.FailureConfig.CancelOnParentFail {
					// 失败传播：递归取消所有下游依赖任务
					cancelled := dm.cancelDependents(res.TaskID)
					atomic.AddInt32(&completedTasks, int32(cancelled))
				}

				if res.Status == Completed {
					// Use adjacency list for O(1) dependent lookup
					dependents := dm.dependents[res.TaskID]
					for _, depID := range dependents {
						node := dm.Nodes[depID]
						if node != nil && node.Status == Pending && !node.Scheduled {
							dm.inDegree[depID]--
							if dm.inDegree[depID] == 0 {
								node.Scheduled = true // Mark as scheduled before enqueue
								readyQueue <- node
							}
						}
					}
				}
				dm.mu.Unlock()

				if int(atomic.LoadInt32(&completedTasks)) >= totalTasks {
					log.Println("[Manager] All tasks in DAG resolved (completed or failed).")
					dispatcherDone <- nil
					return
				}

			case <-timer.C:
				dispatcherDone <- fmt.Errorf("DAG execution timed out after %v", globalTimeout)
				return
			}
		}
	}()

	// Wait for dispatcher to finish
	err := <-dispatcherDone
	wg.Wait()
	return err
}

// cancelDependents recursively cancels all tasks that depend on failedTaskID
// Must be called while holding dm.mu lock
func (dm *DAGManager) cancelDependents(failedTaskID string) int {
	cancelled := 0
	// Use adjacency list for O(1) lookup
	dependents := dm.dependents[failedTaskID]
	for _, depID := range dependents {
		node := dm.Nodes[depID]
		if node == nil || node.Status != Pending {
			continue
		}
		node.Status = Failed
		cancelled++
		log.Printf("[Manager] Task %s cancelled: parent %s failed", depID, failedTaskID)
		// Recursively cancel dependents of this node
		cancelled += dm.cancelDependents(depID)
	}
	return cancelled
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
			// Calculate backoff delay with overflow protection
			exponent := float64(attempt - 1)
			// Cap exponent to prevent overflow: max exponent where 2^n doesn't overflow float64
			const maxExponent = 60
			if exponent > maxExponent {
				exponent = maxExponent
			}
			delay := time.Duration(float64(dm.FailureConfig.RetryDelay) * math.Pow(2, exponent))
			// Cap at reasonable maximum
			const maxDelay = 5 * time.Second
			if delay > maxDelay {
				delay = maxDelay
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
