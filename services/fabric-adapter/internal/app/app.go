package app

import (
	"context"
	"io"
	"log/slog"

	adapterconfig "datab.local/fabric-adapter/internal/config"
	adapterkafka "datab.local/fabric-adapter/internal/kafka"
	"datab.local/fabric-adapter/internal/provider"
	"datab.local/fabric-adapter/internal/service"
	"datab.local/fabric-adapter/internal/store"
)

func Run(ctx context.Context, cfg adapterconfig.Config, logger *slog.Logger) error {
	persist, err := store.New(ctx, cfg.DatabaseURL, cfg.ServiceName)
	if err != nil {
		return err
	}
	defer persist.Close()

	submitter, err := provider.NewSubmissionProvider(cfg)
	if err != nil {
		return err
	}
	if closer, ok := submitter.(io.Closer); ok {
		defer closer.Close()
	}
	processor := service.NewProcessor(cfg.ServiceName, persist, submitter, logger)
	consumer := adapterkafka.NewConsumer(cfg, processor.ProcessMessage, logger)

	logger.Info(
		"fabric-adapter starting",
		"service_name", cfg.ServiceName,
		"app_env", cfg.AppEnv,
		"consumer_group", cfg.ConsumerGroup,
		"audit_anchor_topic", cfg.AuditAnchorTopic,
		"fabric_requests_topic", cfg.FabricRequestsTopic,
		"provider_mode", cfg.ProviderMode,
		"chaincode_name", cfg.ChaincodeName,
		"channel_name", cfg.ChannelName,
		"gateway_endpoint", cfg.GatewayEndpoint,
	)

	return consumer.Run(ctx)
}
