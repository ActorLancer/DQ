package service

import (
	"context"
	"errors"
	"testing"
	"time"

	"datab.local/fabric-adapter/internal/model"
	"datab.local/fabric-adapter/internal/provider"
)

type fakeStore struct {
	request provider.SubmissionRequest
	receipt provider.SubmissionReceipt
	called  bool

	loadStateResult string
	loadStateExists bool
	beginProceed    bool
	beginErr        error
	updateCalls     []updateCall
}

type updateCall struct {
	resultCode string
	errorText  string
	metadata   map[string]any
}

func (fake *fakeStore) LoadProcessingState(_ context.Context, _ string) (string, bool, error) {
	return fake.loadStateResult, fake.loadStateExists, nil
}

func (fake *fakeStore) BeginProcessing(_ context.Context, _ model.CanonicalEnvelope, _ string, _ string) (bool, error) {
	if fake.beginErr != nil {
		return false, fake.beginErr
	}
	if !fake.beginProceed {
		return false, nil
	}
	return true, nil
}

func (fake *fakeStore) UpdateProcessingResult(
	_ context.Context,
	_ model.CanonicalEnvelope,
	resultCode string,
	errorText string,
	metadata map[string]any,
) error {
	fake.updateCalls = append(fake.updateCalls, updateCall{
		resultCode: resultCode,
		errorText:  errorText,
		metadata:   metadata,
	})
	return nil
}

func (fake *fakeStore) PersistSubmission(_ context.Context, request provider.SubmissionRequest, receipt provider.SubmissionReceipt) error {
	fake.request = request
	fake.receipt = receipt
	fake.called = true
	return nil
}

type fakeProvider struct {
	request provider.SubmissionRequest
	receipt provider.SubmissionReceipt
	called  bool
}

func (fake *fakeProvider) Submit(_ context.Context, request provider.SubmissionRequest) (provider.SubmissionReceipt, error) {
	fake.request = request
	fake.called = true
	return fake.receipt, nil
}

type fakeLock struct {
	acquired bool
	err      error
}

func (fake *fakeLock) Acquire(_ context.Context, eventID string) (string, bool, error) {
	if fake.err != nil {
		return "", false, fake.err
	}
	return "datab:v1:fabric-adapter:consumer-lock:" + eventID, fake.acquired, nil
}

func (fake *fakeLock) Release(_ context.Context, _ string) error {
	return nil
}

func TestProcessorProcessesCanonicalAnchorEvent(t *testing.T) {
	store := &fakeStore{beginProceed: true}
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

	processor := NewProcessor("fabric-adapter", store, submitter, &fakeLock{acquired: true}, testLogger())
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
	if !store.called {
		t.Fatalf("expected persister to be called")
	}
	if got, want := store.request.Envelope.EventType, "audit.anchor_requested"; got != want {
		t.Fatalf("persisted EventType = %q, want %q", got, want)
	}
	if got, want := store.receipt.ProviderReference, "mock-tx-001"; got != want {
		t.Fatalf("persisted ProviderReference = %q, want %q", got, want)
	}
	if got, want := submitter.request.SubmissionKind, provider.SubmissionKindEvidenceBatchRoot; got != want {
		t.Fatalf("SubmissionKind = %q, want %q", got, want)
	}
	if got, want := submitter.request.ContractName, "evidence_batch_root"; got != want {
		t.Fatalf("ContractName = %q, want %q", got, want)
	}
	if len(store.updateCalls) != 1 || store.updateCalls[0].resultCode != "processed" {
		t.Fatalf("expected processed update call, got %#v", store.updateCalls)
	}
}

func TestProcessorSkipsDuplicateEvent(t *testing.T) {
	store := &fakeStore{beginProceed: false}
	submitter := &fakeProvider{}
	processor := NewProcessor("fabric-adapter", store, submitter, &fakeLock{acquired: true}, testLogger())
	raw := []byte(`{
	  "event_id":"evt-002",
	  "event_type":"audit.anchor_requested",
	  "event_version":1,
	  "producer_service":"platform-core.audit",
	  "aggregate_type":"audit.anchor_batch",
	  "aggregate_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "anchor_batch_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "payload":{"batch_root":"root-002"}
	}`)

	if err := processor.ProcessMessage(context.Background(), "dtp.audit.anchor", raw); err != nil {
		t.Fatalf("ProcessMessage() error = %v", err)
	}
	if submitter.called {
		t.Fatalf("expected provider to be skipped")
	}
	if store.called {
		t.Fatalf("expected persister to be skipped")
	}
}

func TestProcessorSkipsInFlightEventWhenLockBusy(t *testing.T) {
	store := &fakeStore{loadStateResult: "processing", loadStateExists: true}
	submitter := &fakeProvider{}
	processor := NewProcessor("fabric-adapter", store, submitter, &fakeLock{acquired: false}, testLogger())
	raw := []byte(`{
	  "event_id":"evt-003",
	  "event_type":"fabric.proof_submit_requested",
	  "event_version":1,
	  "producer_service":"platform-core.integration",
	  "aggregate_type":"chain.chain_anchor",
	  "aggregate_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "summary_type":"order_summary",
	  "summary_digest":"root-003",
	  "payload":{"summary_type":"order_summary","summary_digest":"root-003"}
	}`)

	if err := processor.ProcessMessage(context.Background(), "dtp.fabric.requests", raw); err != nil {
		t.Fatalf("ProcessMessage() error = %v", err)
	}
	if submitter.called {
		t.Fatalf("expected provider to be skipped while lock is busy")
	}
}

func TestProcessorMarksFailureWhenProviderSubmitFails(t *testing.T) {
	store := &fakeStore{beginProceed: true}
	submitter := &fakeProvider{}
	processor := NewProcessor("fabric-adapter", store, submitter, &fakeLock{acquired: true}, testLogger())
	raw := []byte(`{
	  "event_id":"evt-004",
	  "event_type":"audit.anchor_requested",
	  "event_version":1,
	  "producer_service":"platform-core.audit",
	  "aggregate_type":"audit.anchor_batch",
	  "aggregate_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "anchor_batch_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "payload":{"batch_root":"root-004"}
	}`)
	submitter.receipt = provider.SubmissionReceipt{}
	submitter.called = false

	processor.provider = provider.SubmissionProvider(submissionProviderFunc(func(_ context.Context, _ provider.SubmissionRequest) (provider.SubmissionReceipt, error) {
		return provider.SubmissionReceipt{}, errors.New("provider down")
	}))

	err := processor.ProcessMessage(context.Background(), "dtp.audit.anchor", raw)
	if err == nil {
		t.Fatalf("expected provider failure")
	}
	if len(store.updateCalls) != 1 || store.updateCalls[0].resultCode != "failed" {
		t.Fatalf("expected failed update call, got %#v", store.updateCalls)
	}
}

type submissionProviderFunc func(context.Context, provider.SubmissionRequest) (provider.SubmissionReceipt, error)

func (fn submissionProviderFunc) Submit(ctx context.Context, request provider.SubmissionRequest) (provider.SubmissionReceipt, error) {
	return fn(ctx, request)
}
