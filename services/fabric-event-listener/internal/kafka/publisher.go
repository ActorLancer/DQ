package kafka

import (
	"context"
	"fmt"
	"log/slog"
	"time"

	"github.com/segmentio/kafka-go"

	listenerconfig "datab.local/fabric-event-listener/internal/config"
)

type Publisher struct {
	topic  string
	writer *kafka.Writer
	logger *slog.Logger
}

func NewPublisher(cfg listenerconfig.Config, logger *slog.Logger) *Publisher {
	return &Publisher{
		topic: cfg.CallbackTopic,
		writer: &kafka.Writer{
			Addr:         kafka.TCP(cfg.KafkaBrokers...),
			Topic:        cfg.CallbackTopic,
			Balancer:     &kafka.Hash{},
			RequiredAcks: kafka.RequireAll,
			BatchTimeout: 50 * time.Millisecond,
		},
		logger: logger,
	}
}

func (publisher *Publisher) Publish(ctx context.Context, key []byte, value []byte) error {
	if err := publisher.writer.WriteMessages(ctx, kafka.Message{
		Key:   key,
		Value: value,
	}); err != nil {
		return fmt.Errorf("write Kafka callback topic %s: %w", publisher.topic, err)
	}

	publisher.logger.Info(
		"fabric-event-listener published Kafka callback",
		"topic", publisher.topic,
	)
	return nil
}

func (publisher *Publisher) Close() error {
	return publisher.writer.Close()
}
