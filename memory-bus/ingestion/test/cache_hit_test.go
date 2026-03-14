package test

import (
	"context"
	"fmt"
	"sma-os/memory-bus/ingestion/internal/cache"
	"testing"
	"time"
)

// TestCacheHitRate validates cache hit ratio meets expectations
// Expected: > 75% hit rate with 80% duplicate keys
func TestCacheHitRate(t *testing.T) {
	// Initialize caches
	localCache, err := cache.NewLocalCache()
	if err != nil {
		t.Fatalf("Failed to create local cache: %v", err)
	}
	defer localCache.Close()

	redisClient, err := cache.NewRedisClient()
	if err != nil {
		t.Logf("Warning: Redis not available, using local cache only: %v", err)
		redisClient = nil
	}

	manager, err := cache.NewCacheManager(localCache, redisClient)
	if err != nil {
		t.Fatalf("Failed to create cache manager: %v", err)
	}
	defer manager.Close()

	// Test configuration
	totalRequests := 1000
	uniqueKeys := 200 // 20% unique = 80% duplicates
	hits := 0
	misses := 0
	loaderCalls := 0

	loader := func(ctx context.Context) (string, error) {
		loaderCalls++
		return fmt.Sprintf("value-for-key-%d", time.Now().UnixNano()), nil
	}

	ctx := context.Background()

	// Send requests
	for i := 0; i < totalRequests; i++ {
		// Generate key (80% will be duplicates of first 80% of unique keys)
		keyID := i % uniqueKeys
		key := fmt.Sprintf("test-key-%d", keyID)

		// Try to get from cache
		_, err := manager.Get(ctx, key, loader)
		if err != nil {
			t.Logf("Request %d failed: %v", i, err)
			misses++
			continue
		}

		// Determine if it was a hit or miss based on loader calls
		// First request for each key is a miss, subsequent are hits
		if loaderCalls <= keyID+1 && i >= uniqueKeys {
			// After warm-up, most should be hits
			hits++
		} else {
			misses++
		}
	}

	// Calculate actual hit rate
	// First `uniqueKeys` requests will be misses (cold cache)
	// Remaining requests should be hits
	expectedHits := totalRequests - uniqueKeys
	expectedHitRate := float64(expectedHits) / float64(totalRequests)
	actualHitRate := float64(hits) / float64(totalRequests)

	t.Logf("Total Requests: %d", totalRequests)
	t.Logf("Unique Keys: %d", uniqueKeys)
	t.Logf("Loader Calls: %d", loaderCalls)
	t.Logf("Expected Hit Rate: %.2f%%", expectedHitRate*100)
	t.Logf("Actual Hit Rate: %.2f%%", actualHitRate*100)

	// Verify hit rate > 75%
	if actualHitRate < 0.75 {
		t.Errorf("Hit rate %.2f%% is below expected 75%%", actualHitRate*100)
	}

	// Verify loader was called at most uniqueKeys times (singleflight dedup)
	if loaderCalls > uniqueKeys {
		t.Errorf("Loader called %d times for %d unique keys (expected <= %d)",
			loaderCalls, uniqueKeys, uniqueKeys)
	}
}

// TestCacheLatency validates cache latency requirements
// Expected: L1 hit < 1ms, L2 hit < 5ms
func TestCacheLatency(t *testing.T) {
	localCache, err := cache.NewLocalCache()
	if err != nil {
		t.Fatalf("Failed to create local cache: %v", err)
	}
	defer localCache.Close()

	redisClient, err := cache.NewRedisClient()
	if err != nil {
		t.Skipf("Redis not available, skipping L2 latency test: %v", err)
	}

	manager, err := cache.NewCacheManager(localCache, redisClient)
	if err != nil {
		t.Fatalf("Failed to create cache manager: %v", err)
	}
	defer manager.Close()

	ctx := context.Background()
	key := "latency-test-key"
	loader := func(ctx context.Context) (string, error) {
		return "test-value", nil
	}

	// First call - cold cache (L1 miss, L2 miss)
	start := time.Now()
	_, _ = manager.Get(ctx, key, loader)
	coldLatency := time.Since(start)
	t.Logf("Cold cache latency: %v", coldLatency)

	// Second call - L1 hit
	start = time.Now()
	_, _ = manager.Get(ctx, key, loader)
	l1Latency := time.Since(start)
	t.Logf("L1 hit latency: %v", l1Latency)

	// Verify L1 hit < 100μs (100 microseconds)
	if l1Latency > 100*time.Microsecond {
		t.Errorf("L1 hit latency %v exceeds 100μs threshold", l1Latency)
	}

	// Verify L1 hit < 1ms (1000 microseconds)
	if l1Latency > time.Millisecond {
		t.Errorf("L1 hit latency %v exceeds 1ms threshold", l1Latency)
	}
}

// BenchmarkCacheOperations provides performance metrics
func BenchmarkCacheGet(b *testing.B) {
	localCache, _ := cache.NewLocalCache()
	defer localCache.Close()

	manager, _ := cache.NewCacheManager(localCache, nil)
	defer manager.Close()

	ctx := context.Background()
	loader := func(ctx context.Context) (string, error) {
		return "benchmark-value", nil
	}

	// Pre-populate cache
	for i := 0; i < 1000; i++ {
		key := fmt.Sprintf("bench-key-%d", i)
		manager.Set(ctx, key, "cached-value")
	}

	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		i := 0
		for pb.Next() {
			key := fmt.Sprintf("bench-key-%d", i%1000)
			manager.Get(ctx, key, loader)
			i++
		}
	})
}
