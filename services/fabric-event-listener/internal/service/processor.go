package service

import (
	"context"
	"fmt"
	"log/slog"
	"time"

	"datab.local/fabric-event-listener/internal/model"
	"datab.local/fabric-event-listener/internal/provider"
)

type PendingSubmissionStore interface {
	ListPendingSubmissions(context.Context, int) ([]model.PendingSubmission, error)
	PersistCallback(context.Context, model.CallbackObservation) error
}

type CallbackPublisher interface {
	Publish(context.Context, []byte, []byte) error
}

type Processor struct {
	serviceName string
	batchSize   int
	store       PendingSubmissionStore
	provider    provider.CallbackProvider
	publisher   CallbackPublisher
	logger      *slog.Logger
}

func NewProcessor(
	serviceName string,
	batchSize int,
	store PendingSubmissionStore,
	callbackProvider provider.CallbackProvider,
	publisher CallbackPublisher,
	logger *slog.Logger,
) *Processor {
	return &Processor{
		serviceName: serviceName,
		batchSize:   batchSize,
		store:       store,
		provider:    callbackProvider,
		publisher:   publisher,
		logger:      logger,
	}
}

func (processor *Processor) Run(ctx context.Context, pollInterval time.Duration) error {
	if err := processor.processBatch(ctx); err != nil {
		return err
	}

	ticker := time.NewTicker(pollInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			if err := processor.processBatch(ctx); err != nil {
				return err
			}
		}
	}
}

func (processor *Processor) processBatch(ctx context.Context) error {
	pending, err := processor.store.ListPendingSubmissions(ctx, processor.batchSize)
	if err != nil {
		return fmt.Errorf("list pending fabric submissions: %w", err)
	}

	for _, submission := range pending {
		observation, err := processor.provider.Observe(ctx, submission)
		if err != nil {
			return fmt.Errorf(
				"observe fabric callback for receipt %s: %w",
				submission.SourceReceiptID,
				err,
			)
		}

		key, envelope, err := observation.MarshalCanonicalEnvelope(processor.serviceName)
		if err != nil {
			return fmt.Errorf(
				"marshal fabric callback envelope for receipt %s: %w",
				submission.SourceReceiptID,
				err,
			)
		}

		if err := processor.publisher.Publish(ctx, key, envelope); err != nil {
			return fmt.Errorf(
				"publish fabric callback for receipt %s: %w",
				submission.SourceReceiptID,
				err,
			)
		}

		if err := processor.store.PersistCallback(ctx, observation); err != nil {
			return fmt.Errorf(
				"persist fabric callback for receipt %s: %w",
				submission.SourceReceiptID,
				err,
			)
		}

		processor.logger.Info(
			"fabric-event-listener processed callback",
			"source_receipt_id", submission.SourceReceiptID,
			"event_id", observation.EventID,
			"event_type", observation.EventType,
			"submission_kind", submission.SubmissionKind,
			"chain_anchor_id", submission.ChainAnchorID,
			"provider_request_id", observation.ProviderRequestID,
			"receipt_status", observation.ReceiptStatus,
		)
	}

	return nil
}
