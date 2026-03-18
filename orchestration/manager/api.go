package main

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"
	"time"
)

// DecompositionRequest is the HTTP body for POST /api/v1/tasks/decompose.
type DecompositionRequest struct {
	Intent      ParsedIntent `json:"intent"`
	MaxDepth    int          `json:"max_depth"`
	MaxSubTasks int          `json:"max_sub_tasks"`
}

// ParsedIntent mirrors memory-bus/ingestion's ParsedIntent.
type ParsedIntent struct {
	Action     string  `json:"action"`
	Target     string  `json:"target"`
	Parameters string  `json:"parameters"`
	Confidence float64 `json:"confidence"`
	Source     string  `json:"source"`
}

// DecomposedTask is a single task returned by the inline decomposer.
type DecomposedTask struct {
	ID           string   `json:"id"`
	ActionName   string   `json:"action_name"`
	Description  string   `json:"description"`
	Dependencies []string `json:"dependencies"`
	Priority     int      `json:"priority"`
}

// DecompositionResponse is the HTTP response for POST /api/v1/tasks/decompose.
type DecompositionResponse struct {
	RootTaskID string           `json:"root_task_id"`
	Tasks      []DecomposedTask `json:"tasks"`
	Duration   string           `json:"duration"`
}

// HandleDecomposeTask handles POST /api/v1/tasks/decompose.
// It performs a rule-based decomposition of the intent and adds the resulting
// TaskNodes to the DAG manager, returning the decomposition result.
func (dm *DAGManager) HandleDecomposeTask(w http.ResponseWriter, r *http.Request) {
	var req DecompositionRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body: "+err.Error(), http.StatusBadRequest)
		return
	}
	if req.Intent.Action == "" {
		http.Error(w, "intent.action is required", http.StatusBadRequest)
		return
	}
	if req.MaxDepth <= 0 {
		req.MaxDepth = 5
	}
	if req.MaxSubTasks <= 0 {
		req.MaxSubTasks = 20
	}

	start := time.Now()
	ctx, cancel := context.WithTimeout(r.Context(), 60*time.Second)
	defer cancel()
	_ = ctx

	tasks := decomposeIntent(req)
	if len(tasks) > req.MaxSubTasks {
		tasks = tasks[:req.MaxSubTasks]
	}

	nodes := make([]TaskNode, len(tasks))
	for i, t := range tasks {
		nodes[i] = TaskNode{
			ID:           t.ID,
			ActionName:   t.ActionName,
			Dependencies: t.Dependencies,
			Status:       Pending,
			Payload:      t.Description,
			IsAtomic:     true,
		}
	}

	if err := dm.AddTasksFromIntent(nodes); err != nil {
		http.Error(w, "failed to add tasks: "+err.Error(), http.StatusInternalServerError)
		return
	}

	rootID := ""
	for _, t := range tasks {
		if len(t.Dependencies) == 0 {
			rootID = t.ID
			break
		}
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(DecompositionResponse{
		RootTaskID: rootID,
		Tasks:      tasks,
		Duration:   time.Since(start).String(),
	})
}

// decomposeIntent produces a minimal deterministic task DAG from a DecompositionRequest.
// In production this would delegate to an LLM; here it provides a sensible default.
func decomposeIntent(req DecompositionRequest) []DecomposedTask {
	action := strings.ToLower(req.Intent.Action)
	target := req.Intent.Target
	params := req.Intent.Parameters

	return []DecomposedTask{
		{
			ID:           "T1",
			ActionName:   "validate_" + action,
			Description:  "Validate preconditions for " + action + " on " + target,
			Dependencies: []string{},
			Priority:     1,
		},
		{
			ID:           "T2",
			ActionName:   "prepare_" + action,
			Description:  "Prepare resources for " + action + " with params: " + params,
			Dependencies: []string{"T1"},
			Priority:     2,
		},
		{
			ID:           "T3",
			ActionName:   action,
			Description:  "Execute " + action + " on " + target,
			Dependencies: []string{"T2"},
			Priority:     3,
		},
		{
			ID:           "T4",
			ActionName:   "verify_" + action,
			Description:  "Verify result of " + action + " on " + target,
			Dependencies: []string{"T3"},
			Priority:     4,
		},
	}
}

