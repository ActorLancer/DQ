package service

import (
	"context"
	"testing"
	"time"

	"datab.local/fabric-adapter/internal/model"
	"datab.local/fabric-adapter/internal/provider"
)

type fakePersister struct {
	envelope model.CanonicalEnvelope
	receipt  provider.SubmissionReceipt
	called   bool
}

func (fake *fakePersister) PersistSubmission(_ context.Context, envelope model.CanonicalEnvelope, receipt provider.SubmissionReceipt) error {
	fake.envelope = envelope
	fake.receipt = receipt
	fake.called = true
	return nil
}

type fakeProvider struct {
	receipt provider.SubmissionReceipt
	called  bool
}

func (fake *fakeProvider) Submit(_ context.Context, _ provider.SubmissionRequest) (provider.SubmissionReceipt, error) {
	fake.called = true
	return fake.receipt, nil
}

func TestProcessorProcessesCanonicalAnchorEvent(t *testing.T) {
	persister := &fakePersister{}
	submitter := &fakeProvider{
		receipt: provider.SubmissionReceipt{
			ProviderType:      "fabric_gateway",
			ProviderKey:       "mock-fabric-gateway",
			ProviderReference: "mock-tx-001",
			ReceiptStatus:     "submitted",
			OccurredAt:        time.Now().UTC(),
			ReceiptPayload:    map[string]any{"chain_id": "fabric-local"},
		},
	}

	processor := NewProcessor("fabric-adapter", persister, submitter, testLogger())
	raw := []byte(`{
	  "event_id":"evt-001",
	  "event_type":"audit.anchor_requested",
	  "event_version":1,
	  "producer_service":"platform-core.audit",
	  "aggregate_type":"audit.anchor_batch",
	  "aggregate_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "request_id":"req-001",
	  "trace_id":"trace-001",
	  "event_schema_version":"v1",
	  "authority_scope":"audit",
	  "source_of_truth":"database",
	  "proof_commit_policy":"async_evidence",
	  "anchor_batch_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "payload":{"batch_root":"root-001"}
	}`)

	if err := processor.ProcessMessage(context.Background(), "dtp.audit.anchor", raw); err != nil {
		t.Fatalf("ProcessMessage() error = %v", err)
	}

	if !submitter.called {
		t.Fatalf("expected provider to be called")
	}
	if !persister.called {
		t.Fatalf("expected persister to be called")
	}
	if got, want := persister.envelope.EventType, "audit.anchor_requested"; got != want {
		t.Fatalf("persisted EventType = %q, want %q", got, want)
	}
	if got, want := persister.receipt.ProviderReference, "mock-tx-001"; got != want {
		t.Fatalf("persisted ProviderReference = %q, want %q", got, want)
	}
}
