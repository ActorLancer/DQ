package config

import (
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"
)

type Config struct {
	ServiceName   string
	AppEnv        string
	DatabaseURL   string
	KafkaBrokers  []string
	CallbackTopic string
	ProviderMode  string
	ChannelName   string
	ChaincodeName string
	PollInterval  time.Duration
	BatchSize     int
}

func Load() (Config, error) {
	cfg := Config{
		ServiceName:   firstNonEmpty(os.Getenv("FABRIC_EVENT_LISTENER_SERVICE_NAME"), "fabric-event-listener"),
		AppEnv:        firstNonEmpty(os.Getenv("APP_ENV"), "local"),
		DatabaseURL:   firstNonEmpty(os.Getenv("DATABASE_URL"), os.Getenv("FABRIC_EVENT_LISTENER_DATABASE_URL")),
		KafkaBrokers:  splitCSV(firstNonEmpty(os.Getenv("KAFKA_BROKERS"), os.Getenv("KAFKA_BOOTSTRAP_SERVERS"), "127.0.0.1:9094")),
		CallbackTopic: firstNonEmpty(os.Getenv("TOPIC_FABRIC_CALLBACKS"), "dtp.fabric.callbacks"),
		ProviderMode:  firstNonEmpty(os.Getenv("FABRIC_EVENT_LISTENER_PROVIDER_MODE"), "mock"),
		ChannelName:   firstNonEmpty(os.Getenv("FABRIC_CHANNEL_NAME"), "datab-channel"),
		ChaincodeName: firstNonEmpty(os.Getenv("FABRIC_CHAINCODE_NAME"), "datab-audit-anchor"),
		PollInterval:  parseDurationEnv("FABRIC_EVENT_LISTENER_POLL_INTERVAL", 1500*time.Millisecond),
		BatchSize:     parseIntEnv("FABRIC_EVENT_LISTENER_BATCH_SIZE", 16),
	}

	if cfg.DatabaseURL == "" {
		return Config{}, fmt.Errorf("DATABASE_URL is required")
	}
	if len(cfg.KafkaBrokers) == 0 {
		return Config{}, fmt.Errorf("KAFKA_BROKERS or KAFKA_BOOTSTRAP_SERVERS is required")
	}
	if strings.TrimSpace(cfg.CallbackTopic) == "" {
		return Config{}, fmt.Errorf("TOPIC_FABRIC_CALLBACKS is required")
	}
	if strings.TrimSpace(cfg.ProviderMode) == "" {
		return Config{}, fmt.Errorf("FABRIC_EVENT_LISTENER_PROVIDER_MODE is required")
	}

	return cfg, nil
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

func parseIntEnv(key string, fallback int) int {
	value := strings.TrimSpace(os.Getenv(key))
	if value == "" {
		return fallback
	}
	parsed, err := strconv.Atoi(value)
	if err != nil || parsed <= 0 {
		return fallback
	}
	return parsed
}

func splitCSV(raw string) []string {
	parts := strings.Split(raw, ",")
	items := make([]string, 0, len(parts))
	for _, item := range parts {
		trimmed := strings.TrimSpace(item)
		if trimmed == "" {
			continue
		}
		items = append(items, trimmed)
	}
	return items
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if trimmed := strings.TrimSpace(value); trimmed != "" {
			return trimmed
		}
	}
	return ""
}
