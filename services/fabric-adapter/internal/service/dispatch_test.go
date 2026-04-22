package service

import (
	"testing"

	"datab.local/fabric-adapter/internal/model"
	"datab.local/fabric-adapter/internal/provider"
)

func TestDispatcherBuildsEvidenceBatchRootRequest(t *testing.T) {
	dispatcher := NewDispatcher()
	envelope := mustEnvelope(t, "dtp.audit.anchor", `{
	  "event_id":"evt-anchor",
	  "event_type":"audit.anchor_requested",
	  "event_version":1,
	  "producer_service":"platform-core.audit",
	  "aggregate_type":"audit.anchor_batch",
	  "aggregate_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "anchor_batch_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "chain_anchor_id":"cbf626f3-6c84-46d1-bb9a-605b6ecadf31",
	  "batch_root":"root-001",
	  "chain_id":"fabric-local"
	}`)

	request, err := dispatcher.BuildRequest(envelope)
	if err != nil {
		t.Fatalf("BuildRequest() error = %v", err)
	}

	if got, want := request.SubmissionKind, provider.SubmissionKindEvidenceBatchRoot; got != want {
		t.Fatalf("SubmissionKind = %q, want %q", got, want)
	}
	if got, want := request.ContractName, "evidence_batch_root"; got != want {
		t.Fatalf("ContractName = %q, want %q", got, want)
	}
	if got, want := request.TransactionName, "SubmitEvidenceBatchRoot"; got != want {
		t.Fatalf("TransactionName = %q, want %q", got, want)
	}
	if got, want := request.AnchorBatchID, "8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d"; got != want {
		t.Fatalf("AnchorBatchID = %q, want %q", got, want)
	}
	if got, want := request.SummaryDigest, "root-001"; got != want {
		t.Fatalf("SummaryDigest = %q, want %q", got, want)
	}
}

func TestDispatcherBuildsProofSummaryRequests(t *testing.T) {
	testCases := []struct {
		name            string
		summaryType     string
		wantKind        provider.SubmissionKind
		wantContract    string
		wantTransaction string
	}{
		{
			name:            "order",
			summaryType:     "order_summary",
			wantKind:        provider.SubmissionKindOrderSummary,
			wantContract:    "order_digest",
			wantTransaction: "SubmitOrderDigest",
		},
		{
			name:            "authorization",
			summaryType:     "authorization_summary",
			wantKind:        provider.SubmissionKindAuthorization,
			wantContract:    "authorization_digest",
			wantTransaction: "SubmitAuthorizationDigest",
		},
		{
			name:            "acceptance",
			summaryType:     "acceptance_summary",
			wantKind:        provider.SubmissionKindAcceptance,
			wantContract:    "acceptance_digest",
			wantTransaction: "SubmitAcceptanceDigest",
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			dispatcher := NewDispatcher()
			envelope := mustEnvelope(t, "dtp.fabric.requests", `{
			  "event_id":"evt-proof",
			  "event_type":"fabric.proof_submit_requested",
			  "event_version":1,
			  "producer_service":"platform-core.integration",
			  "aggregate_type":"chain.chain_anchor",
			  "aggregate_id":"33333333-3333-4333-8333-333333333333",
			  "chain_anchor_id":"33333333-3333-4333-8333-333333333333",
			  "chain_id":"fabric-local",
			  "summary_type":"`+testCase.summaryType+`",
			  "summary_digest":"digest-`+testCase.name+`"
			}`)

			request, err := dispatcher.BuildRequest(envelope)
			if err != nil {
				t.Fatalf("BuildRequest() error = %v", err)
			}

			if got := request.SubmissionKind; got != testCase.wantKind {
				t.Fatalf("SubmissionKind = %q, want %q", got, testCase.wantKind)
			}
			if got := request.ContractName; got != testCase.wantContract {
				t.Fatalf("ContractName = %q, want %q", got, testCase.wantContract)
			}
			if got := request.TransactionName; got != testCase.wantTransaction {
				t.Fatalf("TransactionName = %q, want %q", got, testCase.wantTransaction)
			}
			if got, want := request.ChainAnchorID, "33333333-3333-4333-8333-333333333333"; got != want {
				t.Fatalf("ChainAnchorID = %q, want %q", got, want)
			}
		})
	}
}

func TestDispatcherRejectsUnsupportedSummaryType(t *testing.T) {
	dispatcher := NewDispatcher()
	envelope := mustEnvelope(t, "dtp.fabric.requests", `{
	  "event_id":"evt-proof",
	  "event_type":"fabric.proof_submit_requested",
	  "event_version":1,
	  "producer_service":"platform-core.integration",
	  "aggregate_type":"chain.chain_anchor",
	  "aggregate_id":"33333333-3333-4333-8333-333333333333",
	  "chain_anchor_id":"33333333-3333-4333-8333-333333333333",
	  "summary_type":"unsupported_kind",
	  "summary_digest":"digest-proof"
	}`)

	if _, err := dispatcher.BuildRequest(envelope); err == nil {
		t.Fatal("BuildRequest() expected error for unsupported summary_type")
	}
}

func mustEnvelope(t *testing.T, topic string, raw string) model.CanonicalEnvelope {
	t.Helper()

	envelope, err := model.DecodeCanonicalEnvelope(topic, []byte(raw))
	if err != nil {
		t.Fatalf("DecodeCanonicalEnvelope() error = %v", err)
	}
	return envelope
}
