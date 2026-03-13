package cache

import (
	"context"
	"golang.org/x/sync/singleflight"
	"sync/atomic"
)

// Dedup is a light-weight wrapper around singleflight.Group providing a
// generic Do method that deduplicates concurrent loads for the same key.
// It also exposes a simple in-flight load metric via DedupCount().
type Dedup struct {
	g                 singleflight.Group
	totalCalls        int64 // total number of Do invocations
	loaderInvocations int64 // number of times the loader function actually executed
}

// Do executes the provided loader function for the given key, ensuring that
// concurrent calls for the same key are deduplicated. The loader receives the
// provided ctx and may honor cancellation.
// The loader returns an interface{} to support non-generic usage.
func (d *Dedup) Do(ctx context.Context, key string, fn func(context.Context) (interface{}, error)) (interface{}, error) {
	atomic.AddInt64(&d.totalCalls, 1)
	v, err, _ := d.g.Do(key, func() (interface{}, error) {
		val, e := fn(ctx)
		if e == nil {
			atomic.AddInt64(&d.loaderInvocations, 1)
		}
		return val, e
	})
	if err != nil {
		return nil, err
	}
	return v, nil
}

// DedupCount returns the number of deduplicated calls for the previously loaded keys.
// This equals total Do invocations minus the number of actual loader executions.
func (d *Dedup) DedupCount() int64 {
	total := atomic.LoadInt64(&d.totalCalls)
	loads := atomic.LoadInt64(&d.loaderInvocations)
	return total - loads
}
