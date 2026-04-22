package app

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"time"

	"datab.local/fabric-ca-admin/internal/api"
	"datab.local/fabric-ca-admin/internal/config"
	"datab.local/fabric-ca-admin/internal/provider"
	"datab.local/fabric-ca-admin/internal/service"
	"datab.local/fabric-ca-admin/internal/store"
)

func Run(ctx context.Context, cfg config.Config, logger *slog.Logger) error {
	pgStore, err := store.New(ctx, cfg.DatabaseURL, cfg.ServiceName)
	if err != nil {
		return err
	}
	defer pgStore.Close()

	fabricCAProvider := provider.NewMock(cfg.ServiceName)
	handler := api.New(service.New(pgStore, fabricCAProvider, logger), logger)

	server := &http.Server{
		Addr:         cfg.ListenAddr,
		Handler:      handler.Routes(),
		ReadTimeout:  cfg.ReadTimeout,
		WriteTimeout: cfg.WriteTimeout,
	}

	errCh := make(chan error, 1)
	go func() {
		logger.Info(
			"fabric-ca-admin listening",
			"listen_addr", cfg.ListenAddr,
			"mode", cfg.Mode,
		)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			errCh <- fmt.Errorf("listen: %w", err)
			return
		}
		errCh <- nil
	}()

	select {
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()
		return server.Shutdown(shutdownCtx)
	case err := <-errCh:
		return err
	}
}
