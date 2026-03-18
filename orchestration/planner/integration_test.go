package planner

import (
	"context"
	"testing"
)

// TestIntegration_DecomposeAndConvert tests the full planner pipeline:
// ParsedIntent → Decompose → []TaskNode (ready for DAG manager).
func TestIntegration_DecomposeAndConvert(t *testing.T) {
	llm := &mockLLM{
		response: `[
			{"id":"T1","action_name":"validate","description":"validate intent","dependencies":[],"priority":1},
			{"id":"T2","action_name":"prepare","description":"prepare resources","dependencies":["T1"],"priority":2},
			{"id":"T3","action_name":"execute","description":"execute action","dependencies":["T2"],"priority":3}
		]`,
	}

	decomposer := NewLLMDecomposer(llm, 5, 20)
	bridge := NewBridge(decomposer)

	req := DecompositionRequest{
		Intent: ParsedIntent{
			Action:     "create_vm",
			Target:     "pool-A",
			Parameters: "cpu=2,ram=4G",
		},
		MaxDepth:    3,
		MaxSubTasks: 20,
	}

	nodes, rootID, err := bridge.Decompose(context.Background(), req)
	if err != nil {
		t.Fatalf("Decompose failed: %v", err)
	}

	if len(nodes) != 3 {
		t.Fatalf("expected 3 nodes, got %d", len(nodes))
	}
	if rootID != "T1" {
		t.Errorf("expected root T1, got %s", rootID)
	}

	// Verify all nodes are Pending
	for _, n := range nodes {
		if n.Status != Pending {
			t.Errorf("node %s: expected Pending, got %s", n.ID, n.Status)
		}
	}

	// Verify dependency chain T1 → T2 → T3
	depMap := make(map[string][]string)
	for _, n := range nodes {
		depMap[n.ID] = n.Dependencies
	}
	if len(depMap["T1"]) != 0 {
		t.Errorf("T1 should have no dependencies")
	}
	if len(depMap["T2"]) != 1 || depMap["T2"][0] != "T1" {
		t.Errorf("T2 should depend on T1, got %v", depMap["T2"])
	}
	if len(depMap["T3"]) != 1 || depMap["T3"][0] != "T2" {
		t.Errorf("T3 should depend on T2, got %v", depMap["T3"])
	}
}

// TestIntegration_ConflictDetectionAfterDecompose verifies conflict detection
// runs cleanly on a valid decomposition result.
func TestIntegration_ConflictDetectionAfterDecompose(t *testing.T) {
	llm := &mockLLM{
		response: `[
			{"id":"T1","action_name":"init","description":"initialise pool-A","dependencies":[],"priority":1},
			{"id":"T2","action_name":"run","description":"run on pool-B","dependencies":["T1"],"priority":2}
		]`,
	}

	decomposer := NewLLMDecomposer(llm, 5, 20)
	result, err := decomposer.Decompose(context.Background(), DecompositionRequest{
		Intent: ParsedIntent{Action: "deploy", Target: "cluster"},
	})
	if err != nil {
		t.Fatalf("Decompose failed: %v", err)
	}

	detector := &ConflictDetector{}
	conflicts := detector.Detect(result.Tasks)

	for _, c := range conflicts {
		if c.Severity == Critical {
			t.Errorf("unexpected critical conflict: %+v", c)
		}
	}
}

// TestIntegration_CycleRejected verifies that a cyclic LLM response is rejected.
func TestIntegration_CycleRejected(t *testing.T) {
	llm := &mockLLM{
		response: `[
			{"id":"T1","action_name":"a","description":"","dependencies":["T3"],"priority":1},
			{"id":"T2","action_name":"b","description":"","dependencies":["T1"],"priority":1},
			{"id":"T3","action_name":"c","description":"","dependencies":["T2"],"priority":1}
		]`,
	}
	decomposer := NewLLMDecomposer(llm, 5, 20)
	_, err := decomposer.Decompose(context.Background(), DecompositionRequest{
		Intent: ParsedIntent{Action: "test"},
	})
	if err == nil {
		t.Fatal("expected cycle error to be returned")
	}
}
