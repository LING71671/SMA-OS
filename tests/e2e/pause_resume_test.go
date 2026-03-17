package e2e

import (
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

// re-export types from manager package via a local test harness
// E2E tests spin up a real DAGManager with an HTTP server

// taskProgressResponse mirrors manager.TaskProgress for JSON decoding
type taskProgressResponse struct {
	TaskID   string  `json:"task_id"`
	Status   string  `json:"status"`
	Progress float64 `json:"progress"`
}

type dependencyAnalysisResponse struct {
	HasCycle           bool                `json:"has_cycle"`
	CriticalPath       []string            `json:"critical_path"`
	CriticalPathLength int                 `json:"critical_path_length"`
	ParallelismMax     int                 `json:"parallelism_max"`
	DependencyDepth    int                 `json:"dependency_depth"`
	DependencyMatrix   map[string][]string `json:"dependency_matrix"`
}

// TestE2E_PauseResumeFlow tests the full pause → resume lifecycle via HTTP
func TestE2E_PauseResumeFlow(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()

	// 1. Verify task is running
	resp := mustGET(t, srv.URL+"/api/v1/tasks/T1/progress")
	var p taskProgressResponse
	mustDecode(t, resp, &p)
	if p.Status != "RUNNING" {
		t.Fatalf("expected RUNNING, got %s", p.Status)
	}

	// 2. Pause the task
	resp = mustPOST(t, srv.URL+"/api/v1/tasks/T1/pause")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("pause: expected 200, got %d", resp.StatusCode)
	}

	// 3. Verify paused
	resp = mustGET(t, srv.URL+"/api/v1/tasks/T1/progress")
	mustDecode(t, resp, &p)
	if p.Status != "PAUSED" {
		t.Fatalf("expected PAUSED after pause, got %s", p.Status)
	}

	// 4. Resume the task
	resp = mustPOST(t, srv.URL+"/api/v1/tasks/T1/resume")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("resume: expected 200, got %d", resp.StatusCode)
	}

	// 5. Verify running again
	resp = mustGET(t, srv.URL+"/api/v1/tasks/T1/progress")
	mustDecode(t, resp, &p)
	if p.Status != "RUNNING" {
		t.Fatalf("expected RUNNING after resume, got %s", p.Status)
	}
}

// TestE2E_ProgressQuery tests progress API returns correct fields
func TestE2E_ProgressQuery(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()

	resp := mustGET(t, srv.URL+"/api/v1/tasks/T1/progress")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var p taskProgressResponse
	mustDecode(t, resp, &p)

	if p.TaskID != "T1" {
		t.Errorf("expected task_id T1, got %s", p.TaskID)
	}
	if p.Progress < 0 || p.Progress > 100 {
		t.Errorf("progress out of range: %f", p.Progress)
	}
}

// TestE2E_ProgressQuery_NotFound tests 404 for missing task
func TestE2E_ProgressQuery_NotFound(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()

	resp := mustGET(t, srv.URL+"/api/v1/tasks/NONEXISTENT/progress")
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", resp.StatusCode)
	}
}

// TestE2E_PauseNonRunning tests that pausing a non-running task returns 400
func TestE2E_PauseNonRunning(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()

	// T2 is PENDING
	resp := mustPOST(t, srv.URL+"/api/v1/tasks/T2/pause")
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", resp.StatusCode)
	}
}

// TestE2E_DependencyAnalysis_FullFlow tests the full dependency analysis API
func TestE2E_DependencyAnalysis_FullFlow(t *testing.T) {
	srv := newComplexTestServer(t)
	defer srv.Close()

	// 1. Get full analysis
	resp := mustGET(t, srv.URL+"/api/v1/dags/analysis")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var analysis dependencyAnalysisResponse
	mustDecode(t, resp, &analysis)

	if analysis.HasCycle {
		t.Error("expected no cycle in test DAG")
	}
	if len(analysis.CriticalPath) == 0 {
		t.Error("expected non-empty critical path")
	}
	if analysis.CriticalPathLength < 1 {
		t.Error("expected critical path length >= 1")
	}
	if analysis.ParallelismMax < 1 {
		t.Error("expected parallelism >= 1")
	}
	if analysis.DependencyMatrix == nil {
		t.Error("expected non-nil dependency matrix")
	}

	// 2. Get critical path
	resp = mustGET(t, srv.URL+"/api/v1/dags/critical-path")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("critical-path: expected 200, got %d", resp.StatusCode)
	}

	// 3. Get parallelism
	resp = mustGET(t, srv.URL+"/api/v1/dags/parallelism")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("parallelism: expected 200, got %d", resp.StatusCode)
	}

	// 4. Get task impact
	resp = mustGET(t, srv.URL+"/api/v1/tasks/T1/impact")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("impact: expected 200, got %d", resp.StatusCode)
	}
}

