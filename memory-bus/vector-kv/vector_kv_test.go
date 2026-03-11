package main

import (
	"testing"
)

// TestHybridDBManagerCreation tests creating the hybrid DB manager
func TestHybridDBManagerCreation(t *testing.T) {
	manager := NewHybridDBManager()

	if manager == nil {
		t.Fatal("HybridDBManagerProxy should not be nil")
	}
}

// TestReadWithCache tests the cache read functionality
func TestReadWithCache(t *testing.T) {
	manager := NewHybridDBManager()

	result := manager.ReadWithCache("tenant-alpha", "v1.2")

	if result == "" {
		t.Error("Expected non-empty result from ReadWithCache")
	}

	// Check if result contains expected JSON structure
	expectedSubstrings := []string{"cached_payload", "true", "latency"}
	for _, substr := range expectedSubstrings {
		if !contains(result, substr) {
			t.Errorf("Expected result to contain '%s', got '%s'", substr, result)
		}
	}
}

// TestReadWithCache_DifferentTenants tests reading with different tenant IDs
func TestReadWithCache_DifferentTenants(t *testing.T) {
	manager := NewHybridDBManager()

	tenants := []string{"tenant-1", "tenant-2", "tenant-3"}
	versions := []string{"v1.0", "v1.1", "v2.0"}

	for i, tenant := range tenants {
		version := versions[i]
		result := manager.ReadWithCache(tenant, version)

		if result == "" {
			t.Errorf("Expected non-empty result for tenant %s, version %s", tenant, version)
		}
	}
}

// TestReadWithCache_EmptyStrings tests reading with empty strings
func TestReadWithCache_EmptyStrings(t *testing.T) {
	manager := NewHybridDBManager()

	// Test with empty tenant ID
	result := manager.ReadWithCache("", "v1.0")
	if result == "" {
		t.Error("Expected result even with empty tenant ID")
	}

	// Test with empty version
	result = manager.ReadWithCache("tenant-alpha", "")
	if result == "" {
		t.Error("Expected result even with empty version")
	}

	// Test with both empty
	result = manager.ReadWithCache("", "")
	if result == "" {
		t.Error("Expected result even with both empty")
	}
}

// TestCompactContexts tests the context compression functionality
func TestCompactContexts(t *testing.T) {
	manager := NewHybridDBManager()

	// This should not panic
	manager.CompactContexts()

	// The function is expected to log but not return anything
	// In a real scenario, you might check if compression actually occurred
}

// TestHybridDBManager_MultipleReads tests multiple sequential reads
func TestHybridDBManager_MultipleReads(t *testing.T) {
	manager := NewHybridDBManager()

	// Perform multiple reads
	for i := 0; i < 10; i++ {
		result := manager.ReadWithCache("test-tenant", "v1.0")
		if result == "" {
			t.Errorf("Expected non-empty result for read %d", i)
		}
	}
}

// TestHybridDBManager_ConcurrentReads tests concurrent read access
func TestHybridDBManager_ConcurrentReads(t *testing.T) {
	manager := NewHybridDBManager()

	done := make(chan bool, 10)

	// Perform concurrent reads
	for i := 0; i < 10; i++ {
		go func(id int) {
			_ = manager.ReadWithCache("concurrent-tenant", "v1.0")
			done <- true
		}(i)
	}

	// Wait for all goroutines
	for i := 0; i < 10; i++ {
		<-done
	}
}

// Helper function to check substring
func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > len(substr) &&
		(s[0:len(substr)] == substr || s[len(s)-len(substr):] == substr ||
			findSubstring(s, substr)))
}

func findSubstring(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
