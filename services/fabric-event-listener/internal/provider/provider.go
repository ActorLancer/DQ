package provider

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"
	"time"

	"datab.local/fabric-event-listener/internal/model"
)

type CallbackProvider interface {
	Observe(context.Context, model.PendingSubmission) (model.CallbackObservation, error)
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

func (provider *MockProvider) Observe(
	_ context.Context,
	pending model.PendingSubmission,
) (model.CallbackObservation, error) {
	callbackStatus := normalizeMockCallbackStatus(
		stringFromMaps("mock_callback_status", pending.Metadata, pending.ReceiptPayload),
	)
	txID := defaultString(pending.ProviderReference, deterministicID("mock-tx-", pending.SourceReceiptID))
	occurredAt := time.Now().UTC()
	eventType := "fabric.commit_confirmed"
	receiptStatus := "confirmed"
	providerStatus := "confirmed"
	commitStatus := "VALID"
	if callbackStatus == "failed" {
		eventType = "fabric.commit_failed"
		receiptStatus = "failed"
		providerStatus = "failed"
		commitStatus = "ERROR"
	}

	callbackEventID := deterministicID(
		"fabric-callback-",
		pending.SourceReceiptID+":"+eventType,
	)
	blockNumber := mockBlockNumber(callbackEventID)
	chainID := defaultString(pending.ChainID, stringFromMaps("chain_id", pending.Metadata, pending.ReceiptPayload))
	if chainID == "" {
		chainID = "fabric-local"
	}

	payload := map[string]any{
		"mode":                 "mock",
		"callback_source":      "fabric-event-listener",
		"callback_event_id":    callbackEventID,
		"provider_code":        defaultString(pending.ProviderKey, pending.ProviderType),
		"provider_request_id":  txID,
		"provider_status":      providerStatus,
		"provider_occurred_at": occurredAt.UTC().Format(time.RFC3339Nano),
		"event_type":           eventType,
		"chain_id":             chainID,
		"channel_name":         defaultString(provider.channelName, "datab-channel"),
		"chaincode_name":       defaultString(provider.chaincodeName, "datab-audit-anchor"),
		"transaction_id":       txID,
		"block_number":         blockNumber,
		"chaincode_event_name": mockChaincodeEventName(
			pending.SubmissionKind,
			receiptStatus,
		),
		"commit_status":     commitStatus,
		"source_receipt_id": pending.SourceReceiptID,
		"source_event_id":   pending.SourceEventID,
		"source_event_type": pending.SourceEventType,
		"request_id":        pending.RequestID,
		"trace_id":          pending.TraceID,
		"submission_kind":   pending.SubmissionKind,
		"contract_name":     pending.ContractName,
		"transaction_name":  pending.TransactionName,
		"summary_type":      pending.SummaryType,
		"summary_digest":    pending.SummaryDigest,
		"anchor_batch_id":   pending.AnchorBatchID,
		"chain_anchor_id":   pending.ChainAnchorID,
		"reference_type":    pending.ReferenceType,
		"reference_id":      pending.ReferenceID,
		"source_payload":    pending.ReceiptPayload,
	}

	return model.CallbackObservation{
		Source:             pending,
		EventID:            callbackEventID,
		EventType:          eventType,
		EventVersion:       1,
		OccurredAt:         occurredAt,
		ProviderCode:       defaultString(pending.ProviderKey, pending.ProviderType),
		ProviderRequestID:  txID,
		ProviderStatus:     providerStatus,
		ReceiptStatus:      receiptStatus,
		TransactionID:      txID,
		BlockNumber:        blockNumber,
		ChaincodeEventName: payload["chaincode_event_name"].(string),
		CommitStatus:       commitStatus,
		Payload:            payload,
	}, nil
}

func normalizeMockCallbackStatus(status string) string {
	switch strings.ToLower(strings.TrimSpace(status)) {
	case "failed", "fail", "error":
		return "failed"
	default:
		return "confirmed"
	}
}

func stringFromMaps(key string, maps ...map[string]any) string {
	for _, current := range maps {
		if current == nil {
			continue
		}
		value, ok := current[key]
		if !ok || value == nil {
			continue
		}
		switch typed := value.(type) {
		case string:
			if strings.TrimSpace(typed) != "" {
				return typed
			}
		case fmt.Stringer:
			text := typed.String()
			if strings.TrimSpace(text) != "" {
				return text
			}
		case float64:
			return fmt.Sprintf("%.0f", typed)
		case int:
			return fmt.Sprintf("%d", typed)
		case int64:
			return fmt.Sprintf("%d", typed)
		case bool:
			if typed {
				return "true"
			}
			return "false"
		}
	}
	return ""
}

func deterministicID(prefix string, input string) string {
	sum := sha256.Sum256([]byte(input))
	return prefix + hex.EncodeToString(sum[:8])
}

func mockBlockNumber(seed string) int64 {
	sum := sha256.Sum256([]byte(seed))
	var number uint64
	for _, piece := range sum[:8] {
		number = (number << 8) | uint64(piece)
	}
	return int64(number%900000 + 1000)
}

func mockChaincodeEventName(submissionKind string, receiptStatus string) string {
	if receiptStatus == "failed" {
		return "CommitFailed"
	}
	switch submissionKind {
	case "evidence_batch_root":
		return "EvidenceBatchRootCommitted"
	case "authorization_summary":
		return "AuthorizationDigestCommitted"
	case "acceptance_summary":
		return "AcceptanceDigestCommitted"
	default:
		return "OrderDigestCommitted"
	}
}

func defaultString(value string, fallback string) string {
	if strings.TrimSpace(value) == "" {
		return fallback
	}
	return value
}
