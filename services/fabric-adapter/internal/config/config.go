package config

import (
	"fmt"
	"os"
	"strconv"
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
	ProviderMode        string
	ChannelName         string
	ChaincodeName       string
	ChaincodeVersion    string
	ChaincodeSequence   int
	GatewayEndpoint     string
	GatewayPeer         string
	MSPID               string
	TLSCertPath         string
	SignCertPath        string
	PrivateKeyDir       string
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
		ProviderMode:        firstNonEmpty(os.Getenv("FABRIC_ADAPTER_PROVIDER_MODE"), "mock"),
		ChannelName:         firstNonEmpty(os.Getenv("FABRIC_CHANNEL_NAME"), "datab-channel"),
		ChaincodeName:       firstNonEmpty(os.Getenv("FABRIC_CHAINCODE_NAME"), "datab-audit-anchor"),
		ChaincodeVersion:    firstNonEmpty(os.Getenv("FABRIC_CHAINCODE_VERSION"), "1.0"),
		ChaincodeSequence:   parseIntEnv("FABRIC_CHAINCODE_SEQUENCE", 1),
		GatewayEndpoint:     strings.TrimSpace(os.Getenv("FABRIC_GATEWAY_ENDPOINT")),
		GatewayPeer:         firstNonEmpty(os.Getenv("FABRIC_GATEWAY_PEER"), "peer0.org1.example.com"),
		MSPID:               strings.TrimSpace(os.Getenv("FABRIC_MSP_ID")),
		TLSCertPath:         strings.TrimSpace(os.Getenv("FABRIC_TLS_CERT_PATH")),
		SignCertPath:        strings.TrimSpace(os.Getenv("FABRIC_SIGN_CERT_PATH")),
		PrivateKeyDir:       strings.TrimSpace(os.Getenv("FABRIC_PRIVATE_KEY_DIR")),
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
	if cfg.ProviderMode == "" {
		return Config{}, fmt.Errorf("FABRIC_ADAPTER_PROVIDER_MODE is required")
	}
	if cfg.ProviderMode == "fabric-test-network" {
		if cfg.GatewayEndpoint == "" {
			return Config{}, fmt.Errorf("FABRIC_GATEWAY_ENDPOINT is required when FABRIC_ADAPTER_PROVIDER_MODE=fabric-test-network")
		}
		if cfg.GatewayPeer == "" {
			return Config{}, fmt.Errorf("FABRIC_GATEWAY_PEER is required when FABRIC_ADAPTER_PROVIDER_MODE=fabric-test-network")
		}
		if cfg.MSPID == "" {
			return Config{}, fmt.Errorf("FABRIC_MSP_ID is required when FABRIC_ADAPTER_PROVIDER_MODE=fabric-test-network")
		}
		if cfg.TLSCertPath == "" {
			return Config{}, fmt.Errorf("FABRIC_TLS_CERT_PATH is required when FABRIC_ADAPTER_PROVIDER_MODE=fabric-test-network")
		}
		if cfg.SignCertPath == "" {
			return Config{}, fmt.Errorf("FABRIC_SIGN_CERT_PATH is required when FABRIC_ADAPTER_PROVIDER_MODE=fabric-test-network")
		}
		if cfg.PrivateKeyPath == "" && cfg.PrivateKeyDir == "" {
			return Config{}, fmt.Errorf("FABRIC_PRIVATE_KEY_PATH or FABRIC_PRIVATE_KEY_DIR is required when FABRIC_ADAPTER_PROVIDER_MODE=fabric-test-network")
		}
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
