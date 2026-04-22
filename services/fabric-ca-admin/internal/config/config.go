package config

import (
	"fmt"
	"os"
	"strings"
	"time"
)

type Config struct {
	ServiceName  string
	AppEnv       string
	DatabaseURL  string
	ListenAddr   string
	Mode         string
	ReadTimeout  time.Duration
	WriteTimeout time.Duration
}

func Load() (Config, error) {
	port := strings.TrimSpace(firstNonEmpty(os.Getenv("FABRIC_CA_ADMIN_PORT"), "18112"))
	listenAddr := strings.TrimSpace(firstNonEmpty(
		os.Getenv("FABRIC_CA_ADMIN_LISTEN_ADDR"),
		"127.0.0.1:"+port,
	))

	cfg := Config{
		ServiceName:  firstNonEmpty(os.Getenv("FABRIC_CA_ADMIN_SERVICE_NAME"), "fabric-ca-admin"),
		AppEnv:       firstNonEmpty(os.Getenv("APP_ENV"), "local"),
		DatabaseURL:  firstNonEmpty(os.Getenv("DATABASE_URL"), os.Getenv("FABRIC_CA_ADMIN_DATABASE_URL")),
		ListenAddr:   listenAddr,
		Mode:         normalizeMode(firstNonEmpty(os.Getenv("FABRIC_CA_ADMIN_MODE"), "mock")),
		ReadTimeout:  parseDurationEnv("FABRIC_CA_ADMIN_READ_TIMEOUT", 15*time.Second),
		WriteTimeout: parseDurationEnv("FABRIC_CA_ADMIN_WRITE_TIMEOUT", 15*time.Second),
	}

	if cfg.DatabaseURL == "" {
		return Config{}, fmt.Errorf("DATABASE_URL is required")
	}
	if cfg.ListenAddr == "" {
		return Config{}, fmt.Errorf("FABRIC_CA_ADMIN_LISTEN_ADDR is required")
	}

	return cfg, nil
}

func normalizeMode(mode string) string {
	switch strings.ToLower(strings.TrimSpace(mode)) {
	case "", "mock":
		return "mock"
	default:
		return "mock"
	}
}

func parseDurationEnv(key string, fallback time.Duration) time.Duration {
	value := strings.TrimSpace(os.Getenv(key))
	if value == "" {
		return fallback
	}
	parsed, err := time.ParseDuration(value)
	if err != nil {
		return fallback
	}
	return parsed
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if trimmed := strings.TrimSpace(value); trimmed != "" {
			return trimmed
		}
	}
	return ""
}
