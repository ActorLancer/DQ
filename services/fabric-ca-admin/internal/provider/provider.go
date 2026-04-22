package provider

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"
	"time"

	"datab.local/fabric-ca-admin/internal/model"
)

type CAProvider interface {
	IssueIdentity(context.Context, model.IdentityBinding) (model.ProviderIssueReceipt, error)
	RevokeIdentity(context.Context, model.IdentityBinding, *model.CertificateRecord) (model.ProviderRevokeReceipt, error)
	RevokeCertificate(context.Context, model.CertificateRecord, *model.IdentityBinding) (model.ProviderRevokeReceipt, error)
}

type MockProvider struct {
	serviceName string
}

func NewMock(serviceName string) *MockProvider {
	return &MockProvider{serviceName: serviceName}
}

func (provider *MockProvider) IssueIdentity(
	_ context.Context,
	binding model.IdentityBinding,
) (model.ProviderIssueReceipt, error) {
	occurredAt := time.Now().UTC()
	providerRequestID := newID("fabric-ca-issue-", binding.FabricIdentityBindingID+occurredAt.Format(time.RFC3339Nano))
	serialSeed := sha256.Sum256([]byte(binding.FabricIdentityBindingID + ":" + providerRequestID))
	serialNumber := strings.ToUpper(hex.EncodeToString(serialSeed[:8]))
	certificateDigest := hashHex(
		fmt.Sprintf(
			"%s|%s|%s|%s",
			binding.MSPID,
			binding.EnrollmentID,
			binding.Affiliation,
			providerRequestID,
		),
	)
	subjectDN := fmt.Sprintf(
		"CN=%s,OU=%s,O=%s",
		binding.EnrollmentID,
		defaultString(binding.Affiliation, "fabric"),
		defaultString(binding.MSPID, "DATABMSP"),
	)
	issuerDN := fmt.Sprintf(
		"CN=%s,O=%s",
		defaultString(binding.CAName, defaultString(binding.RegistryName, "fabric-ca")),
		defaultString(binding.MSPID, "DATABMSP"),
	)
	keyRef := fmt.Sprintf("mockkms://fabric-ca/%s", binding.FabricIdentityBindingID)
	notBefore := occurredAt
	notAfter := occurredAt.Add(365 * 24 * time.Hour)

	return model.ProviderIssueReceipt{
		OccurredAt:        occurredAt,
		ProviderRequestID: providerRequestID,
		SerialNumber:      serialNumber,
		CertificateDigest: certificateDigest,
		SubjectDN:         subjectDN,
		IssuerDN:          issuerDN,
		KeyRef:            keyRef,
		NotBefore:         notBefore,
		NotAfter:          notAfter,
		Payload: map[string]any{
			"mode":                "mock",
			"service_name":        provider.serviceName,
			"provider_request_id": providerRequestID,
			"msp_id":              binding.MSPID,
			"affiliation":         binding.Affiliation,
			"enrollment_id":       binding.EnrollmentID,
			"identity_type":       binding.IdentityType,
			"attrs_snapshot":      binding.AttrsSnapshot,
			"subject_dn":          subjectDN,
			"issuer_dn":           issuerDN,
			"serial_number":       serialNumber,
			"certificate_digest":  certificateDigest,
			"key_ref":             keyRef,
			"not_before":          notBefore.Format(time.RFC3339Nano),
			"not_after":           notAfter.Format(time.RFC3339Nano),
		},
	}, nil
}

func (provider *MockProvider) RevokeIdentity(
	_ context.Context,
	binding model.IdentityBinding,
	certificate *model.CertificateRecord,
) (model.ProviderRevokeReceipt, error) {
	return provider.revokeReceipt(
		"identity",
		binding.FabricIdentityBindingID,
		func(payload map[string]any) {
			payload["msp_id"] = binding.MSPID
			payload["affiliation"] = binding.Affiliation
			payload["enrollment_id"] = binding.EnrollmentID
			payload["identity_type"] = binding.IdentityType
			if certificate != nil {
				payload["certificate_id"] = certificate.CertificateID
				payload["certificate_digest"] = certificate.CertificateDigest
			}
		},
	)
}

func (provider *MockProvider) RevokeCertificate(
	_ context.Context,
	certificate model.CertificateRecord,
	binding *model.IdentityBinding,
) (model.ProviderRevokeReceipt, error) {
	return provider.revokeReceipt(
		"certificate",
		certificate.CertificateID,
		func(payload map[string]any) {
			payload["certificate_id"] = certificate.CertificateID
			payload["certificate_digest"] = certificate.CertificateDigest
			payload["serial_number"] = certificate.SerialNumber
			if binding != nil {
				payload["fabric_identity_binding_id"] = binding.FabricIdentityBindingID
				payload["enrollment_id"] = binding.EnrollmentID
				payload["msp_id"] = binding.MSPID
			}
		},
	)
}

func (provider *MockProvider) revokeReceipt(
	scope string,
	targetID string,
	enrich func(map[string]any),
) (model.ProviderRevokeReceipt, error) {
	occurredAt := time.Now().UTC()
	providerRequestID := newID("fabric-ca-revoke-", scope+":"+targetID+":"+occurredAt.Format(time.RFC3339Nano))
	payload := map[string]any{
		"mode":                "mock",
		"service_name":        provider.serviceName,
		"provider_request_id": providerRequestID,
		"scope":               scope,
		"revoked_at":          occurredAt.Format(time.RFC3339Nano),
		"revoke_reason":       "manual_revoke",
	}
	if enrich != nil {
		enrich(payload)
	}
	return model.ProviderRevokeReceipt{
		OccurredAt:        occurredAt,
		ProviderRequestID: providerRequestID,
		RevokeReason:      "manual_revoke",
		Payload:           payload,
	}, nil
}

func newID(prefix string, source string) string {
	return prefix + hashHex(source)[:16]
}

func hashHex(source string) string {
	sum := sha256.Sum256([]byte(source))
	return hex.EncodeToString(sum[:])
}

func defaultString(value string, fallback string) string {
	if strings.TrimSpace(value) == "" {
		return fallback
	}
	return value
}
