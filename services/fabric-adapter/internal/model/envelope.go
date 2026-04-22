package model

import (
	"encoding/json"
	"fmt"
	"strings"
)

type CanonicalEnvelope struct {
	Topic              string
	EventID            string
	EventType          string
	EventVersion       int
	OccurredAt         string
	ProducerService    string
	AggregateType      string
	AggregateID        string
	RequestID          string
	TraceID            string
	IdempotencyKey     string
	EventSchemaVersion string
	AuthorityScope     string
	SourceOfTruth      string
	ProofCommitPolicy  string
	Payload            json.RawMessage
	Extras             map[string]json.RawMessage
}

var reservedEnvelopeKeys = map[string]struct{}{
	"event_id":             {},
	"event_type":           {},
	"event_version":        {},
	"occurred_at":          {},
	"producer_service":     {},
	"aggregate_type":       {},
	"aggregate_id":         {},
	"request_id":           {},
	"trace_id":             {},
	"idempotency_key":      {},
	"event_schema_version": {},
	"authority_scope":      {},
	"source_of_truth":      {},
	"proof_commit_policy":  {},
	"payload":              {},
}

func DecodeCanonicalEnvelope(topic string, raw []byte) (CanonicalEnvelope, error) {
	fields := map[string]json.RawMessage{}
	if err := json.Unmarshal(raw, &fields); err != nil {
		return CanonicalEnvelope{}, fmt.Errorf("decode envelope json: %w", err)
	}

	env := CanonicalEnvelope{
		Topic:   topic,
		Payload: raw,
		Extras:  map[string]json.RawMessage{},
	}

	if payload, ok := fields["payload"]; ok && len(payload) > 0 {
		env.Payload = payload
	}

	decodeString(fields, "event_id", &env.EventID)
	decodeString(fields, "event_type", &env.EventType)
	decodeInt(fields, "event_version", &env.EventVersion)
	decodeString(fields, "occurred_at", &env.OccurredAt)
	decodeString(fields, "producer_service", &env.ProducerService)
	decodeString(fields, "aggregate_type", &env.AggregateType)
	decodeString(fields, "aggregate_id", &env.AggregateID)
	decodeString(fields, "request_id", &env.RequestID)
	decodeString(fields, "trace_id", &env.TraceID)
	decodeString(fields, "idempotency_key", &env.IdempotencyKey)
	decodeString(fields, "event_schema_version", &env.EventSchemaVersion)
	decodeString(fields, "authority_scope", &env.AuthorityScope)
	decodeString(fields, "source_of_truth", &env.SourceOfTruth)
	decodeString(fields, "proof_commit_policy", &env.ProofCommitPolicy)

	for key, value := range fields {
		if _, reserved := reservedEnvelopeKeys[key]; reserved {
			continue
		}
		env.Extras[key] = value
	}

	if strings.TrimSpace(env.EventID) == "" {
		return CanonicalEnvelope{}, fmt.Errorf("event_id is required")
	}
	if strings.TrimSpace(env.EventType) == "" {
		return CanonicalEnvelope{}, fmt.Errorf("event_type is required")
	}
	if strings.TrimSpace(env.AggregateType) == "" {
		return CanonicalEnvelope{}, fmt.Errorf("aggregate_type is required")
	}
	if strings.TrimSpace(env.AggregateID) == "" {
		return CanonicalEnvelope{}, fmt.Errorf("aggregate_id is required")
	}

	return env, nil
}

func (env CanonicalEnvelope) FindString(key string) (string, bool) {
	if value, ok := env.Extras[key]; ok {
		var text string
		if err := json.Unmarshal(value, &text); err == nil && strings.TrimSpace(text) != "" {
			return text, true
		}
	}

	payload := map[string]json.RawMessage{}
	if err := json.Unmarshal(env.Payload, &payload); err == nil {
		if value, ok := payload[key]; ok {
			var text string
			if err := json.Unmarshal(value, &text); err == nil && strings.TrimSpace(text) != "" {
				return text, true
			}
		}
	}

	return "", false
}

func (env CanonicalEnvelope) PayloadObject() map[string]any {
	payload := map[string]any{}
	_ = json.Unmarshal(env.Payload, &payload)
	return payload
}

func (env CanonicalEnvelope) ExtraObject() map[string]any {
	extras := map[string]any{}
	for key, value := range env.Extras {
		var decoded any
		if err := json.Unmarshal(value, &decoded); err != nil {
			extras[key] = string(value)
			continue
		}
		extras[key] = decoded
	}
	return extras
}

func decodeString(fields map[string]json.RawMessage, key string, target *string) {
	value, ok := fields[key]
	if !ok {
		return
	}
	_ = json.Unmarshal(value, target)
}

func decodeInt(fields map[string]json.RawMessage, key string, target *int) {
	value, ok := fields[key]
	if !ok {
		return
	}
	_ = json.Unmarshal(value, target)
}
