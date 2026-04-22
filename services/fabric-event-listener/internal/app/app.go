package app

import (
	"context"
	"log/slog"

	listenerconfig "datab.local/fabric-event-listener/internal/config"
	listenerkafka "datab.local/fabric-event-listener/internal/kafka"
	"datab.local/fabric-event-listener/internal/provider"
	"datab.local/fabric-event-listener/internal/service"
	"datab.local/fabric-event-listener/internal/store"
)

func Run(ctx context.Context, cfg listenerconfig.Config, logger *slog.Logger) error {
	persist, err := store.New(ctx, cfg.DatabaseURL, cfg.ServiceName)
	if err != nil {
		return err
	}
	defer persist.Close()

	publisher := listenerkafka.NewPublisher(cfg, logger)
	defer publisher.Close()

	callbackProvider := provider.NewMock(cfg.ChannelName, cfg.ChaincodeName)
	processor := service.NewProcessor(
		cfg.ServiceName,
		cfg.BatchSize,
		persist,
		callbackProvider,
		publisher,
		logger,
	)

	logger.Info(
		"fabric-event-listener starting",
		"service_name", cfg.ServiceName,
		"app_env", cfg.AppEnv,
		"callback_topic", cfg.CallbackTopic,
		"provider_mode", cfg.ProviderMode,
		"channel_name", cfg.ChannelName,
		"chaincode_name", cfg.ChaincodeName,
		"poll_interval", cfg.PollInterval.String(),
		"batch_size", cfg.BatchSize,
	)

	return processor.Run(ctx, cfg.PollInterval)
}
