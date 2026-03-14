package config

import (
	"context"
	"testing"
	"time"
)

func TestNewFeatureFlags(t *testing.T) {
	ff := NewFeatureFlags()
	if ff == nil {
		t.Fatal("NewFeatureFlags() returned nil")
	}
	if ff.flags == nil {
		t.Fatal("flags map not initialized")
	}
}

func TestFeatureFlagRegister(t *testing.T) {
	ff := NewFeatureFlags()

	flag := FeatureFlag{
		Name:        "test-flag",
		Description: "Test feature flag",
		Default:     false,
		Enabled:     false,
	}

	err := ff.Register(flag)
	if err != nil {
		t.Fatalf("Register() failed: %v", err)
	}

	// Test duplicate registration
	err = ff.Register(flag)
	if err == nil {
		t.Error("Register() should fail for duplicate flag")
	}

	// Test empty name
	err = ff.Register(FeatureFlag{Name: ""})
	if err == nil {
		t.Error("Register() should fail for empty name")
	}
}

func TestFeatureFlagGet(t *testing.T) {
	ff := NewFeatureFlags()

	flag := FeatureFlag{
		Name:        "test-flag",
		Description: "Test feature flag",
		Default:     true,
		Enabled:     true,
	}

	ff.Register(flag)

	retrieved, err := ff.Get("test-flag")
	if err != nil {
		t.Fatalf("Get() failed: %v", err)
	}
	if retrieved.Name != "test-flag" {
		t.Errorf("Get() returned wrong flag: got %s, want test-flag", retrieved.Name)
	}

	// Test non-existent flag
	_, err = ff.Get("non-existent")
	if err == nil {
		t.Error("Get() should fail for non-existent flag")
	}
}

func TestFeatureFlagIsEnabled(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{
		Name:    "enabled-flag",
		Enabled: true,
	})

	ff.Register(FeatureFlag{
		Name:    "disabled-flag",
		Enabled: false,
	})

	if !ff.IsEnabled("enabled-flag") {
		t.Error("IsEnabled() should return true for enabled flag")
	}

	if ff.IsEnabled("disabled-flag") {
		t.Error("IsEnabled() should return false for disabled flag")
	}

	if ff.IsEnabled("non-existent") {
		t.Error("IsEnabled() should return false for non-existent flag")
	}
}

func TestFeatureFlagToggle(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{
		Name:    "toggle-flag",
		Enabled: false,
	})

	enabled, err := ff.Toggle("toggle-flag")
	if err != nil {
		t.Fatalf("Toggle() failed: %v", err)
	}
	if !enabled {
		t.Error("Toggle() should return true after toggling from false")
	}

	// Verify it was toggled
	if !ff.IsEnabled("toggle-flag") {
		t.Error("Flag should be enabled after toggle")
	}

	// Toggle again
	enabled, _ = ff.Toggle("toggle-flag")
	if enabled {
		t.Error("Toggle() should return false after second toggle")
	}

	// Test non-existent flag
	_, err = ff.Toggle("non-existent")
	if err == nil {
		t.Error("Toggle() should fail for non-existent flag")
	}
}

func TestFeatureFlagEnableDisable(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{
		Name:    "control-flag",
		Enabled: false,
	})

	err := ff.Enable("control-flag")
	if err != nil {
		t.Fatalf("Enable() failed: %v", err)
	}
	if !ff.IsEnabled("control-flag") {
		t.Error("Flag should be enabled after Enable()")
	}

	err = ff.Disable("control-flag")
	if err != nil {
		t.Fatalf("Disable() failed: %v", err)
	}
	if ff.IsEnabled("control-flag") {
		t.Error("Flag should be disabled after Disable()")
	}

	// Test non-existent flag
	err = ff.Enable("non-existent")
	if err == nil {
		t.Error("Enable() should fail for non-existent flag")
	}
}

func TestFeatureFlagRollout(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{
		Name:              "rollout-flag",
		Enabled:           true,
		RolloutPercentage: 50,
	})

	// Test invalid percentage
	err := ff.SetRolloutPercentage("rollout-flag", 150)
	if err == nil {
		t.Error("SetRolloutPercentage() should fail for percentage > 100")
	}

	err = ff.SetRolloutPercentage("rollout-flag", -10)
	if err == nil {
		t.Error("SetRolloutPercentage() should fail for negative percentage")
	}

	// Test valid percentage
	err = ff.SetRolloutPercentage("rollout-flag", 75)
	if err != nil {
		t.Fatalf("SetRolloutPercentage() failed: %v", err)
	}

	flag, _ := ff.Get("rollout-flag")
	if flag.RolloutPercentage != 75 {
		t.Errorf("Rollout percentage not set correctly: got %d, want 75", flag.RolloutPercentage)
	}
}

