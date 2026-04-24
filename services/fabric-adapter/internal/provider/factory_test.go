package provider

import (
	"testing"

	adapterconfig "datab.local/fabric-adapter/internal/config"
)

func TestNewSubmissionProviderUsesMockProvider(t *testing.T) {
	cfg := adapterconfig.Config{
		ProviderMode:  "mock",
		ChannelName:   "datab-channel",
		ChaincodeName: "datab-audit-anchor",
	}

	submitter, err := NewSubmissionProvider(cfg)
	if err != nil {
		t.Fatalf("NewSubmissionProvider() error = %v", err)
	}

	if _, ok := submitter.(*MockProvider); !ok {
		t.Fatalf("NewSubmissionProvider() = %T, want *MockProvider", submitter)
	}
}

func TestNewSubmissionProviderRejectsUnsupportedProviderMode(t *testing.T) {
	cfg := adapterconfig.Config{ProviderMode: "unsupported"}

	if _, err := NewSubmissionProvider(cfg); err == nil {
		t.Fatalf("expected unsupported provider mode error")
	}
}