// HandleGetProgress handles GET /api/v1/tasks/{taskID}/progress
func (dm *DAGManager) HandleGetProgress(w http.ResponseWriter, r *http.Request) {
	taskID := r.PathValue("taskID")
	dm.mu.Lock()
	_, exists := dm.Nodes[taskID]
	dm.mu.Unlock()
	if !exists {
		http.Error(w, "task not found", http.StatusNotFound)
		return
	}
	progress := dm.GetProgress(taskID)
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(progress)
}

// HandlePauseTask handles POST /api/v1/tasks/{taskID}/pause
func (dm *DAGManager) HandlePauseTask(w http.ResponseWriter, r *http.Request) {
	taskID := r.PathValue("taskID")
	if err := dm.PauseTask(taskID); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	w.WriteHeader(http.StatusOK)
}

// HandleResumeTask handles POST /api/v1/tasks/{taskID}/resume
func (dm *DAGManager) HandleResumeTask(w http.ResponseWriter, r *http.Request) {
	taskID := r.PathValue("taskID")
	// readyQueue not accessible here; resume is a no-op re-enqueue in HTTP context
	// Callers should use the DAGManager.ResumeTask directly with a queue reference
	dm.mu.Lock()
	task := dm.Nodes[taskID]
	dm.mu.Unlock()
	if task == nil {
		http.Error(w, "task not found", http.StatusNotFound)
		return
	}
	if task.Status != Paused {
		http.Error(w, "task not paused", http.StatusBadRequest)
		return
	}
	dm.mu.Lock()
	err := dm.restoreFromCheckpoint(taskID)
	if err == nil {
		task.Status = Running
	}
	dm.mu.Unlock()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.WriteHeader(http.StatusOK)
}

// HandleDependencyAnalysis handles GET /api/v1/dags/analysis
func (dm *DAGManager) HandleDependencyAnalysis(w http.ResponseWriter, r *http.Request) {
	analysis := dm.AnalyzeDependencies()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(analysis)
}

// HandleCriticalPath handles GET /api/v1/dags/critical-path
func (dm *DAGManager) HandleCriticalPath(w http.ResponseWriter, r *http.Request) {
	path, length := dm.CalculateCriticalPath()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"critical_path": path,
		"length":        length,
	})
}

// HandleParallelism handles GET /api/v1/dags/parallelism
func (dm *DAGManager) HandleParallelism(w http.ResponseWriter, r *http.Request) {
	max, layers := dm.CalculateParallelism()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"max_parallelism": max,
		"layers":          layers,
	})
}

// HandleTaskImpact handles GET /api/v1/tasks/{taskID}/impact
func (dm *DAGManager) HandleTaskImpact(w http.ResponseWriter, r *http.Request) {
	taskID := r.PathValue("taskID")
	impact := dm.CalculateImpactMap()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"task_id":        taskID,
		"affected_tasks": impact[taskID],
		"affected_count": len(impact[taskID]),
	})
}

// RegisterRoutes registers all task management HTTP routes on the given mux.
func (dm *DAGManager) RegisterRoutes(mux *http.ServeMux) {
	mux.HandleFunc("GET /api/v1/tasks/{taskID}/progress", dm.HandleGetProgress)
	mux.HandleFunc("POST /api/v1/tasks/{taskID}/pause", dm.HandlePauseTask)
	mux.HandleFunc("POST /api/v1/tasks/{taskID}/resume", dm.HandleResumeTask)
	mux.HandleFunc("GET /api/v1/tasks/{taskID}/impact", dm.HandleTaskImpact)
	mux.HandleFunc("GET /api/v1/dags/analysis", dm.HandleDependencyAnalysis)
	mux.HandleFunc("GET /api/v1/dags/critical-path", dm.HandleCriticalPath)
	mux.HandleFunc("GET /api/v1/dags/parallelism", dm.HandleParallelism)
	mux.HandleFunc("POST /api/v1/tasks/decompose", dm.HandleDecomposeTask)
}