func TestFeatureFlagTargeting(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{
		Name:              "targeted-flag",
		Enabled:           true,
		TargetTenants:     []string{"tenant-1", "tenant-2"},
		TargetUsers:       []string{"user-1"},
		RolloutPercentage: 100,
	})

	// Test tenant targeting
	if !ff.IsEnabledFor("targeted-flag", "tenant-1", "user-1") {
		t.Error("Flag should be enabled for targeted tenant")
	}

	if ff.IsEnabledFor("targeted-flag", "tenant-3", "user-1") {
		t.Error("Flag should be disabled for non-targeted tenant")
	}

	// Test user targeting
	if ff.IsEnabledFor("targeted-flag", "tenant-1", "user-2") {
		t.Error("Flag should be disabled for non-targeted user")
	}

	// Test SetTargetTenants
	err := ff.SetTargetTenants("targeted-flag", []string{"tenant-3"})
	if err != nil {
		t.Fatalf("SetTargetTenants() failed: %v", err)
	}

	if !ff.IsEnabledFor("targeted-flag", "tenant-3", "user-1") {
		t.Error("Flag should be enabled for new targeted tenant")
	}

	// Test SetTargetUsers
	err = ff.SetTargetUsers("targeted-flag", []string{"user-2"})
	if err != nil {
		t.Fatalf("SetTargetUsers() failed: %v", err)
	}

	if !ff.IsEnabledFor("targeted-flag", "tenant-3", "user-2") {
		t.Error("Flag should be enabled for new targeted user")
	}
}

func TestFeatureFlagContextPropagation(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{
		Name:    "context-flag",
		Enabled: false,
	})

	// Test context override
	ctx := ff.WithFlag(context.Background(), "context-flag", true)
	if !ff.IsEnabledInContext(ctx, "context-flag") {
		t.Error("IsEnabledInContext() should return true with context override")
	}

	// Test without context override
	if ff.IsEnabled("context-flag") {
		t.Error("IsEnabled() should return false without context override")
	}

	// Test WithFlags
	ctx = ff.WithFlags(context.Background(), map[string]bool{
		"context-flag": true,
	})
	if !ff.IsEnabledInContext(ctx, "context-flag") {
		t.Error("IsEnabledInContext() should work with WithFlags")
	}

	// Test IsEnabledInContextFor
	ctx = ff.WithFlag(context.Background(), "context-flag", true)
	if !ff.IsEnabledInContextFor(ctx, "context-flag", "tenant-1", "user-1") {
		t.Error("IsEnabledInContextFor() should respect context override")
	}
}

func TestFeatureFlagList(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{Name: "flag-1", Enabled: true})
	ff.Register(FeatureFlag{Name: "flag-2", Enabled: false})
	ff.Register(FeatureFlag{Name: "flag-3", Enabled: true})

	allFlags := ff.List()
	if len(allFlags) != 3 {
		t.Errorf("List() returned %d flags, want 3", len(allFlags))
	}

	enabledFlags := ff.ListEnabled()
	if len(enabledFlags) != 2 {
		t.Errorf("ListEnabled() returned %d flags, want 2", len(enabledFlags))
	}
}

func TestFeatureFlagDelete(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{Name: "delete-flag"})

	err := ff.Delete("delete-flag")
	if err != nil {
		t.Fatalf("Delete() failed: %v", err)
	}

	if ff.IsEnabled("delete-flag") {
		t.Error("Flag should not exist after deletion")
	}

	// Test deleting non-existent flag
	err = ff.Delete("non-existent")
	if err == nil {
		t.Error("Delete() should fail for non-existent flag")
	}
}

func TestFeatureFlagExportImport(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{
		Name:        "export-flag",
		Description: "Test export",
		Enabled:     true,
	})

	data, err := ff.Export()
	if err != nil {
		t.Fatalf("Export() failed: %v", err)
	}

	// Import into new manager
	ff2 := NewFeatureFlags()
	err = ff2.Import(data)
	if err != nil {
		t.Fatalf("Import() failed: %v", err)
	}

	if !ff2.IsEnabled("export-flag") {
		t.Error("Flag should be enabled after import")
	}

	flag, _ := ff2.Get("export-flag")
	if flag.Description != "Test export" {
		t.Error("Description not preserved after import")
	}
}

func TestFeatureFlagUpdate(t *testing.T) {
	ff := NewFeatureFlags()

	ff.Register(FeatureFlag{
		Name:        "update-flag",
		Description: "Original description",
	})

	err := ff.Update("update-flag", FeatureFlag{
		Description: "Updated description",
		Metadata: map[string]string{
			"key": "value",
		},
	})
	if err != nil {
		t.Fatalf("Update() failed: %v", err)
	}

	flag, _ := ff.Get("update-flag")
	if flag.Description != "Updated description" {
		t.Errorf("Description not updated: got %s, want Updated description", flag.Description)
	}
	if flag.Metadata["key"] != "value" {
		t.Error("Metadata not updated")
	}

	// Test updating non-existent flag
	err = ff.Update("non-existent", FeatureFlag{})
	if err == nil {
		t.Error("Update() should fail for non-existent flag")
	}
}

