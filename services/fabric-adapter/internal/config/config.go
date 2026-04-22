package config

import (
	"fmt"
	"os"
	"strings"
	"time"
)

type Config struct {
	ServiceName         string
	AppEnv              string
	DatabaseURL         string
	KafkaBrokers        []string
	ConsumerGroup       string
	AuditAnchorTopic    string
	FabricRequestsTopic string
	ChannelName         string
	ChaincodeName       string
	GatewayEndpoint     string
	MSPID               string
	TLSCertPath         string
	SignCertPath        string
	PrivateKeyPath      string
	ProviderTimeout     time.Duration
}

func Load() (Config, error) {
	cfg := Config{
		ServiceName:         firstNonEmpty(os.Getenv("FABRIC_ADAPTER_SERVICE_NAME"), "fabric-adapter"),
		AppEnv:              firstNonEmpty(os.Getenv("APP_ENV"), "local"),
		DatabaseURL:         firstNonEmpty(os.Getenv("DATABASE_URL"), os.Getenv("FABRIC_ADAPTER_DATABASE_URL")),
		KafkaBrokers:        splitCSV(firstNonEmpty(os.Getenv("KAFKA_BROKERS"), os.Getenv("KAFKA_BOOTSTRAP_SERVERS"), "127.0.0.1:9094")),
		ConsumerGroup:       firstNonEmpty(os.Getenv("FABRIC_ADAPTER_CONSUMER_GROUP"), "cg-fabric-adapter"),
		AuditAnchorTopic:    firstNonEmpty(os.Getenv("TOPIC_AUDIT_ANCHOR"), "dtp.audit.anchor"),
		FabricRequestsTopic: firstNonEmpty(os.Getenv("TOPIC_FABRIC_REQUESTS"), "dtp.fabric.requests"),
		ChannelName:         firstNonEmpty(os.Getenv("FABRIC_CHANNEL_NAME"), "datab-channel"),
		ChaincodeName:       firstNonEmpty(os.Getenv("FABRIC_CHAINCODE_NAME"), "datab-audit-anchor"),
		GatewayEndpoint:     strings.TrimSpace(os.Getenv("FABRIC_GATEWAY_ENDPOINT")),
		MSPID:               strings.TrimSpace(os.Getenv("FABRIC_MSP_ID")),
		TLSCertPath:         strings.TrimSpace(os.Getenv("FABRIC_TLS_CERT_PATH")),
		SignCertPath:        strings.TrimSpace(os.Getenv("FABRIC_SIGN_CERT_PATH")),
		PrivateKeyPath:      strings.TrimSpace(os.Getenv("FABRIC_PRIVATE_KEY_PATH")),
		ProviderTimeout:     parseDurationEnv("FABRIC_ADAPTER_PROVIDER_TIMEOUT", 15*time.Second),
	}

	if cfg.DatabaseURL == "" {
		return Config{}, fmt.Errorf("DATABASE_URL is required")
	}
	if len(cfg.KafkaBrokers) == 0 {
		return Config{}, fmt.Errorf("KAFKA_BROKERS or KAFKA_BOOTSTRAP_SERVERS is required")
	}
	if cfg.ConsumerGroup == "" {
		return Config{}, fmt.Errorf("FABRIC_ADAPTER_CONSUMER_GROUP is required")
	}
	if cfg.AuditAnchorTopic == "" || cfg.FabricRequestsTopic == "" {
		return Config{}, fmt.Errorf("TOPIC_AUDIT_ANCHOR and TOPIC_FABRIC_REQUESTS are required")
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
