package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"net/http"
	"os"
	"regexp"
	"strings"

	"sma-os/memory-bus/ingestion/internal/cache"
	"sma-os/memory-bus/ingestion/internal/llm"
	metrics "sma-os/memory-bus/ingestion/internal/metrics"
)

// Intent schema that downstream Evaluator expects
type ParsedIntent struct {
	Action     string  `json:"action"`
	Target     string  `json:"target"`
	Parameters string  `json:"parameters"`
	Confidence float64 `json:"confidence"`
	Source     string  `json:"source"`
}

// IntentExtractor handles intent extraction with LLM and fallback
type IntentExtractor struct {
	llmManager   *llm.Manager
	cacheManager *cache.CacheManager
}

// NewIntentExtractor creates a new intent extractor
func NewIntentExtractor() *IntentExtractor {
	return &IntentExtractor{
		llmManager: llm.NewManager(),
	}
}

// WithCache sets the cache manager
func (e *IntentExtractor) WithCache(cacheManager *cache.CacheManager) *IntentExtractor {
	e.cacheManager = cacheManager
	return e
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
func (e *IntentExtractor) ProcessInput(userInput string) (*ParsedIntent, error) {
	log.Printf("\n--- Processing User Input: %s ---", userInput)

	// LLM 调用封装
	callLLMAndParse := func() (*ParsedIntent, error) {
		llmResponse, err := e.llmManager.Invoke(userInput)
		if err != nil {
			return nil, err
		}
		var intent ParsedIntent
		if err := json.Unmarshal([]byte(llmResponse), &intent); err != nil {
			return nil, fmt.Errorf("LLM response unmarshal failed: %w", err)
		}
		intent.Source = "LLM"
		intent.Confidence = 0.85
		return &intent, nil
	}

	// 1. 如果缓存启用，通过缓存管理器调用 LLM
	if e.cacheManager != nil {
		cachedResponse, err := e.cacheManager.Get(context.Background(), userInput, func(ctx context.Context) (string, error) {
			return e.llmManager.Invoke(userInput)
		})

		if err == nil {
			var intent ParsedIntent
			if err := json.Unmarshal([]byte(cachedResponse), &intent); err == nil {
				intent.Source = "LLM"
				intent.Confidence = 0.85
				log.Println("[Ingestion] LLM response retrieved (cached or fresh).")
				return &intent, nil
			}
			log.Printf("[Warning] Failed to unmarshal cached response: %v", err)
		} else {
			log.Printf("[Warning] Cache/LLM error: %v. Skipping duplicate LLM call, falling back to regex.", err)
		}
	} else {
		// 2. 缓存未启用时，直接调用 LLM
		intent, err := callLLMAndParse()
		if err == nil {
			log.Println("[Ingestion] LLM successfully extracted intent.")
			return intent, nil
		}
		log.Printf("[Warning] LLM failed: %v. Falling back to regex.", err)
	}

	// 3. Fallback: Regex 提取
	intent, fallbackErr := fallbackRegexExtractor(userInput)
	if fallbackErr == nil {
		return intent, nil
	}

	return nil, errors.New("pipeline exhausted: both LLM and Fallback failed to understand intent")
}

func main() {
	log.Println("Initializing SMA-OS Memory Bus: Ingestion / Fallback Pipeline v3.0 (Multi-LLM)")

	// Log configured providers
	extractor := NewIntentExtractor()
	providers := extractor.llmManager.GetProviders()
	log.Printf("[LLM] Configured providers: %v", extractor.llmManager.GetProviders())

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
				cacheManager, _ = cache.NewCacheManager(localCache, nil)
			} else {
				cacheManager, _ = cache.NewCacheManager(localCache, redisClient)
				log.Println("[Cache] Multi-level cache initialized (L1: local, L2: Redis)")
			}
		}
	} else {
		log.Println("[Cache] Cache disabled via CACHE_ENABLED=false")
	}

	// Set cache if available
	if cacheManager != nil {
		extractor.WithCache(cacheManager)
	}

	// Ensure cache cleanup on exit
	defer func() {
		if cacheManager != nil {
			cacheManager.Close()
			log.Println("[Cache] Cache resources cleaned up")
		}
	}()

	// Start metrics endpoint
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

	// Test cases
	intent1, _ := extractor.ProcessInput("Please create a VM in pool-A with cpu=2,ram=4G")
	log.Printf("Final Intent 1: %+v\n\n", intent1)

	intent2, _ := extractor.ProcessInput("complex command: create instance in pool B with cpu=8,ram=16G")
	log.Printf("Final Intent 2: %+v\n", intent2)

	// Log provider info
	_ = providers // suppress unused variable warning
}
