package provider

import (
	"context"
	"crypto/x509"
	"encoding/json"
	"fmt"
	"os"
	"path"
	"strings"
	"time"

	adapterconfig "datab.local/fabric-adapter/internal/config"

	"github.com/hyperledger/fabric-gateway/pkg/client"
	"github.com/hyperledger/fabric-gateway/pkg/hash"
	"github.com/hyperledger/fabric-gateway/pkg/identity"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
)

type FabricGatewayProvider struct {
	connection      *grpc.ClientConn
	gateway         *client.Gateway
	network         *client.Network
	channelName     string
	chaincodeName   string
	gatewayEndpoint string
	gatewayPeer     string
	providerTimeout time.Duration
}

func NewFabricGatewayProvider(cfg adapterconfig.Config) (*FabricGatewayProvider, error) {
	connection, err := newFabricGrpcConnection(cfg)
	if err != nil {
		return nil, err
	}

	id, err := newFabricIdentity(cfg)
	if err != nil {
		connection.Close()
		return nil, err
	}

	sign, err := newFabricSigner(cfg)
	if err != nil {
		connection.Close()
		return nil, err
	}

	commitTimeout := cfg.ProviderTimeout * 4
	if commitTimeout < 30*time.Second {
		commitTimeout = 30 * time.Second
	}

	gateway, err := client.Connect(
		id,
		client.WithSign(sign),
		client.WithHash(hash.SHA256),
		client.WithClientConnection(connection),
		client.WithEvaluateTimeout(5*time.Second),
		client.WithEndorseTimeout(cfg.ProviderTimeout),
		client.WithSubmitTimeout(cfg.ProviderTimeout),
		client.WithCommitStatusTimeout(commitTimeout),
	)
	if err != nil {
		connection.Close()
		return nil, fmt.Errorf("connect fabric gateway: %w", err)
	}

	return &FabricGatewayProvider{
		connection:      connection,
		gateway:         gateway,
		network:         gateway.GetNetwork(cfg.ChannelName),
		channelName:     cfg.ChannelName,
		chaincodeName:   cfg.ChaincodeName,
		gatewayEndpoint: cfg.GatewayEndpoint,
		gatewayPeer:     cfg.GatewayPeer,
		providerTimeout: cfg.ProviderTimeout,
	}, nil
}

func (provider *FabricGatewayProvider) Close() error {
	if provider.gateway != nil {
		provider.gateway.Close()
	}
	if provider.connection != nil {
		return provider.connection.Close()
	}
	return nil
}

func (provider *FabricGatewayProvider) Submit(
	ctx context.Context,
	request SubmissionRequest,
) (SubmissionReceipt, error) {
	contract := provider.network.GetContract(provider.chaincodeName)
	arguments, err := submissionArguments(request)
	if err != nil {
		return SubmissionReceipt{}, err
	}

	submitResult, commit, err := contract.SubmitAsync(
		request.TransactionName,
		client.WithArguments(arguments...),
	)
	if err != nil {
		return SubmissionReceipt{}, fmt.Errorf(
			"submit fabric transaction %s: %w",
			request.TransactionName,
			err,
		)
	}

	chaincodeResult := parseChaincodeResult(submitResult)
	providerReference := ""
	if commit != nil {
		providerReference = strings.TrimSpace(commit.TransactionID())
	}
	if providerReference == "" {
		if txID, ok := chaincodeResult["transaction_id"].(string); ok {
			providerReference = txID
		}
	}
	if providerReference == "" {
		return SubmissionReceipt{}, fmt.Errorf("fabric gateway returned empty transaction id")
	}

	statusCtx, cancel := context.WithTimeout(ctx, provider.providerTimeout*4)
	defer cancel()

	commitStatus := "unknown"
	var blockNumber uint64
	if commit != nil {
		status, err := commit.StatusWithContext(statusCtx)
		if err != nil {
			return SubmissionReceipt{}, fmt.Errorf(
				"wait for commit status %s: %w",
				providerReference,
				err,
			)
		}
		commitStatus = status.Code.String()
		blockNumber = status.BlockNumber
		if !status.Successful {
			return SubmissionReceipt{}, fmt.Errorf(
				"fabric transaction %s committed with status %s",
				status.TransactionID,
				status.Code.String(),
			)
		}
	}

	chainID := request.ChainID
	if chainID == "" {
		chainID = "fabric-test-network"
	}

	return SubmissionReceipt{
		ProviderType:      "fabric_gateway",
		ProviderKey:       "fabric-test-network",
		ProviderReference: providerReference,
		ReceiptStatus:     "committed",
		OccurredAt:        time.Now().UTC(),
		ReceiptPayload: map[string]any{
			"mode":              "fabric-test-network",
			"chain_id":          chainID,
			"channel_name":      provider.channelName,
			"chaincode_name":    provider.chaincodeName,
			"event_type":        request.Envelope.EventType,
			"aggregate_type":    request.Envelope.AggregateType,
			"aggregate_id":      request.Envelope.AggregateID,
			"submission_kind":   string(request.SubmissionKind),
			"contract_name":     request.ContractName,
			"transaction_name":  request.TransactionName,
			"summary_type":      request.SummaryType,
			"summary_digest":    request.SummaryDigest,
			"anchor_batch_id":   request.AnchorBatchID,
			"chain_anchor_id":   request.ChainAnchorID,
			"reference_type":    request.ReferenceType,
			"reference_id":      request.ReferenceID,
			"request_id":        request.Envelope.RequestID,
			"trace_id":          request.Envelope.TraceID,
			"tx_id":             providerReference,
			"gateway_status":    "committed",
			"commit_status":     commitStatus,
			"commit_block":      blockNumber,
			"gateway_endpoint":  provider.gatewayEndpoint,
			"gateway_peer":      provider.gatewayPeer,
			"submitted_via":     "fabric-gateway",
			"chaincode_result":  chaincodeResult,
			"business_payload":  request.Envelope.PayloadObject(),
			"flattened_payload": request.Envelope.ExtraObject(),
			"submitter_service": "fabric-adapter",
			"submission_summary": fmt.Sprintf(
				"%s/%s -> %s",
				request.Envelope.Topic,
				request.SubmissionKind,
				providerReference,
			),
		},
	}, nil
}

