package main

import (
	"context"
	"log/slog"
	"os"
	"os/signal"
	"syscall"

	"datab.local/fabric-adapter/internal/app"
	"datab.local/fabric-adapter/internal/config"
)

func main() {
	logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
		Level: slog.LevelInfo,
	}))

	cfg, err := config.Load()
	if err != nil {
		logger.Error("load fabric-adapter config", "error", err)
		os.Exit(1)
	}

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	if err := app.Run(ctx, cfg, logger); err != nil {
		logger.Error("fabric-adapter stopped with error", "error", err)
		os.Exit(1)
	}
}
