// Package config provides configuration management for the SMA-OS orchestration layer.
//
// This package includes feature flag management with support for:
// - Dynamic flag toggling
// - Context propagation
// - Environment-based configuration
// - Percentage-based rollouts
// - User/tenant targeting
//
// Example usage:
//
//	// Initialize feature flags
//	flags := config.NewFeatureFlags()
//
//	// Register a feature flag
//	flags.Register(config.FeatureFlag{
//		Name:        "new-scheduler",
//		Description: "Enable the new scheduler implementation",
//		Default:     false,
//	})
//
//	// Check if feature is enabled
//	if flags.IsEnabled("new-scheduler") {
//		// Use new implementation
//	}
//
//	// With context propagation
//	ctx := flags.WithFlag(context.Background(), "new-scheduler", true)
//	enabled := flags.IsEnabledInContext(ctx, "new-scheduler")
package config

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"
)

// FeatureFlag represents a single feature flag configuration.
type FeatureFlag struct {
	// Name is the unique identifier for the flag
	Name string `json:"name"`

	// Description explains what the flag controls
	Description string `json:"description"`

	// Default is the default state of the flag
	Default bool `json:"default"`

	// Enabled indicates if the flag is currently enabled
	Enabled bool `json:"enabled"`

	// RolloutPercentage controls gradual rollout (0-100)
	RolloutPercentage int `json:"rollout_percentage"`

	// TargetTenants limits the flag to specific tenants
	TargetTenants []string `json:"target_tenants,omitempty"`

	// TargetUsers limits the flag to specific users
	TargetUsers []string `json:"target_users,omitempty"`

	// CreatedAt is when the flag was created
	CreatedAt time.Time `json:"created_at"`

	// UpdatedAt is when the flag was last modified
	UpdatedAt time.Time `json:"updated_at"`

	// Metadata stores additional flag-specific data
	Metadata map[string]string `json:"metadata,omitempty"`
}

// IsEnabledFor checks if the flag is enabled for a specific context.
func (f *FeatureFlag) IsEnabledFor(tenantID, userID string) bool {
	// Check if explicitly disabled
	if !f.Enabled {
		return false
	}

	// Check tenant targeting
	if len(f.TargetTenants) > 0 {
		found := false
		for _, t := range f.TargetTenants {
			if t == tenantID {
				found = true
				break
			}
		}
		if !found {
			return false
		}
	}

	// Check user targeting
	if len(f.TargetUsers) > 0 {
		found := false
		for _, u := range f.TargetUsers {
			if u == userID {
				found = true
				break
			}
		}
		if !found {
			return false
		}
	}

	// Check percentage rollout
	if f.RolloutPercentage < 100 {
		// Use a simple hash-based approach for consistent rollout
		hash := hashString(tenantID + userID + f.Name)
		return hash%100 < uint32(f.RolloutPercentage)
	}

	return true
}

// FeatureFlags manages all feature flags in the system.
type FeatureFlags struct {
	flags map[string]*FeatureFlag
	mu    sync.RWMutex

	// environmentPrefix is the prefix for environment variables
	environmentPrefix string

	// contextKey is used for context propagation
	contextKey struct{}
}

// NewFeatureFlags creates a new feature flag manager.
func NewFeatureFlags() *FeatureFlags {
	return &FeatureFlags{
		flags:             make(map[string]*FeatureFlag),
		environmentPrefix: "SMA_FF_",
	}
}

// NewFeatureFlagsWithPrefix creates a new feature flag manager with custom prefix.
func NewFeatureFlagsWithPrefix(prefix string) *FeatureFlags {
	return &FeatureFlags{
		flags:             make(map[string]*FeatureFlag),
		environmentPrefix: prefix,
	}
}

// Register registers a new feature flag.
func (ff *FeatureFlags) Register(flag FeatureFlag) error {
	ff.mu.Lock()
	defer ff.mu.Unlock()

	if flag.Name == "" {
		return fmt.Errorf("feature flag name cannot be empty")
	}

	// Check if already exists
	if _, exists := ff.flags[flag.Name]; exists {
		return fmt.Errorf("feature flag %q already exists", flag.Name)
	}

	// Set timestamps
	now := time.Now()
	flag.CreatedAt = now
	flag.UpdatedAt = now

	// Apply environment variable override
	envValue := os.Getenv(ff.environmentPrefix + strings.ToUpper(flag.Name))
	if envValue != "" {
		if enabled, err := strconv.ParseBool(envValue); err == nil {
			flag.Enabled = enabled
		}
	}

	ff.flags[flag.Name] = &flag
	return nil
}