func newFabricGrpcConnection(cfg adapterconfig.Config) (*grpc.ClientConn, error) {
	certificatePEM, err := os.ReadFile(cfg.TLSCertPath)
	if err != nil {
		return nil, fmt.Errorf("read fabric TLS certificate %s: %w", cfg.TLSCertPath, err)
	}

	certificate, err := identity.CertificateFromPEM(certificatePEM)
	if err != nil {
		return nil, fmt.Errorf("parse fabric TLS certificate: %w", err)
	}

	certPool := x509.NewCertPool()
	certPool.AddCert(certificate)
	transportCredentials := credentials.NewClientTLSFromCert(certPool, cfg.GatewayPeer)

	connection, err := grpc.NewClient(
		normalizeGatewayEndpoint(cfg.GatewayEndpoint),
		grpc.WithTransportCredentials(transportCredentials),
	)
	if err != nil {
		return nil, fmt.Errorf("create fabric gRPC connection: %w", err)
	}
	return connection, nil
}

func newFabricIdentity(cfg adapterconfig.Config) (*identity.X509Identity, error) {
	certificatePEM, err := os.ReadFile(cfg.SignCertPath)
	if err != nil {
		return nil, fmt.Errorf("read fabric sign certificate %s: %w", cfg.SignCertPath, err)
	}

	certificate, err := identity.CertificateFromPEM(certificatePEM)
	if err != nil {
		return nil, fmt.Errorf("parse fabric sign certificate: %w", err)
	}

	id, err := identity.NewX509Identity(cfg.MSPID, certificate)
	if err != nil {
		return nil, fmt.Errorf("create fabric x509 identity: %w", err)
	}
	return id, nil
}

func newFabricSigner(cfg adapterconfig.Config) (identity.Sign, error) {
	privateKeyPath := strings.TrimSpace(cfg.PrivateKeyPath)
	if privateKeyPath == "" {
		privateKeyPEM, err := readFirstFile(cfg.PrivateKeyDir)
		if err != nil {
			return nil, fmt.Errorf("read fabric private key from %s: %w", cfg.PrivateKeyDir, err)
		}
		privateKey, err := identity.PrivateKeyFromPEM(privateKeyPEM)
		if err != nil {
			return nil, fmt.Errorf("parse fabric private key: %w", err)
		}
		return identity.NewPrivateKeySign(privateKey)
	}

	privateKeyPEM, err := os.ReadFile(privateKeyPath)
	if err != nil {
		return nil, fmt.Errorf("read fabric private key %s: %w", privateKeyPath, err)
	}
	privateKey, err := identity.PrivateKeyFromPEM(privateKeyPEM)
	if err != nil {
		return nil, fmt.Errorf("parse fabric private key: %w", err)
	}
	return identity.NewPrivateKeySign(privateKey)
}

func readFirstFile(dirPath string) ([]byte, error) {
	dir, err := os.Open(dirPath)
	if err != nil {
		return nil, err
	}

	fileNames, err := dir.Readdirnames(1)
	if err != nil {
		return nil, err
	}
	return os.ReadFile(path.Join(dirPath, fileNames[0]))
}

func normalizeGatewayEndpoint(endpoint string) string {
	trimmed := strings.TrimSpace(endpoint)
	if trimmed == "" {
		return "dns:///localhost:7051"
	}
	if strings.HasPrefix(trimmed, "dns:///") || strings.Contains(trimmed, "://") {
		return trimmed
	}
	return "dns:///" + trimmed
}

func submissionArguments(request SubmissionRequest) ([]string, error) {
	requestID := strings.TrimSpace(request.Envelope.RequestID)
	traceID := strings.TrimSpace(request.Envelope.TraceID)

	switch request.SubmissionKind {
	case SubmissionKindEvidenceBatchRoot:
		return []string{
			request.AnchorBatchID,
			request.ChainAnchorID,
			request.ChainID,
			request.SummaryDigest,
			requestID,
			traceID,
			request.ReferenceType,
			request.ReferenceID,
		}, nil
	case SubmissionKindOrderSummary, SubmissionKindAuthorization, SubmissionKindAcceptance:
		return []string{
			request.ChainAnchorID,
			request.ChainID,
			request.SummaryDigest,
			requestID,
			traceID,
			request.ReferenceType,
			request.ReferenceID,
		}, nil
	default:
		return nil, fmt.Errorf("unsupported submission kind %q", request.SubmissionKind)
	}
}

func parseChaincodeResult(submitResult []byte) map[string]any {
	if len(submitResult) == 0 {
		return map[string]any{}
	}

	var result map[string]any
	if err := json.Unmarshal(submitResult, &result); err == nil && result != nil {
		return result
	}

	return map[string]any{
		"raw_result": string(submitResult),
	}
}
