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
	Envelope        model.CanonicalEnvelope
	SubmissionKind  SubmissionKind
	ContractName    string
	TransactionName string
	ChainID         string
	SummaryType     string
	SummaryDigest   string
	AnchorBatchID   string
	ChainAnchorID   string
	ReferenceType   string
	ReferenceID     string
}

type SubmissionReceipt struct {
	ProviderType      string
	ProviderKey       string
	ProviderReference string
	ReceiptStatus     string
	OccurredAt        time.Time
	ReceiptPayload    map[string]any
}

type SubmissionKind string

const (
	SubmissionKindEvidenceBatchRoot SubmissionKind = "evidence_batch_root"
	SubmissionKindOrderSummary      SubmissionKind = "order_summary"
	SubmissionKindAuthorization     SubmissionKind = "authorization_summary"
	SubmissionKindAcceptance        SubmissionKind = "acceptance_summary"
)

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
	chainID := request.ChainID
	if chainID == "" {
		if resolved, ok := request.Envelope.FindString("chain_id"); ok {
			chainID = resolved
		} else {
			chainID = "fabric-local"
		}
	}

	txHash := mockTransactionHash(
		request.Envelope.EventID,
		request.Envelope.AggregateID,
		string(request.SubmissionKind),
	)
	return SubmissionReceipt{
		ProviderType:      "fabric_gateway",
		ProviderKey:       "mock-fabric-gateway",
		ProviderReference: txHash,
		ReceiptStatus:     "submitted",
		OccurredAt:        time.Now().UTC(),
		ReceiptPayload: map[string]any{
			"mode":              "mock",
			"chain_id":          chainID,
			"channel_name":      provider.channelName,
			"chaincode_name":    provider.chaincodeName,
			"event_type":        request.Envelope.EventType,
			"aggregate_type":    request.Envelope.AggregateType,
			"aggregate_id":      request.Envelope.AggregateID,
			"submission_kind":   string(request.SubmissionKind),
			"contract_name":     request.ContractName,
			"transaction_name":  request.TransactionName,
			"summary_type":      request.SummaryType,
			"summary_digest":    request.SummaryDigest,
			"anchor_batch_id":   request.AnchorBatchID,
			"chain_anchor_id":   request.ChainAnchorID,
			"reference_type":    request.ReferenceType,
			"reference_id":      request.ReferenceID,
			"request_id":        request.Envelope.RequestID,
			"trace_id":          request.Envelope.TraceID,
			"tx_hash":           txHash,
			"gateway_status":    "submitted",
			"business_payload":  request.Envelope.PayloadObject(),
			"flattened_payload": request.Envelope.ExtraObject(),
			"submitter_service": "fabric-adapter",
			"submission_summary": fmt.Sprintf(
				"%s/%s -> %s",
				request.Envelope.Topic,
				request.SubmissionKind,
				txHash,
			),
		},
	}, nil
}

func mockTransactionHash(eventID, aggregateID string, submissionKind string) string {
	sum := sha256.Sum256([]byte(eventID + ":" + aggregateID + ":" + submissionKind))
	return "mock-tx-" + hex.EncodeToString(sum[:8])
}
