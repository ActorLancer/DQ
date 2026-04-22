package provider_test

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	adapterconfig "datab.local/fabric-adapter/internal/config"
	"datab.local/fabric-adapter/internal/model"
	adapterprovider "datab.local/fabric-adapter/internal/provider"
	"datab.local/fabric-adapter/internal/service"
	"datab.local/fabric-adapter/internal/store"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
)

func TestFabricGatewayLiveSmoke(t *testing.T) {
	if os.Getenv("FABRIC_ADAPTER_LIVE_SMOKE") != "1" {
		t.Skip("set FABRIC_ADAPTER_LIVE_SMOKE=1 to run")
	}

	cfg, err := adapterconfig.Load()
	if err != nil {
		t.Fatalf("load config: %v", err)
	}
	cfg.ProviderMode = "fabric-test-network"

	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	db, err := pgxpool.New(ctx, cfg.DatabaseURL)
	if err != nil {
		t.Fatalf("connect postgres: %v", err)
	}
	defer db.Close()

	anchorBatchID := uuid.NewString()
	chainAnchorID := uuid.NewString()
	eventID := uuid.NewString()
	requestID := uuid.NewString()
	traceID := uuid.NewString()
	batchRoot := "aud017-batch-root-" + strings.ReplaceAll(anchorBatchID[:8], "-", "")

	if _, err := db.Exec(
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
		   'audit_batch',
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
		"aud017-digest-"+anchorBatchID[:8],
	); err != nil {
		t.Fatalf("insert chain.chain_anchor: %v", err)
	}
	defer func() {
		_, _ = db.Exec(context.Background(), `DELETE FROM audit.anchor_batch WHERE anchor_batch_id = $1::text::uuid`, anchorBatchID)
		_, _ = db.Exec(context.Background(), `DELETE FROM chain.chain_anchor WHERE chain_anchor_id = $1::text::uuid`, chainAnchorID)
	}()

	if _, err := db.Exec(
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
		   3,
		   $2,
		   now() - interval '3 minutes',
		   now() - interval '1 minute',
		   'pending',
		   $3::text::uuid,
		   jsonb_build_object('seed', 'aud017-live-smoke', 'event_id', $4::text)
		 )`,
		anchorBatchID,
		batchRoot,
		chainAnchorID,
		eventID,
	); err != nil {
		t.Fatalf("insert audit.anchor_batch: %v", err)
	}

	raw := []byte(`{
	  "event_id":"` + eventID + `",
	  "event_type":"audit.anchor_requested",
	  "event_version":1,
	  "occurred_at":"2026-04-22T06:28:30Z",
	  "producer_service":"platform-core.audit",
	  "aggregate_type":"audit.anchor_batch",
	  "aggregate_id":"` + anchorBatchID + `",
	  "request_id":"` + requestID + `",
	  "trace_id":"` + traceID + `",
	  "event_schema_version":"v1",
	  "authority_scope":"audit",
	  "source_of_truth":"database",
	  "proof_commit_policy":"async_evidence",
	  "anchor_batch_id":"` + anchorBatchID + `",
	  "chain_anchor_id":"` + chainAnchorID + `",
	  "batch_root":"` + batchRoot + `",
	  "payload":{"batch_root":"` + batchRoot + `","record_count":3}
	}`)

	envelope, err := model.DecodeCanonicalEnvelope("dtp.audit.anchor", raw)
	if err != nil {
		t.Fatalf("decode envelope: %v", err)
	}

	request, err := service.NewDispatcher().BuildRequest(envelope)
	if err != nil {
		t.Fatalf("build request: %v", err)
	}

	submitter, err := adapterprovider.NewFabricGatewayProvider(cfg)
	if err != nil {
		t.Fatalf("new fabric gateway provider: %v", err)
	}
	defer submitter.Close()

	persist, err := store.New(ctx, cfg.DatabaseURL, cfg.ServiceName)
	if err != nil {
		t.Fatalf("new store: %v", err)
	}
	defer persist.Close()

	receipt, err := submitter.Submit(ctx, request)
	if err != nil {
		t.Fatalf("submit: %v", err)
	}
	if got, want := receipt.ReceiptStatus, "committed"; got != want {
		t.Fatalf("receipt status = %q, want %q", got, want)
	}
	if receipt.ProviderReference == "" {
		t.Fatalf("expected provider reference")
	}

	if err := persist.PersistSubmission(ctx, request, receipt); err != nil {
		t.Fatalf("persist submission: %v", err)
	}

	cwd, err := os.Getwd()
	if err != nil {
		t.Fatalf("getwd: %v", err)
	}
	repoRoot := filepath.Clean(filepath.Join(cwd, "../../../.."))
	queryOutput, err := exec.Command(
		"bash",
		"-lc",
		fmt.Sprintf(
			"cd %s && ./infra/fabric/query-anchor.sh anchor_batch %s %s",
			repoRoot,
			anchorBatchID,
			string(adapterprovider.SubmissionKindEvidenceBatchRoot),
		),
	).CombinedOutput()
	if err != nil {
		t.Fatalf("query anchor via script: %v\n%s", err, string(queryOutput))
	}

	var ledgerRecord map[string]any
	if err := json.Unmarshal(queryOutput, &ledgerRecord); err != nil {
		t.Fatalf("decode ledger record: %v", err)
	}
	if got, _ := ledgerRecord["reference_id"].(string); got != anchorBatchID {
		t.Fatalf("ledger reference_id = %q, want %q", got, anchorBatchID)
	}
	if got, _ := ledgerRecord["transaction_id"].(string); got != receipt.ProviderReference {
		t.Fatalf("ledger transaction_id = %q, want %q", got, receipt.ProviderReference)
	}

	var receiptStatus string
	var providerReference string
	if err := db.QueryRow(
		ctx,
		`SELECT receipt_status, provider_reference
		   FROM ops.external_fact_receipt
		  WHERE request_id = $1
		  ORDER BY received_at DESC
		  LIMIT 1`,
		requestID,
	).Scan(&receiptStatus, &providerReference); err != nil {
		t.Fatalf("query ops.external_fact_receipt: %v", err)
	}
	if receiptStatus != "committed" {
		t.Fatalf("ops.external_fact_receipt receipt_status = %q", receiptStatus)
	}
	if providerReference != receipt.ProviderReference {
		t.Fatalf("ops.external_fact_receipt provider_reference = %q, want %q", providerReference, receipt.ProviderReference)
	}

	var chainStatus string
	var txHash *string
	if err := db.QueryRow(
		ctx,
		`SELECT status, tx_hash
		   FROM chain.chain_anchor
		  WHERE chain_anchor_id = $1::text::uuid`,
		chainAnchorID,
	).Scan(&chainStatus, &txHash); err != nil {
		t.Fatalf("query chain.chain_anchor: %v", err)
	}
	if chainStatus != "submitted" {
		t.Fatalf("chain.chain_anchor status = %q, want submitted", chainStatus)
	}
	if txHash == nil || *txHash != receipt.ProviderReference {
		t.Fatalf("chain.chain_anchor tx_hash = %v, want %q", txHash, receipt.ProviderReference)
	}

	var auditCount int
	if err := db.QueryRow(
		ctx,
		`SELECT count(*)
		   FROM audit.audit_event
		  WHERE request_id = $1
		    AND action_name = 'fabric.adapter.submit'`,
		requestID,
	).Scan(&auditCount); err != nil {
		t.Fatalf("query audit.audit_event: %v", err)
	}
	if auditCount == 0 {
		t.Fatalf("expected audit.audit_event for request_id=%s", requestID)
	}

	var systemLogCount int
	if err := db.QueryRow(
		ctx,
		`SELECT count(*)
		   FROM ops.system_log
		  WHERE request_id = $1
		    AND message_text = 'fabric adapter accepted submit event'`,
		requestID,
	).Scan(&systemLogCount); err != nil {
		t.Fatalf("query ops.system_log: %v", err)
	}
	if systemLogCount == 0 {
		t.Fatalf("expected ops.system_log for request_id=%s", requestID)
	}
}
