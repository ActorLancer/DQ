package provider

import (
	"fmt"

	adapterconfig "datab.local/fabric-adapter/internal/config"
)

func NewSubmissionProvider(cfg adapterconfig.Config) (SubmissionProvider, error) {
	switch cfg.ProviderMode {
	case "mock":
		return NewMock(cfg.ChannelName, cfg.ChaincodeName), nil
	case "fabric-test-network":
		return NewFabricGatewayProvider(cfg)
	default:
		return nil, fmt.Errorf("unsupported FABRIC_ADAPTER_PROVIDER_MODE %q", cfg.ProviderMode)
	}
}