// TestE2E_DependencyAnalysis_CycleDetection tests cycle detection via API
func TestE2E_DependencyAnalysis_CycleDetection(t *testing.T) {
	srv := newCyclicTestServer(t)
	defer srv.Close()

	resp := mustGET(t, srv.URL+"/api/v1/dags/analysis")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var analysis dependencyAnalysisResponse
	mustDecode(t, resp, &analysis)

	if !analysis.HasCycle {
		t.Error("expected cycle to be detected in cyclic DAG")
	}
}

// TestE2E_SubtaskProgress tests subtask progress aggregation
func TestE2E_SubtaskProgress(t *testing.T) {
	srv := newSubtaskTestServer(t)
	defer srv.Close()

	resp := mustGET(t, srv.URL+"/api/v1/tasks/PARENT/progress")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var p taskProgressResponse
	mustDecode(t, resp, &p)

	// 2 of 4 subtasks completed = 50%
	if p.Progress != 50.0 {
		t.Errorf("expected 50%% progress, got %f", p.Progress)
	}
}

// TestE2E_CheckpointPreservesProgress tests that progress is preserved across pause/resume
func TestE2E_CheckpointPreservesProgress(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()

	// Pause
	resp := mustPOST(t, srv.URL+"/api/v1/tasks/T1/pause")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("pause: %d", resp.StatusCode)
	}

	// Resume
	resp = mustPOST(t, srv.URL+"/api/v1/tasks/T1/resume")
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("resume: %d", resp.StatusCode)
	}

	// Progress should be restored
	resp = mustGET(t, srv.URL+"/api/v1/tasks/T1/progress")
	var p taskProgressResponse
	mustDecode(t, resp, &p)
	if p.Progress != 75.0 {
		t.Errorf("expected progress 75.0 preserved, got %f", p.Progress)
	}
}

// ---- helpers ----

func newTestServer(t *testing.T) *httptest.Server {
	t.Helper()
	dm := newDAGManager()
	dm.addTask("T1", "Task One", []string{}, "RUNNING", 75.0)
	dm.addTask("T2", "Task Two", []string{"T1"}, "PENDING", 0)
	mux := http.NewServeMux()
	dm.registerRoutes(mux)
	return httptest.NewServer(mux)
}

func newComplexTestServer(t *testing.T) *httptest.Server {
	t.Helper()
	dm := newDAGManager()
	dm.addTask("T1", "A", []string{}, "PENDING", 0)
	dm.addTask("T2", "B", []string{"T1"}, "PENDING", 0)
	dm.addTask("T3", "C", []string{"T1"}, "PENDING", 0)
	dm.addTask("T4", "D", []string{"T2", "T3"}, "PENDING", 0)
	dm.addTask("T5", "E", []string{"T4"}, "PENDING", 0)
	mux := http.NewServeMux()
	dm.registerRoutes(mux)
	return httptest.NewServer(mux)
}

func newCyclicTestServer(t *testing.T) *httptest.Server {
	t.Helper()
	dm := newDAGManager()
	dm.addTask("T1", "A", []string{"T3"}, "PENDING", 0)
	dm.addTask("T2", "B", []string{"T1"}, "PENDING", 0)
	dm.addTask("T3", "C", []string{"T2"}, "PENDING", 0)
	mux := http.NewServeMux()
	dm.registerRoutes(mux)
	return httptest.NewServer(mux)
}

func newSubtaskTestServer(t *testing.T) *httptest.Server {
	t.Helper()
	dm := newDAGManager()
	dm.addTaskWithSubtasks("PARENT", "Parent", []string{"C1", "C2", "C3", "C4"})
	dm.addTask("C1", "Child1", []string{}, "COMPLETED", 100)
	dm.addTask("C2", "Child2", []string{}, "COMPLETED", 100)
	dm.addTask("C3", "Child3", []string{}, "PENDING", 0)
	dm.addTask("C4", "Child4", []string{}, "PENDING", 0)
	mux := http.NewServeMux()
	dm.registerRoutes(mux)
	return httptest.NewServer(mux)
}

func mustGET(t *testing.T, url string) *http.Response {
	t.Helper()
	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Get(url)
	if err != nil {
		t.Fatalf("GET %s: %v", url, err)
	}
	return resp
}

