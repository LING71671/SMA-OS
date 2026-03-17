package llm

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"time"
)

// OpenAIProvider implements Provider for OpenAI API
type OpenAIProvider struct {
	apiKey  string
	baseURL string
	config  Config
	client  *http.Client
}

// NewOpenAIProvider creates a new OpenAI provider
func NewOpenAIProvider() *OpenAIProvider {
	baseURL := os.Getenv("OPENAI_BASE_URL")
	if baseURL == "" {
		baseURL = "https://api.openai.com/v1/chat/completions"
	}

	return &OpenAIProvider{
		apiKey:  os.Getenv("OPENAI_API_KEY"),
		baseURL: baseURL,
		config:  DefaultConfig(),
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

func (p *OpenAIProvider) Name() string {
	return "openai"
}

func (p *OpenAIProvider) IsAvailable() bool {
	return p.apiKey != ""
}

func (p *OpenAIProvider) Invoke(ctx context.Context, prompt, systemPrompt string) (string, error) {
	if !p.IsAvailable() {
		return "", ErrProviderNotAvailable
	}

	model := p.config.Model
	if model == "" {
		model = "gpt-4o-mini" // Default to cost-effective model
	}

	reqBody := ChatRequest{
		Model: model,
		Messages: []Message{
			{Role: "system", Content: systemPrompt},
			{Role: "user", Content: prompt},
		},
		Temperature: p.config.Temperature,
		Stream:      false,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequestWithContext(ctx, "POST", p.baseURL, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+p.apiKey)

	resp, err := p.client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return "", errors.New("OpenAI API error: " + string(bodyBytes))
	}

	var chatResp ChatResponse
	if err := json.NewDecoder(resp.Body).Decode(&chatResp); err != nil {
		return "", err
	}

	if len(chatResp.Choices) > 0 {
		return chatResp.Choices[0].Message.Content, nil
	}

	return "", errors.New("empty response from OpenAI")
}

func (p *OpenAIProvider) SetModel(model string) {
	p.config.Model = model
}

func (p *OpenAIProvider) SetTemperature(temp float64) {
	p.config.Temperature = temp
}

// AnthropicProvider implements Provider for Anthropic Claude API
type AnthropicProvider struct {
	apiKey  string
	baseURL string
	config  Config
	client  *http.Client
}

func NewAnthropicProvider() *AnthropicProvider {
	return &AnthropicProvider{
		apiKey:  os.Getenv("ANTHROPIC_API_KEY"),
		baseURL: "https://api.anthropic.com/v1/messages",
		config:  DefaultConfig(),
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

func (p *AnthropicProvider) Name() string {
	return "anthropic"
}

func (p *AnthropicProvider) IsAvailable() bool {
	return p.apiKey != ""
}

func (p *AnthropicProvider) Invoke(ctx context.Context, prompt, systemPrompt string) (string, error) {
	if !p.IsAvailable() {
		return "", ErrProviderNotAvailable
	}

	model := p.config.Model
	if model == "" {
		model = "claude-3-haiku-20240307"
	}

	reqBody := map[string]interface{}{
		"model": model,
		"messages": []map[string]string{
			{"role": "user", "content": prompt},
		},
		"system":      systemPrompt,
		"max_tokens":  1024,
		"temperature": p.config.Temperature,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequestWithContext(ctx, "POST", p.baseURL, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("x-api-key", p.apiKey)
	req.Header.Set("anthropic-version", "2023-06-01")

	resp, err := p.client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return "", errors.New("Anthropic API error: " + string(bodyBytes))
	}

	var result struct {
		Content []struct {
			Text string `json:"text"`
		} `json:"content"`
	}

	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	if len(result.Content) > 0 {
		return result.Content[0].Text, nil
	}

	return "", errors.New("empty response from Anthropic")
}

func (p *AnthropicProvider) SetModel(model string) {
	p.config.Model = model
}

// DeepSeekProvider implements Provider for DeepSeek API
type DeepSeekProvider struct {
	apiKey  string
	baseURL string
	config  Config
	client  *http.Client
}

func NewDeepSeekProvider() *DeepSeekProvider {
	return &DeepSeekProvider{
		apiKey:  os.Getenv("DEEPSEEK_API_KEY"),
		baseURL: "https://api.deepseek.com/chat/completions",
		config:  DefaultConfig(),
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

func (p *DeepSeekProvider) Name() string {
	return "deepseek"
}

func (p *DeepSeekProvider) IsAvailable() bool {
	return p.apiKey != ""
}

func (p *DeepSeekProvider) Invoke(ctx context.Context, prompt, systemPrompt string) (string, error) {
	if !p.IsAvailable() {
		return "", ErrProviderNotAvailable
	}

	model := p.config.Model
	if model == "" {
		model = "deepseek-chat"
	}

	reqBody := ChatRequest{
		Model: model,
		Messages: []Message{
			{Role: "system", Content: systemPrompt},
			{Role: "user", Content: prompt},
		},
		Temperature: p.config.Temperature,
		Stream:      false,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequestWithContext(ctx, "POST", p.baseURL, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+p.apiKey)

	resp, err := p.client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return "", errors.New("DeepSeek API error: " + string(bodyBytes))
	}

	var chatResp ChatResponse
	if err := json.NewDecoder(resp.Body).Decode(&chatResp); err != nil {
		return "", err
	}

	if len(chatResp.Choices) > 0 {
		return chatResp.Choices[0].Message.Content, nil
	}

	return "", errors.New("empty response from DeepSeek")
}

func (p *DeepSeekProvider) SetModel(model string) {
	p.config.Model = model
}

// OllamaProvider implements Provider for local Ollama
type OllamaProvider struct {
	baseURL string
	config  Config
	client  *http.Client
}

func NewOllamaProvider() *OllamaProvider {
	baseURL := os.Getenv("OLLAMA_BASE_URL")
	if baseURL == "" {
		baseURL = "http://localhost:11434/api/chat"
	}

	return &OllamaProvider{
		baseURL: baseURL,
		config:  DefaultConfig(),
		client: &http.Client{
			Timeout: 60 * time.Second, // Local may be slower
		},
	}
}

func (p *OllamaProvider) Name() string {
	return "ollama"
}

func (p *OllamaProvider) IsAvailable() bool {
	// Ollama doesn't need API key, just check if server is running
	return true
}

func (p *OllamaProvider) Invoke(ctx context.Context, prompt, systemPrompt string) (string, error) {
	model := p.config.Model
	if model == "" {
		model = "llama3" // Default model
	}

	reqBody := map[string]interface{}{
		"model": model,
		"messages": []Message{
			{Role: "system", Content: systemPrompt},
			{Role: "user", Content: prompt},
		},
		"stream": false,
		"options": map[string]interface{}{
			"temperature": p.config.Temperature,
		},
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequestWithContext(ctx, "POST", p.baseURL, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := p.client.Do(req)
	if err != nil {
		return "", fmt.Errorf("Ollama connection failed: %w (is Ollama running?)", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return "", errors.New("Ollama error: " + string(bodyBytes))
	}

	var result struct {
		Message struct {
			Content string `json:"content"`
		} `json:"message"`
	}

	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	return result.Message.Content, nil
}

func (p *OllamaProvider) SetModel(model string) {
	p.config.Model = model
}

// OpenAICompatibleProvider for any OpenAI-compatible API (LM Studio, vLLM, etc.)
type OpenAICompatibleProvider struct {
	apiKey  string
	baseURL string
	config  Config
	client  *http.Client
	name    string
}

func NewOpenAICompatibleProvider(name string) *OpenAICompatibleProvider {
	envKey := fmt.Sprintf("%s_API_KEY", name)
	baseURLEnv := fmt.Sprintf("%s_BASE_URL", name)

	baseURL := os.Getenv(baseURLEnv)
	if baseURL == "" {
		baseURL = "http://localhost:8000/v1/chat/completions"
	}

	return &OpenAICompatibleProvider{
		name:    name,
		apiKey:  os.Getenv(envKey),
		baseURL: baseURL,
		config:  DefaultConfig(),
		client: &http.Client{
			Timeout: 60 * time.Second,
		},
	}
}

func (p *OpenAICompatibleProvider) Name() string {
	return p.name
}

func (p *OpenAICompatibleProvider) IsAvailable() bool {
	return true // Local servers don't need API key
}

func (p *OpenAICompatibleProvider) Invoke(ctx context.Context, prompt, systemPrompt string) (string, error) {
	model := p.config.Model
	if model == "" {
		model = "local-model"
	}

	reqBody := ChatRequest{
		Model: model,
		Messages: []Message{
			{Role: "system", Content: systemPrompt},
			{Role: "user", Content: prompt},
		},
		Temperature: p.config.Temperature,
		Stream:      false,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequestWithContext(ctx, "POST", p.baseURL, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", err
	}

	req.Header.Set("Content-Type", "application/json")
	if p.apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+p.apiKey)
	}

	resp, err := p.client.Do(req)
	if err != nil {
		return "", fmt.Errorf("%s connection failed: %w", p.name, err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("%s error: %s", p.name, string(bodyBytes))
	}

	var chatResp ChatResponse
	if err := json.NewDecoder(resp.Body).Decode(&chatResp); err != nil {
		return "", err
	}

	if len(chatResp.Choices) > 0 {
		return chatResp.Choices[0].Message.Content, nil
	}

	return "", fmt.Errorf("empty response from %s", p.name)
}

func (p *OpenAICompatibleProvider) SetModel(model string) {
	p.config.Model = model
}
