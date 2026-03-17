package main

import (
	"time"
)

// DependencyAnalysis holds the complete dependency analysis result for a DAG
type DependencyAnalysis struct {
	HasCycle           bool                `json:"has_cycle"`
	CyclePath          []string            `json:"cycle_path,omitempty"`
	CriticalPath       []string            `json:"critical_path"`
	CriticalPathLength int                 `json:"critical_path_length"`
	ParallelismMax     int                 `json:"parallelism_max"`
	DependencyDepth    int                 `json:"dependency_depth"`
	DependencyMatrix   map[string][]string `json:"dependency_matrix"`
	ImpactMap          map[string][]string `json:"impact_map"`
	Graph              *DependencyGraph    `json:"graph"`
	GeneratedAt        time.Time           `json:"generated_at"`
}

// DependencyNode is a node in the serializable dependency graph
type DependencyNode struct {
	ID           string     `json:"id"`
	Status       TaskStatus `json:"status"`
	Dependencies []string   `json:"dependencies"`
	Dependents   []string   `json:"dependents"`
	Depth        int        `json:"depth"`
	IsCritical   bool       `json:"is_critical"`
}

// DependencyGraph is the serializable form of the DAG
type DependencyGraph struct {
	Nodes []*DependencyNode `json:"nodes"`
	Edges []DependencyEdge  `json:"edges"`
}

// DependencyEdge represents a directed edge in the dependency graph
type DependencyEdge struct {
	From       string `json:"from"`
	To         string `json:"to"`
	IsCritical bool   `json:"is_critical"`
}

// DetectCycle uses DFS to detect cycles in the DAG.
// Returns (true, cyclePath) if a cycle exists, (false, nil) otherwise.
func (dm *DAGManager) DetectCycle() (bool, []string) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	visited := make(map[string]bool)
	recStack := make(map[string]bool)
	var cyclePath []string
	found := false

	var dfs func(id string) bool
	dfs = func(id string) bool {
		visited[id] = true
		recStack[id] = true

		for _, dep := range dm.Nodes[id].Dependencies {
			if !visited[dep] {
				if dfs(dep) {
					if found {
						cyclePath = append(cyclePath, dep)
					}
					return true
				}
			} else if recStack[dep] {
				found = true
				cyclePath = append(cyclePath, dep, id)
				return true
			}
		}

		recStack[id] = false
		return false
	}

	for id := range dm.Nodes {
		if !visited[id] {
			if dfs(id) {
				break
			}
		}
	}

	if found {
		// Reverse for correct order
		for i, j := 0, len(cyclePath)-1; i < j; i, j = i+1, j-1 {
			cyclePath[i], cyclePath[j] = cyclePath[j], cyclePath[i]
		}
		return true, cyclePath
	}
	return false, nil
}

// topologicalSort returns nodes in topological order (must NOT hold dm.mu)
func (dm *DAGManager) topologicalSort() []string {
	inDeg := make(map[string]int)
	for id, node := range dm.Nodes {
		inDeg[id] = len(node.Dependencies)
	}

	var queue []string
	for id, deg := range inDeg {
		if deg == 0 {
			queue = append(queue, id)
		}
	}

	var result []string
	for len(queue) > 0 {
		id := queue[0]
		queue = queue[1:]
		result = append(result, id)
		for _, dep := range dm.dependents[id] {
			inDeg[dep]--
			if inDeg[dep] == 0 {
				queue = append(queue, dep)
			}
		}
	}
	return result
}

// CalculateCriticalPath computes the longest dependency chain using dynamic programming.
func (dm *DAGManager) CalculateCriticalPath() ([]string, int) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	order := dm.topologicalSort()
	maxDepth := make(map[string]int)
	pathTo := make(map[string][]string)

	for _, id := range order {
		maxDepth[id] = 0
		pathTo[id] = []string{id}

		for _, dep := range dm.Nodes[id].Dependencies {
			if maxDepth[dep]+1 > maxDepth[id] {
				maxDepth[id] = maxDepth[dep] + 1
				newPath := make([]string, len(pathTo[dep]))
				copy(newPath, pathTo[dep])
				pathTo[id] = append(newPath, id)
			}
		}
	}

	var maxLen int
	var maxPath []string
	for id, depth := range maxDepth {
		if depth > maxLen || (depth == maxLen && len(pathTo[id]) > len(maxPath)) {
			maxLen = depth
			maxPath = pathTo[id]
		}
	}

	return maxPath, len(maxPath)
}

