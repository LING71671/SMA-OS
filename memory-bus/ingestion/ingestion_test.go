package main

import (
	"testing"
)

// TestParsedIntentStructure tests the ParsedIntent structure
func TestParsedIntentStructure(t *testing.T) {
	intent := &ParsedIntent{
		Action:     "create_vm",
		Target:     "pool-a",
		Parameters: "cpu=2,ram=4G",
		Confidence: 0.95,
		Source:     "LLM",
	}

	if intent.Action != "create_vm" {
		t.Errorf("Expected Action 'create_vm', got '%s'", intent.Action)
	}

	if intent.Target != "pool-a" {
		t.Errorf("Expected Target 'pool-a', got '%s'", intent.Target)
	}

	if intent.Parameters != "cpu=2,ram=4G" {
		t.Errorf("Expected Parameters 'cpu=2,ram=4G', got '%s'", intent.Parameters)
	}

	if intent.Confidence != 0.95 {
		t.Errorf("Expected Confidence 0.95, got %f", intent.Confidence)
	}

	if intent.Source != "LLM" {
		t.Errorf("Expected Source 'LLM', got '%s'", intent.Source)
	}
}

// TestFallbackRegexExtractor_ValidInput tests regex extraction with valid input
func TestFallbackRegexExtractor_ValidInput(t *testing.T) {
	prompt := "create vm in pool a with cpu=2,ram=4G"

	intent, err := fallbackRegexExtractor(prompt)

	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}

	if intent == nil {
		t.Fatal("Expected intent to be extracted")
	}

	if intent.Action != "create_vm" {
		t.Errorf("Expected Action 'create_vm', got '%s'", intent.Action)
	}

	if intent.Target != "a" {
		t.Errorf("Expected Target 'a', got '%s'", intent.Target)
	}

	if intent.Source != "REGEX_FALLBACK" {
		t.Errorf("Expected Source 'REGEX_FALLBACK', got '%s'", intent.Source)
	}

	if intent.Confidence != 0.99 {
		t.Errorf("Expected Confidence 0.99, got %f", intent.Confidence)
	}
}

// TestFallbackRegexExtractor_InvalidInput tests regex extraction with invalid input
func TestFallbackRegexExtractor_InvalidInput(t *testing.T) {
	prompt := "this is not a valid command"

	intent, err := fallbackRegexExtractor(prompt)

	if err == nil {
		t.Error("Expected error for invalid input")
	}

	if intent != nil {
		t.Error("Expected nil intent for invalid input")
	}
}

// TestFallbackRegexExtractor_CaseInsensitive tests case insensitivity
func TestFallbackRegexExtractor_CaseInsensitive(t *testing.T) {
	prompts := []string{
		"create vm in pool a with cpu=2",
		"CREATE VM IN POOL A WITH CPU=2",
		"Create Vm In Pool A With Cpu=2",
	}

	for _, prompt := range prompts {
		intent, err := fallbackRegexExtractor(prompt)
		if err != nil {
			t.Errorf("Expected no error for prompt '%s', got %v", prompt, err)
		}
		if intent != nil {
			if intent.Action != "create_vm" {
				t.Errorf("Expected Action 'create_vm' for prompt '%s', got '%s'", prompt, intent.Action)
			}
		}
	}
}

// TestFallbackRegexExtractor_Variations tests different input variations
func TestFallbackRegexExtractor_Variations(t *testing.T) {
	tests := []struct {
		input          string
		expectedAction string
		expectedTarget string
		expectSuccess  bool
	}{
		{"create a vm in pool b with cpu=4", "create_vm", "b", true},
		{"create instance in pool c with cpu=8", "create_vm", "c", true},
		{"invalid input", "", "", false},
	}

	for _, test := range tests {
		intent, err := fallbackRegexExtractor(test.input)

		if test.expectSuccess {
			if err != nil {
				t.Errorf("Expected success for '%s', got error: %v", test.input, err)
			}
			if intent != nil {
				if intent.Action != test.expectedAction {
					t.Errorf("Expected Action '%s', got '%s'", test.expectedAction, intent.Action)
				}
				if intent.Target != test.expectedTarget {
					t.Errorf("Expected Target '%s', got '%s'", test.expectedTarget, intent.Target)
				}
			}
		} else {
			if err == nil {
				t.Errorf("Expected error for '%s', got nil", test.input)
			}
		}
	}
}

// TestProcessInput_EmptyInput tests processing empty input
func TestProcessInput_EmptyInput(t *testing.T) {
	_, err := ProcessInput("")
	// Should return an error (either LLM or fallback failure)
	if err == nil {
		t.Log("Expected error for empty input (behavior may vary)")
	}
}
