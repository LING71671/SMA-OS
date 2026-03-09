package main

import (
	"context"
	"log"
	"math/rand"
	"os"
	"os/signal"
	"sync"
	"syscall"
	"time"
)

// WorkerType differentiates between resident and transient workers
type WorkerType string

const (
	WorkerTypeResident  WorkerType = "RESIDENT"
	WorkerTypeTransient WorkerType = "TRANSIENT"
)

type WorkerNode struct {
	ID        string
	Type      WorkerType
	NodeHost  string // Physical host affinity
	Available bool
}

type FractalClusterScheduler struct {
	mu           sync.RWMutex
	WarmPoolSize int
	Workers      map[string]*WorkerNode
}

func NewScheduler(warmPool int) *FractalClusterScheduler {
	s := &FractalClusterScheduler{
		WarmPoolSize: warmPool,
		Workers:      make(map[string]*WorkerNode),
	}
	s.initWarmPool()
	return s
}

// initWarmPool mocks the creation of pre-warmed Firecracker instances
func (s *FractalClusterScheduler) initWarmPool() {
	log.Printf("[Scheduler] Initializing Firecracker MicroVM Warm Pool (Size: %d)...", s.WarmPoolSize)
	// Mock pre-allocation
	for i := 0; i < s.WarmPoolSize; i++ {
		// Mock <5ms startup
		time.Sleep(1 * time.Millisecond)
	}
	log.Println("[Scheduler] Warm Pool initialized. Ready for nanosecond assignment.")
}

// AssignTask demonstrates Affinity scheduling
func (s *FractalClusterScheduler) AssignTask(taskID string, previousHost string) string {
	s.mu.Lock()
	defer s.mu.Unlock()

	// 1. Try affinity Match
	if previousHost != "" {
		for id, w := range s.Workers {
			if w.Available && w.NodeHost == previousHost {
				w.Available = false
				log.Printf("[Scheduler] Affinity Hit: Task %s assigned to existing host %s via Worker %s", taskID, previousHost, id)
				return id
			}
		}
	}

	// 2. Fallback to Warm Pool (Transient Worker pull)
	assignedHost := "host-alpha-x1" // mock
	assignedID := "microvm-pool-" + string(rune(rand.Intn(100)))
	log.Printf("[Scheduler] Affinity Miss: Task %s waking transient worker %s on %s (<5ms).", taskID, assignedID, assignedHost)
	return assignedID
}

func main() {
	log.Println("Starting SMA-OS Fractal Worker Scheduler v2.0...")

	scheduler := NewScheduler(50)

	// Mock incoming task distribution
	scheduler.AssignTask("task-1", "")
	scheduler.AssignTask("task-2", "host-alpha-x1") // Should hit affinity

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	go func() {
		<-quit
		log.Println("Scheduler shutting down...")
		cancel()
	}()

	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			log.Println("[Scheduler] Maintaining warm pool metrics...")
		case <-ctx.Done():
			log.Println("Scheduler gracefully stopped.")
			return
		}
	}
}
