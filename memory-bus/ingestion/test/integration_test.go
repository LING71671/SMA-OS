package test

import (
	"context"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"testing"
	"time"

	"sma-os/memory-bus/ingestion/internal/cache"
)

// TestIntegration_CacheFlow validates full cache flow
func TestIntegration_CacheFlow(t *testing.T) {
	localCache, err := cache.NewLocalCache()
	if err != nil {
		t.Fatalf("Failed to create local cache: %v", err)
	}
	defer localCache.Close()

	manager, err := cache.NewCacheManager(localCache, nil)
	if err != nil {
		t.Fatalf("Failed to create cache manager: %v", err)
	}
	defer manager.Close()

	ctx := context.Background()
	loaderCalls := 0
	loader := func(ctx context.Context) (string, error) {
		loaderCalls++
		return fmt.Sprintf("loaded-value-%d", loaderCalls), nil
	}

	// Step 1: First call should hit loader
	val1, err := manager.Get(ctx, "test-key", loader)
	if err != nil {
		t.Fatalf("First call failed: %v", err)
	}
	if loaderCalls != 1 {
		t.Errorf("Expected 1 loader call, got %d", loaderCalls)
	}
	t.Logf("First call result: %s", val1)

	// Step 2: Second call should use cache
	val2, err := manager.Get(ctx, "test-key", loader)
	if err != nil {
		t.Fatalf("Second call failed: %v", err)
	}
	if loaderCalls != 1 {
		t.Errorf("Expected still 1 loader call, got %d", loaderCalls)
	}
	if val1 != val2 {
		t.Errorf("Cache returned different values: %s vs %s", val1, val2)
	}
	t.Logf("Second call result: %s (from cache)", val2)

	// Step 3: Set new value
	err = manager.Set(ctx, "test-key", "direct-set-value")
	if err != nil {
		t.Fatalf("Set failed: %v", err)
	}

	// Step 4: Get should return new value (cache should be updated)
	val3, err := manager.Get(ctx, "test-key", loader)
	if err != nil {
		t.Fatalf("Third call failed: %v", err)
	}
	// Note: Set updates both caches, so Get should return new value
	// But loader shouldn't be called since value is in cache
	t.Logf("After set, result: %s", val3)

	// Step 5: Delete
	err = manager.Delete(ctx, "test-key")
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	// Step 6: Get after delete should hit loader again
	loaderCallsBefore := loaderCalls
	val4, err := manager.Get(ctx, "test-key", loader)
	if err != nil {
		t.Fatalf("Call after delete failed: %v", err)
	}
	if loaderCalls <= loaderCallsBefore {
		t.Errorf("Expected loader to be called after delete")
	}
	t.Logf("After delete, result: %s", val4)

	t.Log("=== Cache Flow Test Complete ===")
}

// TestIntegration_MetricsEndpoint validates metrics endpoint
func TestIntegration_MetricsEndpoint(t *testing.T) {
	// This test assumes the metrics server is running
	// In real scenario, start the ingestion service

	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Get("http://localhost:8080/metrics")
	if err != nil {
		t.Skipf("Metrics endpoint not available: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Errorf("Metrics endpoint returned %d", resp.StatusCode)
	}

	// Verify content type
	contentType := resp.Header.Get("Content-Type")
	if !strings.Contains(contentType, "text/plain") {
		t.Errorf("Unexpected content type: %s", contentType)
	}

	t.Log("Metrics endpoint is accessible")
}

// TestIntegration_ConcurrentCacheAccess validates concurrent operations
func TestIntegration_ConcurrentCacheAccess(t *testing.T) {
	localCache, _ := cache.NewLocalCache()
	defer localCache.Close()

	manager, _ := cache.NewCacheManager(localCache, nil)
	defer manager.Close()

	ctx := context.Background()
	loader := func(ctx context.Context) (string, error) {
		time.Sleep(10 * time.Millisecond)
		return "concurrent-value", nil
	}

	// Concurrent writes
	for i := 0; i < 100; i++ {
		go func(id int) {
			key := fmt.Sprintf("concurrent-key-%d", id)
			manager.Set(ctx, key, fmt.Sprintf("value-%d", id))
		}(i)
	}

	time.Sleep(100 * time.Millisecond)

	// Concurrent reads
	var wg sync.WaitGroup
	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			key := fmt.Sprintf("concurrent-key-%d", id%50) // 50 unique keys
			manager.Get(ctx, key, loader)
		}(i)
	}
	wg.Wait()

	t.Log("Concurrent cache access completed successfully")
}

// TestIntegration_GracefulShutdown validates cleanup
func TestIntegration_GracefulShutdown(t *testing.T) {
	localCache, _ := cache.NewLocalCache()
	manager, _ := cache.NewCacheManager(localCache, nil)

	// Perform some operations
	ctx := context.Background()
	manager.Set(ctx, "key", "value")

	// Graceful shutdown
	err := manager.Close()
	if err != nil {
		t.Errorf("Close failed: %v", err)
	}

	t.Log("Graceful shutdown completed")
}