// CalculateParallelism computes the maximum number of tasks that can run concurrently.
func (dm *DAGManager) CalculateParallelism() (int, map[int][]string) {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	order := dm.topologicalSort()
	levels := make(map[string]int)

	for _, id := range order {
		level := 0
		for _, dep := range dm.Nodes[id].Dependencies {
			if levels[dep]+1 > level {
				level = levels[dep] + 1
			}
		}
		levels[id] = level
	}

	layerInfo := make(map[int][]string)
	for id, level := range levels {
		layerInfo[level] = append(layerInfo[level], id)
	}

	maxParallelism := 0
	for _, tasks := range layerInfo {
		if len(tasks) > maxParallelism {
			maxParallelism = len(tasks)
		}
	}

	return maxParallelism, layerInfo
}

// CalculateDependencyDepth returns the dependency depth for each task (0 = no deps).
func (dm *DAGManager) CalculateDependencyDepth() map[string]int {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	depths := make(map[string]int)

	var calcDepth func(id string) int
	calcDepth = func(id string) int {
		if depth, ok := depths[id]; ok {
			return depth
		}
		maxDepDepth := -1
		for _, dep := range dm.Nodes[id].Dependencies {
			d := calcDepth(dep)
			if d > maxDepDepth {
				maxDepDepth = d
			}
		}
		depths[id] = maxDepDepth + 1
		return depths[id]
	}

	for id := range dm.Nodes {
		calcDepth(id)
	}
	return depths
}

// CalculateImpactMap returns the set of tasks affected if each task fails.
func (dm *DAGManager) CalculateImpactMap() map[string][]string {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	impact := make(map[string][]string)

	var calcImpact func(id string) []string
	calcImpact = func(id string) []string {
		if imp, ok := impact[id]; ok {
			return imp
		}
		var affected []string
		for _, dep := range dm.dependents[id] {
			affected = append(affected, dep)
			affected = append(affected, calcImpact(dep)...)
		}
		impact[id] = affected
		return affected
	}

	for id := range dm.Nodes {
		calcImpact(id)
	}
	return impact
}

// GenerateDependencyMatrix returns a map of taskID -> direct dependencies.
func (dm *DAGManager) GenerateDependencyMatrix() map[string][]string {
	dm.mu.Lock()
	defer dm.mu.Unlock()

	matrix := make(map[string][]string)
	for id, node := range dm.Nodes {
		deps := make([]string, len(node.Dependencies))
		copy(deps, node.Dependencies)
		matrix[id] = deps
	}
	return matrix
}

// BuildDependencyGraph constructs the serializable dependency graph with critical path info.
func (dm *DAGManager) BuildDependencyGraph() *DependencyGraph {
	criticalPath, _ := dm.CalculateCriticalPath()
	criticalSet := make(map[string]bool)
	for _, id := range criticalPath {
		criticalSet[id] = true
	}

	depths := dm.CalculateDependencyDepth()

	dm.mu.Lock()
	defer dm.mu.Unlock()

	var nodes []*DependencyNode
	var edges []DependencyEdge

	for id, node := range dm.Nodes {
		nodes = append(nodes, &DependencyNode{
			ID:           id,
			Status:       node.Status,
			Dependencies: node.Dependencies,
			Dependents:   dm.dependents[id],
			Depth:        depths[id],
			IsCritical:   criticalSet[id],
		})
		for _, dep := range node.Dependencies {
			edges = append(edges, DependencyEdge{
				From:       dep,
				To:         id,
				IsCritical: criticalSet[dep] && criticalSet[id],
			})
		}
	}

	return &DependencyGraph{Nodes: nodes, Edges: edges}
}

// AnalyzeDependencies runs the full dependency analysis and returns a complete result.
func (dm *DAGManager) AnalyzeDependencies() *DependencyAnalysis {
	hasCycle, cyclePath := dm.DetectCycle()
	criticalPath, criticalLen := dm.CalculateCriticalPath()
	maxParallel, _ := dm.CalculateParallelism()
	depths := dm.CalculateDependencyDepth()
	impact := dm.CalculateImpactMap()
	matrix := dm.GenerateDependencyMatrix()
	graph := dm.BuildDependencyGraph()

	maxDepth := 0
	for _, d := range depths {
		if d > maxDepth {
			maxDepth = d
		}
	}

	return &DependencyAnalysis{
		HasCycle:           hasCycle,
		CyclePath:          cyclePath,
		CriticalPath:       criticalPath,
		CriticalPathLength: criticalLen,
		ParallelismMax:     maxParallel,
		DependencyDepth:    maxDepth,
		DependencyMatrix:   matrix,
		ImpactMap:          impact,
		Graph:              graph,
		GeneratedAt:        time.Now(),
	}
}
