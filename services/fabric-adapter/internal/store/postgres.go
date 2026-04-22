package store

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"

	"datab.local/fabric-adapter/internal/model"
	"datab.local/fabric-adapter/internal/provider"
)

type Store struct {
	pool        *pgxpool.Pool
	serviceName string
}

func New(ctx context.Context, databaseURL string, serviceName string) (*Store, error) {
	pool, err := pgxpool.New(ctx, databaseURL)
	if err != nil {
		return nil, fmt.Errorf("connect PostgreSQL: %w", err)
	}
	return &Store{
		pool:        pool,
		serviceName: serviceName,
	}, nil
}

func (store *Store) Close() {
	store.pool.Close()
}

func (store *Store) PersistSubmission(
	ctx context.Context,
	request provider.SubmissionRequest,
	receipt provider.SubmissionReceipt,
) error {
	envelope := request.Envelope
	tx, err := store.pool.BeginTx(ctx, pgx.TxOptions{})
	if err != nil {
		return fmt.Errorf("begin transaction: %w", err)
	}
	defer tx.Rollback(ctx)

	orderID, err := resolveOrderID(ctx, tx, request)
	if err != nil {
		return err
	}

	refType := normalizedRefType(envelope.AggregateType)
	receiptPayload, err := marshalJSON(receipt.ReceiptPayload)
	if err != nil {
		return err
	}
	receiptHash := hashJSON(receiptPayload)
	metadata := map[string]any{
		"topic":               envelope.Topic,
		"event_id":            envelope.EventID,
		"event_type":          envelope.EventType,
		"event_version":       envelope.EventVersion,
		"producer_service":    envelope.ProducerService,
		"aggregate_type":      envelope.AggregateType,
		"aggregate_id":        envelope.AggregateID,
		"authority_scope":     envelope.AuthorityScope,
		"source_of_truth":     envelope.SourceOfTruth,
		"proof_commit_policy": envelope.ProofCommitPolicy,
		"submission_kind":     string(request.SubmissionKind),
		"contract_name":       request.ContractName,
		"transaction_name":    request.TransactionName,
		"summary_type":        request.SummaryType,
		"summary_digest":      request.SummaryDigest,
		"anchor_batch_id":     request.AnchorBatchID,
		"chain_anchor_id":     request.ChainAnchorID,
		"reference_type":      request.ReferenceType,
		"reference_id":        request.ReferenceID,
		"flattened_payload":   envelope.ExtraObject(),
	}
	metadataJSON, err := marshalJSON(metadata)
	if err != nil {
		return err
	}

	_, err = tx.Exec(
		ctx,
		`INSERT INTO ops.external_fact_receipt (
		   order_id,
		   ref_domain,
		   ref_type,
		   ref_id,
		   fact_type,
		   provider_type,
		   provider_key,
		   provider_reference,
		   receipt_status,
		   receipt_payload,
		   receipt_hash,
		   occurred_at,
		   received_at,
		   request_id,
		   trace_id,
		   metadata
		 ) VALUES (
		   $1::text::uuid,
		   'fabric',
		   $2,
		   $3::text::uuid,
		   $4,
		   $5,
		   $6,
		   $7,
		   $8,
		   $9::jsonb,
		   $10,
		   $11::timestamptz,
		   now(),
		   $12,
		   $13,
		   $14::jsonb
		 )`,
		orderID,
		refType,
		envelope.AggregateID,
		factTypeForRequest(request),
		receipt.ProviderType,
		receipt.ProviderKey,
		receipt.ProviderReference,
		receipt.ReceiptStatus,
		string(receiptPayload),
		receiptHash,
		receipt.OccurredAt,
		nullableString(envelope.RequestID),
		nullableString(envelope.TraceID),
		string(metadataJSON),
	)
	if err != nil {
		return fmt.Errorf("insert ops.external_fact_receipt: %w", err)
	}

	if chainAnchorID := request.ChainAnchorID; strings.TrimSpace(chainAnchorID) != "" {
		_, err = tx.Exec(
			ctx,
			`UPDATE chain.chain_anchor
			 SET tx_hash = COALESCE($2, tx_hash),
			     status = CASE
			       WHEN status IN ('pending', 'retry_requested', 'failed') THEN 'submitted'
			       ELSE status
			     END,
			     reconcile_status = 'pending_check'
			 WHERE chain_anchor_id = $1::text::uuid`,
			chainAnchorID,
			receipt.ProviderReference,
		)
		if err != nil {
			return fmt.Errorf("update chain.chain_anchor: %w", err)
		}
	}

	auditMetadata, err := marshalJSON(map[string]any{
		"topic":              envelope.Topic,
		"event_id":           envelope.EventID,
		"event_type":         envelope.EventType,
		"aggregate_type":     envelope.AggregateType,
		"provider_type":      receipt.ProviderType,
		"provider_key":       receipt.ProviderKey,
		"provider_reference": receipt.ProviderReference,
		"receipt_status":     receipt.ReceiptStatus,
		"submission_kind":    string(request.SubmissionKind),
		"contract_name":      request.ContractName,
		"transaction_name":   request.TransactionName,
		"summary_type":       request.SummaryType,
		"summary_digest":     request.SummaryDigest,
	})
	if err != nil {
		return err
	}

	_, err = tx.Exec(
		ctx,
		`INSERT INTO audit.audit_event (
		   domain_name,
		   ref_type,
		   ref_id,
		   actor_type,
		   action_name,
		   result_code,
		   request_id,
		   trace_id,
		   tx_hash,
		   metadata
		 ) VALUES (
		   'fabric',
		   $1,
		   $2::text::uuid,
		   'service',
		   'fabric.adapter.submit',
		   $3,
		   $4,
		   $5,
		   $6,
		   $7::jsonb
		 )`,
		refType,
		envelope.AggregateID,
		receipt.ReceiptStatus,
		nullableString(envelope.RequestID),
		nullableString(envelope.TraceID),
		nullableString(receipt.ProviderReference),
		string(auditMetadata),
	)
	if err != nil {
		return fmt.Errorf("insert audit.audit_event: %w", err)
	}

	systemLogPayload, err := marshalJSON(map[string]any{
		"topic":               envelope.Topic,
		"event_id":            envelope.EventID,
		"event_type":          envelope.EventType,
		"aggregate_type":      envelope.AggregateType,
		"aggregate_id":        envelope.AggregateID,
		"provider_reference":  receipt.ProviderReference,
		"receipt_status":      receipt.ReceiptStatus,
		"submission_kind":     string(request.SubmissionKind),
		"contract_name":       request.ContractName,
		"transaction_name":    request.TransactionName,
		"summary_type":        request.SummaryType,
		"summary_digest":      request.SummaryDigest,
		"anchor_batch_id":     request.AnchorBatchID,
		"chain_anchor_id":     request.ChainAnchorID,
		"proof_commit_policy": envelope.ProofCommitPolicy,
	})
	if err != nil {
		return err
	}

	_, err = tx.Exec(
		ctx,
		`INSERT INTO ops.system_log (
		   service_name,
		   log_level,
		   request_id,
		   trace_id,
		   message_text,
		   structured_payload
		 ) VALUES (
		   $1,
		   'INFO',
		   $2,
		   $3,
		   'fabric adapter accepted submit event',
		   $4::jsonb
		 )`,
		store.serviceName,
		nullableString(envelope.RequestID),
		nullableString(envelope.TraceID),
		string(systemLogPayload),
	)
	if err != nil {
		return fmt.Errorf("insert ops.system_log: %w", err)
	}

	if err := tx.Commit(ctx); err != nil {
		return fmt.Errorf("commit transaction: %w", err)
	}
	return nil
}

