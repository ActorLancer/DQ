package service

import (
	"context"
	"fmt"
	"os"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	adapterconfig "datab.local/fabric-adapter/internal/config"
	"datab.local/fabric-adapter/internal/provider"
	"datab.local/fabric-adapter/internal/store"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
)

func TestProcessorReliabilityLiveSmoke(t *testing.T) {
	if os.Getenv("FABRIC_ADAPTER_RELIABILITY_SMOKE") != "1" {
		t.Skip("set FABRIC_ADAPTER_RELIABILITY_SMOKE=1 to run")
	}

	cfg, err := adapterconfig.Load()
	if err != nil {
		t.Fatalf("load config: %v", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	pool, err := pgxpool.New(ctx, cfg.DatabaseURL)
	if err != nil {
		t.Fatalf("connect postgres: %v", err)
	}
	defer pool.Close()

	eventID := uuid.NewString()
	anchorBatchID := uuid.NewString()
	chainAnchorID := uuid.NewString()
	requestID := uuid.NewString()
	traceID := uuid.NewString()
	batchRoot := "aud-reliability-root-" + strings.ReplaceAll(anchorBatchID[:8], "-", "")

	if _, err := pool.Exec(
		ctx,
		`INSERT INTO chain.chain_anchor (
		   chain_anchor_id,
		   chain_id,
		   anchor_type,
		   ref_type,
		   ref_id,
		   digest,
		   status,
		   authority_model,
		   reconcile_status,
		   created_at
		 ) VALUES (
		   $1::text::uuid,
		   'fabric-local',
		   'audit_anchor_batch',
		   'anchor_batch',
		   $2::text::uuid,
		   $3,
		   'pending',
		   'dual_authority',
		   'pending_check',
		   now()
		 )`,
		chainAnchorID,
		anchorBatchID,
		batchRoot,
	); err != nil {
		t.Fatalf("insert chain.chain_anchor: %v", err)
	}
	if _, err := pool.Exec(
		ctx,
		`INSERT INTO audit.anchor_batch (
		   anchor_batch_id,
		   batch_scope,
		   chain_id,
		   record_count,
		   batch_root,
		   window_started_at,
		   window_ended_at,
		   status,
		   chain_anchor_id,
		   metadata
		 ) VALUES (
		   $1::text::uuid,
		   'audit_event',
		   'fabric-local',
		   1,
		   $2,
		   now() - interval '1 minute',
		   now(),
		   'retry_requested',
		   $3::text::uuid,
		   jsonb_build_object('seed', 'aud-fabric-reliability')
		 )`,
		anchorBatchID,
		batchRoot,
		chainAnchorID,
	); err != nil {
		t.Fatalf("insert audit.anchor_batch: %v", err)
	}
	defer func() {
		_, _ = pool.Exec(context.Background(), `DELETE FROM ops.consumer_idempotency_record WHERE consumer_name = $1 AND event_id = $2::text::uuid`, cfg.ServiceName, eventID)
		_, _ = pool.Exec(context.Background(), `DELETE FROM ops.external_fact_receipt WHERE request_id = $1`, requestID)
		_, _ = pool.Exec(context.Background(), `DELETE FROM audit.anchor_batch WHERE anchor_batch_id = $1::text::uuid`, anchorBatchID)
		_, _ = pool.Exec(context.Background(), `DELETE FROM chain.chain_anchor WHERE chain_anchor_id = $1::text::uuid`, chainAnchorID)
	}()

	persist, err := store.New(ctx, cfg.DatabaseURL, cfg.ServiceName)
	if err != nil {
		t.Fatalf("new store: %v", err)
	}
	defer persist.Close()

	locker, err := NewRedisShortLock(ctx, cfg.RedisURL, cfg.RedisNamespace, 5*time.Second)
	if err != nil {
		t.Fatalf("new redis short lock: %v", err)
	}
	defer locker.Close()
	defer func() {
		_ = locker.Release(context.Background(), eventID)
	}()

	providerCalls := atomic.Int32{}
	processor := NewProcessor(
		cfg.ServiceName,
		persist,
		submissionProviderFunc(func(ctx context.Context, request provider.SubmissionRequest) (provider.SubmissionReceipt, error) {
			providerCalls.Add(1)
			time.Sleep(1500 * time.Millisecond)
			return provider.SubmissionReceipt{
				ProviderType:      "fabric_gateway",
				ProviderKey:       "mock-fabric-gateway",
				ProviderReference: "mock-tx-" + eventID[:8],
				ReceiptStatus:     "submitted",
				OccurredAt:        time.Now().UTC(),
				ReceiptPayload: map[string]any{
					"mode":             "mock",
					"chain_id":         "fabric-local",
					"event_id":         request.Envelope.EventID,
					"submission_kind":  string(request.SubmissionKind),
					"contract_name":    request.ContractName,
					"transaction_name": request.TransactionName,
				},
			}, nil
		}),
		locker,
		testLogger(),
	)

	raw := []byte(fmt.Sprintf(`{
	  "event_id":"%s",
	  "event_type":"audit.anchor_requested",
	  "event_version":1,
	  "occurred_at":"2026-04-22T08:00:00Z",
	  "producer_service":"platform-core.audit",
	  "aggregate_type":"audit.anchor_batch",
	  "aggregate_id":"%s",
	  "request_id":"%s",
	  "trace_id":"%s",
	  "event_schema_version":"v1",
	  "authority_scope":"audit",
	  "source_of_truth":"postgresql",
	  "proof_commit_policy":"async_evidence",
	  "anchor_batch_id":"%s",
	  "chain_anchor_id":"%s",
	  "batch_root":"%s",
	  "payload":{"batch_root":"%s","record_count":1}
	}`, eventID, anchorBatchID, requestID, traceID, anchorBatchID, chainAnchorID, batchRoot, batchRoot))

	firstDone := make(chan error, 1)
	go func() {
		firstDone <- processor.ProcessMessage(ctx, "dtp.audit.anchor", raw)
	}()

	lockDeadline := time.Now().Add(5 * time.Second)
	for {
		exists, err := locker.Exists(ctx, eventID)
		if err != nil {
			t.Fatalf("check redis lock exists: %v", err)
		}
		if exists {
			break
		}
		if time.Now().After(lockDeadline) {
			t.Fatalf("redis short lock was not observed for event_id=%s", eventID)
		}
		time.Sleep(100 * time.Millisecond)
	}

	if err := processor.ProcessMessage(ctx, "dtp.audit.anchor", raw); err != nil {
		t.Fatalf("process duplicate message: %v", err)
	}
	if err := <-firstDone; err != nil {
		t.Fatalf("process first message: %v", err)
	}

	if got := providerCalls.Load(); got != 1 {
		t.Fatalf("provider call count = %d, want 1", got)
	}

	var receiptCount int
	if err := pool.QueryRow(
		ctx,
		`SELECT count(*)
		   FROM ops.external_fact_receipt
		  WHERE request_id = $1`,
		requestID,
	).Scan(&receiptCount); err != nil {
		t.Fatalf("query receipt count: %v", err)
	}
	if receiptCount != 1 {
		t.Fatalf("receipt count = %d, want 1", receiptCount)
	}

	var resultCode string
	var metadata string
	if err := pool.QueryRow(
		ctx,
		`SELECT result_code, metadata::text
		   FROM ops.consumer_idempotency_record
		  WHERE consumer_name = $1
		    AND event_id = $2::text::uuid`,
		cfg.ServiceName,
		eventID,
	).Scan(&resultCode, &metadata); err != nil {
		t.Fatalf("query consumer idempotency record: %v", err)
	}
	if resultCode != "processed" {
		t.Fatalf("result_code = %q, want processed", resultCode)
	}
	if !strings.Contains(metadata, "fabric-adapter:consumer-lock:"+eventID) {
		t.Fatalf("metadata missing redis lock key: %s", metadata)
	}

	lockExists, err := locker.Exists(ctx, eventID)
	if err != nil {
		t.Fatalf("check redis lock release: %v", err)
	}
	if lockExists {
		t.Fatalf("expected redis lock to be released")
	}
}
