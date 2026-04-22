package service

import (
	"context"
	"fmt"
	"log/slog"

	"datab.local/fabric-adapter/internal/model"
	"datab.local/fabric-adapter/internal/provider"
)

type SubmissionPersister interface {
	PersistSubmission(context.Context, model.CanonicalEnvelope, provider.SubmissionReceipt) error
}

type Processor struct {
	serviceName string
	persister   SubmissionPersister
	provider    provider.SubmissionProvider
	logger      *slog.Logger
}

func NewProcessor(
	serviceName string,
	persister SubmissionPersister,
	submitter provider.SubmissionProvider,
	logger *slog.Logger,
) *Processor {
	return &Processor{
		serviceName: serviceName,
		persister:   persister,
		provider:    submitter,
		logger:      logger,
	}
}

func (processor *Processor) ProcessMessage(ctx context.Context, topic string, value []byte) error {
	envelope, err := model.DecodeCanonicalEnvelope(topic, value)
	if err != nil {
		return err
	}

	// TODO(V1-gap, AUD-013): fold duplicate Kafka deliveries into ops.consumer_idempotency_record
	// plus Redis short-lock semantics once AUD-026 closes Fabric consumer idempotency/DLQ/reprocess.
	receipt, err := processor.provider.Submit(ctx, provider.SubmissionRequest{
		Envelope: envelope,
	})
	if err != nil {
		return fmt.Errorf("submit to fabric provider: %w", err)
	}

	if err := processor.persister.PersistSubmission(ctx, envelope, receipt); err != nil {
		return fmt.Errorf("persist submission receipt: %w", err)
	}

	processor.logger.Info(
		"fabric submission accepted",
		"service_name", processor.serviceName,
		"topic", topic,
		"event_id", envelope.EventID,
		"event_type", envelope.EventType,
		"aggregate_type", envelope.AggregateType,
		"aggregate_id", envelope.AggregateID,
		"provider_reference", receipt.ProviderReference,
		"receipt_status", receipt.ReceiptStatus,
	)
	return nil
}
