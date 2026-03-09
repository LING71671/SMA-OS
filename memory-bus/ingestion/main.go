package main

import (
	"context"
	"encoding/json"
	"log"
	"math/rand"
	"os"
	"os/signal"
	"regexp"
	"syscall"
	"time"

	"github.com/google/uuid"
)

// StructuredMemoryObject represents a verified output stored in the memory bus
type StructuredMemoryObject struct {
	ObjectID   string          `json:"object_id"`
	Version    uint64          `json:"version"`
	TenantID   string          `json:"tenant_id"`
	TraceID    string          `json:"trace_id"`
	Confidence float64         `json:"confidence"`
	Payload    json.RawMessage `json:"payload"`
	CreatedAt  time.Time       `json:"created_at"`
}

type IngestionPipeline struct {
	FallbackRegex *regexp.Regexp
}

func NewIngestionPipeline() *IngestionPipeline {
	return &IngestionPipeline{
		FallbackRegex: regexp.MustCompile(`(?i)(extract|find|create):\s*(.*)`),
	}
}

// Process ingestion stream with dual-fallback logic
func (p *IngestionPipeline) Process(tenantID, rawText string, version uint64) *StructuredMemoryObject {
	traceID := uuid.New().String()
	objID := uuid.New().String()

	log.Printf("[Ingestion] Processing text for trace: %s", traceID)

	// 1. Primary path: Try Local SLM (e.g., Llama-3-8B)
	confidence := p.mockSLMInfer(rawText)

	var payload json.RawMessage
	if confidence >= 0.98 {
		log.Printf("[Ingestion] SLM Confidence %.2f > 98%%. Accepting direct structure.", confidence)
		payload = json.RawMessage(`{"status": "slm_extracted", "content": "verified"}`)
	} else {
		// 2. Dual fallback: Rule engine / Regex
		log.Printf("[Ingestion] SLM Confidence %.2f < 98%%. Falling back to deterministic rules.", confidence)
		matches := p.FallbackRegex.FindStringSubmatch(rawText)
		if len(matches) > 2 {
			payload = json.RawMessage(`{"status": "regex_extracted", "keyword": "` + matches[2] + `"}`)
		} else {
			payload = json.RawMessage(`{"status": "failed", "content": "unrecognized"}`)
		}
	}

	// Forge the immutable audit chain record
	return &StructuredMemoryObject{
		ObjectID:   objID,
		Version:    version,
		TenantID:   tenantID,
		TraceID:    traceID,
		Confidence: confidence,
		Payload:    payload,
		CreatedAt:  time.Now(),
	}
}

func (p *IngestionPipeline) mockSLMInfer(text string) float64 {
	// Mock returns a random confidence between 90% and 100%
	return 0.90 + rand.Float64()*0.10
}

func main() {
	log.Println("Starting SMA-OS SLM Ingestion Pipeline v2.0...")

	pipeline := NewIngestionPipeline()

	obj1 := pipeline.Process("tenant-alpha", "extract: payment_info", 1)
	log.Printf("Ingested Object: %+v\n", obj1)

	obj2 := pipeline.Process("tenant-beta", "Some random unstructured chat that SLM might fail to parse perfectly.", 1)
	log.Printf("Ingested Object: %+v\n", obj2)

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	go func() {
		<-quit
		log.Println("Ingestion pipeline shutting down...")
		cancel()
	}()

	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			log.Println("[Ingestion] Waiting for raw text streams...")
		case <-ctx.Done():
			log.Println("Ingestion gracefully stopped.")
			return
		}
	}
}
