package main

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"
)

// VersionedReject represents an explicitly versioned fallback/reject instruction
type VersionedReject struct {
	TaskID          string
	RejectedVersion uint64
	Reason          string
	RollbackTo      uint64
}

// EvaluatorAgent maintains independent verification of the cognitive decisions.
type EvaluatorAgent struct {
	// e.g. connected to Memory Bus via gRPC
}

func NewEvaluatorAgent() *EvaluatorAgent {
	return &EvaluatorAgent{}
}

func (e *EvaluatorAgent) AuditTaskResult(taskID string, version uint64, result string) *VersionedReject {
	// 规则驱动的验证：根据结果内容判断是否需要回滚
	var reason string
	switch {
	case result == "invalid_schema":
		reason = "schema_mismatch"
	case result == "":
		reason = "empty_result"
	case result == "empty_action":
		reason = "missing_action_field"
	default:
		log.Printf("[Critic] Task %s Version %d alignment OK.", taskID, version)
		return nil
	}

	log.Printf("[Critic] Rejecting Task %s at Version %d: %s", taskID, version, reason)

	// 防止 uint64 下溢：version 为 0 时不能减 1
	rollbackVersion := uint64(0)
	if version > 0 {
		rollbackVersion = version - 1
	}

	return &VersionedReject{
		TaskID:          taskID,
		RejectedVersion: version,
		Reason:          reason,
		RollbackTo:      rollbackVersion,
	}
}

func main() {
	log.Println("Starting SMA-OS Evaluator/Critic Agent v2.0...")

	evaluator := NewEvaluatorAgent()

	// Example usage
	rejectCmd := evaluator.AuditTaskResult("task-uuid-1234", 102, "invalid_schema")
	if rejectCmd != nil {
		log.Printf("Emit versioned rollback Command: Rollback task %s to version %d", rejectCmd.TaskID, rejectCmd.RollbackTo)
	}

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	go func() {
		<-quit
		log.Println("Evaluator shutting down...")
		cancel()
	}()

	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			log.Println("[Critic] Waiting for alignment audit tasks...")
		case <-ctx.Done():
			log.Println("Evaluator gracefully stopped.")
			return
		}
	}
}
