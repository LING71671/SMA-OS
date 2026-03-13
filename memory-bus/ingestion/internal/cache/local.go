package cache

import (
	"sync"
	"time"

	"github.com/dgraph-io/ristretto"
)

// LocalCache is a wrapper around ristretto.Cache with per-key TTL support.
// It provides SetWithTTL and Get methods and a Close method for cleanup.
type LocalCache struct {
	cache *ristretto.Cache
	ttl   map[string]time.Time
	mu    sync.RWMutex
}

// NewLocalCache initializes a ristretto-based cache with the required
// configuration: 10000 counters and a max cost of 100MB.
func NewLocalCache() (*LocalCache, error) {
	cfg := &ristretto.Config{
		NumCounters: 10000,             // number of keys to track
		MaxCost:     100 * 1024 * 1024, // maximum cost of cache (bytes)
		BufferItems: 64,                // recommended default
	}
	c, err := ristretto.NewCache(cfg)
	if err != nil {
		return nil, err
	}
	return &LocalCache{cache: c, ttl: make(map[string]time.Time)}, nil
}

// SetWithTTL stores a value with an optional TTL. If ttl is zero, the
// value will live without expiration.
func (l *LocalCache) SetWithTTL(key string, value interface{}, ttl time.Duration) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.cache.Set(key, value, 1)
	if ttl > 0 {
		l.ttl[key] = time.Now().Add(ttl)
	} else {
		delete(l.ttl, key)
	}
}

// Get retrieves a value if present and not expired.
func (l *LocalCache) Get(key string) (interface{}, bool) {
	l.mu.Lock()
	defer l.mu.Unlock()
	// Check expiration first
	if exp, ok := l.ttl[key]; ok {
		if time.Now().After(exp) {
			l.cache.Del(key)
			delete(l.ttl, key)
			return nil, false
		}
	}
	val, ok := l.cache.Get(key)
	return val, ok
}

// Close cleans up underlying resources.
func (l *LocalCache) Close() {
	l.mu.Lock()
	defer l.mu.Unlock()
	if l.cache != nil {
		l.cache.Close()
		l.cache = nil
	}
	l.ttl = nil
}