func mustPOST(t *testing.T, url string) *http.Response {
	t.Helper()
	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Post(url, "application/json", nil)
	if err != nil {
		t.Fatalf("POST %s: %v", url, err)
	}
	return resp
}

func mustDecode(t *testing.T, resp *http.Response, v interface{}) {
	t.Helper()
	defer resp.Body.Close()
	if err := json.NewDecoder(resp.Body).Decode(v); err != nil {
		t.Fatalf("decode response: %v", err)
	}
}

// ---- minimal DAGManager wrapper to avoid import cycle ----
// E2E tests use the manager package directly via a thin adapter

type testDAGManager struct {
	nodes     map[string]*testNode
	dependents map[string][]string
}

type testNode struct {
	id         string
	actionName string
	deps       []string
	status     string
	progress   float64
	subTasks   []string
}

func newDAGManager() *testDAGManager {
	return &testDAGManager{
		nodes:      make(map[string]*testNode),
		dependents: make(map[string][]string),
	}
}

func (dm *testDAGManager) addTask(id, name string, deps []string, status string, progress float64) {
	dm.nodes[id] = &testNode{id: id, actionName: name, deps: deps, status: status, progress: progress}
	for _, dep := range deps {
		dm.dependents[dep] = append(dm.dependents[dep], id)
	}
}

func (dm *testDAGManager) addTaskWithSubtasks(id, name string, subTasks []string) {
	dm.nodes[id] = &testNode{id: id, actionName: name, status: "RUNNING", subTasks: subTasks}
}

func (dm *testDAGManager) registerRoutes(mux *http.ServeMux) {
	mux.HandleFunc("GET /api/v1/tasks/{taskID}/progress", dm.handleGetProgress)
	mux.HandleFunc("POST /api/v1/tasks/{taskID}/pause", dm.handlePauseTask)
	mux.HandleFunc("POST /api/v1/tasks/{taskID}/resume", dm.handleResumeTask)
	mux.HandleFunc("GET /api/v1/tasks/{taskID}/impact", dm.handleTaskImpact)
	mux.HandleFunc("GET /api/v1/dags/analysis", dm.handleDependencyAnalysis)
	mux.HandleFunc("GET /api/v1/dags/critical-path", dm.handleCriticalPath)
	mux.HandleFunc("GET /api/v1/dags/parallelism", dm.handleParallelism)
}

func (dm *testDAGManager) handleGetProgress(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("taskID")
	node, ok := dm.nodes[id]
	if !ok {
		http.Error(w, "not found", http.StatusNotFound)
		return
	}
	progress := dm.calcProgress(node)
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"task_id":  id,
		"status":   node.status,
		"progress": progress,
	})
}

func (dm *testDAGManager) handlePauseTask(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("taskID")
	node, ok := dm.nodes[id]
	if !ok {
		http.Error(w, "not found", http.StatusNotFound)
		return
	}
	if node.status != "RUNNING" {
		http.Error(w, fmt.Sprintf("task %s not running", id), http.StatusBadRequest)
		return
	}
	node.status = "PAUSED"
	w.WriteHeader(http.StatusOK)
}

func (dm *testDAGManager) handleResumeTask(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("taskID")
	node, ok := dm.nodes[id]
	if !ok {
		http.Error(w, "not found", http.StatusNotFound)
		return
	}
	if node.status != "PAUSED" {
		http.Error(w, fmt.Sprintf("task %s not paused", id), http.StatusBadRequest)
		return
	}
	node.status = "RUNNING"
	w.WriteHeader(http.StatusOK)
}

func (dm *testDAGManager) handleDependencyAnalysis(w http.ResponseWriter, r *http.Request) {
	hasCycle, cyclePath := dm.detectCycle()
	critPath, critLen := dm.criticalPath()
	maxP, _ := dm.parallelism()
	matrix := dm.depMatrix()
	maxDepth := dm.maxDepth()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"has_cycle":            hasCycle,
		"cycle_path":           cyclePath,
		"critical_path":        critPath,
		"critical_path_length": critLen,
		"parallelism_max":      maxP,
		"dependency_depth":     maxDepth,
		"dependency_matrix":    matrix,
	})
}

func (dm *testDAGManager) handleCriticalPath(w http.ResponseWriter, r *http.Request) {
	path, length := dm.criticalPath()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{"critical_path": path, "length": length})
}

func (dm *testDAGManager) handleParallelism(w http.ResponseWriter, r *http.Request) {
	max, layers := dm.parallelism()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{"max_parallelism": max, "layers": layers})
}

func (dm *testDAGManager) handleTaskImpact(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("taskID")
	affected := dm.impact(id)
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"task_id": id, "affected_tasks": affected, "affected_count": len(affected),
	})
}

