package cache

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"time"

	"sma-os/memory-bus/ingestion/internal/metrics"
)

// CacheManager implements a two-level cache: L1 (local) and L2 (Redis).
// It supports singleflight deduplication and automatic refill on cache misses.
type CacheManager struct {
	l1    *LocalCache  // Local ristretto cache (L1)
	l2    *RedisClient // Redis client (L2)
	dedup *Dedup       // Singleflight deduplication

	l1TTL time.Duration // L1 TTL (default: 5 minutes)
	l2TTL time.Duration // L2 TTL (default: 1 hour)
}

// NewCacheManager creates a new CacheManager with the provided clients.
// It uses sensible defaults: L1 TTL = 5 min, L2 TTL = 1 hour.
// Returns an error if l1 is nil (L1 cache is required).
func NewCacheManager(l1 *LocalCache, l2 *RedisClient) (*CacheManager, error) {
	if l1 == nil {
		return nil, fmt.Errorf("L1 cache is required but was nil")
	}
	return &CacheManager{
		l1:    l1,
		l2:    l2,
		dedup: &Dedup{},
		l1TTL: 5 * time.Minute,
		l2TTL: time.Hour,
	}, nil
}

// cacheKey generates a deterministic cache key from the input string.
func cacheKey(input string) string {
	hash := sha256.Sum256([]byte(input))
	return fmt.Sprintf("intent:%s", hex.EncodeToString(hash[:]))
}

// Get retrieves a value from the cache hierarchy: L1 -> L2 -> loader.
// It uses singleflight to prevent duplicate loader calls for the same key.
func (m *CacheManager) Get(ctx context.Context, input string, loader func(context.Context) (string, error)) (string, error) {
	key := cacheKey(input)

	// 1. Try L1 (local cache)
	if val, ok := m.l1.Get(key); ok {
		if str, valid := val.(string); valid {
			metrics.RecordHit("l1")
			return str, nil
		}
	}

	// L1 miss - record it
	metrics.RecordMiss("l1")

	// 2. Try L2 (Redis)
	if m.l2 != nil {
		val, err := m.l2.Get(ctx, key)
		if err == nil && val != "" {
			// Refill L1
			m.l1.SetWithTTL(key, val, m.l1TTL)
			metrics.RecordHit("l2")
			return val, nil
		}
		// L2 miss (Redis returned empty or error)
		metrics.RecordMiss("l2")
	}

	// 3. Cache miss: use singleflight to prevent duplicate loads
	result, err := m.dedup.Do(ctx, key, func(ctx context.Context) (interface{}, error) {
		val, err := loader(ctx)
		if err != nil {
			return nil, err
		}

		// Store in both caches
		m.l1.SetWithTTL(key, val, m.l1TTL)
		if m.l2 != nil {
			_ = m.l2.Set(ctx, key, val, m.l2TTL)
		}

		return val, nil
	})

	if err != nil {
		return "", err
	}

	return result.(string), nil
}

// Set stores a value in both L1 and L2 caches.
func (m *CacheManager) Set(ctx context.Context, input string, value string) error {
	key := cacheKey(input)

	m.l1.SetWithTTL(key, value, m.l1TTL)
	if m.l2 != nil {
		return m.l2.Set(ctx, key, value, m.l2TTL)
	}
	return nil
}

// Delete removes a value from both L1 and L2 caches.
func (m *CacheManager) Delete(ctx context.Context, input string) error {
	key := cacheKey(input)

	m.l1.Delete(key)
	if m.l2 != nil {
		return m.l2.Delete(ctx, key)
	}
	return nil
}

// Close cleans up resources.
func (m *CacheManager) Close() error {
	m.l1.Close()
	if m.l2 != nil {
		return m.l2.Close()
	}
	return nil
}

// DedupStats returns deduplication statistics.
func (m *CacheManager) DedupStats() int64 {
	return m.dedup.DedupCount()
}
