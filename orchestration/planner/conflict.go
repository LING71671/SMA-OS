package planner

import (
	"fmt"
	"strings"
)

// ConflictDetector detects conflicts between decomposed tasks.
type ConflictDetector struct{}

// Detect runs all conflict checks and returns any conflicts found.
func (d *ConflictDetector) Detect(tasks []DecomposedTask) []Conflict {
	var conflicts []Conflict
	conflicts = append(conflicts, d.detectResourceConflicts(tasks)...)
	conflicts = append(conflicts, d.detectGoalConflicts(tasks)...)
	conflicts = append(conflicts, d.detectCycleConflicts(tasks)...)
	return conflicts
}

// detectResourceConflicts finds tasks that appear to target the same resource.
func (d *ConflictDetector) detectResourceConflicts(tasks []DecomposedTask) []Conflict {
	resourceUsage := make(map[string][]string)
	for _, t := range tasks {
		for _, r := range extractResources(t) {
			resourceUsage[r] = append(resourceUsage[r], t.ID)
		}
	}

	var conflicts []Conflict
	for resource, taskIDs := range resourceUsage {
		if len(taskIDs) > 1 {
			conflicts = append(conflicts, Conflict{
				Type:     ResourceConflict,
				TaskIDs:  taskIDs,
				Message:  fmt.Sprintf("multiple tasks reference resource '%s': %v", resource, taskIDs),
				Severity: Warning,
			})
		}
	}
	return conflicts
}

// detectGoalConflicts finds tasks with identical action names targeting the same object.
func (d *ConflictDetector) detectGoalConflicts(tasks []DecomposedTask) []Conflict {
	seen := make(map[string][]string)
	for _, t := range tasks {
		key := strings.ToLower(t.ActionName)
		seen[key] = append(seen[key], t.ID)
	}

	var conflicts []Conflict
	for action, taskIDs := range seen {
		if len(taskIDs) > 1 {
			conflicts = append(conflicts, Conflict{
				Type:     GoalConflict,
				TaskIDs:  taskIDs,
				Message:  fmt.Sprintf("multiple tasks share action '%s': %v", action, taskIDs),
				Severity: Warning,
			})
		}
	}
	return conflicts
}

// detectCycleConflicts reports a critical conflict if the task graph contains a cycle.
func (d *ConflictDetector) detectCycleConflicts(tasks []DecomposedTask) []Conflict {
	if hasCycle(tasks) {
		ids := make([]string, len(tasks))
		for i, t := range tasks {
			ids[i] = t.ID
		}
		return []Conflict{{
			Type:     CycleConflict,
			TaskIDs:  ids,
			Message:  "task graph contains a cyclic dependency",
			Severity: Critical,
		}}
	}
	return nil
}

// extractResources heuristically extracts resource tokens from a task's action and description.
func extractResources(t DecomposedTask) []string {
	var resources []string
	text := strings.ToLower(t.ActionName + " " + t.Description)
	for _, word := range strings.Fields(text) {
		// Consider words that look like resource identifiers (contain digits or hyphens)
		if strings.ContainsAny(word, "0123456789-_") && len(word) > 2 {
			resources = append(resources, word)
		}
	}
	return resources
}
