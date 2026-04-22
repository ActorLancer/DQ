package provider

import (
	"context"
	"testing"

	"datab.local/fabric-ca-admin/internal/model"
)

func TestMockProviderIssueAndRevoke(t *testing.T) {
	provider := NewMock("fabric-ca-admin")
	binding := model.IdentityBinding{
		FabricIdentityBindingID: "11111111-1111-4111-8111-111111111111",
		MSPID:                   "DATABMSP",
		Affiliation:             "platform.security",
		EnrollmentID:            "aud016-user",
		IdentityType:            "user",
		RegistryName:            "local-ca",
		CAName:                  "fabric-ca",
	}

	issueReceipt, err := provider.IssueIdentity(context.Background(), binding)
	if err != nil {
		t.Fatalf("IssueIdentity() error = %v", err)
	}
	if issueReceipt.ProviderRequestID == "" || issueReceipt.CertificateDigest == "" {
		t.Fatalf("issue receipt missing fields: %+v", issueReceipt)
	}

	revokeReceipt, err := provider.RevokeIdentity(context.Background(), binding, nil)
	if err != nil {
		t.Fatalf("RevokeIdentity() error = %v", err)
	}
	if revokeReceipt.ProviderRequestID == "" || revokeReceipt.RevokeReason != "manual_revoke" {
		t.Fatalf("revoke receipt missing fields: %+v", revokeReceipt)
	}
}
