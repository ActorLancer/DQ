package kafka

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"sync"

	"github.com/segmentio/kafka-go"

	adapterconfig "datab.local/fabric-adapter/internal/config"
)

type MessageHandler func(context.Context, string, []byte) error

type Consumer struct {
	readers []*kafka.Reader
	handler MessageHandler
	logger  *slog.Logger
}

func NewConsumer(cfg adapterconfig.Config, handler MessageHandler, logger *slog.Logger) *Consumer {
	return &Consumer{
		readers: []*kafka.Reader{
			newReader(cfg, cfg.AuditAnchorTopic),
			newReader(cfg, cfg.FabricRequestsTopic),
		},
		handler: handler,
		logger:  logger,
	}
}

func (consumer *Consumer) Run(ctx context.Context) error {
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	errCh := make(chan error, len(consumer.readers))
	var waitGroup sync.WaitGroup

	for _, reader := range consumer.readers {
		waitGroup.Add(1)
		go func(current *kafka.Reader) {
			defer waitGroup.Done()
			if err := consumer.consumeLoop(ctx, current); err != nil {
				select {
				case errCh <- err:
				default:
				}
				cancel()
			}
		}(reader)
	}

	waitGroup.Wait()
	close(errCh)

	var result error
	for err := range errCh {
		result = errors.Join(result, err)
	}
	return result
}

func (consumer *Consumer) consumeLoop(ctx context.Context, reader *kafka.Reader) error {
	defer reader.Close()

	for {
		message, err := reader.FetchMessage(ctx)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return fmt.Errorf("fetch Kafka message for topic %s: %w", reader.Config().Topic, err)
		}

		if err := consumer.handler(ctx, reader.Config().Topic, message.Value); err != nil {
			return fmt.Errorf("handle Kafka message for topic %s offset %d: %w", reader.Config().Topic, message.Offset, err)
		}

		if err := reader.CommitMessages(ctx, message); err != nil {
			return fmt.Errorf("commit Kafka message for topic %s offset %d: %w", reader.Config().Topic, message.Offset, err)
		}

		consumer.logger.Info(
			"fabric-adapter committed Kafka message",
			"topic", reader.Config().Topic,
			"partition", message.Partition,
			"offset", message.Offset,
		)
	}
}

func newReader(cfg adapterconfig.Config, topic string) *kafka.Reader {
	return kafka.NewReader(kafka.ReaderConfig{
		Brokers:        cfg.KafkaBrokers,
		GroupID:        cfg.ConsumerGroup,
		Topic:          topic,
		CommitInterval: 0,
		MinBytes:       1,
		MaxBytes:       10e6,
	})
}
