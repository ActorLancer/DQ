package store

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"

	"datab.local/fabric-event-listener/internal/model"
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

func (store *Store) ListPendingSubmissions(
	ctx context.Context,
	limit int,
) ([]model.PendingSubmission, error) {
	rows, err := store.pool.Query(
		ctx,
		`SELECT
		   external_fact_receipt_id::text,
		   COALESCE(order_id::text, ''),
		   ref_domain,
		   ref_type,
		   ref_id::text,
		   fact_type,
		   provider_type,
		   COALESCE(provider_key, ''),
		   COALESCE(provider_reference, ''),
		   receipt_status,
		   COALESCE(request_id, ''),
		   COALESCE(trace_id, ''),
		   receipt_payload::text,
		   metadata::text,
		   occurred_at,
		   received_at
		 FROM ops.external_fact_receipt
		 WHERE ref_domain = 'fabric'
		   AND fact_type IN ('fabric_submit_receipt', 'fabric_anchor_submit_receipt')
		   AND receipt_status = 'submitted'
		   AND COALESCE(metadata ->> 'listener_callback_event_id', '') = ''
		 ORDER BY received_at ASC, external_fact_receipt_id ASC
		 LIMIT $1`,
		&limit,
	)
	if err != nil {
		return nil, fmt.Errorf("query pending fabric receipts: %w", err)
	}
	defer rows.Close()

	items := make([]model.PendingSubmission, 0)
	for rows.Next() {
		var payloadText string
		var metadataText string
		var submission model.PendingSubmission
		if err := rows.Scan(
			&submission.SourceReceiptID,
			&submission.OrderID,
			&submission.RefDomain,
			&submission.RefType,
			&submission.RefID,
			&submission.FactType,
			&submission.ProviderType,
			&submission.ProviderKey,
			&submission.ProviderReference,
			&submission.ReceiptStatus,
			&submission.RequestID,
			&submission.TraceID,
			&payloadText,
			&metadataText,
			&submission.OccurredAt,
			&submission.ReceivedAt,
		); err != nil {
			return nil, fmt.Errorf("scan pending fabric receipt: %w", err)
		}

		payload, err := parseObject(payloadText)
		if err != nil {
			return nil, err
		}
		metadata, err := parseObject(metadataText)
		if err != nil {
			return nil, err
		}

		submission.ReceiptPayload = payload
		submission.Metadata = metadata
		submission.Topic = stringFromMaps("topic", metadata, payload)
		submission.SourceEventID = stringFromMaps("event_id", metadata, payload)
		submission.SourceEventType = stringFromMaps("event_type", metadata, payload)
		submission.SourceEventVersion = intFromMaps("event_version", metadata, payload)
		submission.AuthorityScope = stringFromMaps("authority_scope", metadata, payload)
		submission.SourceOfTruth = stringFromMaps("source_of_truth", metadata, payload)
		submission.ProofCommitPolicy = stringFromMaps("proof_commit_policy", metadata, payload)
		submission.SubmissionKind = stringFromMaps("submission_kind", metadata, payload)
		submission.ContractName = stringFromMaps("contract_name", metadata, payload)
		submission.TransactionName = stringFromMaps("transaction_name", metadata, payload)
		submission.SummaryType = stringFromMaps("summary_type", metadata, payload)
		submission.SummaryDigest = stringFromMaps("summary_digest", metadata, payload)
		submission.AnchorBatchID = stringFromMaps("anchor_batch_id", metadata, payload)
		submission.ChainAnchorID = stringFromMaps("chain_anchor_id", metadata, payload)
		submission.ReferenceType = stringFromMaps("reference_type", metadata, payload)
		submission.ReferenceID = stringFromMaps("reference_id", metadata, payload)
		submission.ChainID = stringFromMaps("chain_id", metadata, payload)

		items = append(items, submission)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate pending fabric receipts: %w", err)
	}

	return items, nil
}

