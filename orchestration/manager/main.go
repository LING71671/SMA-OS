package main

import (
	"encoding/json"
	"log"
	"sync"
	"time"
)

// TaskStatus represents the lifecycle of a task
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
	Nodes    map[string]*TaskNode
	inDegree map[string]int
	mu       sync.Mutex
}

func NewDAGManager() *DAGManager {
	return &DAGManager{
		Nodes:    make(map[string]*TaskNode),
		inDegree: make(map[string]int),
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
	completionChan := make(chan string, len(dm.Nodes))
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
	completedTasks := 0

	// 2. Dispatch loop
	go func() {
		for {
			select {
			case task := <-readyQueue:
				wg.Add(1)
				// Simulating Worker assignment (e.g. gRPC to Firecracker MicroVM pool)
				go dm.dispatchWorker(task, completionChan, &wg)

			case completedID := <-completionChan:
				completedTasks++
				log.Printf("[Manager] Registered COMPLETION event for Task: %s", completedID)

				// Calculate dependents and decrement their in-degrees
				dm.mu.Lock()
				for id, node := range dm.Nodes {
					if node.Status == Pending {
						// Check if completedID is in dependencies
						hasDep := false
						for _, d := range node.Dependencies {
							if d == completedID {
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

				if completedTasks == totalTasks {
					log.Println("[Manager] All tasks in DAG completed successfully!")
					return // End execution mapping
				}
			}
		}
	}()

	// Wait for all actual worker routines to finish
	// (in a real scenario we'd track the overarching DAG finish)
	// We use sleep here to let the dispatch daemon finish for the demo.
	time.Sleep(2 * time.Second)
	wg.Wait()
	return nil
}

func (dm *DAGManager) dispatchWorker(task *TaskNode, done chan<- string, wg *sync.WaitGroup) {
	defer wg.Done()

	task.Status = Running
	log.Printf("[Worker Scheduler] -> Dispatching Task [%s] (%s) to Firecracker Warm Pool...", task.ID, task.ActionName)

	// Simulate work duration
	time.Sleep(500 * time.Millisecond)

	task.Status = Completed
	log.Printf("[Worker Scheduler] <- Task [%s] completed successfully. Cascading state...", task.ID)

	done <- task.ID
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

	manager := NewDAGManager()
	for _, t := range tasks {
		t.Status = Pending
		manager.AddTask(t)
	}

	// Begin High-Performance execution
	manager.Execute()
}
