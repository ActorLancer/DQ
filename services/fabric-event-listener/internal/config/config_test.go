package config

import (
	"os"
	"testing"
)

func TestLoadUsesCanonicalDefaults(t *testing.T) {
	t.Setenv("DATABASE_URL", "postgres://datab:datab_local_pass@127.0.0.1:5432/datab")
	t.Setenv("KAFKA_BROKERS", "127.0.0.1:9094")
	t.Setenv("TOPIC_FABRIC_CALLBACKS", "")
	t.Setenv("FABRIC_EVENT_LISTENER_PROVIDER_MODE", "")

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	if got, want := cfg.CallbackTopic, "dtp.fabric.callbacks"; got != want {
		t.Fatalf("CallbackTopic = %q, want %q", got, want)
	}
	if got, want := cfg.ProviderMode, "mock"; got != want {
		t.Fatalf("ProviderMode = %q, want %q", got, want)
	}
	if got, want := cfg.BatchSize, 16; got != want {
		t.Fatalf("BatchSize = %d, want %d", got, want)
	}
}

func TestLoadRequiresDatabaseURL(t *testing.T) {
	t.Setenv("DATABASE_URL", "")
	t.Setenv("FABRIC_EVENT_LISTENER_DATABASE_URL", "")
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
