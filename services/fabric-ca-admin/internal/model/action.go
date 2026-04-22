package model

import "time"

type ActorContext struct {
	RequestID         string
	TraceID           string
	ActorRole         string
	ActorUserID       string
	StepUpChallengeID string
	PermissionCode    string
}

type IdentityBinding struct {
	FabricIdentityBindingID string
	FabricCARegistryID      string
	CertificateID           string
	MSPID                   string
	Affiliation             string
	EnrollmentID            string
	IdentityType            string
	Status                  string
	RegistryName            string
	CAName                  string
	CAURL                   string
	CAType                  string
	EnrollmentProfile       string
	AttrsSnapshot           map[string]any
}

type CertificateRecord struct {
	CertificateID      string
	FabricCARegistryID string
	Status             string
	SerialNumber       string
	CertificateDigest  string
	SubjectDN          string
	IssuerDN           string
	KeyRef             string
}

type ProviderIssueReceipt struct {
	OccurredAt        time.Time
	ProviderRequestID string
	SerialNumber      string
	CertificateDigest string
	SubjectDN         string
	IssuerDN          string
	KeyRef            string
	NotBefore         time.Time
	NotAfter          time.Time
	Payload           map[string]any
}

type ProviderRevokeReceipt struct {
	OccurredAt        time.Time
	ProviderRequestID string
	RevokeReason      string
	Payload           map[string]any
}

type ActionResult struct {
	TargetID              string `json:"target_id"`
	Status                string `json:"status"`
	CertificateID         string `json:"certificate_id,omitempty"`
	ExternalFactReceiptID string `json:"external_fact_receipt_id,omitempty"`
	ProviderReference     string `json:"provider_reference,omitempty"`
}
