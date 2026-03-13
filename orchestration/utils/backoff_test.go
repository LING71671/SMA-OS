package utils

import (
	"math"
	"math/rand"
	"testing"
	"time"
)

// Ensure deterministic randomness in tests
func seedRand(t *testing.T) {
	t.Helper()
	rand.Seed(42)
}

func TestCalculateBackoff_Scenario1(t *testing.T) {
	seedRand(t)
	cfg := DefaultBackoffConfig()

	// attempt 0: expect ~BaseDelay with jitter up to Jitter*BaseDelay
	got := CalculateBackoff(0, cfg)
	base := cfg.BaseDelay
	max := time.Duration(float64(base) * (1.0 + cfg.Jitter))
	if got < base || got > max {
		t.Fatalf("attempt 0: got %v, want in [%v, %v]", got, base, max)
	}

	// attempt 1
	got = CalculateBackoff(1, cfg)
	exp := float64(base) * math.Pow(cfg.Multiplier, 1)
	if exp > float64(cfg.MaxDelay) {
		exp = float64(cfg.MaxDelay)
	}
	base1 := time.Duration(exp)
	max1 := time.Duration(float64(base1) * (1.0 + cfg.Jitter))
	if got < base1 || got > max1 {
		t.Fatalf("attempt 1: got %v, want in [%v, %v]", got, base1, max1)
	}

	// attempt 2
	got = CalculateBackoff(2, cfg)
	exp2 := float64(base) * math.Pow(cfg.Multiplier, 2)
	if exp2 > float64(cfg.MaxDelay) {
		exp2 = float64(cfg.MaxDelay)
	}
	base2 := time.Duration(exp2)
	max2 := time.Duration(float64(base2) * (1.0 + cfg.Jitter))
	if got < base2 || got > max2 {
		t.Fatalf("attempt 2: got %v, want in [%v, %v]", got, base2, max2)
	}
}

func TestCalculateBackoff_MaxCap(t *testing.T) {
	seedRand(t)
	cfg := DefaultBackoffConfig()

	// attempt that would exceed max delay without cap
	got := CalculateBackoff(10, cfg)
	// Should be capped by MaxDelay, with jitter addition up to (1+Jitter)
	max := time.Duration(float64(cfg.MaxDelay) * (1.0 + cfg.Jitter))
	if got < cfg.MaxDelay || got > max {
		t.Fatalf("max-cap: got %v, want in [%v, %v]", got, cfg.MaxDelay, max)
	}
}