func (store *Store) PersistCallback(
	ctx context.Context,
	observation model.CallbackObservation,
) error {
	tx, err := store.pool.BeginTx(ctx, pgx.TxOptions{})
	if err != nil {
		return fmt.Errorf("begin transaction: %w", err)
	}
	defer tx.Rollback(ctx)

	var existingCallbackID string
	err = tx.QueryRow(
		ctx,
		`SELECT COALESCE(metadata ->> 'listener_callback_event_id', '')
		 FROM ops.external_fact_receipt
		 WHERE external_fact_receipt_id = $1::text::uuid
		 FOR UPDATE`,
		observation.Source.SourceReceiptID,
	).Scan(&existingCallbackID)
	if err != nil {
		return fmt.Errorf("lock source receipt: %w", err)
	}
	if existingCallbackID == observation.EventID {
		return tx.Commit(ctx)
	}

	payloadJSON, err := marshalJSON(observation.Payload)
	if err != nil {
		return err
	}
	payloadHash := hashJSON(payloadJSON)

	refType := callbackRefType(observation.Source)
	refID := callbackRefID(observation.Source)
	confirmedAt := any(nil)
	if observation.ReceiptStatus == "confirmed" {
		confirmedAt = observation.OccurredAt
	}

	metadataJSON, err := marshalJSON(map[string]any{
		"topic":                "dtp.fabric.callbacks",
		"event_id":             observation.EventID,
		"event_type":           observation.EventType,
		"event_version":        observation.EventVersion,
		"producer_service":     store.serviceName,
		"provider_code":        observation.ProviderCode,
		"provider_request_id":  observation.ProviderRequestID,
		"callback_event_id":    observation.EventID,
		"provider_status":      observation.ProviderStatus,
		"provider_occurred_at": observation.OccurredAt.UTC().Format(time.RFC3339Nano),
		"payload_hash":         payloadHash,
		"source_receipt_id":    observation.Source.SourceReceiptID,
		"source_event_id":      observation.Source.SourceEventID,
		"source_event_type":    observation.Source.SourceEventType,
		"source_fact_type":     observation.Source.FactType,
		"source_request_id":    observation.Source.RequestID,
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
	})
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
		   confirmed_at,
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
		   $12::timestamptz,
		   $13,
		   $14,
		   $15::jsonb
		 )`,
		nullableString(observation.Source.OrderID),
		refType,
		refID,
		callbackFactType(observation.Source.FactType),
		observation.Source.ProviderType,
		nullableString(observation.Source.ProviderKey),
		nullableString(observation.TransactionID),
		observation.ReceiptStatus,
		string(payloadJSON),
		payloadHash,
		observation.OccurredAt,
		confirmedAt,
		nullableString(observation.Source.RequestID),
		nullableString(observation.Source.TraceID),
		string(metadataJSON),
	)
	if err != nil {
		return fmt.Errorf("insert callback external_fact_receipt: %w", err)
	}

	_, err = tx.Exec(
		ctx,
		`UPDATE ops.external_fact_receipt
		 SET metadata = metadata || jsonb_build_object(
		   'listener_callback_event_id', $2::text,
		   'listener_callback_status', $3::text,
		   'listener_callback_occurred_at', $4::text,
		   'listener_callback_topic', 'dtp.fabric.callbacks',
		   'listener_callback_payload_hash', $5::text,
		   'listener_service_name', $6::text
		 )
		 WHERE external_fact_receipt_id = $1::text::uuid`,
		observation.Source.SourceReceiptID,
		observation.EventID,
		observation.ReceiptStatus,
		observation.OccurredAt.UTC().Format(time.RFC3339Nano),
		payloadHash,
		store.serviceName,
	)
	if err != nil {
		return fmt.Errorf("mark source fabric receipt as callback-emitted: %w", err)
	}

	if chainAnchorID := strings.TrimSpace(observation.Source.ChainAnchorID); chainAnchorID != "" {
		if observation.ReceiptStatus == "confirmed" {
			_, err = tx.Exec(
				ctx,
				`UPDATE chain.chain_anchor
				 SET tx_hash = COALESCE($2::text, tx_hash),
				     status = 'anchored',
				     anchored_at = COALESCE(anchored_at, $3::timestamptz),
				     reconcile_status = 'matched',
				     last_reconciled_at = $3::timestamptz
				 WHERE chain_anchor_id = $1::text::uuid`,
				chainAnchorID,
				nullableString(observation.TransactionID),
				observation.OccurredAt,
			)
		} else {
			_, err = tx.Exec(
				ctx,
				`UPDATE chain.chain_anchor
				 SET tx_hash = COALESCE($2::text, tx_hash),
				     status = 'failed',
				     reconcile_status = 'pending_check',
				     last_reconciled_at = $3::timestamptz
				 WHERE chain_anchor_id = $1::text::uuid`,
				chainAnchorID,
				nullableString(observation.TransactionID),
				observation.OccurredAt,
			)
		}
		if err != nil {
			return fmt.Errorf("update chain.chain_anchor from callback: %w", err)
		}
	}

	if anchorBatchID := strings.TrimSpace(observation.Source.AnchorBatchID); anchorBatchID != "" {
		if observation.ReceiptStatus == "confirmed" {
			_, err = tx.Exec(
				ctx,
				`UPDATE audit.anchor_batch
				 SET status = 'anchored',
				     anchored_at = COALESCE(anchored_at, $2::timestamptz)
				 WHERE anchor_batch_id = $1::text::uuid`,
				anchorBatchID,
				observation.OccurredAt,
			)
		} else {
			_, err = tx.Exec(
				ctx,
				`UPDATE audit.anchor_batch
				 SET status = 'failed'
				 WHERE anchor_batch_id = $1::text::uuid`,
				anchorBatchID,
			)
		}
		if err != nil {
			return fmt.Errorf("update audit.anchor_batch from callback: %w", err)
		}
	}

	auditMetadata, err := marshalJSON(map[string]any{
		"event_id":             observation.EventID,
		"event_type":           observation.EventType,
		"provider_code":        observation.ProviderCode,
		"provider_request_id":  observation.ProviderRequestID,
		"provider_status":      observation.ProviderStatus,
		"callback_event_id":    observation.EventID,
		"source_receipt_id":    observation.Source.SourceReceiptID,
		"submission_kind":      observation.Source.SubmissionKind,
		"transaction_id":       observation.TransactionID,
		"block_number":         observation.BlockNumber,
		"chaincode_event_name": observation.ChaincodeEventName,
		"chain_anchor_id":      observation.Source.ChainAnchorID,
		"anchor_batch_id":      observation.Source.AnchorBatchID,
		"payload_hash":         payloadHash,
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
		   'fabric.event_listener.callback',
		   $3,
		   $4,
		   $5,
		   $6,
		   $7::jsonb
		 )`,
		refType,
		refID,
		observation.ReceiptStatus,
		nullableString(observation.Source.RequestID),
		nullableString(observation.Source.TraceID),
		nullableString(observation.TransactionID),
		string(auditMetadata),
	)
	if err != nil {
		return fmt.Errorf("insert audit.audit_event callback trail: %w", err)
	}

	logPayload, err := marshalJSON(map[string]any{
		"event_id":             observation.EventID,
		"event_type":           observation.EventType,
		"source_receipt_id":    observation.Source.SourceReceiptID,
		"provider_code":        observation.ProviderCode,
		"provider_request_id":  observation.ProviderRequestID,
		"provider_status":      observation.ProviderStatus,
		"payload_hash":         payloadHash,
		"transaction_id":       observation.TransactionID,
		"block_number":         observation.BlockNumber,
		"chaincode_event_name": observation.ChaincodeEventName,
		"chain_anchor_id":      observation.Source.ChainAnchorID,
		"anchor_batch_id":      observation.Source.AnchorBatchID,
		"submission_kind":      observation.Source.SubmissionKind,
		"callback_topic":       "dtp.fabric.callbacks",
	})
	if err != nil {
		return err
	}

	logLevel := "INFO"
	if observation.ReceiptStatus != "confirmed" {
		logLevel = "WARN"
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
		   $2,
		   $3,
		   $4,
		   'fabric event listener published callback',
		   $5::jsonb
		 )`,
		store.serviceName,
		logLevel,
		nullableString(observation.Source.RequestID),
		nullableString(observation.Source.TraceID),
		string(logPayload),
	)
	if err != nil {
		return fmt.Errorf("insert ops.system_log callback trail: %w", err)
	}

	if err := tx.Commit(ctx); err != nil {
		return fmt.Errorf("commit callback persistence: %w", err)
	}
	return nil
}

func parseObject(raw string) (map[string]any, error) {
	if strings.TrimSpace(raw) == "" {
		return map[string]any{}, nil
	}
	result := map[string]any{}
	if err := json.Unmarshal([]byte(raw), &result); err != nil {
		return nil, fmt.Errorf("decode json object: %w", err)
	}
	return result, nil
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
		case float64:
			return strconv.FormatInt(int64(typed), 10)
		case int:
			return strconv.Itoa(typed)
		case int64:
			return strconv.FormatInt(typed, 10)
		}
	}
	return ""
}

func intFromMaps(key string, maps ...map[string]any) int {
	for _, current := range maps {
		if current == nil {
			continue
		}
		value, ok := current[key]
		if !ok || value == nil {
			continue
		}
		switch typed := value.(type) {
		case int:
			return typed
		case int64:
			return int(typed)
		case float64:
			return int(typed)
		case string:
			parsed, err := strconv.Atoi(strings.TrimSpace(typed))
			if err == nil {
				return parsed
			}
		}
	}
	return 0
}

func marshalJSON(value any) ([]byte, error) {
	raw, err := json.Marshal(value)
	if err != nil {
		return nil, fmt.Errorf("marshal json: %w", err)
	}
	return raw, nil
}

func hashJSON(raw []byte) string {
	sum := sha256.Sum256(raw)
	return hex.EncodeToString(sum[:])
}

func callbackRefType(source model.PendingSubmission) string {
	if strings.TrimSpace(source.ChainAnchorID) != "" {
		return "chain_anchor"
	}
	if strings.TrimSpace(source.ReferenceType) != "" {
		return source.ReferenceType
	}
	return source.RefType
}

func callbackRefID(source model.PendingSubmission) string {
	if strings.TrimSpace(source.ChainAnchorID) != "" {
		return source.ChainAnchorID
	}
	if strings.TrimSpace(source.ReferenceID) != "" {
		return source.ReferenceID
	}
	return source.RefID
}

func callbackFactType(sourceFactType string) string {
	switch sourceFactType {
	case "fabric_anchor_submit_receipt":
		return "fabric_anchor_commit_receipt"
	default:
		return "fabric_commit_receipt"
	}
}

func nullableString(value string) any {
	if strings.TrimSpace(value) == "" {
		return nil
	}
	return value
}
