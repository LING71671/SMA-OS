# Intent Extraction Module Guide

**Location**: `memory-bus/ingestion/`  
**Domain**: Multi-provider LLM intent extraction with regex fallback  
**Language**: Go  
**Score**: 20/25 (Multi-LLM integration, extensible architecture)

## Overview

Natural language understanding layer supporting multiple LLM providers (OpenAI, Anthropic, DeepSeek, Ollama, local models) with deterministic fallback. Extracts structured intent from user commands, bridging natural language to actionable system operations.

## Architecture

```
ingestion/
├── main.go                    # Intent extractor with fallback pipeline
├── go.mod                     # Module dependencies
└── internal/
    ├── llm/
    │   ├── provider.go        # Provider interface definition
    │   ├── providers.go       # All provider implementations
    │   └── manager.go         # Multi-provider manager with fallback
    ├── cache/
    │   └── cache.go           # Redis + local cache
    └── metrics/
        └── metrics.go         # Prometheus metrics
```

## Supported LLM Providers

| Provider | Type | Environment Variables | Default Model |
|----------|------|----------------------|---------------|
| OpenAI | Cloud API | `OPENAI_API_KEY`, `OPENAI_BASE_URL` | gpt-4o-mini |
| Anthropic | Cloud API | `ANTHROPIC_API_KEY` | claude-3-haiku |
| DeepSeek | Cloud API | `DEEPSEEK_API_KEY` | deepseek-chat |
| Ollama | Local | `OLLAMA_BASE_URL` | llama3 |
| LM Studio | Local | `LMSTUDIO_BASE_URL` | local-model |
| vLLM | Local | `VLLM_BASE_URL` | local-model |
| Custom | Any | `<NAME>_API_KEY`, `<NAME>_BASE_URL` | configurable |

## Configuration

### Environment Variables

```bash
# Provider Selection (optional - auto-detected if not set)
LLM_PROVIDER=openai  # Options: openai, anthropic, deepseek, ollama, lmstudio, vllm

# Cloud API Keys
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
DEEPSEEK_API_KEY=sk-...

# Local LLM Configuration
OLLAMA_BASE_URL=http://localhost:11434/api/chat
LMSTUDIO_BASE_URL=http://localhost:1234/v1/chat/completions
VLLM_BASE_URL=http://localhost:8000/v1/chat/completions

# Cache Configuration
CACHE_ENABLED=true
REDIS_URL=redis://localhost:6379
```

### Auto-Detection Priority

If `LLM_PROVIDER` is not set, the system auto-detects in this order:
1. OpenAI (if `OPENAI_API_KEY` is set)
2. Anthropic (if `ANTHROPIC_API_KEY` is set)
3. DeepSeek (if `DEEPSEEK_API_KEY` is set)
4. Ollama (always available as fallback)

## Usage

### Basic Usage

```go
extractor := NewIntentExtractor()
intent, err := extractor.ProcessInput("create vm in pool A with cpu=2")
```

### With Specific Provider

```go
manager := llm.NewManager(
    llm.WithProviders(
        llm.NewOllamaProvider(),
    ),
)
extractor := &IntentExtractor{llmManager: manager}
```

### With Custom System Prompt

```go
manager := llm.NewManager(
    llm.WithSystemPrompt("Custom prompt here..."),
)
```

## Intent Schema

```go
type ParsedIntent struct {
    Action     string  `json:"action"`     // e.g., "create_vm"
    Target     string  `json:"target"`     // e.g., "pool-A"
    Parameters string  `json:"parameters"` // e.g., "cpu=2,ram=4G"
    Confidence float64 `json:"confidence"` // 0.0-1.0
    Source     string  `json:"source"`     // "LLM" or "REGEX_FALLBACK"
}
```

## Fallback Pipeline

```
User Input
    │
    ▼
┌─────────────────┐
│  Check Cache    │ ──► Hit? ──► Return cached result
└─────────────────┘
    │ Miss
    ▼
┌─────────────────┐
│  Try LLM        │ ──► Success? ──► Cache & Return
│  (in order)     │
└─────────────────┘
    │ All fail
    ▼
┌─────────────────┐
│  Regex Fallback │ ──► Match? ──► Return
└─────────────────┘
    │ No match
    ▼
  Error
```

## Commands

```bash
# Build
cd memory-bus/ingestion && go build -o bin/ingestion .

# Run with OpenAI
export OPENAI_API_KEY=sk-...
go run main.go

# Run with local Ollama
ollama serve &
ollama pull llama3
export LLM_PROVIDER=ollama
go run main.go

# Run with LM Studio
# (Start LM Studio server first)
export LLM_PROVIDER=lmstudio
export LMSTUDIO_BASE_URL=http://localhost:1234/v1/chat/completions
go run main.go
```

## Dependencies

| Package | Purpose |
|---------|---------|
| net/http | LLM API calls |
| encoding/json | Request/response serialization |
| regexp | Fallback pattern matching |
| context | Request cancellation |
| os | Environment configuration |

## Anti-Patterns

### Forbidden
```go
// NEVER: Hardcode API keys
const APIKey = "sk-..."

// NEVER: Block on LLM without timeout
resp, err := client.Do(req) // No timeout set

// NEVER: Ignore provider errors
_, _ = provider.Invoke(ctx, prompt)
```

### Required
```go
// ALWAYS: Use environment variables
apiKey := os.Getenv("OPENAI_API_KEY")

// ALWAYS: Set timeouts
client := &http.Client{Timeout: 30 * time.Second}

// ALWAYS: Handle errors with fallback
response, err := manager.Invoke(prompt)
if err != nil {
    return fallbackRegexExtractor(prompt)
}
```

## Metrics

The module exposes Prometheus metrics at `:8080/metrics`:
- `llm_api_calls_total` - Total LLM API calls
- `llm_api_errors_total` - Failed API calls
- `llm_latency_seconds` - Request latency histogram

## Notes

- **Timeout**: 30s for cloud APIs, 60s for local
- **Temperature**: 0.1 for deterministic JSON output
- **Cache**: Multi-level (L1: local memory, L2: Redis)
- **Fallback**: Regex patterns for common commands
