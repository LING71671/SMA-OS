package llm

import (
	"context"
	"fmt"
	"log"
	"os"
	"strings"
)

// ProviderType defines the type of LLM provider
type ProviderType string

const (
	ProviderOpenAI       ProviderType = "openai"
	ProviderAnthropic    ProviderType = "anthropic"
	ProviderDeepSeek     ProviderType = "deepseek"
	ProviderOllama       ProviderType = "ollama"
	ProviderLMStudio     ProviderType = "lmstudio"
	ProviderVLLM         ProviderType = "vllm"
	ProviderOpenAICompat ProviderType = "openai-compatible"
)

// Manager manages multiple LLM providers with fallback
type Manager struct {
	providers    []Provider
	rootCtx      context.Context
	systemPrompt string
}

// ManagerOption configures the manager
type ManagerOption func(*Manager)

// WithProviders sets the providers to use
func WithProviders(providers ...Provider) ManagerOption {
	return func(m *Manager) {
		m.providers = providers
	}
}

// WithSystemPrompt sets the system prompt
func WithSystemPrompt(prompt string) ManagerOption {
	return func(m *Manager) {
		m.systemPrompt = prompt
	}
}

// NewManager creates a new LLM manager
func NewManager(opts ...ManagerOption) *Manager {
	m := &Manager{
		rootCtx:      context.Background(),
		systemPrompt: `You are the SMA-OS Intent Extractor. Extract the user's command into EXACTLY this JSON format, NO markdown formatting: {"action": "string", "target": "string", "parameters": "string"}. E.g for "create vm pool A cpu=2", return {"action": "create_vm", "target": "pool-A", "parameters": "cpu=2"}`,
	}

	for _, opt := range opts {
		opt(m)
	}

	// If no providers specified, auto-detect from environment
	if len(m.providers) == 0 {
		m.providers = m.autoDetectProviders()
	}

	return m
}

// autoDetectProviders detects available providers from environment
func (m *Manager) autoDetectProviders() []Provider {
	var providers []Provider

	// Priority order based on environment variables
	providerType := os.Getenv("LLM_PROVIDER")
	if providerType == "" {
		providerType = os.Getenv("LLM_PROVIDER_TYPE")
	}

	// If specific provider is set, use only that
	switch strings.ToLower(providerType) {
	case "openai":
		if p := NewOpenAIProvider(); p.IsAvailable() {
			providers = append(providers, p)
		}
	case "anthropic":
		if p := NewAnthropicProvider(); p.IsAvailable() {
			providers = append(providers, p)
		}
	case "deepseek":
		if p := NewDeepSeekProvider(); p.IsAvailable() {
			providers = append(providers, p)
		}
	case "ollama":
		providers = append(providers, NewOllamaProvider())
	case "lmstudio":
		p := NewOpenAICompatibleProvider("lmstudio")
		providers = append(providers, p)
	case "vllm":
		p := NewOpenAICompatibleProvider("vllm")
		providers = append(providers, p)
	default:
		// Auto-detect: try each provider in order
		if p := NewOpenAIProvider(); p.IsAvailable() {
			providers = append(providers, p)
		}
		if p := NewAnthropicProvider(); p.IsAvailable() {
			providers = append(providers, p)
		}
		if p := NewDeepSeekProvider(); p.IsAvailable() {
			providers = append(providers, p)
		}
		// Always add local providers at the end as fallback
		providers = append(providers, NewOllamaProvider())
	}

	log.Printf("[LLM Manager] Configured providers: %v", m.providerNames(providers))
	return providers
}

func (m *Manager) providerNames(providers []Provider) []string {
	names := make([]string, len(providers))
	for i, p := range providers {
		names[i] = p.Name()
	}
	return names
}

// Invoke tries each provider in order until one succeeds
func (m *Manager) Invoke(prompt string) (string, error) {
	return m.InvokeWithContext(m.rootCtx, prompt)
}

// InvokeWithContext tries each provider with context
func (m *Manager) InvokeWithContext(ctx context.Context, prompt string) (string, error) {
	if len(m.providers) == 0 {
		return "", fmt.Errorf("no LLM providers configured")
	}

	var lastErr error
	for _, provider := range m.providers {
		log.Printf("[LLM Manager] Trying provider: %s", provider.Name())

		response, err := provider.Invoke(ctx, prompt, m.systemPrompt)
		if err == nil {
			log.Printf("[LLM Manager] Success with provider: %s", provider.Name())
			return response, nil
		}

		log.Printf("[LLM Manager] Provider %s failed: %v", provider.Name(), err)
		lastErr = err
	}

	return "", fmt.Errorf("all providers failed, last error: %w", lastErr)
}

// GetProviders returns the configured providers
func (m *Manager) GetProviders() []Provider {
	return m.providers
}

// SetSystemPrompt updates the system prompt
func (m *Manager) SetSystemPrompt(prompt string) {
	m.systemPrompt = prompt
}

// GetSystemPrompt returns the current system prompt
func (m *Manager) GetSystemPrompt() string {
	return m.systemPrompt
}

// AddProvider adds a provider to the list
func (m *Manager) AddProvider(provider Provider) {
	m.providers = append(m.providers, provider)
}

// HasAvailableProvider checks if any provider is available
func (m *Manager) HasAvailableProvider() bool {
	return len(m.providers) > 0
}
