package utils

import (
	"math"
	"math/rand"
	"time"
)

// BackoffConfig defines backoff parameters
type BackoffConfig struct {
	BaseDelay  time.Duration
	MaxDelay   time.Duration
	Multiplier float64
	Jitter     float64 // 0.0 to 1.0
}

// DefaultBackoffConfig returns sensible defaults
func DefaultBackoffConfig() BackoffConfig {
	return BackoffConfig{
		BaseDelay:  100 * time.Millisecond,
		MaxDelay:   5 * time.Second,
		Multiplier: 2.0,
		Jitter:     0.25,
	}
}

// CalculateBackoff computes delay for a given retry attempt
// Formula: min(base * multiplier^attempt, max) + jitter
func CalculateBackoff(attempt int, config BackoffConfig) time.Duration {
	// Calculate exponential component
	exp := float64(config.BaseDelay) * math.Pow(config.Multiplier, float64(attempt))

	// Cap at max delay
	capped := math.Min(exp, float64(config.MaxDelay))

	// Add jitter (random percentage of capped)
	jitter := rand.Float64() * config.Jitter * capped

	return time.Duration(capped + jitter)
}

// CalculateBackoffWithDefaults uses default config
func CalculateBackoffWithDefaults(attempt int) time.Duration {
	return CalculateBackoff(attempt, DefaultBackoffConfig())
}

// LinearBackoff returns linear delay (for comparison/testing)
func LinearBackoff(attempt int, baseDelay time.Duration) time.Duration {
	return baseDelay * time.Duration(attempt+1)
}
