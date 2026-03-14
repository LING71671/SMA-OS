package test

import (
	"context"
	"sma-os/memory-bus/ingestion/internal/cache"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// TestCacheStampedeProtection validates singleflight prevents cache stampede
func TestCacheStampedeProtection(t *testing.T) {
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

	concurrentRequests := 100
	loaderCallCount := int32(0)
	key := "stampede-test-key"

	loader := func(ctx context.Context) (string, error) {
		time.Sleep(100 * time.Millisecond)
		atomic.AddInt32(&loaderCallCount, 1)
		return "expensive-value", nil
	}

	ctx := context.Background()
	var wg sync.WaitGroup
	results := make(chan string, concurrentRequests)

	for i := 0; i < concurrentRequests; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			val, err := manager.Get(ctx, key, loader)
			if err != nil {
				return
			}
			results <- val
		}(i)
	}

	wg.Wait()
	close(results)

	resultCount := 0
	for range results {
		resultCount++
	}

	t.Logf("Concurrent requests: %d", concurrentRequests)
	t.Logf("Successful results: %d", resultCount)
	t.Logf("Loader calls: %d", loaderCallCount)

	if loaderCallCount != 1 {
		t.Errorf("Loader called %d times for %d concurrent requests (expected 1)",
			loaderCallCount, concurrentRequests)
	}
}

// BenchmarkStampedeProtection benchmarks singleflight performance
func BenchmarkStampedeProtection(b *testing.B) {
	localCache, _ := cache.NewLocalCache()
	defer localCache.Close()

	manager, _ := cache.NewCacheManager(localCache, nil)
	defer manager.Close()

	ctx := context.Background()
	loaderCalls := int32(0)

	loader := func(ctx context.Context) (string, error) {
		atomic.AddInt32(&loaderCalls, 1)
		time.Sleep(10 * time.Millisecond)
		return "value", nil
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var wg sync.WaitGroup
		for j := 0; j < 10; j++ {
			wg.Add(1)
			go func() {
				defer wg.Done()
				manager.Get(ctx, "bench-key", loader)
			}()
		}
		wg.Wait()
	}
}
