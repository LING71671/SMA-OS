package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"log"
	"net/http"
	"os"
	"regexp"
	"strings"
	"time"

	"sma-os/memory-bus/ingestion/internal/cache"
	metrics "sma-os/memory-bus/ingestion/internal/metrics"
)

var DeepSeekAPIKey = os.Getenv("DEEPSEEK_API_KEY")

const DeepSeekEndpoint = "https://api.deepseek.com/chat/completions"

// DeepSeek Request/Response Structures
type DeepSeekMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type DeepSeekRequest struct {
	Model       string            `json:"model"`
	Messages    []DeepSeekMessage `json:"messages"`
	Temperature float64           `json:"temperature"`
}

type DeepSeekResponse struct {
	Choices []struct {
		Message struct {
			Content string `json:"content"`
		} `json:"message"`
	} `json:"choices"`
}

// Intent schema that downstream Evaluator expects
type ParsedIntent struct {
	Action     string  `json:"action"`
	Target     string  `json:"target"`
	Parameters string  `json:"parameters"`
	Confidence float64 `json:"confidence"`
	Source     string  `json:"source"`
}

// Real DeepSeek API call
func invokeLLM(prompt string) (string, error) {
	log.Printf("[LLM Invocation] Querying DeepSeek API with prompt length %d...", len(prompt))
	// Record an API call metric
	metrics.RecordAPICall()

	reqBody := DeepSeekRequest{
		Model: "deepseek-chat", // standard DeepSeek V3 chat model
		Messages: []DeepSeekMessage{
			{
				Role: "system",
				Content: `You are the SMA-OS Intent Extractor. Extract the user's command into EXACTLY this JSON format, NO markdown formatting: {"action": "string", "target": "string", "parameters": "string"}. E.g for "create vm pool A cpu=2", return {"action": "create_vm", "target": "pool-A", "parameters": "cpu=2"}`,
			},
			{
				Role:    "user",
				Content: prompt,
			},
		},
		Temperature: 0.1, // Deterministic
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequest("POST", DeepSeekEndpoint, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+DeepSeekAPIKey)

	client := &http.Client{Timeout: 15 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return "", errors.New("DeepSeek API Error: " + string(bodyBytes))
	}

	var dsResp DeepSeekResponse
	if err := json.NewDecoder(resp.Body).Decode(&dsResp); err != nil {
		return "", err
	}

	if len(dsResp.Choices) > 0 {
		return dsResp.Choices[0].Message.Content, nil
	}

	return "", errors.New("empty choices from DeepSeek")
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

// ProcessInput processes user input with optional caching support
func ProcessInput(userInput string, cacheManager *cache.CacheManager) (*ParsedIntent, error) {
	log.Printf("\n--- Processing User Input: %s ---", userInput)

	// Check if caching is enabled
	if cacheManager != nil {
		// Try to get from cache
		cachedResponse, err := cacheManager.Get(context.Background(), userInput, func(ctx context.Context) (string, error) {
			// This is the loader function - only called on cache miss
			return invokeLLM(userInput)
		})

		if err == nil {
			var intent ParsedIntent
			if err := json.Unmarshal([]byte(cachedResponse), &intent); err == nil {
				intent.Source = "LLM"
				intent.Confidence = 0.85
				log.Println("[Ingestion] Cache hit - LLM response retrieved from cache.")
				return &intent, nil
			}
			log.Printf("[Warning] Failed to unmarshal cached response: %v", err)
		} else {
			// Log cache errors but continue with direct LLM call
			log.Printf("[Warning] Cache error: %v. Falling back to direct LLM call.", err)
		}
	}

	// 1. Attempt LLM JSON Extraction (direct call if cache miss or disabled)
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
	log.Println("Initializing SMA-OS Memory Bus: Ingestion / Fallback Pipeline v2.0 (DeepSeek Engine)")

	var cacheManager *cache.CacheManager

	// Initialize cache manager if CACHE_ENABLED is not set to "false"
	if os.Getenv("CACHE_ENABLED") != "false" {
		localCache, err := cache.NewLocalCache()
		if err != nil {
			log.Printf("[Warning] Failed to initialize local cache: %v. Continuing without cache.", err)
		} else {
			redisClient, err := cache.NewRedisClient()
			if err != nil {
				log.Printf("[Warning] Failed to connect to Redis: %v. Using local cache only.", err)
				// Use nil for Redis, local cache will still work
				cacheManager, _ = cache.NewCacheManager(localCache, nil)
			} else {
				cacheManager, _ = cache.NewCacheManager(localCache, redisClient)
				log.Println("[Cache] Multi-level cache initialized (L1: local, L2: Redis)")
			}
		}
	} else {
		log.Println("[Cache] Cache disabled via CACHE_ENABLED=false")
	}

	// Ensure cache cleanup on exit
	defer func() {
		if cacheManager != nil {
			cacheManager.Close()
			log.Println("[Cache] Cache resources cleaned up")
		}
	}()

	// Start a lightweight /metrics endpoint for Prometheus-like scraping (text format)
	go func() {
		http.HandleFunc("/metrics", func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "text/plain; version=0.0.4")
			w.Write([]byte(metrics.MetricsText()))
		})
		log.Println("Metrics endpoint available at http://localhost:8080/metrics")
		if err := http.ListenAndServe(":8080", nil); err != nil {
			log.Printf("[Warning] metrics server failed: %v", err)
		}
	}()

	// Case 1: Simple command handled by the REAL DeepSeek LLM appropriately
	intent1, _ := ProcessInput("Please create a VM in pool-A with cpu=2,ram=4G", cacheManager)
	log.Printf("Final Intent 1: %+v\n\n", intent1)

	// Case 2: Intentional failure to test fallback. We'll pass a prompt that is garbage.
	// Since DeepSeek might still try to parse it, we intentionally force the Regex Fallback by breaking the schema logic
	// or we can test the fallback directly with a regex that triggers but an LLM that might act weird.
	// For demonstration, let's just show a normal fallback test string.
	intent2, _ := ProcessInput("complex command: create instance in pool B with cpu=8,ram=16G", cacheManager)
	log.Printf("Final Intent 2: %+v\n", intent2)
}
