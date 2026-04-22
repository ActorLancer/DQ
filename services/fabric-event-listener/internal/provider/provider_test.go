package provider

import (
	"context"
	"testing"

	"datab.local/fabric-event-listener/internal/model"
)

func TestMockProviderDefaultsToConfirmed(t *testing.T) {
	callbackProvider := NewMock("datab-channel", "datab-audit-anchor")
	observation, err := callbackProvider.Observe(context.Background(), model.PendingSubmission{
		SourceReceiptID:   "receipt-001",
		ProviderType:      "fabric_gateway",
		ProviderKey:       "mock-fabric-gateway",
		ProviderReference: "mock-tx-001",
		SubmissionKind:    "order_summary",
		ChainAnchorID:     "d6df4d6c-99fe-4ec0-a609-e14fd3c48dc7",
		ReferenceType:     "chain_anchor",
		ReferenceID:       "d6df4d6c-99fe-4ec0-a609-e14fd3c48dc7",
		ChainID:           "fabric-local",
	})
	if err != nil {
		t.Fatalf("Observe() error = %v", err)
	}

	if got, want := observation.EventType, "fabric.commit_confirmed"; got != want {
		t.Fatalf("EventType = %q, want %q", got, want)
	}
	if got, want := observation.ReceiptStatus, "confirmed"; got != want {
		t.Fatalf("ReceiptStatus = %q, want %q", got, want)
	}
	if got, want := observation.CommitStatus, "VALID"; got != want {
		t.Fatalf("CommitStatus = %q, want %q", got, want)
	}
}

func TestMockProviderHonorsFailedOverride(t *testing.T) {
	callbackProvider := NewMock("datab-channel", "datab-audit-anchor")
	observation, err := callbackProvider.Observe(context.Background(), model.PendingSubmission{
		SourceReceiptID:   "receipt-002",
		ProviderType:      "fabric_gateway",
		ProviderReference: "mock-tx-002",
		SubmissionKind:    "evidence_batch_root",
		AnchorBatchID:     "78e879c0-4ef0-4bdf-9a72-a46a389731f2",
		ChainAnchorID:     "e283eb78-f1c5-43cb-b40f-619d964d5244",
		ReferenceType:     "anchor_batch",
		ReferenceID:       "78e879c0-4ef0-4bdf-9a72-a46a389731f2",
		Metadata:          map[string]any{"mock_callback_status": "failed"},
	})
	if err != nil {
		t.Fatalf("Observe() error = %v", err)
	}

	if got, want := observation.EventType, "fabric.commit_failed"; got != want {
		t.Fatalf("EventType = %q, want %q", got, want)
	}
	if got, want := observation.ReceiptStatus, "failed"; got != want {
		t.Fatalf("ReceiptStatus = %q, want %q", got, want)
	}
	if got, want := observation.CommitStatus, "ERROR"; got != want {
		t.Fatalf("CommitStatus = %q, want %q", got, want)
	}
}