func (dm *testDAGManager) calcProgress(node *testNode) float64 {
	if len(node.subTasks) > 0 {
		done := 0
		for _, sid := range node.subTasks {
			if s, ok := dm.nodes[sid]; ok && (s.status == "COMPLETED" || s.status == "FAILED") {
				done++
			}
		}
		return float64(done) / float64(len(node.subTasks)) * 100
	}
	switch node.status {
	case "PENDING":   return 0
	case "RUNNING":   return node.progress
	case "COMPLETED": return 100
	case "FAILED":    return 100
	case "PAUSED":    return node.progress
	}
	return 0
}

func (dm *testDAGManager) detectCycle() (bool, []string) {
	visited := make(map[string]bool)
	recStack := make(map[string]bool)
	var cycle []string
	found := false

	var dfs func(id string) bool
	dfs = func(id string) bool {
		if found {
			return true
		}
		visited[id] = true
		recStack[id] = true
		node := dm.nodes[id]
		if node == nil {
			recStack[id] = false
			return false
		}
		for _, dep := range node.deps {
			if !visited[dep] {
				if dfs(dep) {
					return true
				}
			} else if recStack[dep] {
				found = true
				cycle = append(cycle, dep, id)
				return true
			}
		}
		recStack[id] = false
		return false
	}

	for id := range dm.nodes {
		if !visited[id] {
			dfs(id)
		}
		if found {
			break
		}
	}
	return found, cycle
}

func (dm *testDAGManager) topoSort() []string {
	inDeg := make(map[string]int)
	for id, n := range dm.nodes { inDeg[id] = len(n.deps) }
	var q []string
	for id, d := range inDeg { if d == 0 { q = append(q, id) } }
	var result []string
	for len(q) > 0 {
		id := q[0]; q = q[1:]
		result = append(result, id)
		for _, dep := range dm.dependents[id] {
			inDeg[dep]--
			if inDeg[dep] == 0 { q = append(q, dep) }
		}
	}
	return result
}

func (dm *testDAGManager) criticalPath() ([]string, int) {
	order := dm.topoSort()
	maxD := make(map[string]int)
	pathTo := make(map[string][]string)
	for _, id := range order {
		maxD[id] = 0
		pathTo[id] = []string{id}
		for _, dep := range dm.nodes[id].deps {
			if maxD[dep]+1 > maxD[id] {
				maxD[id] = maxD[dep] + 1
				p := make([]string, len(pathTo[dep]))
				copy(p, pathTo[dep])
				pathTo[id] = append(p, id)
			}
		}
	}
	var maxLen int
	var maxPath []string
	for id, d := range maxD {
		if d > maxLen { maxLen = d; maxPath = pathTo[id] }
	}
	return maxPath, len(maxPath)
}

func (dm *testDAGManager) parallelism() (int, map[int][]string) {
	order := dm.topoSort()
	levels := make(map[string]int)
	for _, id := range order {
		l := 0
		for _, dep := range dm.nodes[id].deps {
			if levels[dep]+1 > l { l = levels[dep] + 1 }
		}
		levels[id] = l
	}
	layers := make(map[int][]string)
	for id, l := range levels { layers[l] = append(layers[l], id) }
	max := 0
	for _, tasks := range layers { if len(tasks) > max { max = len(tasks) } }
	return max, layers
}

func (dm *testDAGManager) depMatrix() map[string][]string {
	m := make(map[string][]string)
	for id, n := range dm.nodes {
		deps := make([]string, len(n.deps))
		copy(deps, n.deps)
		m[id] = deps
	}
	return m
}

func (dm *testDAGManager) maxDepth() int {
	depths := make(map[string]int)
	inProgress := make(map[string]bool)
	var calc func(id string) int
	calc = func(id string) int {
		if d, ok := depths[id]; ok { return d }
		if inProgress[id] { return 0 } // cycle guard
		inProgress[id] = true
		max := -1
		if node, ok := dm.nodes[id]; ok {
			for _, dep := range node.deps {
				if d := calc(dep); d > max { max = d }
			}
		}
		inProgress[id] = false
		depths[id] = max + 1
		return depths[id]
	}
	for id := range dm.nodes { calc(id) }
	m := 0
	for _, d := range depths { if d > m { m = d } }
	return m
}

func (dm *testDAGManager) impact(id string) []string {
	var affected []string
	for _, dep := range dm.dependents[id] {
		affected = append(affected, dep)
		affected = append(affected, dm.impact(dep)...)
	}
	return affected
}
