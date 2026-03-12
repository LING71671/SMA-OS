# Intent Extraction Module Guide

**Location**: `memory-bus/ingestion/`  
**Domain**: SLM-powered intent extraction with regex fallback  
**Language**: Go  
**Score**: 15/25 (LLM integration, distinct NLP domain)

## Overview

Natural language understanding layer using DeepSeek API with deterministic fallback. Extracts structured intent from user commands, bridging natural language to actionable system operations.

## Structure

```
ingestion/
├── main.go              # DeepSeek API client + fallback logic
├── go.mod              # Uses sma-os/memory-bus module
└── main_test.go        # (if exists)
```

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| DeepSeek API | `main.go:50-103` | HTTP POST with auth header |
| Intent schema | `main.go:41-47` | ParsedIntent JSON structure |
| Fallback regex | `main.go:106-124` | Pattern-based extraction |
| LLM prompt | `main.go:57-58` | System prompt with JSON format |

## Conventions (This Module)

### API Key Management
```go
var DeepSeekAPIKey = os.Getenv("DEEPSEEK_API_KEY")
// NEVER hardcode - must come from environment
```

### Request/Response Types
```go
type DeepSeekRequest struct {
    Model       string              `json:"model"`
    Messages    []DeepSeekMessage   `json:"messages"`
    Temperature float64             `json:"temperature"`  // 0.1 for deterministic
}
```

### Fallback Pipeline
```go
// 1. Try LLM first
llmResponse, err := invokeLLM(userInput)
if err == nil { return intent, nil }

// 2. Fallback to regex
intent, fallbackErr := fallbackRegexExtractor(userInput)
if fallbackErr == nil { return intent, nil }

// 3. Exhaustion error
return nil, errors.New("pipeline exhausted")
```

## Anti-Patterns (This Module)

### Forbidden
```go
// NEVER: Hardcode API key
const DeepSeekAPIKey = "sk-..."  // SECURITY RISK

// ALWAYS: Environment variable
var DeepSeekAPIKey = os.Getenv("DEEPSEEK_API_KEY")
```

### JSON Parsing
```go
// WRONG: Fails on partial response
json.Unmarshal([]byte(llmResponse), &intent)

// CORRECT: Validate structure, handle errors
if err := json.Unmarshal(...); err == nil {
    intent.Source = "LLM"
    return &intent, nil
}
```

### Error Handling
```go
// WRONG: Silent failure
defer resp.Body.Close()

// CORRECT: Check status, read body for error
if resp.StatusCode != http.StatusOK {
    bodyBytes, _ := io.ReadAll(resp.Body)
    return "", errors.New("API Error: " + string(bodyBytes))
}
```

## Unique Styles

### Temperature Setting
```go
// Low temperature for deterministic JSON
Temperature: 0.1  // vs 0.7+ for creative tasks
```

### System Prompt Engineering
```go
Content: `You are the SMA-OS Intent Extractor. 
Extract the user's command into EXACTLY this JSON format, 
NO markdown formatting: {"action": "...", "target": "...", "parameters": "..."}`
```

### Fallback Confidence
```go
// Different confidence for LLM vs fallback
intent.Source = "LLM"
intent.Confidence = 0.85

intent.Source = "REGEX_FALLBACK"
intent.Confidence = 0.99  // Deterministic
```

## Commands

```bash
# Build
cd memory-bus/ingestion && go build -o bin/ingestion .

# Run (requires DEEPSEEK_API_KEY env var)
export DEEPSEEK_API_KEY=your-key
go run main.go
```

## Dependencies

| Package | Purpose |
|---------|---------|
| net/http | DeepSeek API calls |
| encoding/json | Request/response serialization |
| regexp | Fallback pattern matching |
| bytes | Request body construction |
| io | Response body reading |
| time | HTTP client timeout |

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| DEEPSEEK_API_KEY | Yes | DeepSeek API authentication |

## Notes

- **Timeout**: 15 seconds for API calls
- **Model**: deepseek-chat (DeepSeek V3)
- **Endpoint**: https://api.deepseek.com/chat/completions
- **Regex patterns**: Currently limited to VM creation commands
- **No caching**: Each request hits API (add Redis for production)
