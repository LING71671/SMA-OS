package cache

import (
	"context"
	"testing"
	"time"
)

// TestCacheManager_Get tests the multi-level cache hierarchy
func TestCacheManager_Get(t *testing.T) {
	// Create local cache
	localCache, err := NewLocalCache()
	if err != nil {
		t.Fatalf("Failed to create local cache: %v", err)
	}
	defer localCache.Close()

	// Create manager without Redis (L2 disabled for test)
	manager, err := NewCacheManager(localCache, nil)
	if err != nil {
		t.Fatalf("Failed to create cache manager: %v", err)
	}

	// Test loader function
	loaderCallCount := 0
	loader := func(ctx context.Context) (string, error) {
		loaderCallCount++
		return "test-value", nil
	}

	// First call - should hit loader
	ctx := context.Background()
	val1, err := manager.Get(ctx, "test-key", loader)
	if err != nil {
		t.Fatalf("First Get failed: %v", err)
	}
	if val1 != "test-value" {
		t.Errorf("Expected 'test-value', got '%s'", val1)
	}
	if loaderCallCount != 1 {
		t.Errorf("Expected loader called once, got %d", loaderCallCount)
	}

	// Second call - should hit L1 cache, loader not called
	val2, err := manager.Get(ctx, "test-key", loader)
	if err != nil {
		t.Fatalf("Second Get failed: %v", err)
	}
	if val2 != "test-value" {
		t.Errorf("Expected 'test-value', got '%s'", val2)
	}
	if loaderCallCount != 1 {
		t.Errorf("Expected loader still called once (cache hit), got %d", loaderCallCount)
	}
}

// TestCacheManager_Set tests setting values
func TestCacheManager_Set(t *testing.T) {
	localCache, err := NewLocalCache()
	if err != nil {
		t.Fatalf("Failed to create local cache: %v", err)
	}
	defer localCache.Close()

	manager, err := NewCacheManager(localCache, nil)
	if err != nil {
		t.Fatalf("Failed to create cache manager: %v", err)
	}

	ctx := context.Background()

	// Set a value
	err = manager.Set(ctx, "set-key", "set-value")
	if err != nil {
		t.Fatalf("Set failed: %v", err)
	}

	// Verify via Get
	loader := func(ctx context.Context) (string, error) {
		return "loader-value", nil
	}

	val, err := manager.Get(ctx, "set-key", loader)
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if val != "set-value" {
		t.Errorf("Expected 'set-value', got '%s'", val)
	}
}

// TestCacheKey tests key generation is deterministic
func TestCacheKey(t *testing.T) {
	key1 := cacheKey("test-input")
	key2 := cacheKey("test-input")

	if key1 != key2 {
		t.Errorf("Cache key not deterministic: '%s' vs '%s'", key1, key2)
	}

	// Should be different for different inputs
	key3 := cacheKey("different-input")
	if key1 == key3 {
		t.Error("Cache keys should be different for different inputs")
	}

	// Should have prefix
	if len(key1) < 7 || key1[:7] != "intent:" {
		t.Errorf("Cache key should have 'intent:' prefix, got: %s", key1)
	}
}

// TestCacheManager_Dedup tests singleflight deduplication
func TestCacheManager_Dedup(t *testing.T) {
	localCache, err := NewLocalCache()
	if err != nil {
		t.Fatalf("Failed to create local cache: %v", err)
	}
	defer localCache.Close()

	manager, err := NewCacheManager(localCache, nil)
	if err != nil {
		t.Fatalf("Failed to create cache manager: %v", err)
	}

	// Track loader calls
	loaderCalls := 0
	loader := func(ctx context.Context) (string, error) {
		time.Sleep(50 * time.Millisecond) // Simulate slow operation
		loaderCalls++
		return "dedup-value", nil
	}

	ctx := context.Background()
	key := "dedup-test-key"

	// Start 10 concurrent requests
	done := make(chan bool, 10)
	for i := 0; i < 10; i++ {
		go func() {
			_, _ = manager.Get(ctx, key, loader)
			done <- true
		}()
	}

	// Wait for all to complete
	for i := 0; i < 10; i++ {
		<-done
	}

	// Loader should only be called once due to singleflight
	if loaderCalls != 1 {
		t.Errorf("Expected loader called once (singleflight), got %d", loaderCalls)
	}

	// Verify dedup stats
	stats := manager.DedupStats()
	if stats < 9 {
		t.Errorf("Expected at least 9 deduplicated requests, got %d", stats)
	}
}
