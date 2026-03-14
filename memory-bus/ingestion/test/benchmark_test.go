package test

import (
	"context"
	"fmt"
	"sma-os/memory-bus/ingestion/internal/cache"
	"testing"
	"time"
)

// BenchmarkCacheHit measures L1 cache hit performance
// Expected: < 100μs
func BenchmarkCacheHit(b *testing.B) {
	localCache, _ := cache.NewLocalCache()
	defer localCache.Close()

	manager, _ := cache.NewCacheManager(localCache, nil)
	defer manager.Close()

	ctx := context.Background()
	loader := func(ctx context.Context) (string, error) {
		return "value", nil
	}

	// Pre-populate cache
	manager.Set(ctx, "bench-key", "cached-value")

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		manager.Get(ctx, "bench-key", loader)
	}
}

// BenchmarkCacheMiss measures cache miss + loader performance
// Expected: ~ loader latency + small overhead
func BenchmarkCacheMiss(b *testing.B) {
	localCache, _ := cache.NewLocalCache()
	defer localCache.Close()

	manager, _ := cache.NewCacheManager(localCache, nil)
	defer manager.Close()

	ctx := context.Background()
	loader := func(ctx context.Context) (string, error) {
		// Simulate API call
		time.Sleep(500 * time.Millisecond)
		return "api-value", nil
	}

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		key := fmt.Sprintf("miss-key-%d", i)
		manager.Get(ctx, key, loader)
	}
}

// BenchmarkCacheConcurrency measures concurrent access performance
func BenchmarkCacheConcurrency(b *testing.B) {
	localCache, _ := cache.NewLocalCache()
	defer localCache.Close()

	manager, _ := cache.NewCacheManager(localCache, nil)
	defer manager.Close()

	ctx := context.Background()
	loader := func(ctx context.Context) (string, error) {
		return "value", nil
	}

	// Pre-populate
	for i := 0; i < 100; i++ {
		key := fmt.Sprintf("concurrent-key-%d", i)
		manager.Set(ctx, key, "cached")
	}

	b.ResetTimer()
	b.ReportAllocs()

	b.RunParallel(func(pb *testing.PB) {
		i := 0
		for pb.Next() {
			key := fmt.Sprintf("concurrent-key-%d", i%100)
			manager.Get(ctx, key, loader)
			i++
		}
	})
}

// BenchmarkComparison compares cache vs no-cache performance
func BenchmarkComparison(b *testing.B) {
	// Without cache (naive loader)
	b.Run("WithoutCache", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			// Simulate API call without caching
			time.Sleep(1 * time.Millisecond)
		}
	})

	// With cache
	b.Run("WithCache", func(b *testing.B) {
		localCache, _ := cache.NewLocalCache()
		defer localCache.Close()
		manager, _ := cache.NewCacheManager(localCache, nil)
		defer manager.Close()

		ctx := context.Background()
		loader := func(ctx context.Context) (string, error) {
			time.Sleep(1 * time.Millisecond)
			return "value", nil
		}

		// Pre-populate
		manager.Set(ctx, "key", "cached")

		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			manager.Get(ctx, "key", loader)
		}
	})
}

// TestPerformanceSummary generates performance metrics
func TestPerformanceSummary(t *testing.T) {
	t.Log("=== Cache Performance Summary ===")

	localCache, _ := cache.NewLocalCache()
	defer localCache.Close()

	manager, _ := cache.NewCacheManager(localCache, nil)
	defer manager.Close()

	ctx := context.Background()
	loader := func(ctx context.Context) (string, error) {
		time.Sleep(500 * time.Millisecond) // Simulate slow API
		return "value", nil
	}

	// Test 1: Cold cache latency
	start := time.Now()
	manager.Get(ctx, "cold-key", loader)
	coldLatency := time.Since(start)
	t.Logf("Cold cache latency: %v", coldLatency)

	// Test 2: Warm cache latency
	start = time.Now()
	for i := 0; i < 1000; i++ {
		manager.Get(ctx, "cold-key", loader)
	}
	warmLatency := time.Since(start) / 1000
	t.Logf("Average warm cache latency: %v", warmLatency)

	// Test 3: Throughput
	start = time.Now()
	for i := 0; i < 10000; i++ {
		manager.Get(ctx, "cold-key", loader)
	}
	duration := time.Since(start)
	throughput := float64(10000) / duration.Seconds()
	t.Logf("Throughput: %.2f ops/sec", throughput)

	t.Log("=== End Performance Summary ===")
}
