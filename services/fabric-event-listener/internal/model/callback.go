package model

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"strings"
	"time"
)

type PendingSubmission struct {
	SourceReceiptID    string
	OrderID            string
	RefDomain          string
	RefType            string
	RefID              string
	FactType           string
	ProviderType       string
	ProviderKey        string
	ProviderReference  string
	ReceiptStatus      string
	RequestID          string
	TraceID            string
	Topic              string
	SourceEventID      string
	SourceEventType    string
	SourceEventVersion int
	AuthorityScope     string
	SourceOfTruth      string
	ProofCommitPolicy  string
	SubmissionKind     string
	ContractName       string
	TransactionName    string
	SummaryType        string
	SummaryDigest      string
	AnchorBatchID      string
	ChainAnchorID      string
	ReferenceType      string
	ReferenceID        string
	ChainID            string
	OccurredAt         *time.Time
	ReceivedAt         time.Time
	ReceiptPayload     map[string]any
	Metadata           map[string]any
}

type CallbackObservation struct {
	Source             PendingSubmission
	EventID            string
	EventType          string
	EventVersion       int
	OccurredAt         time.Time
	ProviderCode       string
	ProviderRequestID  string
	ProviderStatus     string
	ReceiptStatus      string
	TransactionID      string
	BlockNumber        int64
	ChaincodeEventName string
	CommitStatus       string
	Payload            map[string]any
}

func (observation CallbackObservation) AggregateType() string {
	if strings.TrimSpace(observation.Source.ChainAnchorID) != "" {
		return "chain.chain_anchor"
	}
	switch observation.Source.RefType {
	case "anchor_batch":
		return "audit.anchor_batch"
	case "chain_anchor":
		return "chain.chain_anchor"
	default:
		if observation.Source.RefType == "" {
			return "chain.chain_anchor"
		}
		return observation.Source.RefType
	}
}

func (observation CallbackObservation) AggregateID() string {
	if strings.TrimSpace(observation.Source.ChainAnchorID) != "" {
		return observation.Source.ChainAnchorID
	}
	if strings.TrimSpace(observation.Source.ReferenceID) != "" {
		return observation.Source.ReferenceID
	}
	return observation.Source.RefID
}

func (observation CallbackObservation) IdempotencyKey() string {
	return fmt.Sprintf(
		"fabric:callback:%s:%s",
		observation.Source.SourceReceiptID,
		observation.EventType,
	)
}

func (observation CallbackObservation) MarshalCanonicalEnvelope(serviceName string) ([]byte, []byte, error) {
	payloadHash, err := HashJSON(observation.Payload)
	if err != nil {
		return nil, nil, err
	}

	body := map[string]any{
		"event_id":             observation.EventID,
		"event_type":           observation.EventType,
		"event_version":        observation.EventVersion,
		"occurred_at":          observation.OccurredAt.UTC().Format(time.RFC3339Nano),
		"producer_service":     serviceName,
		"aggregate_type":       observation.AggregateType(),
		"aggregate_id":         observation.AggregateID(),
		"request_id":           observation.Source.RequestID,
		"trace_id":             observation.Source.TraceID,
		"idempotency_key":      observation.IdempotencyKey(),
		"event_schema_version": "v1",
		"authority_scope":      defaultString(observation.Source.AuthorityScope, "governance"),
		"source_of_truth":      defaultString(observation.Source.SourceOfTruth, "fabric"),
		"proof_commit_policy":  defaultString(observation.Source.ProofCommitPolicy, "async_evidence"),
		"payload":              observation.Payload,
		"provider_code":        observation.ProviderCode,
		"provider_request_id":  observation.ProviderRequestID,
		"callback_event_id":    observation.EventID,
		"provider_status":      observation.ProviderStatus,
		"provider_occurred_at": observation.OccurredAt.UTC().Format(time.RFC3339Nano),
		"payload_hash":         payloadHash,
		"source_receipt_id":    observation.Source.SourceReceiptID,
		"source_event_id":      observation.Source.SourceEventID,
		"source_event_type":    observation.Source.SourceEventType,
		"submission_kind":      observation.Source.SubmissionKind,
		"contract_name":        observation.Source.ContractName,
		"transaction_name":     observation.Source.TransactionName,
		"summary_type":         observation.Source.SummaryType,
		"summary_digest":       observation.Source.SummaryDigest,
		"anchor_batch_id":      observation.Source.AnchorBatchID,
		"chain_anchor_id":      observation.Source.ChainAnchorID,
		"reference_type":       observation.Source.ReferenceType,
		"reference_id":         observation.Source.ReferenceID,
		"chain_id":             observation.Source.ChainID,
		"transaction_id":       observation.TransactionID,
		"block_number":         observation.BlockNumber,
		"chaincode_event_name": observation.ChaincodeEventName,
		"commit_status":        observation.CommitStatus,
	}

	value, err := json.Marshal(body)
	if err != nil {
		return nil, nil, fmt.Errorf("marshal callback envelope: %w", err)
	}
	return []byte(observation.AggregateID()), value, nil
}

func HashJSON(value any) (string, error) {
	raw, err := json.Marshal(value)
	if err != nil {
		return "", fmt.Errorf("marshal payload for hash: %w", err)
	}
	sum := sha256.Sum256(raw)
	return hex.EncodeToString(sum[:]), nil
}

func defaultString(value string, fallback string) string {
	if strings.TrimSpace(value) == "" {
		return fallback
	}
	return value
}
