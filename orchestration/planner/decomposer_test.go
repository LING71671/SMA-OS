package planner

import (
	"context"
	"fmt"
	"testing"
)

// mockLLM is a test double for LLMClient.
type mockLLM struct {
	response string
	err      error
}

func (m *mockLLM) InvokeWithContext(_ context.Context, _ string) (string, error) {
	return m.response, m.err
}

func TestLLMDecomposer_Decompose_Success(t *testing.T) {
	llm := &mockLLM{
		response: `[
			{"id":"T1","action_name":"step one","description":"first step","dependencies":[],"priority":1},
			{"id":"T2","action_name":"step two","description":"second step","dependencies":["T1"],"priority":2}
		]`,
	}
	d := NewLLMDecomposer(llm, 5, 20)
	result, err := d.Decompose(context.Background(), DecompositionRequest{
		Intent: ParsedIntent{Action: "create_vm", Target: "pool-A", Parameters: "cpu=2"},
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(result.Tasks) != 2 {
		t.Fatalf("expected 2 tasks, got %d", len(result.Tasks))
	}
	if result.RootTaskID != "T1" {
		t.Errorf("expected root T1, got %s", result.RootTaskID)
	}
}

func TestLLMDecomposer_Decompose_MalformedJSON(t *testing.T) {
	llm := &mockLLM{response: `not json`}
	d := NewLLMDecomposer(llm, 5, 20)
	_, err := d.Decompose(context.Background(), DecompositionRequest{
		Intent: ParsedIntent{Action: "test"},
	})
	if err == nil {
		t.Fatal("expected error for malformed JSON")
	}
}

func TestLLMDecomposer_Decompose_LLMError(t *testing.T) {
	llm := &mockLLM{err: fmt.Errorf("provider unavailable")}
	d := NewLLMDecomposer(llm, 5, 20)
	_, err := d.Decompose(context.Background(), DecompositionRequest{
		Intent: ParsedIntent{Action: "test"},
	})
	if err == nil {
		t.Fatal("expected error when LLM fails")
	}
}

func TestLLMDecomposer_Decompose_CycleDetected(t *testing.T) {
	llm := &mockLLM{
		response: `[
			{"id":"T1","action_name":"a","description":"","dependencies":["T2"],"priority":1},
			{"id":"T2","action_name":"b","description":"","dependencies":["T1"],"priority":1}
		]`,
	}
	d := NewLLMDecomposer(llm, 5, 20)
	_, err := d.Decompose(context.Background(), DecompositionRequest{
		Intent: ParsedIntent{Action: "test"},
	})
	if err == nil {
		t.Fatal("expected cycle error")
	}
}

func TestLLMDecomposer_Decompose_MaxSubTasksTruncation(t *testing.T) {
	llm := &mockLLM{
		response: `[
			{"id":"T1","action_name":"a","description":"","dependencies":[],"priority":1},
			{"id":"T2","action_name":"b","description":"","dependencies":["T1"],"priority":1},
			{"id":"T3","action_name":"c","description":"","dependencies":["T2"],"priority":1}
		]`,
	}
	d := NewLLMDecomposer(llm, 5, 2)
	result, err := d.Decompose(context.Background(), DecompositionRequest{
		Intent: ParsedIntent{Action: "test"},
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(result.Tasks) != 2 {
		t.Errorf("expected 2 tasks after truncation, got %d", len(result.Tasks))
	}
}

func TestValidateDecomposition_UnknownDependency(t *testing.T) {
	d := NewLLMDecomposer(&mockLLM{}, 5, 20)
	result := &DecompositionResult{
		Tasks: []DecomposedTask{
			{ID: "T1", ActionName: "a", Dependencies: []string{"T99"}},
		},
	}
	if err := d.ValidateDecomposition(result); err == nil {
		t.Fatal("expected error for unknown dependency")
	}
}
