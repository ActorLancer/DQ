package service

import (
	"context"
	"errors"
	"testing"

	"datab.local/fabric-event-listener/internal/model"
)

type fakeStore struct {
	pending      []model.PendingSubmission
	persisted    []model.CallbackObservation
	persistError error
}

func (fake *fakeStore) ListPendingSubmissions(_ context.Context, _ int) ([]model.PendingSubmission, error) {
	return fake.pending, nil
}

func (fake *fakeStore) PersistCallback(_ context.Context, observation model.CallbackObservation) error {
	if fake.persistError != nil {
		return fake.persistError
	}
	fake.persisted = append(fake.persisted, observation)
	return nil
}

type fakePublisher struct {
	keys   [][]byte
	values [][]byte
	err    error
}

func (fake *fakePublisher) Publish(_ context.Context, key []byte, value []byte) error {
	if fake.err != nil {
		return fake.err
	}
	fake.keys = append(fake.keys, key)
	fake.values = append(fake.values, value)
	return nil
}

type fakeCallbackProvider struct {
	observation model.CallbackObservation
	err         error
}

func (fake *fakeCallbackProvider) Observe(
	_ context.Context,
	submission model.PendingSubmission,
) (model.CallbackObservation, error) {
	if fake.err != nil {
		return model.CallbackObservation{}, fake.err
	}
	observation := fake.observation
	observation.Source = submission
	return observation, nil
}

func TestProcessorProcessesPendingSubmission(t *testing.T) {
	store := &fakeStore{
		pending: []model.PendingSubmission{{
			SourceReceiptID: "receipt-001",
			SubmissionKind:  "order_summary",
			ChainAnchorID:   "a6241cf1-9a22-48d4-9690-e83df3c9923f",
			ReferenceType:   "chain_anchor",
			ReferenceID:     "a6241cf1-9a22-48d4-9690-e83df3c9923f",
			AuthorityScope:  "governance",
			SourceOfTruth:   "fabric",
		}},
	}
	callbackProvider := &fakeCallbackProvider{
		observation: model.CallbackObservation{
			EventID:            "callback-001",
			EventType:          "fabric.commit_confirmed",
			EventVersion:       1,
			ProviderCode:       "mock-fabric-gateway",
			ProviderRequestID:  "mock-tx-001",
			ProviderStatus:     "confirmed",
			ReceiptStatus:      "confirmed",
			TransactionID:      "mock-tx-001",
			BlockNumber:        42,
			ChaincodeEventName: "OrderDigestCommitted",
			CommitStatus:       "VALID",
			Payload:            map[string]any{"callback_event_id": "callback-001"},
		},
	}
	publisher := &fakePublisher{}

	processor := NewProcessor(
		"fabric-event-listener",
		8,
		store,
		callbackProvider,
		publisher,
		testLogger(),
	)

	if err := processor.processBatch(context.Background()); err != nil {
		t.Fatalf("processBatch() error = %v", err)
	}
	if len(store.persisted) != 1 {
		t.Fatalf("persisted callbacks = %d, want 1", len(store.persisted))
	}
	if len(publisher.values) != 1 {
		t.Fatalf("published callbacks = %d, want 1", len(publisher.values))
	}
}

func TestProcessorReturnsPublishError(t *testing.T) {
	store := &fakeStore{
		pending: []model.PendingSubmission{{
			SourceReceiptID: "receipt-002",
			ChainAnchorID:   "0b3fa65f-d20e-4a03-aa0b-3ea622f39721",
		}},
	}
	callbackProvider := &fakeCallbackProvider{
		observation: model.CallbackObservation{
			EventID:            "callback-002",
			EventType:          "fabric.commit_failed",
			EventVersion:       1,
			ProviderCode:       "mock-fabric-gateway",
			ProviderRequestID:  "mock-tx-002",
			ProviderStatus:     "failed",
			ReceiptStatus:      "failed",
			TransactionID:      "mock-tx-002",
			BlockNumber:        7,
			ChaincodeEventName: "CommitFailed",
			CommitStatus:       "ERROR",
			Payload:            map[string]any{"callback_event_id": "callback-002"},
		},
	}
	publisher := &fakePublisher{err: errors.New("boom")}

	processor := NewProcessor(
		"fabric-event-listener",
		8,
		store,
		callbackProvider,
		publisher,
		testLogger(),
	)

	if err := processor.processBatch(context.Background()); err == nil {
		t.Fatalf("expected publish error")
	}
	if len(store.persisted) != 0 {
		t.Fatalf("persisted callbacks = %d, want 0", len(store.persisted))
	}
}