// RegisterMultiple registers multiple feature flags at once.
func (ff *FeatureFlags) RegisterMultiple(flags []FeatureFlag) error {
	for _, flag := range flags {
		if err := ff.Register(flag); err != nil {
			return err
		}
	}
	return nil
}

// Get retrieves a feature flag by name.
func (ff *FeatureFlags) Get(name string) (*FeatureFlag, error) {
	ff.mu.RLock()
	defer ff.mu.RUnlock()

	flag, exists := ff.flags[name]
	if !exists {
		return nil, fmt.Errorf("feature flag %q not found", name)
	}

	return flag, nil
}

// IsEnabled checks if a feature flag is enabled globally.
func (ff *FeatureFlags) IsEnabled(name string) bool {
	ff.mu.RLock()
	defer ff.mu.RUnlock()

	flag, exists := ff.flags[name]
	if !exists {
		return false
	}

	return flag.Enabled
}

// IsEnabledFor checks if a feature flag is enabled for a specific tenant/user.
func (ff *FeatureFlags) IsEnabledFor(name, tenantID, userID string) bool {
	ff.mu.RLock()
	defer ff.mu.RUnlock()

	flag, exists := ff.flags[name]
	if !exists {
		return false
	}

	return flag.IsEnabledFor(tenantID, userID)
}

// Enable enables a feature flag.
func (ff *FeatureFlags) Enable(name string) error {
	ff.mu.Lock()
	defer ff.mu.Unlock()

	flag, exists := ff.flags[name]
	if !exists {
		return fmt.Errorf("feature flag %q not found", name)
	}

	flag.Enabled = true
	flag.UpdatedAt = time.Now()
	return nil
}

// Disable disables a feature flag.
func (ff *FeatureFlags) Disable(name string) error {
	ff.mu.Lock()
	defer ff.mu.Unlock()

	flag, exists := ff.flags[name]
	if !exists {
		return fmt.Errorf("feature flag %q not found", name)
	}

	flag.Enabled = false
	flag.UpdatedAt = time.Now()
	return nil
}

// Toggle toggles a feature flag's state.
func (ff *FeatureFlags) Toggle(name string) (bool, error) {
	ff.mu.Lock()
	defer ff.mu.Unlock()

	flag, exists := ff.flags[name]
	if !exists {
		return false, fmt.Errorf("feature flag %q not found", name)
	}

	flag.Enabled = !flag.Enabled
	flag.UpdatedAt = time.Now()
	return flag.Enabled, nil
}

// SetRolloutPercentage sets the rollout percentage for a feature flag.
func (ff *FeatureFlags) SetRolloutPercentage(name string, percentage int) error {
	if percentage < 0 || percentage > 100 {
		return fmt.Errorf("rollout percentage must be between 0 and 100")
	}

	ff.mu.Lock()
	defer ff.mu.Unlock()

	flag, exists := ff.flags[name]
	if !exists {
		return fmt.Errorf("feature flag %q not found", name)
	}

	flag.RolloutPercentage = percentage
	flag.UpdatedAt = time.Now()
	return nil
}

// SetTargetTenants sets the target tenants for a feature flag.
func (ff *FeatureFlags) SetTargetTenants(name string, tenants []string) error {
	ff.mu.Lock()
	defer ff.mu.Unlock()

	flag, exists := ff.flags[name]
	if !exists {
		return fmt.Errorf("feature flag %q not found", name)
	}

	flag.TargetTenants = tenants
	flag.UpdatedAt = time.Now()
	return nil
}

// SetTargetUsers sets the target users for a feature flag.
func (ff *FeatureFlags) SetTargetUsers(name string, users []string) error {
	ff.mu.Lock()
	defer ff.mu.Unlock()

	flag, exists := ff.flags[name]
	if !exists {
		return fmt.Errorf("feature flag %q not found", name)
	}

	flag.TargetUsers = users
	flag.UpdatedAt = time.Now()
	return nil
}

