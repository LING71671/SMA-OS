package metrics

import (
	"fmt"
	"sync"
	"sync/atomic"

	"github.com/prometheus/client_golang/prometheus"
)

var (
	cacheHitsTotal      *prometheus.CounterVec
	cacheMissesTotal    *prometheus.CounterVec
	cacheHitRatio       *prometheus.GaugeVec
	dedupPreventedTotal prometheus.Counter
	apiCallsTotal       prometheus.Counter

	// per-tier counters for ratio calculation
	tierHits   = make(map[string]uint64)
	tierMisses = make(map[string]uint64)
	mu         sync.Mutex
	// textual metric store (in-process counter for API calls)
	apiCalls uint64
)

func init() {
	cacheHitsTotal = prometheus.NewCounterVec(
		prometheus.CounterOpts{Namespace: "memory_bus", Subsystem: "ingestion_cache", Name: "cache_hits_total", Help: "Cache hits by tier"},
		[]string{"tier"},
	)
	cacheMissesTotal = prometheus.NewCounterVec(
		prometheus.CounterOpts{Namespace: "memory_bus", Subsystem: "ingestion_cache", Name: "cache_misses_total", Help: "Cache misses by tier"},
		[]string{"tier"},
	)
	cacheHitRatio = prometheus.NewGaugeVec(
		prometheus.GaugeOpts{Namespace: "memory_bus", Subsystem: "ingestion_cache", Name: "cache_hit_ratio", Help: "Cache hit ratio by tier"},
		[]string{"tier"},
	)
	dedupPreventedTotal = prometheus.NewCounter(prometheus.CounterOpts{Namespace: "memory_bus", Subsystem: "ingestion_cache", Name: "dedup_prevented_total", Help: "Total deduped loader requests"})
	apiCallsTotal = prometheus.NewCounter(prometheus.CounterOpts{Namespace: "memory_bus", Subsystem: "ingestion_cache", Name: "api_calls_total", Help: "API calls to underlying cache service"})

	// register metrics
	prometheus.MustRegister(cacheHitsTotal, cacheMissesTotal, cacheHitRatio, dedupPreventedTotal, apiCallsTotal)
}

// RecordHit records a hit for a given tier and updates the ratio.
func RecordHit(tier string) {
	mu.Lock()
	tierHits[tier]++
	hits := tierHits[tier]
	misses := tierMisses[tier]
	total := hits + misses
	var ratio float64
	if total > 0 {
		ratio = float64(hits) / float64(total)
	}
	mu.Unlock()

	// Update Prometheus counters after releasing lock (they're thread-safe)
	cacheHitRatio.WithLabelValues(tier).Set(ratio)
	cacheHitsTotal.WithLabelValues(tier).Inc()
}

// RecordMiss records a miss for a given tier and updates the ratio.
func RecordMiss(tier string) {
	mu.Lock()
	tierMisses[tier]++
	hits := tierHits[tier]
	misses := tierMisses[tier]
	total := hits + misses
	var ratio float64
	if total > 0 {
		ratio = float64(hits) / float64(total)
	}
	mu.Unlock()

	// Update Prometheus counters after releasing lock (they're thread-safe)
	cacheHitRatio.WithLabelValues(tier).Set(ratio)
	cacheMissesTotal.WithLabelValues(tier).Inc()
}

// RecordAPICall increments the API call counter for the cache layer.
func RecordAPICall() {
	atomic.AddUint64(&apiCalls, 1)
}

// MetricsText returns a simple text representation of the most relevant cache metrics
// to satisfy the /metrics endpoint in environments without Prometheus exposition endpoint.
func MetricsText() string {
	mu.Lock()
	defer mu.Unlock()
	hits := tierHits["l1"]
	misses := tierMisses["l1"]
	total := hits + misses
	ratio := 0.0
	if total > 0 {
		ratio = float64(hits) / float64(total)
	}
	api := atomic.LoadUint64(&apiCalls)
	return fmt.Sprintf("cache_hits_total{tier=\"l1\"} %d\ncache_misses_total{tier=\"l1\"} %d\ncache_hit_ratio{tier=\"l1\"} %.6f\napi_calls_total %d\n", hits, misses, ratio, api)
}
