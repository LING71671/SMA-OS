package main

import (
	"encoding/json"
	"errors"
	"log"
	"regexp"
	"strings"
	"time"
)

// Intent schema that downstream Evaluator expects
type ParsedIntent struct {
	Action     string  `json:"action"`
	Target     string  `json:"target"`
	Parameters string  `json:"parameters"`
	Confidence float64 `json:"confidence"`
	Source     string  `json:"source"`
}

// Simulate an LLM API call (e.g. to a local Ollama Llama-3 instance)
func invokeLLM(prompt string) (string, error) {
	log.Printf("[LLM Invocation] Querying LLM API with prompt length %d...", len(prompt))
	time.Sleep(300 * time.Millisecond) // Simulated inference latency

	// We'll intentionally simulate a failure or a hallucinated / malformed JSON string for demonstration
	if strings.Contains(prompt, "complex command") {
		return "", errors.New("timeout or rate limited from LLM Endpoint")
	}

	return `{"action": "create_vm", "target": "pool-A", "parameters": "cpu=2,ram=4G"}`, nil
}

// Fallback logic using deterministic Regex
func fallbackRegexExtractor(prompt string) (*ParsedIntent, error) {
	log.Println("[Fallback Engine] LLM failed! Activating high-confidence Regex extraction...")

	// Example Regex matching "create vm <pool> <params>"
	re := regexp.MustCompile(`(?i)create\s+(?:a\s+)?(?:vm|instance)\s+in\s+pool\s+(\w+)\s+with\s+(.+)`)
	matches := re.FindStringSubmatch(prompt)

	if len(matches) == 3 {
		return &ParsedIntent{
			Action:     "create_vm",
			Target:     matches[1],
			Parameters: strings.ReplaceAll(matches[2], " ", ""),
			Confidence: 0.99,
			Source:     "REGEX_FALLBACK",
		}, nil
	}

	return nil, errors.New("no matching pre-defined regex rules found")
}

func ProcessInput(userInput string) (*ParsedIntent, error) {
	log.Printf("\n--- Processing User Input: %s ---", userInput)

	// 1. Attempt LLM JSON Extraction
	llmResponse, err := invokeLLM(userInput)
	if err == nil {
		var intent ParsedIntent
		if err := json.Unmarshal([]byte(llmResponse), &intent); err == nil {
			intent.Source = "LLM"
			intent.Confidence = 0.85
			log.Println("[Ingestion] LLM successfully extracted intent.")
			return &intent, nil
		}
	}

	// 2. Fallback to Regex / AST Parsing if LLM fails, hallucinates, or times out
	intent, fallbackErr := fallbackRegexExtractor(userInput)
	if fallbackErr == nil {
		return intent, nil
	}

	return nil, errors.New("pipeline exhausted: both LLM and Fallback failed to understand intent")
}

func main() {
	log.Println("Initializing SMA-OS Memory Bus: Ingestion / Fallback Pipeline v2.0")

	// Case 1: Simple command handled by LLM appropriately
	intent1, _ := ProcessInput("Please create a VM in pool-A with cpu=2,ram=4G")
	log.Printf("Final Intent 1: %+v\n", intent1)

	// Case 2: Complex command that causes LLM to hallucinate or timeout, gracefully degraded
	intent2, _ := ProcessInput("complex command: create a vm in pool B with cpu=8,ram=16G")
	log.Printf("Final Intent 2: %+v\n", intent2)
}
