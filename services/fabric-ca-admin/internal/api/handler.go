package api

import (
	"context"
	"encoding/json"
	"errors"
	"log/slog"
	"net/http"
	"strings"
	"time"

	"datab.local/fabric-ca-admin/internal/model"
	"datab.local/fabric-ca-admin/internal/service"
	"datab.local/fabric-ca-admin/internal/store"
)

type Handler struct {
	service *service.Service
	logger  *slog.Logger
}

func New(service *service.Service, logger *slog.Logger) *Handler {
	return &Handler{
		service: service,
		logger:  logger,
	}
}

func (handler *Handler) Routes() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /healthz", handler.healthz)
	mux.HandleFunc("POST /internal/fabric-identities/{id}/issue", handler.issueIdentity)
	mux.HandleFunc("POST /internal/fabric-identities/{id}/revoke", handler.revokeIdentity)
	mux.HandleFunc("POST /internal/certificates/{id}/revoke", handler.revokeCertificate)
	return mux
}

func (handler *Handler) healthz(w http.ResponseWriter, r *http.Request) {
	ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
	defer cancel()

	if err := handler.service.Ping(ctx); err != nil {
		writeError(w, http.StatusServiceUnavailable, "FABRIC_CA_ADMIN_UNAVAILABLE", err.Error())
		return
	}

	writeJSON(w, http.StatusOK, map[string]any{
		"status":  "ok",
		"service": "fabric-ca-admin",
	})
}

func (handler *Handler) issueIdentity(w http.ResponseWriter, r *http.Request) {
	actor, ok := requireActorContext(w, r, "iam.fabric_identity.issue")
	if !ok {
		return
	}
	result, err := handler.service.IssueIdentity(r.Context(), r.PathValue("id"), actor)
	if err != nil {
		handler.respondError(w, err)
		return
	}
	writeJSON(w, http.StatusOK, result)
}

func (handler *Handler) revokeIdentity(w http.ResponseWriter, r *http.Request) {
	actor, ok := requireActorContext(w, r, "iam.fabric_identity.revoke")
	if !ok {
		return
	}
	result, err := handler.service.RevokeIdentity(r.Context(), r.PathValue("id"), actor)
	if err != nil {
		handler.respondError(w, err)
		return
	}
	writeJSON(w, http.StatusOK, result)
}

func (handler *Handler) revokeCertificate(w http.ResponseWriter, r *http.Request) {
	actor, ok := requireActorContext(w, r, "iam.certificate.revoke")
	if !ok {
		return
	}
	result, err := handler.service.RevokeCertificate(r.Context(), r.PathValue("id"), actor)
	if err != nil {
		handler.respondError(w, err)
		return
	}
	writeJSON(w, http.StatusOK, result)
}

func (handler *Handler) respondError(w http.ResponseWriter, err error) {
	var actionErr *store.ActionError
	if errors.As(err, &actionErr) {
		writeError(w, actionErr.StatusCode, actionErr.Code, actionErr.Message)
		return
	}
	handler.logger.Error("fabric-ca-admin request failed", "error", err)
	writeError(w, http.StatusInternalServerError, "FABRIC_CA_ADMIN_INTERNAL_ERROR", err.Error())
}

func requireActorContext(
	w http.ResponseWriter,
	r *http.Request,
	expectedPermission string,
) (model.ActorContext, bool) {
	actor := model.ActorContext{
		RequestID:         strings.TrimSpace(r.Header.Get("x-request-id")),
		TraceID:           strings.TrimSpace(r.Header.Get("x-trace-id")),
		ActorRole:         strings.TrimSpace(r.Header.Get("x-role")),
		ActorUserID:       strings.TrimSpace(r.Header.Get("x-user-id")),
		StepUpChallengeID: strings.TrimSpace(r.Header.Get("x-step-up-challenge-id")),
		PermissionCode:    strings.TrimSpace(r.Header.Get("x-permission-code")),
	}

	if actor.ActorRole == "" {
		writeError(w, http.StatusForbidden, "FABRIC_CA_ACTION_FORBIDDEN", "x-role is required")
		return model.ActorContext{}, false
	}
	if actor.ActorUserID == "" {
		writeError(w, http.StatusBadRequest, "FABRIC_CA_ACTOR_REQUIRED", "x-user-id is required")
		return model.ActorContext{}, false
	}
	if actor.StepUpChallengeID == "" {
		writeError(w, http.StatusBadRequest, "STEP_UP_REQUIRED", "x-step-up-challenge-id is required")
		return model.ActorContext{}, false
	}
	if actor.PermissionCode != expectedPermission {
		writeError(w, http.StatusForbidden, "FABRIC_CA_ACTION_FORBIDDEN", "unexpected x-permission-code")
		return model.ActorContext{}, false
	}
	return actor, true
}

func writeJSON(w http.ResponseWriter, statusCode int, payload any) {
	w.Header().Set("content-type", "application/json")
	w.WriteHeader(statusCode)
	_ = json.NewEncoder(w).Encode(payload)
}

func writeError(w http.ResponseWriter, statusCode int, code string, message string) {
	writeJSON(w, statusCode, map[string]string{
		"code":    code,
		"message": message,
	})
}
