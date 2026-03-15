package main

import (
	"context"
	"fmt"
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

// WorkerHealthStatus represents the health status of a worker
type WorkerHealthStatus string

const (
	WorkerHealthy   WorkerHealthStatus = "HEALTHY"
	WorkerUnhealthy WorkerHealthStatus = "UNHEALTHY"
	WorkerUnknown   WorkerHealthStatus = "UNKNOWN"
)

// WorkerHealth tracks health metrics
type WorkerHealth struct {
	Status            WorkerHealthStatus
	LastHeartbeat     time.Time
	FailureCount      int
	RecoveryStartedAt time.Time // When recovery began (for cooldown)
	ConsecutiveGood   int       // Count of consecutive good heartbeats
}

// HeartbeatMessage represents a heartbeat message from a worker
type HeartbeatMessage struct {
	WorkerID  string
	Timestamp time.Time
}

// WorkerFailureEvent represents a worker failure event for broadcasting
type WorkerFailureEvent struct {
	WorkerID     string
	Reason       string
	Timestamp    time.Time
	FailureCount int
}

// WorkerHealthConfig defines health check configuration
type WorkerHealthConfig struct {
	HeartbeatInterval     time.Duration
	HealthCheckThreshold  int
	FailureThreshold      int
	RecoveryCooldown      time.Duration // Minimum time before marking healthy
	ConsecutiveHeartbeats int           // Required good heartbeats for recovery
}

// DefaultWorkerHealthConfig returns default configuration
func DefaultWorkerHealthConfig() WorkerHealthConfig {
	return WorkerHealthConfig{
		HeartbeatInterval:     10 * time.Second,
		HealthCheckThreshold:  3,
		FailureThreshold:      3,
		RecoveryCooldown:      30 * time.Second,
		ConsecutiveHeartbeats: 3,
	}
}

type WorkerNode struct {
	ID        string
	Type      WorkerType
	NodeHost  string // Physical host affinity
	Available bool
	Health    WorkerHealth // Health status for the worker
}

type FractalClusterScheduler struct {
	mu           sync.RWMutex
	WarmPoolSize int
	Workers      map[string]*WorkerNode
	HealthConfig WorkerHealthConfig // Health check configuration
	// Add health check tracking
	healthCheckTicker *time.Ticker
	quit              chan struct{}
}

func NewScheduler(warmPool int) *FractalClusterScheduler {
	s := &FractalClusterScheduler{
		WarmPoolSize: warmPool,
		Workers:      make(map[string]*WorkerNode),
		HealthConfig: DefaultWorkerHealthConfig(),
		quit:         make(chan struct{}),
	}
	s.initWarmPool()
	return s
}

// initWarmPool mocks the creation of pre-warmed Firecracker instances
func (s *FractalClusterScheduler) initWarmPool() {
	log.Printf("[Scheduler] Initializing Firecracker MicroVM Warm Pool (Size: %d)...", s.WarmPoolSize)
	s.mu.Lock()
	defer s.mu.Unlock()

	// Create worker nodes with initialized health
	for i := 0; i < s.WarmPoolSize; i++ {
		id := fmt.Sprintf("worker-%d", i)
		s.Workers[id] = &WorkerNode{
			ID:        id,
			Type:      WorkerTypeResident,
			NodeHost:  fmt.Sprintf("host-%d", i%10), // Distribute across 10 mock hosts
			Available: true,
			Health: WorkerHealth{
				Status:        WorkerHealthy,
				LastHeartbeat: time.Now(),
				FailureCount:  0,
			},
		}
		// Mock <5ms startup
		time.Sleep(1 * time.Millisecond)
	}
	log.Println("[Scheduler] Warm Pool initialized. Ready for nanosecond assignment.")
}

// AssignTask demonstrates Affinity scheduling with health checks
func (s *FractalClusterScheduler) AssignTask(taskID string, previousHost string) string {
	s.mu.Lock()
	defer s.mu.Unlock()

	// 1. Try affinity Match with health check
	if previousHost != "" {
		for id, w := range s.Workers {
			if w.Available && w.NodeHost == previousHost && w.Health.Status == WorkerHealthy {
				w.Available = false
				log.Printf("[Scheduler] Affinity Hit: Task %s assigned to existing host %s via Worker %s", taskID, previousHost, id)
				return id
			}
		}
	}

	// 2. Try any healthy worker (respecting affinity preference but not requiring it)
	for id, w := range s.Workers {
		if w.Available && w.Health.Status == WorkerHealthy {
			w.Available = false
			log.Printf("[Scheduler] Healthy Worker: Task %s assigned to Worker %s on %s", taskID, id, w.NodeHost)
			return id
		}
	}

	// 3. Fallback to Transient Worker (no healthy workers available)
	assignedHost := "host-alpha-x1" // mock
	assignedID := fmt.Sprintf("microvm-pool-%d", rand.Intn(100))
	log.Printf("[Scheduler] Transient Fallback: Task %s waking transient worker %s on %s (<5ms). No healthy workers available.", taskID, assignedID, assignedHost)
	return assignedID
}

// ReassignTaskFromUnhealthy reassigns a task from an unhealthy worker to a healthy one
func (s *FractalClusterScheduler) ReassignTaskFromUnhealthy(workerID string, taskID string) (string, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	// Verify the worker exists and is unhealthy
	unhealthyWorker, exists := s.Workers[workerID]
	if !exists {
		return "", fmt.Errorf("worker %s not found", workerID)
	}

	if unhealthyWorker.Health.Status == WorkerHealthy {
		return "", fmt.Errorf("worker %s is healthy, no reassignment needed", workerID)
	}

	// Mark the unhealthy worker as unavailable
	unhealthyWorker.Available = false

	// Find a healthy worker for reassignment
	for id, w := range s.Workers {
		if w.Available && w.Health.Status == WorkerHealthy {
			w.Available = false
			log.Printf("[Scheduler] Reassignment: Task %s moved from unhealthy Worker %s to healthy Worker %s on %s",
				taskID, workerID, id, w.NodeHost)
			return id, nil
		}
	}

	// No healthy workers available - return transient fallback
	assignedHost := "host-alpha-x1" // mock
	assignedID := fmt.Sprintf("microvm-pool-%d", rand.Intn(100))
	log.Printf("[Scheduler] Reassignment Fallback: Task %s moved from unhealthy Worker %s to transient worker %s on %s (<5ms). No healthy workers available.",
		taskID, workerID, assignedID, assignedHost)
	return assignedID, nil
}

// GetHealthyWorkers returns a list of healthy worker IDs
func (s *FractalClusterScheduler) GetHealthyWorkers() []string {
	s.mu.RLock()
	defer s.mu.RUnlock()

	healthyWorkers := make([]string, 0)
	for id, worker := range s.Workers {
		if worker.Health.Status == WorkerHealthy {
			healthyWorkers = append(healthyWorkers, id)
		}
	}
	return healthyWorkers
}

// MarkWorkerUnhealthy marks a worker as unhealthy with a reason
func (s *FractalClusterScheduler) MarkWorkerUnhealthy(workerID string, reason string) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	worker, exists := s.Workers[workerID]
	if !exists {
		return fmt.Errorf("worker %s not found", workerID)
	}

	worker.Health.Status = WorkerUnhealthy
	worker.Health.FailureCount++
	worker.Available = false
	log.Printf("[Scheduler] Worker %s marked UNHEALTHY: %s (failure count: %d)", workerID, reason, worker.Health.FailureCount)
	return nil
}

