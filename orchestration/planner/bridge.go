package planner

import (
	"context"
	"fmt"
)

// Bridge connects the TaskDecomposer output to the DAG manager input.
type Bridge struct {
	decomposer TaskDecomposer
}

// NewBridge creates a Bridge with the given decomposer.
func NewBridge(decomposer TaskDecomposer) *Bridge {
	return &Bridge{decomposer: decomposer}
}

// DecomposedTaskToTaskNode converts []DecomposedTask to []TaskNode for the DAG manager.
func DecomposedTaskToTaskNode(tasks []DecomposedTask) []TaskNode {
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
	return nodes
}

// Decompose runs the full pipeline: decompose intent → validate → convert to TaskNodes.
// Returns the TaskNodes and the root task ID.
func (b *Bridge) Decompose(ctx context.Context, req DecompositionRequest) ([]TaskNode, string, error) {
	result, err := b.decomposer.Decompose(ctx, req)
	if err != nil {
		return nil, "", fmt.Errorf("decomposition failed: %w", err)
	}

	nodes := DecomposedTaskToTaskNode(result.Tasks)
	return nodes, result.RootTaskID, nil
}
