package main

import (
	"context"
	"log/slog"
	"os"
	"os/signal"
	"syscall"

	"datab.local/fabric-ca-admin/internal/app"
	"datab.local/fabric-ca-admin/internal/config"
)

func main() {
	logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
		Level: slog.LevelInfo,
	}))

	cfg, err := config.Load()
	if err != nil {
		logger.Error("load fabric-ca-admin config", "error", err)
		os.Exit(1)
	}

	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	if err := app.Run(ctx, cfg, logger); err != nil {
		logger.Error("fabric-ca-admin stopped with error", "error", err)
		os.Exit(1)
	}
}
