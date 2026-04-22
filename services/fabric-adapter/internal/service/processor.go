package service

import (
	"context"
	"fmt"
	"log/slog"

	"datab.local/fabric-adapter/internal/model"
	"datab.local/fabric-adapter/internal/provider"
)

type ProcessingStateStore interface {
	LoadProcessingState(context.Context, string) (string, bool, error)
	BeginProcessing(context.Context, model.CanonicalEnvelope, string, string) (bool, error)
	UpdateProcessingResult(context.Context, model.CanonicalEnvelope, string, string, map[string]any) error
	PersistSubmission(context.Context, provider.SubmissionRequest, provider.SubmissionReceipt) error
}

type ShortLock interface {
	Acquire(context.Context, string) (string, bool, error)
	Release(context.Context, string) error
}

type Processor struct {
	serviceName string
	store       ProcessingStateStore
	provider    provider.SubmissionProvider
	locker      ShortLock
	dispatcher  *Dispatcher
	logger      *slog.Logger
}

func NewProcessor(
	serviceName string,
	store ProcessingStateStore,
	submitter provider.SubmissionProvider,
	locker ShortLock,
	logger *slog.Logger,
) *Processor {
	return &Processor{
		serviceName: serviceName,
		store:       store,
		provider:    submitter,
		locker:      locker,
		dispatcher:  NewDispatcher(),
		logger:      logger,
	}
}

func (processor *Processor) ProcessMessage(ctx context.Context, topic string, value []byte) error {
	envelope, err := model.DecodeCanonicalEnvelope(topic, value)
	if err != nil {
		return err
	}

	request, err := processor.dispatcher.BuildRequest(envelope)
	if err != nil {
		return fmt.Errorf("build submission request: %w", err)
	}

	lockKey, acquired, err := processor.locker.Acquire(ctx, envelope.EventID)
	if err != nil {
		return fmt.Errorf("acquire redis short lock: %w", err)
	}
	if !acquired {
		resultCode, exists, err := processor.store.LoadProcessingState(ctx, envelope.EventID)
		if err != nil {
			return fmt.Errorf("load processing state: %w", err)
		}
		if exists && (resultCode == "processed" || resultCode == "dead_lettered") {
			processor.logger.Info(
				"fabric submission duplicate skipped",
				"service_name", processor.serviceName,
				"topic", topic,
				"event_id", envelope.EventID,
				"event_type", envelope.EventType,
				"result_code", resultCode,
			)
			return nil
		}

		processor.logger.Warn(
			"fabric submission skipped because event is already in flight",
			"service_name", processor.serviceName,
			"topic", topic,
			"event_id", envelope.EventID,
			"event_type", envelope.EventType,
			"lock_key", lockKey,
		)
		return nil
	}
	defer func() {
		if releaseErr := processor.locker.Release(ctx, envelope.EventID); releaseErr != nil {
			processor.logger.Warn(
				"fabric submission lock release failed",
				"service_name", processor.serviceName,
				"event_id", envelope.EventID,
				"lock_key", lockKey,
				"error", releaseErr,
			)
		}
	}()

	proceed, err := processor.store.BeginProcessing(ctx, envelope, topic, lockKey)
	if err != nil {
		return fmt.Errorf("begin consumer idempotency gate: %w", err)
	}
	if !proceed {
		processor.logger.Info(
			"fabric submission duplicate skipped",
			"service_name", processor.serviceName,
			"topic", topic,
			"event_id", envelope.EventID,
			"event_type", envelope.EventType,
			"lock_key", lockKey,
		)
		return nil
	}

	receipt, err := processor.provider.Submit(ctx, request)
	if err != nil {
		if updateErr := processor.store.UpdateProcessingResult(
			ctx,
			envelope,
			"failed",
			err.Error(),
			map[string]any{
				"source_topic": topic,
				"lock_backend": "redis",
				"lock_key":     lockKey,
			},
		); updateErr != nil {
			return fmt.Errorf("submit to fabric provider: %w (mark failed: %v)", err, updateErr)
		}
		return fmt.Errorf("submit to fabric provider: %w", err)
	}

	if err := processor.store.PersistSubmission(ctx, request, receipt); err != nil {
		if updateErr := processor.store.UpdateProcessingResult(
			ctx,
			envelope,
			"failed",
			err.Error(),
			map[string]any{
				"source_topic":       topic,
				"lock_backend":       "redis",
				"lock_key":           lockKey,
				"provider_reference": receipt.ProviderReference,
				"receipt_status":     receipt.ReceiptStatus,
			},
		); updateErr != nil {
			return fmt.Errorf("persist submission receipt: %w (mark failed: %v)", err, updateErr)
		}
		return fmt.Errorf("persist submission receipt: %w", err)
	}
	if err := processor.store.UpdateProcessingResult(
		ctx,
		envelope,
		"processed",
		"",
		map[string]any{
			"source_topic":       topic,
			"lock_backend":       "redis",
			"lock_key":           lockKey,
			"provider_reference": receipt.ProviderReference,
			"receipt_status":     receipt.ReceiptStatus,
			"submission_kind":    string(request.SubmissionKind),
			"contract_name":      request.ContractName,
			"transaction_name":   request.TransactionName,
		},
	); err != nil {
		return fmt.Errorf("update consumer idempotency result: %w", err)
	}

	processor.logger.Info(
		"fabric submission accepted",
		"service_name", processor.serviceName,
		"topic", topic,
		"event_id", envelope.EventID,
		"event_type", envelope.EventType,
		"submission_kind", request.SubmissionKind,
		"contract_name", request.ContractName,
		"transaction_name", request.TransactionName,
		"aggregate_type", envelope.AggregateType,
		"aggregate_id", envelope.AggregateID,
		"provider_reference", receipt.ProviderReference,
		"receipt_status", receipt.ReceiptStatus,
		"lock_key", lockKey,
	)
	return nil
}
