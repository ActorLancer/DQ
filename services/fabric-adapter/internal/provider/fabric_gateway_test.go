package provider

import (
	"testing"

	"datab.local/fabric-adapter/internal/model"
)

func TestSubmissionArgumentsForEvidenceBatchRoot(t *testing.T) {
	request := SubmissionRequest{
		SubmissionKind: SubmissionKindEvidenceBatchRoot,
		ChainID:        "fabric-test-network",
		SummaryDigest:  "digest-001",
		AnchorBatchID:  "anchor-batch-001",
		ChainAnchorID:  "chain-anchor-001",
		ReferenceType:  "anchor_batch",
		ReferenceID:    "anchor-batch-001",
		Envelope: model.CanonicalEnvelope{
			RequestID: "req-001",
			TraceID:   "trace-001",
		},
	}

	arguments, err := submissionArguments(request)
	if err != nil {
		t.Fatalf("submissionArguments() error = %v", err)
	}
	if len(arguments) != 8 {
		t.Fatalf("submissionArguments length = %d, want 8", len(arguments))
	}
	if arguments[0] != "anchor-batch-001" || arguments[1] != "chain-anchor-001" {
		t.Fatalf("unexpected arguments = %#v", arguments)
	}
}

func TestNormalizeGatewayEndpointAddsDNSPrefix(t *testing.T) {
	if got, want := normalizeGatewayEndpoint("localhost:7051"), "dns:///localhost:7051"; got != want {
		t.Fatalf("normalizeGatewayEndpoint() = %q, want %q", got, want)
	}
	if got, want := normalizeGatewayEndpoint("dns:///localhost:7051"), "dns:///localhost:7051"; got != want {
		t.Fatalf("normalizeGatewayEndpoint() = %q, want %q", got, want)
	}
}

func TestParseChaincodeResultFallsBackToRawText(t *testing.T) {
	result := parseChaincodeResult([]byte("plain-text"))
	if result["raw_result"] != "plain-text" {
		t.Fatalf("parseChaincodeResult() = %#v", result)
	}
}
