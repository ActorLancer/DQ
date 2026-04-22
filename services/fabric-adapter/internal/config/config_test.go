package config

import (
	"os"
	"testing"
)

func TestLoadUsesCanonicalFabricDefaults(t *testing.T) {
	t.Setenv("DATABASE_URL", "postgres://datab:datab_local_pass@127.0.0.1:5432/datab")
	t.Setenv("KAFKA_BROKERS", "127.0.0.1:9094")
	t.Setenv("TOPIC_AUDIT_ANCHOR", "")
	t.Setenv("TOPIC_FABRIC_REQUESTS", "")
	t.Setenv("FABRIC_ADAPTER_CONSUMER_GROUP", "")

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	if got, want := cfg.AuditAnchorTopic, "dtp.audit.anchor"; got != want {
		t.Fatalf("AuditAnchorTopic = %q, want %q", got, want)
	}
	if got, want := cfg.FabricRequestsTopic, "dtp.fabric.requests"; got != want {
		t.Fatalf("FabricRequestsTopic = %q, want %q", got, want)
	}
	if got, want := cfg.ConsumerGroup, "cg-fabric-adapter"; got != want {
		t.Fatalf("ConsumerGroup = %q, want %q", got, want)
	}
}

func TestLoadRequiresDatabaseURL(t *testing.T) {
	t.Setenv("DATABASE_URL", "")
	t.Setenv("FABRIC_ADAPTER_DATABASE_URL", "")
	t.Setenv("KAFKA_BROKERS", "127.0.0.1:9094")

	if _, err := Load(); err == nil {
		t.Fatalf("expected missing DATABASE_URL error")
	}
}

func TestSplitCSVIgnoresEmptyValues(t *testing.T) {
	got := splitCSV("127.0.0.1:9094, ,kafka:9092")
	if len(got) != 2 {
		t.Fatalf("splitCSV length = %d, want 2", len(got))
	}
	if got[0] != "127.0.0.1:9094" || got[1] != "kafka:9092" {
		t.Fatalf("splitCSV = %#v", got)
	}
}

func TestMain(m *testing.M) {
	os.Exit(m.Run())
}
