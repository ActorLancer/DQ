package service

import (
	"context"
	"io"
	"log/slog"
	"testing"

	"datab.local/fabric-ca-admin/internal/model"
	"datab.local/fabric-ca-admin/internal/provider"
)

type fakeExecutor struct {
	issuedIdentityID     string
	revokedIdentityID    string
	revokedCertificateID string
}

func (executor *fakeExecutor) IssueIdentity(
	_ context.Context,
	identityID string,
	_ model.ActorContext,
	_ provider.CAProvider,
) (model.ActionResult, error) {
	executor.issuedIdentityID = identityID
	return model.ActionResult{TargetID: identityID, Status: "issued"}, nil
}

func (executor *fakeExecutor) RevokeIdentity(
	_ context.Context,
	identityID string,
	_ model.ActorContext,
	_ provider.CAProvider,
) (model.ActionResult, error) {
	executor.revokedIdentityID = identityID
	return model.ActionResult{TargetID: identityID, Status: "revoked"}, nil
}

func (executor *fakeExecutor) RevokeCertificate(
	_ context.Context,
	certificateID string,
	_ model.ActorContext,
	_ provider.CAProvider,
) (model.ActionResult, error) {
	executor.revokedCertificateID = certificateID
	return model.ActionResult{TargetID: certificateID, Status: "revoked"}, nil
}

func (executor *fakeExecutor) Ping(_ context.Context) error {
	return nil
}

func TestServiceDispatchesActions(t *testing.T) {
	logger := slog.New(slog.NewTextHandler(io.Discard, nil))
	executor := &fakeExecutor{}
	service := New(executor, provider.NewMock("fabric-ca-admin"), logger)

	if _, err := service.IssueIdentity(context.Background(), "binding-1", model.ActorContext{}); err != nil {
		t.Fatalf("IssueIdentity() error = %v", err)
	}
	if _, err := service.RevokeIdentity(context.Background(), "binding-2", model.ActorContext{}); err != nil {
		t.Fatalf("RevokeIdentity() error = %v", err)
	}
	if _, err := service.RevokeCertificate(context.Background(), "certificate-1", model.ActorContext{}); err != nil {
		t.Fatalf("RevokeCertificate() error = %v", err)
	}

	if executor.issuedIdentityID != "binding-1" {
		t.Fatalf("issuedIdentityID = %s", executor.issuedIdentityID)
	}
	if executor.revokedIdentityID != "binding-2" {
		t.Fatalf("revokedIdentityID = %s", executor.revokedIdentityID)
	}
	if executor.revokedCertificateID != "certificate-1" {
		t.Fatalf("revokedCertificateID = %s", executor.revokedCertificateID)
	}
}