func resolveOrderID(
	ctx context.Context,
	tx pgx.Tx,
	request provider.SubmissionRequest,
) (*string, error) {
	envelope := request.Envelope
	if orderID, ok := envelope.FindString("order_id"); ok {
		return &orderID, nil
	}
	if refType, ok := envelope.FindString("ref_type"); ok && refType == "order" {
		if refID, ok := envelope.FindString("ref_id"); ok {
			return &refID, nil
		}
	}

	switch envelope.AggregateType {
	case "order":
		var orderID string
		err := tx.QueryRow(
			ctx,
			`SELECT order_id::text
			 FROM trade.order_main
			 WHERE order_id = $1::text::uuid`,
			envelope.AggregateID,
		).Scan(&orderID)
		if err == nil {
			return &orderID, nil
		}
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, fmt.Errorf(
			"lookup trade.order_main for aggregate_id=%s: %w",
			envelope.AggregateID,
			err,
		)
	case "chain.chain_anchor", "chain_anchor":
		chainAnchorID := request.ChainAnchorID
		if strings.TrimSpace(chainAnchorID) == "" {
			chainAnchorID = envelope.AggregateID
		}

		var refType string
		var refID *string
		err := tx.QueryRow(
			ctx,
			`SELECT ref_type, ref_id::text
			 FROM chain.chain_anchor
			 WHERE chain_anchor_id = $1::text::uuid`,
			chainAnchorID,
		).Scan(&refType, &refID)
		if err == nil {
			if refType == "order" && refID != nil && strings.TrimSpace(*refID) != "" {
				return refID, nil
			}
			return nil, nil
		}
		if err == pgx.ErrNoRows {
			return nil, nil
		}
		return nil, fmt.Errorf(
			"lookup chain.chain_anchor for chain_anchor_id=%s: %w",
			chainAnchorID,
			err,
		)
	default:
		return nil, nil
	}
}

func factTypeForRequest(request provider.SubmissionRequest) string {
	switch request.SubmissionKind {
	case provider.SubmissionKindEvidenceBatchRoot:
		return "fabric_anchor_submit_receipt"
	default:
		return "fabric_submit_receipt"
	}
}

func normalizedRefType(aggregateType string) string {
	if parts := strings.SplitN(aggregateType, ".", 2); len(parts) == 2 {
		return parts[1]
	}
	return aggregateType
}

func hashJSON(payload []byte) string {
	sum := sha256.Sum256(payload)
	return hex.EncodeToString(sum[:])
}

func marshalJSON(value any) ([]byte, error) {
	encoded, err := json.Marshal(value)
	if err != nil {
		return nil, fmt.Errorf("marshal json: %w", err)
	}
	return encoded, nil
}

func nullableString(value string) *string {
	if strings.TrimSpace(value) == "" {
		return nil
	}
	return &value
}

func valueOrEmpty(envelope model.CanonicalEnvelope, key string) string {
	if value, ok := envelope.FindString(key); ok {
		return value
	}
	return ""
}
