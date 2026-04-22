package service

import (
	"fmt"

	"datab.local/fabric-adapter/internal/model"
	"datab.local/fabric-adapter/internal/provider"
)

type SubmissionHandler interface {
	Kind() provider.SubmissionKind
	BuildRequest(model.CanonicalEnvelope) (provider.SubmissionRequest, error)
}

type Dispatcher struct {
	handlers map[provider.SubmissionKind]SubmissionHandler
}

func NewDispatcher(handlers ...SubmissionHandler) *Dispatcher {
	if len(handlers) == 0 {
		handlers = []SubmissionHandler{
			evidenceBatchRootHandler{},
			orderSummaryHandler{},
			authorizationSummaryHandler{},
			acceptanceSummaryHandler{},
		}
	}

	registry := make(map[provider.SubmissionKind]SubmissionHandler, len(handlers))
	for _, handler := range handlers {
		registry[handler.Kind()] = handler
	}

	return &Dispatcher{handlers: registry}
}

func (dispatcher *Dispatcher) BuildRequest(
	envelope model.CanonicalEnvelope,
) (provider.SubmissionRequest, error) {
	kind, err := classifySubmissionKind(envelope)
	if err != nil {
		return provider.SubmissionRequest{}, err
	}

	handler, ok := dispatcher.handlers[kind]
	if !ok {
		return provider.SubmissionRequest{}, fmt.Errorf("no handler registered for %s", kind)
	}
	return handler.BuildRequest(envelope)
}

type evidenceBatchRootHandler struct{}

func (evidenceBatchRootHandler) Kind() provider.SubmissionKind {
	return provider.SubmissionKindEvidenceBatchRoot
}

func (evidenceBatchRootHandler) BuildRequest(
	envelope model.CanonicalEnvelope,
) (provider.SubmissionRequest, error) {
	anchorBatchID, err := requireString(envelope, "anchor_batch_id")
	if err != nil {
		return provider.SubmissionRequest{}, err
	}
	batchRoot, err := requireAnyString(envelope, "batch_root", "summary_digest", "digest")
	if err != nil {
		return provider.SubmissionRequest{}, err
	}

	chainAnchorID, _ := envelope.FindString("chain_anchor_id")
	return provider.SubmissionRequest{
		Envelope:        envelope,
		SubmissionKind:  provider.SubmissionKindEvidenceBatchRoot,
		ContractName:    "evidence_batch_root",
		TransactionName: "SubmitEvidenceBatchRoot",
		ChainID:         chainIDOrDefault(envelope),
		SummaryType:     string(provider.SubmissionKindEvidenceBatchRoot),
		SummaryDigest:   batchRoot,
		AnchorBatchID:   anchorBatchID,
		ChainAnchorID:   chainAnchorID,
		ReferenceType:   "anchor_batch",
		ReferenceID:     anchorBatchID,
	}, nil
}

type orderSummaryHandler struct{}

func (orderSummaryHandler) Kind() provider.SubmissionKind {
	return provider.SubmissionKindOrderSummary
}

func (orderSummaryHandler) BuildRequest(
	envelope model.CanonicalEnvelope,
) (provider.SubmissionRequest, error) {
	return buildProofSummaryRequest(
		envelope,
		provider.SubmissionKindOrderSummary,
		"order_digest",
		"SubmitOrderDigest",
	)
}

type authorizationSummaryHandler struct{}

func (authorizationSummaryHandler) Kind() provider.SubmissionKind {
	return provider.SubmissionKindAuthorization
}

func (authorizationSummaryHandler) BuildRequest(
	envelope model.CanonicalEnvelope,
) (provider.SubmissionRequest, error) {
	return buildProofSummaryRequest(
		envelope,
		provider.SubmissionKindAuthorization,
		"authorization_digest",
		"SubmitAuthorizationDigest",
	)
}

type acceptanceSummaryHandler struct{}

func (acceptanceSummaryHandler) Kind() provider.SubmissionKind {
	return provider.SubmissionKindAcceptance
}

func (acceptanceSummaryHandler) BuildRequest(
	envelope model.CanonicalEnvelope,
) (provider.SubmissionRequest, error) {
	return buildProofSummaryRequest(
		envelope,
		provider.SubmissionKindAcceptance,
		"acceptance_digest",
		"SubmitAcceptanceDigest",
	)
}

func buildProofSummaryRequest(
	envelope model.CanonicalEnvelope,
	kind provider.SubmissionKind,
	contractName string,
	transactionName string,
) (provider.SubmissionRequest, error) {
	chainAnchorID, err := chainAnchorIDForSummary(envelope)
	if err != nil {
		return provider.SubmissionRequest{}, err
	}
	summaryDigest, err := requireAnyString(envelope, "summary_digest", "digest")
	if err != nil {
		return provider.SubmissionRequest{}, err
	}

	return provider.SubmissionRequest{
		Envelope:        envelope,
		SubmissionKind:  kind,
		ContractName:    contractName,
		TransactionName: transactionName,
		ChainID:         chainIDOrDefault(envelope),
		SummaryType:     string(kind),
		SummaryDigest:   summaryDigest,
		ChainAnchorID:   chainAnchorID,
		ReferenceType:   "chain_anchor",
		ReferenceID:     chainAnchorID,
	}, nil
}

func classifySubmissionKind(envelope model.CanonicalEnvelope) (provider.SubmissionKind, error) {
	switch envelope.EventType {
	case "audit.anchor_requested":
		return provider.SubmissionKindEvidenceBatchRoot, nil
	case "fabric.proof_submit_requested":
		summaryType, err := requireString(envelope, "summary_type")
		if err != nil {
			return "", err
		}
		switch summaryType {
		case string(provider.SubmissionKindOrderSummary):
			return provider.SubmissionKindOrderSummary, nil
		case string(provider.SubmissionKindAuthorization):
			return provider.SubmissionKindAuthorization, nil
		case string(provider.SubmissionKindAcceptance):
			return provider.SubmissionKindAcceptance, nil
		default:
			return "", fmt.Errorf(
				"unsupported summary_type %q for fabric.proof_submit_requested",
				summaryType,
			)
		}
	default:
		return "", fmt.Errorf("unsupported event_type %q", envelope.EventType)
	}
}

func requireString(envelope model.CanonicalEnvelope, key string) (string, error) {
	if value, ok := envelope.FindString(key); ok {
		return value, nil
	}
	return "", fmt.Errorf("%s is required for %s", key, envelope.EventType)
}

func requireAnyString(
	envelope model.CanonicalEnvelope,
	keys ...string,
) (string, error) {
	for _, key := range keys {
		if value, ok := envelope.FindString(key); ok {
			return value, nil
		}
	}
	return "", fmt.Errorf("%v is required for %s", keys, envelope.EventType)
}

func chainIDOrDefault(envelope model.CanonicalEnvelope) string {
	if chainID, ok := envelope.FindString("chain_id"); ok {
		return chainID
	}
	return "fabric-local"
}

func chainAnchorIDForSummary(envelope model.CanonicalEnvelope) (string, error) {
	if chainAnchorID, ok := envelope.FindString("chain_anchor_id"); ok {
		return chainAnchorID, nil
	}
	switch envelope.AggregateType {
	case "chain.chain_anchor", "chain_anchor":
		return envelope.AggregateID, nil
	default:
		return "", fmt.Errorf("chain_anchor_id is required for %s", envelope.EventType)
	}
}
