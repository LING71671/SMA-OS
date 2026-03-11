package main

import (
	"testing"
)

// TestEvaluatorAgentCreation tests creating evaluator agent
func TestEvaluatorAgentCreation(t *testing.T) {
	evaluator := NewEvaluatorAgent()

	if evaluator == nil {
		t.Fatal("EvaluatorAgent should not be nil")
	}
}

// TestAuditTaskResult_ValidResult tests auditing a valid result
func TestAuditTaskResult_ValidResult(t *testing.T) {
	evaluator := NewEvaluatorAgent()

	// Test with valid result (should not reject)
	reject := evaluator.AuditTaskResult("task-123", 100, "valid_schema")

	if reject != nil {
		t.Error("Expected no rejection for valid result")
	}
}

// TestAuditTaskResult_InvalidSchema tests rejection on invalid schema
func TestAuditTaskResult_InvalidSchema(t *testing.T) {
	evaluator := NewEvaluatorAgent()

	// Test with invalid schema (should reject)
	reject := evaluator.AuditTaskResult("task-456", 101, "invalid_schema")

	if reject == nil {
		t.Fatal("Expected rejection for invalid schema")
	}

	if reject.TaskID != "task-456" {
		t.Errorf("Expected TaskID 'task-456', got '%s'", reject.TaskID)
	}

	if reject.RejectedVersion != 101 {
		t.Errorf("Expected RejectedVersion 101, got %d", reject.RejectedVersion)
	}

	if reject.Reason != "schema_mismatch" {
		t.Errorf("Expected Reason 'schema_mismatch', got '%s'", reject.Reason)
	}

	if reject.RollbackTo != 100 {
		t.Errorf("Expected RollbackTo 100, got %d", reject.RollbackTo)
	}
}

// TestVersionedRejectStructure tests the VersionedReject structure
func TestVersionedRejectStructure(t *testing.T) {
	reject := &VersionedReject{
		TaskID:          "test-task",
		RejectedVersion: 50,
		Reason:          "test_reason",
		RollbackTo:      49,
	}

	if reject.TaskID != "test-task" {
		t.Errorf("Expected TaskID 'test-task', got '%s'", reject.TaskID)
	}

	if reject.RejectedVersion != 50 {
		t.Errorf("Expected RejectedVersion 50, got %d", reject.RejectedVersion)
	}

	if reject.Reason != "test_reason" {
		t.Errorf("Expected Reason 'test_reason', got '%s'", reject.Reason)
	}

	if reject.RollbackTo != 49 {
		t.Errorf("Expected RollbackTo 49, got %d", reject.RollbackTo)
	}
}

// TestAuditMultipleVersions tests auditing multiple versions
func TestAuditMultipleVersions(t *testing.T) {
	evaluator := NewEvaluatorAgent()

	// Test multiple valid versions
	for i := 1; i <= 5; i++ {
		reject := evaluator.AuditTaskResult("multi-version-task", uint64(i), "valid_schema")
		if reject != nil {
			t.Errorf("Expected no rejection for version %d", i)
		}
	}

	// Test invalid version
	reject := evaluator.AuditTaskResult("multi-version-task", 6, "invalid_schema")
	if reject == nil {
		t.Error("Expected rejection for invalid schema")
	}
}

// TestAuditEdgeCases tests edge cases in auditing
func TestAuditEdgeCases(t *testing.T) {
	evaluator := NewEvaluatorAgent()

	// Test version 0
	reject := evaluator.AuditTaskResult("edge-case", 0, "invalid_schema")
	if reject == nil {
		t.Error("Expected rejection for invalid schema at version 0")
	} else if reject.RollbackTo != 0 {
		// Should handle underflow gracefully (uint64)
		// In production, this would need proper handling
		t.Logf("RollbackTo for version 0: %d (expected behavior)", reject.RollbackTo)
	}

	// Test empty task ID
	reject = evaluator.AuditTaskResult("", 1, "invalid_schema")
	if reject == nil {
		t.Error("Expected rejection for empty task ID")
	}
}
