package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func setupTestManager() *DAGManager {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "T1", ActionName: "A", Status: Running, Progress: 50.0})
	dm.AddTask(TaskNode{ID: "T2", ActionName: "B", Status: Pending, Dependencies: []string{"T1"}})
	return dm
}

func TestHandleGetProgress_Found(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("GET", "/api/v1/tasks/T1/progress", nil)
	req.SetPathValue("taskID", "T1")
	w := httptest.NewRecorder()

	dm.HandleGetProgress(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", w.Code)
	}
	var p TaskProgress
	if err := json.NewDecoder(w.Body).Decode(&p); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if p.TaskID != "T1" {
		t.Errorf("expected task_id T1, got %s", p.TaskID)
	}
}

func TestHandleGetProgress_NotFound(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("GET", "/api/v1/tasks/MISSING/progress", nil)
	req.SetPathValue("taskID", "MISSING")
	w := httptest.NewRecorder()

	dm.HandleGetProgress(w, req)

	if w.Code != http.StatusNotFound {
		t.Errorf("expected 404, got %d", w.Code)
	}
}

func TestHandlePauseTask_Success(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("POST", "/api/v1/tasks/T1/pause", nil)
	req.SetPathValue("taskID", "T1")
	w := httptest.NewRecorder()

	dm.HandlePauseTask(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", w.Code)
	}
}

func TestHandlePauseTask_BadRequest(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("POST", "/api/v1/tasks/T2/pause", nil)
	req.SetPathValue("taskID", "T2")
	w := httptest.NewRecorder()

	dm.HandlePauseTask(w, req)

	if w.Code != http.StatusBadRequest {
		t.Errorf("expected 400, got %d", w.Code)
	}
}

func TestHandleResumeTask_Success(t *testing.T) {
	dm := setupTestManager()
	// Pause first
	dm.Nodes["T1"].Status = Paused
	globalCheckpointStore.mu.Lock()
	cp, _ := NewCheckpoint(99, dm.Nodes["T1"])
	globalCheckpointStore.data["T1"] = cp
	globalCheckpointStore.mu.Unlock()

	req := httptest.NewRequest("POST", "/api/v1/tasks/T1/resume", nil)
	req.SetPathValue("taskID", "T1")
	w := httptest.NewRecorder()

	dm.HandleResumeTask(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", w.Code)
	}
}

func TestHandleResumeTask_NotFound(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("POST", "/api/v1/tasks/MISSING/resume", nil)
	req.SetPathValue("taskID", "MISSING")
	w := httptest.NewRecorder()

	dm.HandleResumeTask(w, req)

	if w.Code != http.StatusNotFound {
		t.Errorf("expected 404, got %d", w.Code)
	}
}

func TestHandleResumeTask_NotPaused(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("POST", "/api/v1/tasks/T1/resume", nil)
	req.SetPathValue("taskID", "T1")
	w := httptest.NewRecorder()

	dm.HandleResumeTask(w, req)

	if w.Code != http.StatusBadRequest {
		t.Errorf("expected 400, got %d", w.Code)
	}
}

func TestHandleDependencyAnalysis(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("GET", "/api/v1/dags/analysis", nil)
	w := httptest.NewRecorder()

	dm.HandleDependencyAnalysis(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", w.Code)
	}
	var result DependencyAnalysis
	if err := json.NewDecoder(w.Body).Decode(&result); err != nil {
		t.Fatalf("decode: %v", err)
	}
}

func TestHandleCriticalPath(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("GET", "/api/v1/dags/critical-path", nil)
	w := httptest.NewRecorder()

	dm.HandleCriticalPath(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", w.Code)
	}
}

func TestHandleParallelism(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("GET", "/api/v1/dags/parallelism", nil)
	w := httptest.NewRecorder()

	dm.HandleParallelism(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", w.Code)
	}
}

func TestHandleTaskImpact(t *testing.T) {
	dm := setupTestManager()
	req := httptest.NewRequest("GET", "/api/v1/tasks/T1/impact", nil)
	req.SetPathValue("taskID", "T1")
	w := httptest.NewRecorder()

	dm.HandleTaskImpact(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", w.Code)
	}
}

func TestRegisterRoutes(t *testing.T) {
	dm := setupTestManager()
	mux := http.NewServeMux()
	dm.RegisterRoutes(mux) // just verify no panic
}

func TestCancelDependents(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "A", ActionName: "A", Status: Pending, Dependencies: []string{}})
	dm.AddTask(TaskNode{ID: "B", ActionName: "B", Status: Pending, Dependencies: []string{"A"}})
	dm.AddTask(TaskNode{ID: "C", ActionName: "C", Status: Pending, Dependencies: []string{"B"}})

	dm.mu.Lock()
	cancelled := dm.cancelDependents("A")
	dm.mu.Unlock()

	if cancelled != 2 {
		t.Errorf("expected 2 cancelled, got %d", cancelled)
	}
	if dm.Nodes["B"].Status != Failed {
		t.Errorf("expected B Failed, got %s", dm.Nodes["B"].Status)
	}
	if dm.Nodes["C"].Status != Failed {
		t.Errorf("expected C Failed, got %s", dm.Nodes["C"].Status)
	}
}