func TestFeatureFlagIsEnabledFor(t *testing.T) {
	ff := NewFeatureFlags()

	// Test with rollout percentage
	ff.Register(FeatureFlag{
		Name:              "rollout-test",
		Enabled:           true,
		RolloutPercentage: 100,
	})

	if !ff.IsEnabledFor("rollout-test", "tenant-1", "user-1") {
		t.Error("Flag should be enabled with 100% rollout")
	}

	// Test disabled flag
	ff.Register(FeatureFlag{
		Name:    "disabled-test",
		Enabled: false,
	})

	if ff.IsEnabledFor("disabled-test", "tenant-1", "user-1") {
		t.Error("Flag should be disabled when Enabled is false")
	}

	// Test non-existent flag
	if ff.IsEnabledFor("non-existent", "tenant-1", "user-1") {
		t.Error("Non-existent flag should return false")
	}
}

func TestDefaultFeatureFlags(t *testing.T) {
	flags := DefaultFeatureFlags()
	if len(flags) == 0 {
		t.Error("DefaultFeatureFlags() returned empty slice")
	}

	// Check that all predefined constants exist
	flagMap := make(map[string]bool)
	for _, f := range flags {
		flagMap[f.Name] = true
	}

	expectedFlags := []string{
		FeatureNewScheduler,
		FeatureEnhancedMetrics,
		FeatureAsyncEvaluator,
		FeatureVectorCompression,
		FeatureHotWarmStorage,
		FeatureCircuitBreaker,
		FeatureRateLimiting,
		FeatureAuditLogging,
		FeatureMultiRegion,
		FeatureAutoScaling,
	}

	for _, name := range expectedFlags {
		if !flagMap[name] {
			t.Errorf("Expected flag %s not found in default flags", name)
		}
	}
}

func TestInitializeDefaultFlags(t *testing.T) {
	ff := NewFeatureFlags()

	err := ff.InitializeDefaultFlags()
	if err != nil {
		t.Fatalf("InitializeDefaultFlags() failed: %v", err)
	}

	// Verify some flags were registered
	if !ff.IsEnabled(FeatureEnhancedMetrics) {
		t.Error("FeatureEnhancedMetrics should be enabled by default")
	}

	if ff.IsEnabled(FeatureNewScheduler) {
		t.Error("FeatureNewScheduler should be disabled by default")
	}
}

func TestFeatureFlagTimestamps(t *testing.T) {
	ff := NewFeatureFlags()

	before := time.Now()
	ff.Register(FeatureFlag{Name: "timestamp-flag"})
	after := time.Now()

	flag, _ := ff.Get("timestamp-flag")
	if flag.CreatedAt.Before(before) || flag.CreatedAt.After(after) {
		t.Error("CreatedAt timestamp not set correctly")
	}
	if flag.UpdatedAt.Before(before) || flag.UpdatedAt.After(after) {
		t.Error("UpdatedAt timestamp not set correctly")
	}

	// Update and check UpdatedAt changes
	time.Sleep(10 * time.Millisecond)
	beforeUpdate := time.Now()
	ff.Enable("timestamp-flag")
	afterUpdate := time.Now()

	flag, _ = ff.Get("timestamp-flag")
	if flag.UpdatedAt.Before(beforeUpdate) || flag.UpdatedAt.After(afterUpdate) {
		t.Error("UpdatedAt not updated after Enable()")
	}
}

func TestFeatureFlagRegisterMultiple(t *testing.T) {
	ff := NewFeatureFlags()

	flags := []FeatureFlag{
		{Name: "multi-1"},
		{Name: "multi-2"},
		{Name: "multi-3"},
	}

	err := ff.RegisterMultiple(flags)
	if err != nil {
		t.Fatalf("RegisterMultiple() failed: %v", err)
	}

	if len(ff.List()) != 3 {
		t.Errorf("Expected 3 flags, got %d", len(ff.List()))
	}

	// Test partial failure (duplicate)
	flagsWithDuplicate := []FeatureFlag{
		{Name: "multi-4"},
		{Name: "multi-1"}, // Duplicate
	}

	err = ff.RegisterMultiple(flagsWithDuplicate)
	if err == nil {
		t.Error("RegisterMultiple() should fail on duplicate")
	}
}

func BenchmarkIsEnabled(b *testing.B) {
	ff := NewFeatureFlags()
	ff.Register(FeatureFlag{
		Name:    "bench-flag",
		Enabled: true,
	})

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		ff.IsEnabled("bench-flag")
	}
}

func BenchmarkIsEnabledFor(b *testing.B) {
	ff := NewFeatureFlags()
	ff.Register(FeatureFlag{
		Name:              "bench-targeted",
		Enabled:           true,
		RolloutPercentage: 50,
	})

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		ff.IsEnabledFor("bench-targeted", "tenant-1", "user-1")
	}
}

func BenchmarkContextPropagation(b *testing.B) {
	ff := NewFeatureFlags()
	ff.Register(FeatureFlag{
		Name:    "bench-context",
		Enabled: false,
	})

	ctx := ff.WithFlag(context.Background(), "bench-context", true)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		ff.IsEnabledInContext(ctx, "bench-context")
	}
}