// Update updates a feature flag's configuration.
func (ff *FeatureFlags) Update(name string, updates FeatureFlag) error {
	ff.mu.Lock()
	defer ff.mu.Unlock()

	flag, exists := ff.flags[name]
	if !exists {
		return fmt.Errorf("feature flag %q not found", name)
	}

	if updates.Description != "" {
		flag.Description = updates.Description
	}
	if updates.Metadata != nil {
		flag.Metadata = updates.Metadata
	}
	flag.UpdatedAt = time.Now()
	return nil
}

// Delete removes a feature flag.
func (ff *FeatureFlags) Delete(name string) error {
	ff.mu.Lock()
	defer ff.mu.Unlock()

	if _, exists := ff.flags[name]; !exists {
		return fmt.Errorf("feature flag %q not found", name)
	}

	delete(ff.flags, name)
	return nil
}

// List returns all registered feature flags.
func (ff *FeatureFlags) List() []*FeatureFlag {
	ff.mu.RLock()
	defer ff.mu.RUnlock()

	flags := make([]*FeatureFlag, 0, len(ff.flags))
	for _, flag := range ff.flags {
		flags = append(flags, flag)
	}
	return flags
}

// ListEnabled returns all enabled feature flags.
func (ff *FeatureFlags) ListEnabled() []*FeatureFlag {
	ff.mu.RLock()
	defer ff.mu.RUnlock()

	flags := make([]*FeatureFlag, 0)
	for _, flag := range ff.flags {
		if flag.Enabled {
			flags = append(flags, flag)
		}
	}
	return flags
}

// WithFlag adds a feature flag override to the context.
func (ff *FeatureFlags) WithFlag(ctx context.Context, name string, enabled bool) context.Context {
	overrides := ff.getContextOverrides(ctx)
	overrides[name] = enabled
	return context.WithValue(ctx, ff.contextKey, overrides)
}

// WithFlags adds multiple feature flag overrides to the context.
func (ff *FeatureFlags) WithFlags(ctx context.Context, flags map[string]bool) context.Context {
	overrides := ff.getContextOverrides(ctx)
	for name, enabled := range flags {
		overrides[name] = enabled
	}
	return context.WithValue(ctx, ff.contextKey, overrides)
}

// IsEnabledInContext checks if a feature flag is enabled, considering context overrides.
func (ff *FeatureFlags) IsEnabledInContext(ctx context.Context, name string) bool {
	// Check context override first
	overrides := ff.getContextOverrides(ctx)
	if enabled, exists := overrides[name]; exists {
		return enabled
	}

	// Fall back to global setting
	return ff.IsEnabled(name)
}

// IsEnabledInContextFor checks if a feature flag is enabled for a specific tenant/user,
// considering context overrides.
func (ff *FeatureFlags) IsEnabledInContextFor(ctx context.Context, name, tenantID, userID string) bool {
	// Check context override first
	overrides := ff.getContextOverrides(ctx)
	if enabled, exists := overrides[name]; exists {
		return enabled
	}

	// Fall back to targeted check
	return ff.IsEnabledFor(name, tenantID, userID)
}

// getContextOverrides retrieves feature flag overrides from context.
func (ff *FeatureFlags) getContextOverrides(ctx context.Context) map[string]bool {
	if ctx == nil {
		return make(map[string]bool)
	}

	if overrides, ok := ctx.Value(ff.contextKey).(map[string]bool); ok {
		return overrides
	}

	return make(map[string]bool)
}

// Export exports all feature flags to JSON.
func (ff *FeatureFlags) Export() ([]byte, error) {
	ff.mu.RLock()
	defer ff.mu.RUnlock()

	return json.MarshalIndent(ff.flags, "", "  ")
}

// Import imports feature flags from JSON.
func (ff *FeatureFlags) Import(data []byte) error {
	var flags map[string]*FeatureFlag
	if err := json.Unmarshal(data, &flags); err != nil {
		return fmt.Errorf("failed to unmarshal feature flags: %w", err)
	}

	ff.mu.Lock()
	defer ff.mu.Unlock()

	for name, flag := range flags {
		flag.Name = name // Ensure name consistency
		ff.flags[name] = flag
	}

	return nil
}

