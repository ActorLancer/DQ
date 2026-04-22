package chaincode

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/hyperledger/fabric-contract-api-go/v2/contractapi"
)

type AnchorContract struct {
	contractapi.Contract
}

type AnchorRecord struct {
	LedgerKey       string `json:"ledger_key"`
	SubmissionKind  string `json:"submission_kind"`
	EventName       string `json:"event_name"`
	ReferenceType   string `json:"reference_type"`
	ReferenceID     string `json:"reference_id"`
	ChainID         string `json:"chain_id"`
	AnchorBatchID   string `json:"anchor_batch_id,omitempty"`
	ChainAnchorID   string `json:"chain_anchor_id,omitempty"`
	SummaryDigest   string `json:"summary_digest"`
	RequestID       string `json:"request_id,omitempty"`
	TraceID         string `json:"trace_id,omitempty"`
	TransactionID   string `json:"transaction_id"`
	SubmittedAt     string `json:"submitted_at"`
	ContractName    string `json:"contract_name"`
	TransactionName string `json:"transaction_name"`
}

func (contract *AnchorContract) Ping(_ contractapi.TransactionContextInterface) string {
	return "ok"
}

func (contract *AnchorContract) SubmitEvidenceBatchRoot(
	ctx contractapi.TransactionContextInterface,
	anchorBatchID string,
	chainAnchorID string,
	chainID string,
	summaryDigest string,
	requestID string,
	traceID string,
	referenceType string,
	referenceID string,
) (*AnchorRecord, error) {
	return contract.persistSubmission(
		ctx,
		"evidence_batch_root",
		"EvidenceBatchRootCommitted",
		"evidence_batch_root",
		"SubmitEvidenceBatchRoot",
		anchorBatchID,
		chainAnchorID,
		chainID,
		summaryDigest,
		requestID,
		traceID,
		referenceType,
		referenceID,
	)
}

func (contract *AnchorContract) SubmitOrderDigest(
	ctx contractapi.TransactionContextInterface,
	chainAnchorID string,
	chainID string,
	summaryDigest string,
	requestID string,
	traceID string,
	referenceType string,
	referenceID string,
) (*AnchorRecord, error) {
	return contract.persistSubmission(
		ctx,
		"order_summary",
		"OrderDigestCommitted",
		"order_digest",
		"SubmitOrderDigest",
		"",
		chainAnchorID,
		chainID,
		summaryDigest,
		requestID,
		traceID,
		referenceType,
		referenceID,
	)
}

func (contract *AnchorContract) SubmitAuthorizationDigest(
	ctx contractapi.TransactionContextInterface,
	chainAnchorID string,
	chainID string,
	summaryDigest string,
	requestID string,
	traceID string,
	referenceType string,
	referenceID string,
) (*AnchorRecord, error) {
	return contract.persistSubmission(
		ctx,
		"authorization_summary",
		"AuthorizationDigestCommitted",
		"authorization_digest",
		"SubmitAuthorizationDigest",
		"",
		chainAnchorID,
		chainID,
		summaryDigest,
		requestID,
		traceID,
		referenceType,
		referenceID,
	)
}

func (contract *AnchorContract) SubmitAcceptanceDigest(
	ctx contractapi.TransactionContextInterface,
	chainAnchorID string,
	chainID string,
	summaryDigest string,
	requestID string,
	traceID string,
	referenceType string,
	referenceID string,
) (*AnchorRecord, error) {
	return contract.persistSubmission(
		ctx,
		"acceptance_summary",
		"AcceptanceDigestCommitted",
		"acceptance_digest",
		"SubmitAcceptanceDigest",
		"",
		chainAnchorID,
		chainID,
		summaryDigest,
		requestID,
		traceID,
		referenceType,
		referenceID,
	)
}

func (contract *AnchorContract) GetAnchorByReference(
	ctx contractapi.TransactionContextInterface,
	referenceType string,
	referenceID string,
	submissionKind string,
) (*AnchorRecord, error) {
	if referenceType == "" || referenceID == "" || submissionKind == "" {
		return nil, fmt.Errorf("reference_type, reference_id and submission_kind are required")
	}

	recordJSON, err := ctx.GetStub().GetState(anchorKey(referenceType, referenceID, submissionKind))
	if err != nil {
		return nil, fmt.Errorf("get anchor state: %w", err)
	}
	if len(recordJSON) == 0 {
		return nil, fmt.Errorf("anchor not found for %s/%s/%s", referenceType, referenceID, submissionKind)
	}

	var record AnchorRecord
	if err := json.Unmarshal(recordJSON, &record); err != nil {
		return nil, fmt.Errorf("decode anchor state: %w", err)
	}
	return &record, nil
}

func (contract *AnchorContract) persistSubmission(
	ctx contractapi.TransactionContextInterface,
	submissionKind string,
	eventName string,
	contractName string,
	transactionName string,
	anchorBatchID string,
	chainAnchorID string,
	chainID string,
	summaryDigest string,
	requestID string,
	traceID string,
	referenceType string,
	referenceID string,
) (*AnchorRecord, error) {
	if referenceType == "" {
		return nil, fmt.Errorf("reference_type is required")
	}
	if referenceID == "" {
		return nil, fmt.Errorf("reference_id is required")
	}
	if chainID == "" {
		return nil, fmt.Errorf("chain_id is required")
	}
	if summaryDigest == "" {
		return nil, fmt.Errorf("summary_digest is required")
	}

	txTimestamp, err := ctx.GetStub().GetTxTimestamp()
	if err != nil {
		return nil, fmt.Errorf("get tx timestamp: %w", err)
	}
	submittedAt := time.Unix(txTimestamp.Seconds, int64(txTimestamp.Nanos)).UTC().Format(time.RFC3339Nano)

	record := AnchorRecord{
		LedgerKey:       anchorKey(referenceType, referenceID, submissionKind),
		SubmissionKind:  submissionKind,
		EventName:       eventName,
		ReferenceType:   referenceType,
		ReferenceID:     referenceID,
		ChainID:         chainID,
		AnchorBatchID:   anchorBatchID,
		ChainAnchorID:   chainAnchorID,
		SummaryDigest:   summaryDigest,
		RequestID:       requestID,
		TraceID:         traceID,
		TransactionID:   ctx.GetStub().GetTxID(),
		SubmittedAt:     submittedAt,
		ContractName:    contractName,
		TransactionName: transactionName,
	}

	recordJSON, err := json.Marshal(record)
	if err != nil {
		return nil, fmt.Errorf("marshal anchor record: %w", err)
	}
	if err := ctx.GetStub().PutState(record.LedgerKey, recordJSON); err != nil {
		return nil, fmt.Errorf("put anchor state: %w", err)
	}
	if err := ctx.GetStub().SetEvent(eventName, recordJSON); err != nil {
		return nil, fmt.Errorf("set chaincode event: %w", err)
	}
	return &record, nil
}

func anchorKey(referenceType string, referenceID string, submissionKind string) string {
	return fmt.Sprintf("anchor:%s:%s:%s", referenceType, referenceID, submissionKind)
}
