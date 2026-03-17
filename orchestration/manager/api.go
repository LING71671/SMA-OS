package main

import (
	"encoding/json"
	"net/http"
)

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
}
