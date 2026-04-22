package service

import (
	"context"
	"log/slog"

	"datab.local/fabric-ca-admin/internal/model"
	"datab.local/fabric-ca-admin/internal/provider"
)

type Executor interface {
	IssueIdentity(context.Context, string, model.ActorContext, provider.CAProvider) (model.ActionResult, error)
	RevokeIdentity(context.Context, string, model.ActorContext, provider.CAProvider) (model.ActionResult, error)
	RevokeCertificate(context.Context, string, model.ActorContext, provider.CAProvider) (model.ActionResult, error)
	Ping(context.Context) error
}

type Service struct {
	executor Executor
	provider provider.CAProvider
	logger   *slog.Logger
}

func New(executor Executor, fabricCA provider.CAProvider, logger *slog.Logger) *Service {
	return &Service{
		executor: executor,
		provider: fabricCA,
		logger:   logger,
	}
}

func (service *Service) Ping(ctx context.Context) error {
	return service.executor.Ping(ctx)
}

func (service *Service) IssueIdentity(
	ctx context.Context,
	identityID string,
	actor model.ActorContext,
) (model.ActionResult, error) {
	result, err := service.executor.IssueIdentity(ctx, identityID, actor, service.provider)
	if err == nil {
		service.logger.Info(
			"fabric-ca-admin issued identity",
			"target_id", result.TargetID,
			"certificate_id", result.CertificateID,
			"request_id", actor.RequestID,
			"trace_id", actor.TraceID,
		)
	}
	return result, err
}

func (service *Service) RevokeIdentity(
	ctx context.Context,
	identityID string,
	actor model.ActorContext,
) (model.ActionResult, error) {
	result, err := service.executor.RevokeIdentity(ctx, identityID, actor, service.provider)
	if err == nil {
		service.logger.Info(
			"fabric-ca-admin revoked identity",
			"target_id", result.TargetID,
			"certificate_id", result.CertificateID,
			"request_id", actor.RequestID,
			"trace_id", actor.TraceID,
		)
	}
	return result, err
}

func (service *Service) RevokeCertificate(
	ctx context.Context,
	certificateID string,
	actor model.ActorContext,
) (model.ActionResult, error) {
	result, err := service.executor.RevokeCertificate(ctx, certificateID, actor, service.provider)
	if err == nil {
		service.logger.Info(
			"fabric-ca-admin revoked certificate",
			"target_id", result.TargetID,
			"certificate_id", result.CertificateID,
			"request_id", actor.RequestID,
			"trace_id", actor.TraceID,
		)
	}
	return result, err
}
