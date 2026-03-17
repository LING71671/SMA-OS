package main

import (
	"testing"
)

// helpers

func buildLinear(dm *DAGManager) {
	// A -> B -> C -> D
	dm.AddTask(TaskNode{ID: "A", ActionName: "A", Status: Pending, Dependencies: []string{}})
	dm.AddTask(TaskNode{ID: "B", ActionName: "B", Status: Pending, Dependencies: []string{"A"}})
	dm.AddTask(TaskNode{ID: "C", ActionName: "C", Status: Pending, Dependencies: []string{"B"}})
	dm.AddTask(TaskNode{ID: "D", ActionName: "D", Status: Pending, Dependencies: []string{"C"}})
}

func buildDiamond(dm *DAGManager) {
	// A -> B, A -> C, B -> D, C -> D
	dm.AddTask(TaskNode{ID: "A", ActionName: "A", Status: Pending, Dependencies: []string{}})
	dm.AddTask(TaskNode{ID: "B", ActionName: "B", Status: Pending, Dependencies: []string{"A"}})
	dm.AddTask(TaskNode{ID: "C", ActionName: "C", Status: Pending, Dependencies: []string{"A"}})
	dm.AddTask(TaskNode{ID: "D", ActionName: "D", Status: Pending, Dependencies: []string{"B", "C"}})
}

// --- Cycle detection ---

func TestDetectCycle_NoCycle(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	buildLinear(dm)

	hasCycle, path := dm.DetectCycle()
	if hasCycle {
		t.Errorf("expected no cycle, got path %v", path)
	}
}

func TestDetectCycle_SimpleLoop(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	// A -> B -> C -> A (cycle)
	dm.AddTask(TaskNode{ID: "A", ActionName: "A", Status: Pending, Dependencies: []string{"C"}})
	dm.AddTask(TaskNode{ID: "B", ActionName: "B", Status: Pending, Dependencies: []string{"A"}})
	dm.AddTask(TaskNode{ID: "C", ActionName: "C", Status: Pending, Dependencies: []string{"B"}})

	hasCycle, path := dm.DetectCycle()
	if !hasCycle {
		t.Error("expected cycle to be detected")
	}
	if len(path) == 0 {
		t.Error("expected non-empty cycle path")
	}
}

func TestDetectCycle_ComplexGraph(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	// 10 tasks, 3 form a cycle
	for _, id := range []string{"T1", "T2", "T3", "T4", "T5", "T6", "T7", "T8", "T9", "T10"} {
		dm.AddTask(TaskNode{ID: id, ActionName: id, Status: Pending, Dependencies: []string{}})
	}
	// Normal deps
	dm.Nodes["T2"].Dependencies = []string{"T1"}
	dm.Nodes["T4"].Dependencies = []string{"T3"}
	// Cycle: T8 -> T9 -> T10 -> T8
	dm.Nodes["T9"].Dependencies = []string{"T8"}
	dm.Nodes["T10"].Dependencies = []string{"T9"}
	dm.Nodes["T8"].Dependencies = []string{"T10"}

	hasCycle, _ := dm.DetectCycle()
	if !hasCycle {
		t.Error("expected cycle in complex graph")
	}
}

// --- Critical path ---

func TestCalculateCriticalPath_Linear(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	buildLinear(dm)

	path, length := dm.CalculateCriticalPath()
	if length != 4 {
		t.Errorf("expected length 4, got %d (path: %v)", length, path)
	}
}

func TestCalculateCriticalPath_Diamond(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	buildDiamond(dm)

	path, length := dm.CalculateCriticalPath()
	if length < 3 {
		t.Errorf("expected length >= 3, got %d (path: %v)", length, path)
	}
}

func TestCalculateCriticalPath_Single(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	dm.AddTask(TaskNode{ID: "X", ActionName: "X", Status: Pending, Dependencies: []string{}})

	path, length := dm.CalculateCriticalPath()
	if length != 1 {
		t.Errorf("expected length 1, got %d (path: %v)", length, path)
	}
}

// --- Parallelism ---

func TestCalculateParallelism(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	// Layer 0: A
	// Layer 1: B, C, D (all depend on A)
	// Layer 2: E, F, G, H, I (all depend on B/C/D)
	dm.AddTask(TaskNode{ID: "A", ActionName: "A", Status: Pending, Dependencies: []string{}})
	for _, id := range []string{"B", "C", "D"} {
		dm.AddTask(TaskNode{ID: id, ActionName: id, Status: Pending, Dependencies: []string{"A"}})
	}
	for _, id := range []string{"E", "F", "G", "H", "I"} {
		dm.AddTask(TaskNode{ID: id, ActionName: id, Status: Pending, Dependencies: []string{"B"}})
	}

	max, layers := dm.CalculateParallelism()
	if max < 3 {
		t.Errorf("expected max parallelism >= 3, got %d (layers: %v)", max, layers)
	}
}

// --- Dependency depth ---

func TestCalculateDependencyDepth(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	buildLinear(dm)

	depths := dm.CalculateDependencyDepth()
	if depths["A"] != 0 {
		t.Errorf("expected A depth 0, got %d", depths["A"])
	}
	if depths["B"] != 1 {
		t.Errorf("expected B depth 1, got %d", depths["B"])
	}
	if depths["C"] != 2 {
		t.Errorf("expected C depth 2, got %d", depths["C"])
	}
	if depths["D"] != 3 {
		t.Errorf("expected D depth 3, got %d", depths["D"])
	}
}

// --- Impact map ---

func TestCalculateImpactMap(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	// A -> B, A -> C, B -> D
	dm.AddTask(TaskNode{ID: "A", ActionName: "A", Status: Pending, Dependencies: []string{}})
	dm.AddTask(TaskNode{ID: "B", ActionName: "B", Status: Pending, Dependencies: []string{"A"}})
	dm.AddTask(TaskNode{ID: "C", ActionName: "C", Status: Pending, Dependencies: []string{"A"}})
	dm.AddTask(TaskNode{ID: "D", ActionName: "D", Status: Pending, Dependencies: []string{"B"}})

	impact := dm.CalculateImpactMap()
	aImpact := impact["A"]
	if len(aImpact) < 3 {
		t.Errorf("expected A to affect at least B, C, D; got %v", aImpact)
	}
}

// --- Full analysis ---

func TestAnalyzeDependencies_FullAnalysis(t *testing.T) {
	dm := NewDAGManager(DefaultFailureConfig())
	buildDiamond(dm)

	result := dm.AnalyzeDependencies()
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	if result.HasCycle {
		t.Error("diamond should not have cycle")
	}
	if len(result.CriticalPath) == 0 {
		t.Error("expected non-empty critical path")
	}
	if result.ParallelismMax < 1 {
		t.Error("expected parallelism >= 1")
	}
	if result.DependencyMatrix == nil {
		t.Error("expected non-nil dependency matrix")
	}
	if result.Graph == nil {
		t.Error("expected non-nil graph")
	}
}