// MarkWorkerHealthy recovers a worker to healthy status
func (s *FractalClusterScheduler) MarkWorkerHealthy(workerID string) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	worker, exists := s.Workers[workerID]
	if !exists {
		return fmt.Errorf("worker %s not found", workerID)
	}

	worker.Health.Status = WorkerHealthy
	worker.Health.FailureCount = 0
	worker.Health.LastHeartbeat = time.Now()
	worker.Available = true
	log.Printf("[Scheduler] Worker %s marked HEALTHY and recovered", workerID)
	return nil
}

// RemoveUnhealthyWorkers removes all unhealthy workers from the pool
func (s *FractalClusterScheduler) RemoveUnhealthyWorkers() int {
	s.mu.Lock()
	defer s.mu.Unlock()

	removedCount := 0
	for id, worker := range s.Workers {
		if worker.Health.Status == WorkerUnhealthy {
			delete(s.Workers, id)
			removedCount++
			log.Printf("[Scheduler] Removed unhealthy worker %s from pool", id)
		}
	}
	return removedCount
}

// GetWorkerHealthStatus returns the health status of a specific worker
func (s *FractalClusterScheduler) GetWorkerHealthStatus(workerID string) (WorkerHealthStatus, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	worker, exists := s.Workers[workerID]
	if !exists {
		return WorkerUnknown, false
	}
	return worker.Health.Status, true
}

// PrintHealthSummary logs the current health status of all workers
func (s *FractalClusterScheduler) PrintHealthSummary() {
	s.mu.RLock()
	defer s.mu.RUnlock()

	healthyCount := 0
	unhealthyCount := 0
	unknownCount := 0

	for _, worker := range s.Workers {
		switch worker.Health.Status {
		case WorkerHealthy:
			healthyCount++
		case WorkerUnhealthy:
			unhealthyCount++
		case WorkerUnknown:
			unknownCount++
		}
	}

	totalWorkers := len(s.Workers)
	log.Printf("[Scheduler] Health Summary: Total=%d, Healthy=%d, Unhealthy=%d, Unknown=%d",
		totalWorkers, healthyCount, unhealthyCount, unknownCount)
}

