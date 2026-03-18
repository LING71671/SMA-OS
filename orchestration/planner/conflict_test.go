package planner

import (
	"testing"
)

func TestConflictDetector_NoConflicts(t *testing.T) {
	tasks := []DecomposedTask{
		{ID: "T1", ActionName: "create_vm", Description: "create vm-1", Dependencies: []string{}},
		{ID: "T2", ActionName: "start_vm", Description: "start vm-2", Dependencies: []string{"T1"}},
	}
	d := &ConflictDetector{}
	conflicts := d.Detect(tasks)
	// Resource conflicts only fire on shared tokens; these have distinct tokens
	for _, c := range conflicts {
		if c.Severity == Critical {
			t.Errorf("unexpected critical conflict: %+v", c)
		}
	}
}

func TestConflictDetector_GoalConflict(t *testing.T) {
	tasks := []DecomposedTask{
		{ID: "T1", ActionName: "create_vm", Description: "a", Dependencies: []string{}},
		{ID: "T2", ActionName: "create_vm", Description: "b", Dependencies: []string{"T1"}},
	}
	d := &ConflictDetector{}
	conflicts := d.Detect(tasks)
	found := false
	for _, c := range conflicts {
		if c.Type == GoalConflict {
			found = true
		}
	}
	if !found {
		t.Error("expected GoalConflict")
	}
}

func TestConflictDetector_CycleConflict(t *testing.T) {
	tasks := []DecomposedTask{
		{ID: "T1", ActionName: "a", Description: "", Dependencies: []string{"T2"}},
		{ID: "T2", ActionName: "b", Description: "", Dependencies: []string{"T1"}},
	}
	d := &ConflictDetector{}
	conflicts := d.Detect(tasks)
	found := false
	for _, c := range conflicts {
		if c.Type == CycleConflict && c.Severity == Critical {
			found = true
		}
	}
	if !found {
		t.Error("expected CycleConflict with Critical severity")
	}
}

func TestConflictDetector_ResourceConflict(t *testing.T) {
	tasks := []DecomposedTask{
		{ID: "T1", ActionName: "write pool-1", Description: "write to pool-1", Dependencies: []string{}},
		{ID: "T2", ActionName: "read pool-1", Description: "read from pool-1", Dependencies: []string{"T1"}},
	}
	d := &ConflictDetector{}
	conflicts := d.Detect(tasks)
	found := false
	for _, c := range conflicts {
		if c.Type == ResourceConflict {
			found = true
		}
	}
	if !found {
		t.Error("expected ResourceConflict for shared pool-1 token")
	}
}
