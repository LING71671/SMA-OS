// Package framework provides the core chaos testing infrastructure
// for SMA-OS distributed system resilience validation.
package framework

import (
	"context"
	"fmt"
	"log"
	"sync"
	"time"
)

// TestResult represents the outcome of a chaos test execution
type TestResult struct {
	Name      string
	Passed    bool
	Duration  time.Duration
	Error     error
	Timestamp time.Time
	Details   map[string]interface{}
}

// ChaosTest defines the interface for all chaos test scenarios
// Each test must implement setup, failure injection, verification, and cleanup phases
type ChaosTest interface {
	// Name returns the unique identifier for this test scenario
	Name() string

	// Setup prepares the test environment and dependencies
	// Returns error if setup fails
	Setup() error

	// InjectFailure introduces the chaos/failure condition
	// This is the core chaos engineering action
	InjectFailure() error

	// Verify checks if the system handled the failure correctly
	// Returns error if verification fails (system not resilient)
	Verify() error

	// Cleanup restores the environment to pre-test state
	// Must be called even if previous steps failed
	Cleanup() error
}

// ChaosRunner orchestrates the execution of multiple chaos tests
type ChaosRunner struct {
	tests    []ChaosTest
	results  []TestResult
	mu       sync.RWMutex
	reporter *Reporter
	timeout  time.Duration
	parallel bool
}

// NewChaosRunner creates a new chaos test runner with default configuration
func NewChaosRunner() *ChaosRunner {
	return &ChaosRunner{
		tests:    make([]ChaosTest, 0),
		results:  make([]TestResult, 0),
		timeout:  5 * time.Minute,
		parallel: false,
		reporter: NewReporter(),
	}
}

// AddTest registers a chaos test for execution
func (r *ChaosRunner) AddTest(test ChaosTest) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.tests = append(r.tests, test)
}

// SetTimeout configures the maximum duration for each test phase
func (r *ChaosRunner) SetTimeout(timeout time.Duration) {
	r.timeout = timeout
}

// SetParallel enables concurrent test execution
func (r *ChaosRunner) SetParallel(enabled bool) {
	r.parallel = enabled
}

// RunAll executes all registered chaos tests sequentially or in parallel
// Returns the aggregated results for all tests
func (r *ChaosRunner) RunAll() []TestResult {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.results = make([]TestResult, 0, len(r.tests))

	if r.parallel {
		r.runParallel()
	} else {
		r.runSequential()
	}

	// Generate report
	if err := r.reporter.GenerateReport(r.results); err != nil {
		log.Printf("[ChaosRunner] Failed to generate report: %v", err)
	}

	return r.results
}

// runSequential executes tests one at a time
func (r *ChaosRunner) runSequential() {
	for _, test := range r.tests {
		result := r.executeTest(test)
		r.results = append(r.results, result)
	}
}

// runParallel executes tests concurrently with isolation
func (r *ChaosRunner) runParallel() {
	var wg sync.WaitGroup
	resultsChan := make(chan TestResult, len(r.tests))

	for _, test := range r.tests {
		wg.Add(1)
		go func(t ChaosTest) {
			defer wg.Done()
			result := r.executeTest(t)
			resultsChan <- result
		}(test)
	}

	go func() {
		wg.Wait()
		close(resultsChan)
	}()

	for result := range resultsChan {
		r.results = append(r.results, result)
	}
}

