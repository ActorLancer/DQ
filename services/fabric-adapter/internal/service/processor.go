package service

import (
	"context"
	"fmt"
	"log/slog"

	"datab.local/fabric-adapter/internal/model"
	"datab.local/fabric-adapter/internal/provider"
)

type SubmissionPersister interface {
	PersistSubmission(context.Context, provider.SubmissionRequest, provider.SubmissionReceipt) error
}

type Processor struct {
	serviceName string
	persister   SubmissionPersister
	provider    provider.SubmissionProvider
	dispatcher  *Dispatcher
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

	// TODO(V1-gap, AUD-013): fold duplicate Kafka deliveries into ops.consumer_idempotency_record
	// plus Redis short-lock semantics once AUD-026 closes Fabric consumer idempotency/DLQ/reprocess.
	receipt, err := processor.provider.Submit(ctx, request)
	if err != nil {
		return fmt.Errorf("submit to fabric provider: %w", err)
	}

	if err := processor.persister.PersistSubmission(ctx, request, receipt); err != nil {
		return fmt.Errorf("persist submission receipt: %w", err)
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
	)
	return nil
}
