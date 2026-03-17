package llm

import (
	"context"
	"errors"
)

// Provider defines the interface for LLM providers
type Provider interface {
	// Name returns the provider name
	Name() string

	// Invoke calls the LLM with a prompt and returns the response
	Invoke(ctx context.Context, prompt string, systemPrompt string) (string, error)

	// IsAvailable checks if the provider is properly configured
	IsAvailable() bool
}

// Config holds common LLM configuration
type Config struct {
	// Model identifier
	Model string
	// Temperature for response generation (0.0-2.0)
	Temperature float64
	// MaxTokens limits response length (0 = unlimited)
	MaxTokens int
	// Timeout for API calls in seconds
	Timeout int
}

// DefaultConfig returns sensible defaults
func DefaultConfig() Config {
	return Config{
		Model:       "",
		Temperature: 0.1, // Low for deterministic JSON output
		MaxTokens:   0,   // Unlimited
		Timeout:     30,  // 30 seconds
	}
}

// ErrProviderNotAvailable is returned when a provider is not configured
var ErrProviderNotAvailable = errors.New("LLM provider not available")

// Message represents a chat message
type Message struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

// ChatRequest represents a standard chat completion request
type ChatRequest struct {
	Model       string    `json:"model"`
	Messages    []Message `json:"messages"`
	Temperature float64   `json:"temperature,omitempty"`
	MaxTokens   int       `json:"max_tokens,omitempty"`
	Stream      bool      `json:"stream,omitempty"`
}

// ChatResponse represents a standard chat completion response
type ChatResponse struct {
	Choices []struct {
		Message struct {
			Content string `json:"content"`
		} `json:"message"`
		FinishReason string `json:"finish_reason"`
	} `json:"choices"`
	Usage struct {
		PromptTokens     int `json:"prompt_tokens"`
		CompletionTokens int `json:"completion_tokens"`
		TotalTokens      int `json:"total_tokens"`
	} `json:"usage"`
}
