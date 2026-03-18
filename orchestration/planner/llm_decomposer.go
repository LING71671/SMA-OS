package planner

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"
)

// LLMClient is a minimal interface for invoking an LLM, matching llm.Manager.
type LLMClient interface {
	InvokeWithContext(ctx context.Context, prompt string) (string, error)
}

// LLMDecomposer implements TaskDecomposer using an LLM backend.
type LLMDecomposer struct {
	llm         LLMClient
	maxDepth    int
	maxSubTasks int
}

// NewLLMDecomposer creates a new LLMDecomposer with the given LLM client.
func NewLLMDecomposer(llm LLMClient, maxDepth, maxSubTasks int) *LLMDecomposer {
	if maxDepth <= 0 {
		maxDepth = 5
	}
	if maxSubTasks <= 0 {
		maxSubTasks = 20
	}
	return &LLMDecomposer{llm: llm, maxDepth: maxDepth, maxSubTasks: maxSubTasks}
}

// Decompose calls the LLM to break the intent into a task DAG.
func (d *LLMDecomposer) Decompose(ctx context.Context, req DecompositionRequest) (*DecompositionResult, error) {
	if req.MaxDepth <= 0 {
		req.MaxDepth = d.maxDepth
	}
	if req.MaxSubTasks <= 0 {
		req.MaxSubTasks = d.maxSubTasks
	}

	start := time.Now()
	prompt := buildDecompositionPrompt(req)

	response, err := d.llm.InvokeWithContext(ctx, prompt)
	if err != nil {
		return nil, fmt.Errorf("LLM decomposition failed: %w", err)
	}

	tasks, err := parseLLMResponse(response)
	if err != nil {
		return nil, fmt.Errorf("failed to parse LLM response: %w", err)
	}

	if len(tasks) > req.MaxSubTasks {
		tasks = tasks[:req.MaxSubTasks]
	}

	result := &DecompositionResult{
		Tasks:      tasks,
		RootTaskID: findRootTask(tasks),
		Duration:   time.Since(start),
	}

	if err := d.ValidateDecomposition(result); err != nil {
		return nil, err
	}

	return result, nil
}

// ValidateDecomposition checks for cycles and missing dependency references.
func (d *LLMDecomposer) ValidateDecomposition(result *DecompositionResult) error {
	if result == nil {
		return fmt.Errorf("nil decomposition result")
	}

	idSet := make(map[string]bool, len(result.Tasks))
	for _, t := range result.Tasks {
		idSet[t.ID] = true
	}

	// Check all dependency references exist
	for _, t := range result.Tasks {
		for _, dep := range t.Dependencies {
			if !idSet[dep] {
				return fmt.Errorf("task %s depends on unknown task %s", t.ID, dep)
			}
		}
	}

	// Cycle detection via DFS
	if hasCycle(result.Tasks) {
		return fmt.Errorf("decomposition contains cyclic dependencies")
	}

	return nil
}

// buildDecompositionPrompt constructs the LLM prompt for task decomposition.
func buildDecompositionPrompt(req DecompositionRequest) string {
	return fmt.Sprintf(`你是一个任务规划专家。请将以下目标分解为可执行的子任务。

目标: %s %s
参数: %s

要求:
1. 分解为具体的子任务，每个任务有清晰的描述
2. 识别任务之间的依赖关系（dependencies 字段填写前置任务的 id）
3. 输出纯 JSON 数组，不要包含任何其他文字

输出格式（JSON 数组）:
[{"id":"T1","action_name":"任务名称","description":"详细描述","dependencies":[],"priority":1}]

限制:
- 最大 %d 层深度
- 最多 %d 个子任务
`, req.Intent.Action, req.Intent.Target, req.Intent.Parameters, req.MaxDepth, req.MaxSubTasks)
}

// parseLLMResponse extracts a []DecomposedTask from raw LLM output.
func parseLLMResponse(response string) ([]DecomposedTask, error) {
	// Strip markdown code fences if present
	response = strings.TrimSpace(response)
	if idx := strings.Index(response, "["); idx >= 0 {
		response = response[idx:]
	}
	if idx := strings.LastIndex(response, "]"); idx >= 0 {
		response = response[:idx+1]
	}

	var tasks []DecomposedTask
	if err := json.Unmarshal([]byte(response), &tasks); err != nil {
		return nil, fmt.Errorf("parse failed: %w", err)
	}
	if len(tasks) == 0 {
		return nil, fmt.Errorf("parse failed: LLM returned empty task list")
	}
	return tasks, nil
}

// findRootTask returns the ID of the task with no dependencies (the DAG root).
// If multiple roots exist, returns the first one found.
func findRootTask(tasks []DecomposedTask) string {
	for _, t := range tasks {
		if len(t.Dependencies) == 0 {
			return t.ID
		}
	}
	if len(tasks) > 0 {
		return tasks[0].ID
	}
	return ""
}

// hasCycle detects cycles in the decomposed task DAG using DFS.
func hasCycle(tasks []DecomposedTask) bool {
	adj := make(map[string][]string, len(tasks))
	for _, t := range tasks {
		adj[t.ID] = t.Dependencies
	}

	visited := make(map[string]bool)
	inStack := make(map[string]bool)

	var dfs func(id string) bool
	dfs = func(id string) bool {
		visited[id] = true
		inStack[id] = true
		for _, dep := range adj[id] {
			if !visited[dep] {
				if dfs(dep) {
					return true
				}
			} else if inStack[dep] {
				return true
			}
		}
		inStack[id] = false
		return false
	}

	for _, t := range tasks {
		if !visited[t.ID] {
			if dfs(t.ID) {
				return true
			}
		}
	}
	return false
}
