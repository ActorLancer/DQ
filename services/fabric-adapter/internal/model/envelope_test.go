package model

import "testing"

func TestDecodeCanonicalEnvelopeKeepsFlattenedExtras(t *testing.T) {
	raw := []byte(`{
	  "event_id":"evt-001",
	  "event_type":"audit.anchor_requested",
	  "event_version":1,
	  "producer_service":"platform-core.audit",
	  "aggregate_type":"audit.anchor_batch",
	  "aggregate_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "request_id":"req-001",
	  "trace_id":"trace-001",
	  "event_schema_version":"v1",
	  "authority_scope":"audit",
	  "source_of_truth":"database",
	  "proof_commit_policy":"async_evidence",
	  "anchor_batch_id":"8f3aa947-8e7d-4f1f-b2f2-6bf63bfbdb4d",
	  "chain_anchor_id":"cbf626f3-6c84-46d1-bb9a-605b6ecadf31",
	  "payload":{"batch_root":"root-001"}
	}`)

	env, err := DecodeCanonicalEnvelope("dtp.audit.anchor", raw)
	if err != nil {
		t.Fatalf("DecodeCanonicalEnvelope() error = %v", err)
	}

	if got, want := env.EventType, "audit.anchor_requested"; got != want {
		t.Fatalf("EventType = %q, want %q", got, want)
	}
	if got, want := env.AggregateType, "audit.anchor_batch"; got != want {
		t.Fatalf("AggregateType = %q, want %q", got, want)
	}
	if _, ok := env.FindString("chain_anchor_id"); !ok {
		t.Fatalf("expected chain_anchor_id in extras")
	}
	if _, ok := env.Extras["anchor_batch_id"]; !ok {
		t.Fatalf("expected anchor_batch_id in extras")
	}
}
