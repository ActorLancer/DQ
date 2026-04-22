package store

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"

	"datab.local/fabric-ca-admin/internal/model"
	"datab.local/fabric-ca-admin/internal/provider"
)

const (
	messageIssueIdentity       = "fabric ca admin issued identity"
	messageRevokeIdentity      = "fabric ca admin revoked identity"
	messageRevokeCertificate   = "fabric ca admin revoked certificate"
	factTypeIssueReceipt       = "certificate_issue_receipt"
	factTypeRevokeReceipt      = "certificate_revocation_receipt"
	eventTypeCertificateIssue  = "ca.certificate_issued"
	eventTypeCertificateRevoke = "ca.certificate_revoked"
)

type ActionError struct {
	StatusCode int
	Code       string
	Message    string
}

func (err *ActionError) Error() string {
	return err.Message
}

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

func (store *Store) Ping(ctx context.Context) error {
	return store.pool.Ping(ctx)
}

func (store *Store) IssueIdentity(
	ctx context.Context,
	identityID string,
	actor model.ActorContext,
	ca provider.CAProvider,
) (model.ActionResult, error) {
	tx, err := store.pool.BeginTx(ctx, pgx.TxOptions{})
	if err != nil {
		return model.ActionResult{}, fmt.Errorf("begin transaction: %w", err)
	}
	defer tx.Rollback(ctx)

	binding, currentCertificate, err := loadIdentityBindingForUpdate(ctx, tx, identityID)
	if err != nil {
		return model.ActionResult{}, err
	}

	if binding.Status != "approved" && binding.Status != "issued" {
		return model.ActionResult{}, &ActionError{
			StatusCode: http.StatusConflict,
			Code:       "FABRIC_IDENTITY_NOT_APPROVED",
			Message:    "fabric identity must be in approved state before issue",
		}
	}

	if binding.Status == "issued" && currentCertificate != nil && currentCertificate.Status == "active" {
		return model.ActionResult{
			TargetID:      binding.FabricIdentityBindingID,
			Status:        "issued",
			CertificateID: currentCertificate.CertificateID,
		}, tx.Commit(ctx)
	}

	receipt, err := ca.IssueIdentity(ctx, binding)
	if err != nil {
		return model.ActionResult{}, &ActionError{
			StatusCode: http.StatusBadGateway,
			Code:       "FABRIC_CA_UPSTREAM_FAILED",
			Message:    fmt.Sprintf("issue fabric identity via CA provider: %v", err),
		}
	}

	certificateID, err := store.insertCertificateRecord(ctx, tx, binding, receipt)
	if err != nil {
		return model.ActionResult{}, err
	}

	if _, err := tx.Exec(
		ctx,
		`UPDATE iam.fabric_identity_binding
		 SET certificate_id = $2::text::uuid,
		     msp_id = $3,
		     affiliation = $4,
		     status = 'issued',
		     issued_at = $5,
		     revoked_at = NULL,
		     updated_at = now()
		 WHERE fabric_identity_binding_id = $1::text::uuid`,
		binding.FabricIdentityBindingID,
		certificateID,
		binding.MSPID,
		nullableString(binding.Affiliation),
		receipt.OccurredAt,
	); err != nil {
		return model.ActionResult{}, fmt.Errorf("update iam.fabric_identity_binding for issue: %w", err)
	}

	externalFactReceiptID, err := store.insertExternalFactReceipt(
		ctx,
		tx,
		"fabric",
		"fabric_identity_binding",
		binding.FabricIdentityBindingID,
		factTypeIssueReceipt,
		eventTypeCertificateIssue,
		"confirmed",
		binding.RegistryName,
		receipt.ProviderRequestID,
		actor,
		receipt.OccurredAt,
		receipt.Payload,
		map[string]any{
			"certificate_id":        certificateID,
			"certificate_digest":    receipt.CertificateDigest,
			"serial_number":         receipt.SerialNumber,
			"msp_id":                binding.MSPID,
			"affiliation":           binding.Affiliation,
			"enrollment_id":         binding.EnrollmentID,
			"identity_type":         binding.IdentityType,
			"fabric_ca_registry_id": binding.FabricCARegistryID,
		},
	)
	if err != nil {
		return model.ActionResult{}, err
	}

	if err := store.insertSystemLog(
		ctx,
		tx,
		"info",
		messageIssueIdentity,
		actor,
		map[string]any{
			"target_id":                binding.FabricIdentityBindingID,
			"certificate_id":           certificateID,
			"external_fact_receipt_id": externalFactReceiptID,
			"provider_reference":       receipt.ProviderRequestID,
			"event_type":               eventTypeCertificateIssue,
		},
	); err != nil {
		return model.ActionResult{}, err
	}

	if err := tx.Commit(ctx); err != nil {
		return model.ActionResult{}, err
	}

	return model.ActionResult{
		TargetID:              binding.FabricIdentityBindingID,
		Status:                "issued",
		CertificateID:         certificateID,
		ExternalFactReceiptID: externalFactReceiptID,
		ProviderReference:     receipt.ProviderRequestID,
	}, nil
}

