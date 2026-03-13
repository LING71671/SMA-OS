package cache

import (
	"fmt"
	"testing"
	"time"
)

// TestLocalCache validates that SetWithTTL and Get operate correctly and that
// a large number of entries can be stored and retrieved without errors.
func TestLocalCache(t *testing.T) {
	lc, err := NewLocalCache()
	if err != nil {
		t.Fatalf("NewLocalCache error: %v", err)
	}
	defer lc.Close()

	ttl := 5 * time.Second
	// Populate cache with 10k entries
	for i := 0; i < 10000; i++ {
		key := fmt.Sprintf("k%d", i)
		lc.SetWithTTL(key, i, ttl)
	}

	// Retrieve all and verify correctness
	hits := 0
	for i := 0; i < 10000; i++ {
		key := fmt.Sprintf("k%d", i)
		v, ok := lc.Get(key)
		if ok {
			if val, ok := v.(int); ok && val == i {
				hits++
			}
		}
	}
	if hits != 10000 {
		t.Fatalf("expected 10000 hits, got %d", hits)
	}
}
