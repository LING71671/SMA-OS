package planner

import (
	"context"
	"testing"
)

func TestBridge_Decompose_Success(t *testing.T) {
	llm := &mockLLM{
		response: `[
			{"id":"T1","action_name":"init","description":"initialise","dependencies":[],"priority":1},
			{"id":"T2","action_name":"run","description":"execute","dependencies":["T1"],"priority":2}
		]`,
	}
	bridge := NewBridge(NewLLMDecomposer(llm, 5, 20))
	nodes, rootID, err := bridge.Decompose(context.Background(), DecompositionRequest{
		Intent: ParsedIntent{Action: "deploy", Target: "service-A"},
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(nodes) != 2 {
		t.Fatalf("expected 2 nodes, got %d", len(nodes))
	}
	if rootID != "T1" {
		t.Errorf("expected root T1, got %s", rootID)
	}
	if nodes[0].Status != Pending {
		t.Errorf("expected Pending status, got %s", nodes[0].Status)
	}
}

func TestDecomposedTaskToTaskNode_Mapping(t *testing.T) {
	tasks := []DecomposedTask{
		{ID: "T1", ActionName: "step", Description: "do it", Dependencies: []string{"T0"}},
	}
	nodes := DecomposedTaskToTaskNode(tasks)
	if len(nodes) != 1 {
		t.Fatalf("expected 1 node")
	}
	n := nodes[0]
	if n.ID != "T1" || n.ActionName != "step" || n.Payload != "do it" {
		t.Errorf("mapping incorrect: %+v", n)
	}
	if len(n.Dependencies) != 1 || n.Dependencies[0] != "T0" {
		t.Errorf("dependencies not mapped: %v", n.Dependencies)
	}
	if !n.IsAtomic {
		t.Error("expected IsAtomic=true")
	}
}