// LoadFromEnvironment loads feature flags from environment variables.
func (ff *FeatureFlags) LoadFromEnvironment() error {
	for _, env := range os.Environ() {
		parts := strings.SplitN(env, "=", 2)
		if len(parts) != 2 {
			continue
		}

		key, value := parts[0], parts[1]
		if !strings.HasPrefix(key, ff.environmentPrefix) {
			continue
		}

		flagName := strings.ToLower(strings.TrimPrefix(key, ff.environmentPrefix))
		enabled, err := strconv.ParseBool(value)
		if err != nil {
			continue // Skip invalid values
		}

		// Create flag if it doesn't exist
		if _, exists := ff.flags[flagName]; !exists {
			ff.flags[flagName] = &FeatureFlag{
				Name:      flagName,
				Default:   enabled,
				Enabled:   enabled,
				CreatedAt: time.Now(),
				UpdatedAt: time.Now(),
			}
		} else {
			ff.flags[flagName].Enabled = enabled
			ff.flags[flagName].UpdatedAt = time.Now()
		}
	}

	return nil
}

// hashString creates a simple hash from a string for consistent rollout.
func hashString(s string) uint32 {
	h := uint32(2166136261)
	for _, c := range s {
		h ^= uint32(c)
		h *= 16777619
	}
	return h
}

// Predefined feature flags for SMA-OS.
const (
	// FeatureNewScheduler enables the new DAG scheduler implementation
	FeatureNewScheduler = "new-scheduler"

	// FeatureEnhancedMetrics enables enhanced metrics collection
	FeatureEnhancedMetrics = "enhanced-metrics"

	// FeatureAsyncEvaluator enables async output evaluation
	FeatureAsyncEvaluator = "async-evaluator"

	// FeatureVectorCompression enables vector compression in memory bus
	FeatureVectorCompression = "vector-compression"

	// FeatureHotWarmStorage enables hot/warm storage tiering
	FeatureHotWarmStorage = "hot-warm-storage"

	// FeatureCircuitBreaker enables circuit breaker pattern
	FeatureCircuitBreaker = "circuit-breaker"

	// FeatureRateLimiting enables rate limiting
	FeatureRateLimiting = "rate-limiting"

	// FeatureAuditLogging enables audit logging
	FeatureAuditLogging = "audit-logging"

	// FeatureMultiRegion enables multi-region deployment
	FeatureMultiRegion = "multi-region"

	// FeatureAutoScaling enables auto-scaling
	FeatureAutoScaling = "auto-scaling"
)

// DefaultFeatureFlags returns the default set of feature flags for SMA-OS.
func DefaultFeatureFlags() []FeatureFlag {
	return []FeatureFlag{
		{
			Name:        FeatureNewScheduler,
			Description: "Enable the new DAG scheduler implementation with improved performance",
			Default:     false,
			Enabled:     false,
		},
		{
			Name:        FeatureEnhancedMetrics,
			Description: "Enable enhanced metrics collection with detailed performance data",
			Default:     true,
			Enabled:     true,
		},
		{
			Name:        FeatureAsyncEvaluator,
			Description: "Enable async output evaluation for better throughput",
			Default:     false,
			Enabled:     false,
		},
		{
			Name:        FeatureVectorCompression,
			Description: "Enable vector compression in memory bus to reduce storage",
			Default:     true,
			Enabled:     true,
		},
		{
			Name:        FeatureHotWarmStorage,
			Description: "Enable hot/warm storage tiering for cost optimization",
			Default:     false,
			Enabled:     false,
		},
		{
			Name:        FeatureCircuitBreaker,
			Description: "Enable circuit breaker pattern for fault tolerance",
			Default:     true,
			Enabled:     true,
		},
		{
			Name:        FeatureRateLimiting,
			Description: "Enable rate limiting for API protection",
			Default:     true,
			Enabled:     true,
		},
		{
			Name:        FeatureAuditLogging,
			Description: "Enable comprehensive audit logging",
			Default:     true,
			Enabled:     true,
		},
		{
			Name:        FeatureMultiRegion,
			Description: "Enable multi-region deployment capabilities",
			Default:     false,
			Enabled:     false,
		},
		{
			Name:        FeatureAutoScaling,
			Description: "Enable auto-scaling based on load",
			Default:     false,
			Enabled:     false,
		},
	}
}

// InitializeDefaultFlags initializes the feature flags with default values.
func (ff *FeatureFlags) InitializeDefaultFlags() error {
	return ff.RegisterMultiple(DefaultFeatureFlags())
}
