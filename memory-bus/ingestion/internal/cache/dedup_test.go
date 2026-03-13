package cache

import (
	"context"
	"runtime"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

func TestSingleflightDedup(t *testing.T) {
	d := &Dedup{}
	var loaderCalls int32

	loader := func(ctx context.Context) (interface{}, error) {
		atomic.AddInt32(&loaderCalls, 1)
		// simulate some work
		time.Sleep(20 * time.Millisecond)
		return "value", nil
	}

	ctx := context.Background()
	var wg sync.WaitGroup
	results := make([]interface{}, 10)
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			v, err := d.Do(ctx, "shared-key", loader)
			if err != nil {
				t.Errorf("unexpected error: %v", err)
				return
			}
			results[idx] = v
		}(i)
	}
	wg.Wait()

	// All results should be identical and come from a single loader invocation
	if loaderCalls != 1 {
		t.Fatalf("expected loader to be called once, got %d", loaderCalls)
	}
	// verify that all results are equal
	for i := 1; i < len(results); i++ {
		if results[i] != results[0] {
			t.Fatalf("mismatched results at %d: %#v vs %#v", i, results[0], results[i])
		}
	}
}

func TestSingleflightDedup_ContextCancellation(t *testing.T) {
	d := &Dedup{}
	// loader that respects context cancellation
	loader := func(ctx context.Context) (interface{}, error) {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		default:
		}
		time.Sleep(40 * time.Millisecond)
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		default:
		}
		return "ok", nil
	}

	// short deadline to trigger cancellation
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Millisecond)
	defer cancel()
	_, err := d.Do(ctx, "cancel-key", loader)
	if err == nil {
		t.Fatalf("expected cancellation error, got nil")
	}
}