func (store *Store) RevokeIdentity(
	ctx context.Context,
	identityID string,
	actor model.ActorContext,
	ca provider.CAProvider,
) (model.ActionResult, error) {
	tx, err := store.pool.BeginTx(ctx, pgx.TxOptions{})
	if err != nil {
		return model.ActionResult{}, fmt.Errorf("begin transaction: %w", err)
	}
	defer tx.Rollback(ctx)

	binding, currentCertificate, err := loadIdentityBindingForUpdate(ctx, tx, identityID)
	if err != nil {
		return model.ActionResult{}, err
	}

	if binding.Status != "issued" && binding.Status != "revoked" {
		return model.ActionResult{}, &ActionError{
			StatusCode: http.StatusConflict,
			Code:       "FABRIC_IDENTITY_NOT_ACTIVE",
			Message:    "fabric identity must be in issued state before revoke",
		}
	}

	if binding.Status == "revoked" {
		return model.ActionResult{
			TargetID:      binding.FabricIdentityBindingID,
			Status:        "revoked",
			CertificateID: binding.CertificateID,
		}, tx.Commit(ctx)
	}

	receipt, err := ca.RevokeIdentity(ctx, binding, currentCertificate)
	if err != nil {
		return model.ActionResult{}, &ActionError{
			StatusCode: http.StatusBadGateway,
			Code:       "FABRIC_CA_UPSTREAM_FAILED",
			Message:    fmt.Sprintf("revoke fabric identity via CA provider: %v", err),
		}
	}

	if currentCertificate != nil && currentCertificate.CertificateID != "" {
		if _, err := tx.Exec(
			ctx,
			`UPDATE iam.certificate_record
			 SET status = 'revoked',
			     updated_at = now()
			 WHERE certificate_id = $1::text::uuid`,
			currentCertificate.CertificateID,
		); err != nil {
			return model.ActionResult{}, fmt.Errorf("update iam.certificate_record for identity revoke: %w", err)
		}

		if err := insertRevocationRecord(ctx, tx, currentCertificate.CertificateID, actor.ActorUserID, receipt.RevokeReason, "fabric_ca_admin", receipt.Payload); err != nil {
			return model.ActionResult{}, err
		}
	}

	if _, err := tx.Exec(
		ctx,
		`UPDATE iam.fabric_identity_binding
		 SET status = 'revoked',
		     revoked_at = $2,
		     updated_at = now()
		 WHERE fabric_identity_binding_id = $1::text::uuid`,
		binding.FabricIdentityBindingID,
		receipt.OccurredAt,
	); err != nil {
		return model.ActionResult{}, fmt.Errorf("update iam.fabric_identity_binding for revoke: %w", err)
	}

	externalFactReceiptID, err := store.insertExternalFactReceipt(
		ctx,
		tx,
		"fabric",
		"fabric_identity_binding",
		binding.FabricIdentityBindingID,
		factTypeRevokeReceipt,
		eventTypeCertificateRevoke,
		"confirmed",
		binding.RegistryName,
		receipt.ProviderRequestID,
		actor,
		receipt.OccurredAt,
		receipt.Payload,
		map[string]any{
			"certificate_id":        binding.CertificateID,
			"msp_id":                binding.MSPID,
			"affiliation":           binding.Affiliation,
			"enrollment_id":         binding.EnrollmentID,
			"identity_type":         binding.IdentityType,
			"fabric_ca_registry_id": binding.FabricCARegistryID,
		},
	)
	if err != nil {
		return model.ActionResult{}, err
	}

	if err := store.insertSystemLog(
		ctx,
		tx,
		"warn",
		messageRevokeIdentity,
		actor,
		map[string]any{
			"target_id":                binding.FabricIdentityBindingID,
			"certificate_id":           binding.CertificateID,
			"external_fact_receipt_id": externalFactReceiptID,
			"provider_reference":       receipt.ProviderRequestID,
			"event_type":               eventTypeCertificateRevoke,
		},
	); err != nil {
		return model.ActionResult{}, err
	}

	if err := tx.Commit(ctx); err != nil {
		return model.ActionResult{}, err
	}

	return model.ActionResult{
		TargetID:              binding.FabricIdentityBindingID,
		Status:                "revoked",
		CertificateID:         binding.CertificateID,
		ExternalFactReceiptID: externalFactReceiptID,
		ProviderReference:     receipt.ProviderRequestID,
	}, nil
}

