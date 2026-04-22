package config

import "testing"

func TestLoadDefaults(t *testing.T) {
	t.Setenv("DATABASE_URL", "postgres://datab:datab_local_pass@127.0.0.1:5432/datab")
	t.Setenv("FABRIC_CA_ADMIN_PORT", "")
	t.Setenv("FABRIC_CA_ADMIN_LISTEN_ADDR", "")
	t.Setenv("FABRIC_CA_ADMIN_MODE", "")

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}

	if cfg.ServiceName != "fabric-ca-admin" {
		t.Fatalf("ServiceName = %s", cfg.ServiceName)
	}
	if cfg.ListenAddr != "127.0.0.1:18112" {
		t.Fatalf("ListenAddr = %s", cfg.ListenAddr)
	}
	if cfg.Mode != "mock" {
		t.Fatalf("Mode = %s", cfg.Mode)
	}
}