// executeTest runs a single chaos test with full lifecycle management
func (r *ChaosRunner) executeTest(test ChaosTest) TestResult {
	name := test.Name()
	startTime := time.Now()
	result := TestResult{
		Name:      name,
		Timestamp: startTime,
		Details:   make(map[string]interface{}),
	}

	log.Printf("[ChaosTest] Starting: %s", name)

	// Phase 1: Setup with timeout
	ctx, cancel := context.WithTimeout(context.Background(), r.timeout)
	setupDone := make(chan error, 1)

	go func() {
		setupDone <- test.Setup()
	}()

	select {
	case err := <-setupDone:
		if err != nil {
			result.Error = fmt.Errorf("setup failed: %w", err)
			result.Passed = false
			cancel()
			r.cleanupSafely(test)
			result.Duration = time.Since(startTime)
			log.Printf("[ChaosTest] FAILED (Setup): %s - %v", name, err)
			return result
		}
		log.Printf("[ChaosTest] Setup complete: %s", name)
	case <-ctx.Done():
		result.Error = fmt.Errorf("setup timeout exceeded")
		result.Passed = false
		cancel()
		r.cleanupSafely(test)
		result.Duration = time.Since(startTime)
		log.Printf("[ChaosTest] FAILED (Setup Timeout): %s", name)
		return result
	}
	cancel()

	// Phase 2: Inject Failure
	ctx, cancel = context.WithTimeout(context.Background(), r.timeout)
	injectDone := make(chan error, 1)

	go func() {
		injectDone <- test.InjectFailure()
	}()

	select {
	case err := <-injectDone:
		if err != nil {
			result.Error = fmt.Errorf("failure injection failed: %w", err)
			result.Passed = false
			cancel()
			r.cleanupSafely(test)
			result.Duration = time.Since(startTime)
			log.Printf("[ChaosTest] FAILED (Inject): %s - %v", name, err)
			return result
		}
		log.Printf("[ChaosTest] Failure injected: %s", name)
	case <-ctx.Done():
		result.Error = fmt.Errorf("failure injection timeout exceeded")
		result.Passed = false
		cancel()
		r.cleanupSafely(test)
		result.Duration = time.Since(startTime)
		log.Printf("[ChaosTest] FAILED (Inject Timeout): %s", name)
		return result
	}
	cancel()

	// Phase 3: Verify resilience
	ctx, cancel = context.WithTimeout(context.Background(), r.timeout)
	verifyDone := make(chan error, 1)

	go func() {
		verifyDone <- test.Verify()
	}()

	select {
	case err := <-verifyDone:
		if err != nil {
			result.Error = fmt.Errorf("verification failed: %w", err)
			result.Passed = false
			result.Details["resilient"] = false
		} else {
			result.Passed = true
			result.Details["resilient"] = true
			log.Printf("[ChaosTest] Verification passed: %s", name)
		}
	case <-ctx.Done():
		result.Error = fmt.Errorf("verification timeout exceeded")
		result.Passed = false
		result.Details["resilient"] = false
		log.Printf("[ChaosTest] FAILED (Verify Timeout): %s", name)
	}
	cancel()

	// Phase 4: Cleanup (always execute)
	r.cleanupSafely(test)

	result.Duration = time.Since(startTime)
	if result.Passed {
		log.Printf("[ChaosTest] PASSED: %s (duration: %v)", name, result.Duration)
	} else {
		log.Printf("[ChaosTest] FAILED: %s - %v (duration: %v)", name, result.Error, result.Duration)
	}

	return result
}

// cleanupSafely executes cleanup with panic recovery
func (r *ChaosRunner) cleanupSafely(test ChaosTest) {
	defer func() {
		if rec := recover(); rec != nil {
			log.Printf("[ChaosTest] Cleanup panic recovered for %s: %v", test.Name(), rec)
		}
	}()

	ctx, cancel := context.WithTimeout(context.Background(), r.timeout)
	defer cancel()

	cleanupDone := make(chan error, 1)
	go func() {
		cleanupDone <- test.Cleanup()
	}()

	select {
	case err := <-cleanupDone:
		if err != nil {
			log.Printf("[ChaosTest] Cleanup error for %s: %v", test.Name(), err)
		} else {
			log.Printf("[ChaosTest] Cleanup complete: %s", test.Name())
		}
	case <-ctx.Done():
		log.Printf("[ChaosTest] Cleanup timeout for %s", test.Name())
	}
}

// GetResults returns the results of the last test run
func (r *ChaosRunner) GetResults() []TestResult {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.results
}

// GetSummary returns a summary of test results
func (r *ChaosRunner) GetSummary() map[string]interface{} {
	r.mu.RLock()
	defer r.mu.RUnlock()

	passed := 0
	failed := 0
	var totalDuration time.Duration

	for _, result := range r.results {
		if result.Passed {
			passed++
		} else {
			failed++
		}
		totalDuration += result.Duration
	}

	return map[string]interface{}{
		"total":          len(r.results),
		"passed":         passed,
		"failed":         failed,
		"success_rate":   float64(passed) / float64(len(r.results)) * 100,
		"total_duration": totalDuration.String(),
	}
}