func (store *Store) RevokeCertificate(
	ctx context.Context,
	certificateID string,
	actor model.ActorContext,
	ca provider.CAProvider,
) (model.ActionResult, error) {
	tx, err := store.pool.BeginTx(ctx, pgx.TxOptions{})
	if err != nil {
		return model.ActionResult{}, fmt.Errorf("begin transaction: %w", err)
	}
	defer tx.Rollback(ctx)

	certificate, binding, err := loadCertificateForUpdate(ctx, tx, certificateID)
	if err != nil {
		return model.ActionResult{}, err
	}

	if certificate.Status != "active" && certificate.Status != "revoked" {
		return model.ActionResult{}, &ActionError{
			StatusCode: http.StatusConflict,
			Code:       "CERTIFICATE_NOT_ACTIVE",
			Message:    "certificate must be active before revoke",
		}
	}

	if certificate.Status == "revoked" {
		return model.ActionResult{
			TargetID:      certificate.CertificateID,
			Status:        "revoked",
			CertificateID: certificate.CertificateID,
		}, tx.Commit(ctx)
	}

	receipt, err := ca.RevokeCertificate(ctx, certificate, binding)
	if err != nil {
		return model.ActionResult{}, &ActionError{
			StatusCode: http.StatusBadGateway,
			Code:       "FABRIC_CA_UPSTREAM_FAILED",
			Message:    fmt.Sprintf("revoke certificate via CA provider: %v", err),
		}
	}

	if _, err := tx.Exec(
		ctx,
		`UPDATE iam.certificate_record
		 SET status = 'revoked',
		     updated_at = now()
		 WHERE certificate_id = $1::text::uuid`,
		certificate.CertificateID,
	); err != nil {
		return model.ActionResult{}, fmt.Errorf("update iam.certificate_record for certificate revoke: %w", err)
	}

	if binding != nil {
		if _, err := tx.Exec(
			ctx,
			`UPDATE iam.fabric_identity_binding
			 SET status = 'revoked',
			     revoked_at = $2,
			     updated_at = now()
			 WHERE fabric_identity_binding_id = $1::text::uuid`,
			binding.FabricIdentityBindingID,
			receipt.OccurredAt,
		); err != nil {
			return model.ActionResult{}, fmt.Errorf("update iam.fabric_identity_binding for certificate revoke: %w", err)
		}
	}

	if err := insertRevocationRecord(ctx, tx, certificate.CertificateID, actor.ActorUserID, receipt.RevokeReason, "fabric_ca_admin", receipt.Payload); err != nil {
		return model.ActionResult{}, err
	}

	externalFactReceiptID, err := store.insertExternalFactReceipt(
		ctx,
		tx,
		"fabric",
		"certificate_record",
		certificate.CertificateID,
		factTypeRevokeReceipt,
		eventTypeCertificateRevoke,
		"confirmed",
		"",
		receipt.ProviderRequestID,
		actor,
		receipt.OccurredAt,
		receipt.Payload,
		map[string]any{
			"certificate_digest": certificate.CertificateDigest,
			"serial_number":      certificate.SerialNumber,
			"linked_identity_id": linkedBindingID(binding),
		},
	)
	if err != nil {
		return model.ActionResult{}, err
	}

	if err := store.insertSystemLog(
		ctx,
		tx,
		"warn",
		messageRevokeCertificate,
		actor,
		map[string]any{
			"target_id":                certificate.CertificateID,
			"linked_identity_id":       linkedBindingID(binding),
			"external_fact_receipt_id": externalFactReceiptID,
			"provider_reference":       receipt.ProviderRequestID,
			"event_type":               eventTypeCertificateRevoke,
		},
	); err != nil {
		return model.ActionResult{}, err
	}

	if err := tx.Commit(ctx); err != nil {
		return model.ActionResult{}, err
	}

	return model.ActionResult{
		TargetID:              certificate.CertificateID,
		Status:                "revoked",
		CertificateID:         certificate.CertificateID,
		ExternalFactReceiptID: externalFactReceiptID,
		ProviderReference:     receipt.ProviderRequestID,
	}, nil
}

