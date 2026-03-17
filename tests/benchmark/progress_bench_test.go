package benchmark

import (
	"encoding/json"
	"testing"
	"time"
)

// ---- minimal in-process types mirroring manager package ----

type taskStatus string

const (
	statusPending   taskStatus = "PENDING"
	statusRunning   taskStatus = "RUNNING"
	statusCompleted taskStatus = "COMPLETED"
	statusFailed    taskStatus = "FAILED"
	statusPaused    taskStatus = "PAUSED"
)

type taskNode struct {
	ID         string
	ActionName string
	Status     taskStatus
	Progress   float64
	Payload    string
	SubTasks   []string
}

type taskCheckpoint struct {
	Version   uint64         `json:"version"`
	StateData []byte         `json:"state_data"`
	Position  string         `json:"position"`
	CreatedAt time.Time      `json:"created_at"`
	Metadata  map[string]any `json:"metadata,omitempty"`
}

type taskStateSnapshot struct {
	TaskID     string     `json:"task_id"`
	Status     taskStatus `json:"status"`
	Progress   float64    `json:"progress"`
	ActionName string     `json:"action_name"`
	Payload    string     `json:"payload"`
}

func newCheckpoint(version uint64, task *taskNode) (*taskCheckpoint, error) {
	snap := taskStateSnapshot{
		TaskID:     task.ID,
		Status:     task.Status,
		Progress:   task.Progress,
		ActionName: task.ActionName,
		Payload:    task.Payload,
	}
	data, err := json.Marshal(snap)
	if err != nil {
		return nil, err
	}
	return &taskCheckpoint{
		Version:   version,
		StateData: data,
		Position:  task.ActionName,
		CreatedAt: time.Now(),
	}, nil
}

func calcProgress(task *taskNode, nodes map[string]*taskNode) float64 {
	if len(task.SubTasks) > 0 {
		done := 0
		for _, sid := range task.SubTasks {
			if s, ok := nodes[sid]; ok && (s.Status == statusCompleted || s.Status == statusFailed) {
				done++
			}
		}
		return float64(done) / float64(len(task.SubTasks)) * 100
	}
	switch task.Status {
	case statusPending:
		return 0
	case statusRunning:
		return 50
	case statusCompleted:
		return 100
	case statusFailed:
		return 100
	case statusPaused:
		return task.Progress
	}
	return 0
}

// ---- Benchmarks ----

// BenchmarkProgressCalc_Atomic measures progress calculation for atomic tasks
func BenchmarkProgressCalc_Atomic(b *testing.B) {
	nodes := map[string]*taskNode{
		"T1": {ID: "T1", ActionName: "Task", Status: statusRunning, Progress: 50},
	}
	task := nodes["T1"]
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		calcProgress(task, nodes)
	}
}

// BenchmarkProgressCalc_SubTasks measures progress aggregation with 100 subtasks
func BenchmarkProgressCalc_SubTasks(b *testing.B) {
	nodes := make(map[string]*taskNode, 101)
	subIDs := make([]string, 100)
	for i := 0; i < 100; i++ {
		id := string(rune('A'+i%26)) + string(rune('0'+i/26))
		subIDs[i] = id
		status := statusPending
		if i < 50 {
			status = statusCompleted
		}
		nodes[id] = &taskNode{ID: id, Status: status}
	}
	parent := &taskNode{ID: "P", Status: statusRunning, SubTasks: subIDs}
	nodes["P"] = parent

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		calcProgress(parent, nodes)
	}
}

// BenchmarkCheckpointSave measures checkpoint serialization
func BenchmarkCheckpointSave(b *testing.B) {
	task := &taskNode{
		ID:         "T1",
		ActionName: "Benchmark Task",
		Status:     statusRunning,
		Progress:   42.5,
		Payload:    `{"key":"value","data":"some payload content here"}`,
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := newCheckpoint(uint64(i), task)
		if err != nil {
			b.Fatal(err)
		}
	}
}

// BenchmarkCheckpointSerialize measures full checkpoint JSON serialization
func BenchmarkCheckpointSerialize(b *testing.B) {
	task := &taskNode{
		ID:         "T1",
		ActionName: "Benchmark Task",
		Status:     statusRunning,
		Progress:   42.5,
		Payload:    `{"key":"value"}`,
	}
	cp, _ := newCheckpoint(1, task)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := json.Marshal(cp)
		if err != nil {
			b.Fatal(err)
		}
	}
}

// BenchmarkCheckpointDeserialize measures checkpoint JSON deserialization
func BenchmarkCheckpointDeserialize(b *testing.B) {
	task := &taskNode{
		ID: "T1", ActionName: "Task", Status: statusRunning, Progress: 42.5,
	}
	cp, _ := newCheckpoint(1, task)
	data, _ := json.Marshal(cp)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var c taskCheckpoint
		if err := json.Unmarshal(data, &c); err != nil {
			b.Fatal(err)
		}
	}
}

// BenchmarkDependencyDepth measures dependency depth calculation on a 50-node chain
func BenchmarkDependencyDepth(b *testing.B) {
	nodes := make(map[string]*taskNode, 50)
	prev := ""
	for i := 0; i < 50; i++ {
		id := string(rune('A' + i%26))
		if i >= 26 {
			id = id + "2"
		}
		nodes[id] = &taskNode{ID: id, Status: statusPending}
		if prev != "" {
			nodes[prev].SubTasks = append(nodes[prev].SubTasks, id)
		}
		prev = id
	}

	calcDepth := func() map[string]int {
		depths := make(map[string]int)
		var calc func(id string) int
		calc = func(id string) int {
			if d, ok := depths[id]; ok {
				return d
			}
			max := -1
			for _, dep := range nodes[id].SubTasks {
				if d := calc(dep); d > max {
					max = d
				}
			}
			depths[id] = max + 1
			return depths[id]
		}
		for id := range nodes {
			calc(id)
		}
		return depths
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		calcDepth()
	}
}

// BenchmarkProgressReport_Async measures that async progress reporting doesn't block
func BenchmarkProgressReport_Async(b *testing.B) {
	nodes := map[string]*taskNode{
		"T1": {ID: "T1", Status: statusRunning, Progress: 50},
	}
	task := nodes["T1"]
	reportCh := make(chan float64, 1000)

	reportProgress := func(t *taskNode) {
		go func() {
			p := calcProgress(t, nodes)
			select {
			case reportCh <- p:
			default:
			}
		}()
	}

	b.ResetTimer()
	start := time.Now()
	for i := 0; i < b.N; i++ {
		reportProgress(task)
	}
	elapsed := time.Since(start)

	// Drain channel
	for len(reportCh) > 0 {
		<-reportCh
	}

	// Report ns/op for visibility
	b.ReportMetric(float64(elapsed.Nanoseconds())/float64(b.N), "ns/report")
}