// StartHeartbeatServer starts the heartbeat server with a ticker
func (s *FractalClusterScheduler) StartHeartbeatServer() {
	s.mu.Lock()
	if s.healthCheckTicker != nil {
		s.mu.Unlock()
		log.Println("[Scheduler] Heartbeat server already running")
		return
	}
	s.healthCheckTicker = time.NewTicker(s.HealthConfig.HeartbeatInterval)
	s.mu.Unlock()

	log.Printf("[Scheduler] Heartbeat server started with interval: %v", s.HealthConfig.HeartbeatInterval)

	go func() {
		for {
			select {
			case <-s.healthCheckTicker.C:
				s.checkWorkerHealth()
			case <-s.quit:
				return
			}
		}
	}()
}

// checkWorkerHealth checks all workers' last heartbeat and marks unhealthy if stale
func (s *FractalClusterScheduler) checkWorkerHealth() {
	s.mu.Lock()
	defer s.mu.Unlock()

	staleThreshold := s.HealthConfig.HeartbeatInterval * time.Duration(s.HealthConfig.HealthCheckThreshold)
	now := time.Now()

	for id, worker := range s.Workers {
		// Skip workers that are already unhealthy
		if worker.Health.Status == WorkerUnhealthy {
			continue
		}

		// Check if worker hasn't sent heartbeat within threshold
		if now.Sub(worker.Health.LastHeartbeat) > staleThreshold {
			worker.Health.FailureCount++
			log.Printf("[Scheduler] Worker %s heartbeat stale (last: %v ago), failure count: %d",
				id, now.Sub(worker.Health.LastHeartbeat), worker.Health.FailureCount)

			// If failure count exceeds threshold, mark as unhealthy
			if worker.Health.FailureCount >= s.HealthConfig.FailureThreshold {
				worker.Health.Status = WorkerUnhealthy
				worker.Available = false
				log.Printf("[Scheduler] Worker %s marked UNHEALTHY due to stale heartbeat", id)
			}
		}
	}
}

// ReceiveHeartbeat updates the worker's last heartbeat time
func (s *FractalClusterScheduler) ReceiveHeartbeat(msg HeartbeatMessage) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	worker, exists := s.Workers[msg.WorkerID]
	if !exists {
		return fmt.Errorf("worker %s not found", msg.WorkerID)
	}

	// Update heartbeat time
	worker.Health.LastHeartbeat = msg.Timestamp

	// Handle unhealthy worker recovery with cooldown and consecutive heartbeat check
	if worker.Health.Status == WorkerUnhealthy {
		// Start recovery timer if not already started
		if worker.Health.RecoveryStartedAt.IsZero() {
			worker.Health.RecoveryStartedAt = msg.Timestamp
			worker.Health.ConsecutiveGood = 0
			log.Printf("[Scheduler] Worker %s started recovery period", msg.WorkerID)
		}

		// Increment consecutive good heartbeats
		worker.Health.ConsecutiveGood++

		// Check if recovery conditions are met
		recoveryDuration := msg.Timestamp.Sub(worker.Health.RecoveryStartedAt)
		if recoveryDuration >= s.HealthConfig.RecoveryCooldown &&
			worker.Health.ConsecutiveGood >= s.HealthConfig.ConsecutiveHeartbeats {
			worker.Health.Status = WorkerHealthy
			worker.Health.FailureCount = 0
			worker.Health.ConsecutiveGood = 0
			worker.Health.RecoveryStartedAt = time.Time{} // Reset
			worker.Available = true
			log.Printf("[Scheduler] Worker %s recovered after %v and %d good heartbeats",
				msg.WorkerID, recoveryDuration, s.HealthConfig.ConsecutiveHeartbeats)
		} else {
			log.Printf("[Scheduler] Worker %s recovery in progress: %v elapsed, %d/%d good heartbeats",
				msg.WorkerID, recoveryDuration, worker.Health.ConsecutiveGood, s.HealthConfig.ConsecutiveHeartbeats)
		}
	} else {
		// Healthy worker: reset failure count on successful heartbeat
		if worker.Health.FailureCount > 0 {
			worker.Health.FailureCount = 0
			log.Printf("[Scheduler] Worker %s failure count reset after heartbeat", msg.WorkerID)
		}
	}

	log.Printf("[Scheduler] Heartbeat received from worker %s", msg.WorkerID)
	return nil
}

// StopHeartbeatServer stops the heartbeat server ticker
func (s *FractalClusterScheduler) StopHeartbeatServer() {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.healthCheckTicker != nil {
		s.healthCheckTicker.Stop()
		s.healthCheckTicker = nil
		log.Println("[Scheduler] Heartbeat server stopped")
	}

	// Signal the goroutine to quit
	close(s.quit)
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