func (store *Store) insertCertificateRecord(
	ctx context.Context,
	tx pgx.Tx,
	binding model.IdentityBinding,
	receipt model.ProviderIssueReceipt,
) (string, error) {
	var certificateID string
	metadata, err := marshalJSON(map[string]any{
		"event_type":          eventTypeCertificateIssue,
		"provider_request_id": receipt.ProviderRequestID,
		"msp_id":              binding.MSPID,
		"affiliation":         binding.Affiliation,
		"enrollment_id":       binding.EnrollmentID,
		"identity_type":       binding.IdentityType,
	})
	if err != nil {
		return "", err
	}

	if err := tx.QueryRow(
		ctx,
		`INSERT INTO iam.certificate_record (
		   fabric_ca_registry_id,
		   certificate_scope,
		   serial_number,
		   certificate_digest,
		   subject_dn,
		   issuer_dn,
		   key_ref,
		   not_before,
		   not_after,
		   status,
		   metadata
		 ) VALUES (
		   $1::text::uuid,
		   'fabric_identity',
		   $2,
		   $3,
		   $4,
		   $5,
		   $6,
		   $7,
		   $8,
		   'active',
		   $9::jsonb
		 )
		 RETURNING certificate_id::text`,
		binding.FabricCARegistryID,
		receipt.SerialNumber,
		receipt.CertificateDigest,
		receipt.SubjectDN,
		receipt.IssuerDN,
		receipt.KeyRef,
		receipt.NotBefore,
		receipt.NotAfter,
		string(metadata),
	).Scan(&certificateID); err != nil {
		return "", fmt.Errorf("insert iam.certificate_record: %w", err)
	}

	return certificateID, nil
}

