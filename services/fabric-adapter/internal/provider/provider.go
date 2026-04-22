package provider

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"time"

	"datab.local/fabric-adapter/internal/model"
)

type SubmissionRequest struct {
	Envelope model.CanonicalEnvelope
}

type SubmissionReceipt struct {
	ProviderType      string
	ProviderKey       string
	ProviderReference string
	ReceiptStatus     string
	OccurredAt        time.Time
	ReceiptPayload    map[string]any
}

type SubmissionProvider interface {
	Submit(ctx context.Context, request SubmissionRequest) (SubmissionReceipt, error)
}

type MockProvider struct {
	channelName   string
	chaincodeName string
}

func NewMock(channelName, chaincodeName string) *MockProvider {
	return &MockProvider{
		channelName:   channelName,
		chaincodeName: chaincodeName,
	}
}

func (provider *MockProvider) Submit(_ context.Context, request SubmissionRequest) (SubmissionReceipt, error) {
	chainID, ok := request.Envelope.FindString("chain_id")
	if !ok {
		chainID = "fabric-local"
	}

	txHash := mockTransactionHash(request.Envelope.EventID, request.Envelope.AggregateID)
	return SubmissionReceipt{
		ProviderType:      "fabric_gateway",
		ProviderKey:       "mock-fabric-gateway",
		ProviderReference: txHash,
		ReceiptStatus:     "submitted",
		OccurredAt:        time.Now().UTC(),
		ReceiptPayload: map[string]any{
			"mode":               "mock",
			"chain_id":           chainID,
			"channel_name":       provider.channelName,
			"chaincode_name":     provider.chaincodeName,
			"event_type":         request.Envelope.EventType,
			"aggregate_type":     request.Envelope.AggregateType,
			"aggregate_id":       request.Envelope.AggregateID,
			"request_id":         request.Envelope.RequestID,
			"trace_id":           request.Envelope.TraceID,
			"tx_hash":            txHash,
			"gateway_status":     "submitted",
			"business_payload":   request.Envelope.PayloadObject(),
			"flattened_payload":  request.Envelope.ExtraObject(),
			"submitter_service":  "fabric-adapter",
			"submission_summary": fmt.Sprintf("%s -> %s", request.Envelope.Topic, txHash),
		},
	}, nil
}

func mockTransactionHash(eventID, aggregateID string) string {
	sum := sha256.Sum256([]byte(eventID + ":" + aggregateID))
	return "mock-tx-" + hex.EncodeToString(sum[:8])
}