func (store *Store) insertExternalFactReceipt(
	ctx context.Context,
	tx pgx.Tx,
	refDomain string,
	refType string,
	refID string,
	factType string,
	eventType string,
	receiptStatus string,
	providerKey string,
	providerReference string,
	actor model.ActorContext,
	occurredAt interface{},
	payload map[string]any,
	extraMetadata map[string]any,
) (string, error) {
	payloadJSON, err := marshalJSON(payload)
	if err != nil {
		return "", err
	}

	metadataMap := map[string]any{
		"event_type":           eventType,
		"actor_role":           actor.ActorRole,
		"actor_user_id":        actor.ActorUserID,
		"step_up_challenge_id": actor.StepUpChallengeID,
		"permission_code":      actor.PermissionCode,
		"provider_reference":   providerReference,
		"service_name":         store.serviceName,
	}
	for key, value := range extraMetadata {
		metadataMap[key] = value
	}
	metadataJSON, err := marshalJSON(metadataMap)
	if err != nil {
		return "", err
	}

	receiptHash := hashJSON(payloadJSON)
	var externalFactReceiptID string
	if err := tx.QueryRow(
		ctx,
		`INSERT INTO ops.external_fact_receipt (
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
		   confirmed_at,
		   request_id,
		   trace_id,
		   metadata
		 ) VALUES (
		   $1,
		   $2,
		   $3::text::uuid,
		   $4,
		   'fabric_ca',
		   NULLIF($5, ''),
		   NULLIF($6, ''),
		   $7,
		   $8::jsonb,
		   $9,
		   $10,
		   $10,
		   NULLIF($11, ''),
		   NULLIF($12, ''),
		   $13::jsonb
		 )
		 RETURNING external_fact_receipt_id::text`,
		refDomain,
		refType,
		refID,
		factType,
		providerKey,
		providerReference,
		receiptStatus,
		string(payloadJSON),
		receiptHash,
		occurredAt,
		actor.RequestID,
		actor.TraceID,
		string(metadataJSON),
	).Scan(&externalFactReceiptID); err != nil {
		return "", fmt.Errorf("insert ops.external_fact_receipt: %w", err)
	}

	return externalFactReceiptID, nil
}

func (store *Store) insertSystemLog(
	ctx context.Context,
	tx pgx.Tx,
	logLevel string,
	messageText string,
	actor model.ActorContext,
	payload map[string]any,
) error {
	payload["actor_role"] = actor.ActorRole
	payload["actor_user_id"] = actor.ActorUserID
	payload["step_up_challenge_id"] = actor.StepUpChallengeID
	payload["permission_code"] = actor.PermissionCode
	payloadJSON, err := marshalJSON(payload)
	if err != nil {
		return err
	}

	if _, err := tx.Exec(
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
		   NULLIF($3, ''),
		   NULLIF($4, ''),
		   $5,
		   $6::jsonb
		 )`,
		store.serviceName,
		logLevel,
		actor.RequestID,
		actor.TraceID,
		messageText,
		string(payloadJSON),
	); err != nil {
		return fmt.Errorf("insert ops.system_log: %w", err)
	}
	return nil
}

func loadIdentityBindingForUpdate(
	ctx context.Context,
	tx pgx.Tx,
	identityID string,
) (model.IdentityBinding, *model.CertificateRecord, error) {
	row := tx.QueryRow(
		ctx,
		`SELECT
		   fib.fabric_identity_binding_id::text,
		   fib.fabric_ca_registry_id::text,
		   COALESCE(fib.certificate_id::text, ''),
		   fib.msp_id,
		   COALESCE(fib.affiliation, ''),
		   fib.enrollment_id,
		   fib.identity_type,
		   fib.status,
		   fcr.registry_name,
		   fcr.ca_name,
		   COALESCE(fcr.ca_url, ''),
		   fcr.ca_type,
		   COALESCE(fcr.enrollment_profile, ''),
		   fib.attrs_snapshot::text,
		   COALESCE(cr.certificate_id::text, ''),
		   COALESCE(cr.status, ''),
		   COALESCE(cr.serial_number, ''),
		   COALESCE(cr.certificate_digest, ''),
		   COALESCE(cr.subject_dn, ''),
		   COALESCE(cr.issuer_dn, ''),
		   COALESCE(cr.key_ref, '')
		 FROM iam.fabric_identity_binding fib
		 JOIN iam.fabric_ca_registry fcr
		   ON fcr.fabric_ca_registry_id = fib.fabric_ca_registry_id
		 LEFT JOIN iam.certificate_record cr
		   ON cr.certificate_id = fib.certificate_id
		 WHERE fib.fabric_identity_binding_id = $1::text::uuid
		 FOR UPDATE OF fib`,
		identityID,
	)

	var attrsSnapshotText string
	binding := model.IdentityBinding{}
	certificate := model.CertificateRecord{}
	if err := row.Scan(
		&binding.FabricIdentityBindingID,
		&binding.FabricCARegistryID,
		&binding.CertificateID,
		&binding.MSPID,
		&binding.Affiliation,
		&binding.EnrollmentID,
		&binding.IdentityType,
		&binding.Status,
		&binding.RegistryName,
		&binding.CAName,
		&binding.CAURL,
		&binding.CAType,
		&binding.EnrollmentProfile,
		&attrsSnapshotText,
		&certificate.CertificateID,
		&certificate.Status,
		&certificate.SerialNumber,
		&certificate.CertificateDigest,
		&certificate.SubjectDN,
		&certificate.IssuerDN,
		&certificate.KeyRef,
	); err != nil {
		if err == pgx.ErrNoRows {
			return model.IdentityBinding{}, nil, &ActionError{
				StatusCode: http.StatusNotFound,
				Code:       "FABRIC_IDENTITY_NOT_FOUND",
				Message:    "fabric identity binding not found",
			}
		}
		return model.IdentityBinding{}, nil, fmt.Errorf("query fabric identity binding: %w", err)
	}

	attrsSnapshot := map[string]any{}
	if strings.TrimSpace(attrsSnapshotText) != "" {
		if err := json.Unmarshal([]byte(attrsSnapshotText), &attrsSnapshot); err != nil {
			return model.IdentityBinding{}, nil, fmt.Errorf("unmarshal attrs_snapshot: %w", err)
		}
	}
	binding.AttrsSnapshot = attrsSnapshot

	if certificate.CertificateID == "" {
		return binding, nil, nil
	}
	certificate.FabricCARegistryID = binding.FabricCARegistryID
	return binding, &certificate, nil
}

func loadCertificateForUpdate(
	ctx context.Context,
	tx pgx.Tx,
	certificateID string,
) (model.CertificateRecord, *model.IdentityBinding, error) {
	row := tx.QueryRow(
		ctx,
		`SELECT
		   cr.certificate_id::text,
		   cr.fabric_ca_registry_id::text,
		   cr.status,
		   cr.serial_number,
		   cr.certificate_digest,
		   cr.subject_dn,
		   cr.issuer_dn,
		   cr.key_ref,
		   COALESCE(fib.fabric_identity_binding_id::text, ''),
		   COALESCE(fib.fabric_ca_registry_id::text, ''),
		   COALESCE(fib.certificate_id::text, ''),
		   COALESCE(fib.msp_id, ''),
		   COALESCE(fib.affiliation, ''),
		   COALESCE(fib.enrollment_id, ''),
		   COALESCE(fib.identity_type, ''),
		   COALESCE(fib.status, ''),
		   COALESCE(fcr.registry_name, ''),
		   COALESCE(fcr.ca_name, ''),
		   COALESCE(fcr.ca_url, ''),
		   COALESCE(fcr.ca_type, ''),
		   COALESCE(fcr.enrollment_profile, ''),
		   COALESCE(fib.attrs_snapshot::text, '{}')
		 FROM iam.certificate_record cr
		 LEFT JOIN iam.fabric_identity_binding fib
		   ON fib.certificate_id = cr.certificate_id
		 LEFT JOIN iam.fabric_ca_registry fcr
		   ON fcr.fabric_ca_registry_id = cr.fabric_ca_registry_id
		 WHERE cr.certificate_id = $1::text::uuid
		 FOR UPDATE OF cr`,
		certificateID,
	)

	certificate := model.CertificateRecord{}
	binding := model.IdentityBinding{}
	var attrsSnapshotText string
	if err := row.Scan(
		&certificate.CertificateID,
		&certificate.FabricCARegistryID,
		&certificate.Status,
		&certificate.SerialNumber,
		&certificate.CertificateDigest,
		&certificate.SubjectDN,
		&certificate.IssuerDN,
		&certificate.KeyRef,
		&binding.FabricIdentityBindingID,
		&binding.FabricCARegistryID,
		&binding.CertificateID,
		&binding.MSPID,
		&binding.Affiliation,
		&binding.EnrollmentID,
		&binding.IdentityType,
		&binding.Status,
		&binding.RegistryName,
		&binding.CAName,
		&binding.CAURL,
		&binding.CAType,
		&binding.EnrollmentProfile,
		&attrsSnapshotText,
	); err != nil {
		if err == pgx.ErrNoRows {
			return model.CertificateRecord{}, nil, &ActionError{
				StatusCode: http.StatusNotFound,
				Code:       "CERTIFICATE_NOT_FOUND",
				Message:    "certificate not found",
			}
		}
		return model.CertificateRecord{}, nil, fmt.Errorf("query certificate record: %w", err)
	}

	if binding.FabricIdentityBindingID == "" {
		return certificate, nil, nil
	}
	attrsSnapshot := map[string]any{}
	if strings.TrimSpace(attrsSnapshotText) != "" {
		if err := json.Unmarshal([]byte(attrsSnapshotText), &attrsSnapshot); err != nil {
			return model.CertificateRecord{}, nil, fmt.Errorf("unmarshal attrs_snapshot: %w", err)
		}
	}
	binding.AttrsSnapshot = attrsSnapshot
	return certificate, &binding, nil
}

func insertRevocationRecord(
	ctx context.Context,
	tx pgx.Tx,
	certificateID string,
	actorUserID string,
	revokeReason string,
	revokeSource string,
	payload map[string]any,
) error {
	payloadJSON, err := marshalJSON(payload)
	if err != nil {
		return err
	}
	if _, err := tx.Exec(
		ctx,
		`INSERT INTO iam.certificate_revocation_record (
		   certificate_id,
		   revoked_by_user_id,
		   revoke_reason,
		   revoke_source,
		   revoked_at,
		   metadata
		 ) VALUES (
		   $1::text::uuid,
		   NULLIF($2, '')::uuid,
		   $3,
		   $4,
		   now(),
		   $5::jsonb
		 )
		 ON CONFLICT (certificate_id)
		 DO UPDATE
		   SET revoked_by_user_id = EXCLUDED.revoked_by_user_id,
		       revoke_reason = EXCLUDED.revoke_reason,
		       revoke_source = EXCLUDED.revoke_source,
		       revoked_at = now(),
		       metadata = EXCLUDED.metadata`,
		certificateID,
		actorUserID,
		revokeReason,
		revokeSource,
		string(payloadJSON),
	); err != nil {
		return fmt.Errorf("upsert iam.certificate_revocation_record: %w", err)
	}
	return nil
}

func marshalJSON(value any) ([]byte, error) {
	encoded, err := json.Marshal(value)
	if err != nil {
		return nil, fmt.Errorf("marshal JSON: %w", err)
	}
	return encoded, nil
}

func hashJSON(payload []byte) string {
	sum := sha256.Sum256(payload)
	return hex.EncodeToString(sum[:])
}

func nullableString(value string) any {
	if strings.TrimSpace(value) == "" {
		return nil
	}
	return value
}

func linkedBindingID(binding *model.IdentityBinding) string {
	if binding == nil {
		return ""
	}
	return binding.FabricIdentityBindingID
}
