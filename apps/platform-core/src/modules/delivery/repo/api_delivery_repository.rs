use crate::modules::delivery::dto::{CommitOrderDeliveryRequest, CommitOrderDeliveryResponseData};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, not_found, write_delivery_audit_event,
};
use crate::modules::order::domain::LayeredOrderStatus;
use crate::modules::order::repo::{
    ensure_order_deliverable_and_prepare_delivery, map_db_error, write_trade_audit_event,
};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn commit_api_delivery(
    client: &mut Client,
    order_id: &str,
    tenant_id: Option<&str>,
    payload: &CommitOrderDeliveryRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<CommitOrderDeliveryResponseData, (StatusCode, Json<ErrorResponse>)> {
    validate_api_commit_request(payload, request_id)?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "SELECT
               o.status,
               o.payment_status,
               o.delivery_status,
               o.acceptance_status,
               o.settlement_status,
               o.dispute_status,
               o.buyer_org_id::text,
               o.seller_org_id::text,
               o.asset_version_id::text,
               s.sku_type
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE OF o",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(not_found(order_id, request_id));
    };

    let current_state: String = row.get(0);
    let payment_status: String = row.get(1);
    let current_delivery_status: String = row.get(2);
    let current_acceptance_status: String = row.get(3);
    let current_settlement_status: String = row.get(4);
    let current_dispute_status: String = row.get(5);
    let buyer_org_id: String = row.get(6);
    let seller_org_id: String = row.get(7);
    let asset_version_id: String = row.get(8);
    let sku_type: String = row.get(9);

    if !matches!(sku_type.as_str(), "API_SUB" | "API_PPU") {
        return Err(conflict(
            &format!(
                "API_DELIVERY_COMMIT_FORBIDDEN: order sku_type `{sku_type}` is not API_SUB/API_PPU"
            ),
            request_id,
        ));
    }

    enforce_api_delivery_scope(
        actor_role,
        tenant_id,
        &buyer_org_id,
        &seller_org_id,
        request_id,
    )?;

    let endpoint = resolve_api_endpoint_binding(
        &tx,
        &asset_version_id,
        payload.asset_object_id.as_deref(),
        request_id,
    )
    .await?;

    let current_binding = load_active_api_binding(&tx, order_id).await?;
    if let Some(existing) = &current_binding {
        let same_requested_app = payload
            .app_id
            .as_deref()
            .is_none_or(|app_id| app_id == existing.app_id);
        if same_requested_app && is_idempotent_enabled_state(&sku_type, &current_state) {
            write_delivery_audit_event(
                &tx,
                "api_credential",
                &existing.api_credential_id,
                actor_role,
                "delivery.api.enable",
                "already_enabled",
                request_id,
                trace_id,
                json!({
                    "order_id": order_id,
                    "branch": "api",
                    "sku_type": sku_type,
                    "app_id": existing.app_id,
                    "api_credential_id": existing.api_credential_id,
                    "endpoint_uri": endpoint.endpoint_uri,
                }),
            )
            .await?;
            tx.commit().await.map_err(map_db_error)?;
            return Ok(CommitOrderDeliveryResponseData {
                order_id: order_id.to_string(),
                delivery_id: existing.delivery_id.clone(),
                branch: "api".to_string(),
                previous_state: current_state.clone(),
                current_state,
                payment_status,
                delivery_status: current_delivery_status,
                acceptance_status: current_acceptance_status,
                settlement_status: current_settlement_status,
                dispute_status: current_dispute_status,
                object_id: None,
                envelope_id: None,
                ticket_id: None,
                bucket_name: None,
                object_key: None,
                expires_at: existing.valid_to.clone(),
                download_limit: None,
                receipt_hash: None,
                delivery_commit_hash: None,
                committed_at: existing.committed_at.clone(),
                app_id: Some(existing.app_id.clone()),
                app_name: Some(existing.app_name.clone()),
                app_type: Some(existing.app_type.clone()),
                client_id: Some(existing.client_id.clone()),
                api_credential_id: Some(existing.api_credential_id.clone()),
                api_key: None,
                api_key_hint: Some(mask_hash_hint(&existing.api_key_hash)),
                quota_json: Some(existing.quota_json.clone()),
                rate_limit_json: Some(existing.rate_limit_json.clone()),
                upstream_mode: Some(existing.upstream_mode.clone()),
                operation: Some("already_enabled".to_string()),
                endpoint_uri: Some(endpoint.endpoint_uri),
                credential_status: Some(existing.status.clone()),
                report_artifact_id: None,
                report_type: None,
                report_version_no: None,
                report_status: None,
                report_hash: None,
            });
        }
    }

    if !is_api_delivery_start_state_allowed(&sku_type, &current_state) {
        return Err(conflict(
            &format!(
                "API_DELIVERY_COMMIT_FORBIDDEN: current_state `{current_state}` cannot open API delivery for sku_type `{sku_type}`"
            ),
            request_id,
        ));
    }

    let prepared = ensure_order_deliverable_and_prepare_delivery(
        &tx, order_id, actor_role, request_id, trace_id,
    )
    .await?;

    let rate_limit_json = payload
        .rate_limit_json
        .clone()
        .or_else(|| {
            endpoint
                .access_constraints
                .get("rate_limit_profile")
                .cloned()
        })
        .or_else(|| endpoint.metadata.get("rate_limit_profile").cloned())
        .unwrap_or_else(default_rate_limit_json);
    let quota_json = payload
        .quota_json
        .clone()
        .unwrap_or_else(|| default_quota_json(&sku_type, &rate_limit_json));
    let upstream_mode = payload
        .upstream_mode
        .clone()
        .unwrap_or_else(|| "platform_proxy".to_string());

    let app = ensure_application_binding(
        &tx,
        order_id,
        &buyer_org_id,
        &sku_type,
        &endpoint,
        payload,
        &rate_limit_json,
        request_id,
    )
    .await?;

    let api_key_plaintext = generate_api_key(order_id, &app.client_id);
    let api_key_hash = hash_token(&api_key_plaintext);
    let api_key_hint = mask_plaintext_key(&api_key_plaintext);

    tx.execute(
        "UPDATE delivery.api_credential
         SET status = 'superseded',
             updated_at = now()
         WHERE order_id = $1::text::uuid
           AND status = 'active'",
        &[&order_id],
    )
    .await
    .map_err(map_db_error)?;

    let credential_row = tx
        .query_one(
            r#"INSERT INTO delivery.api_credential (
               order_id,
               app_id,
               source_binding_id,
               api_key_hash,
               upstream_mode,
               quota_json,
               status,
               valid_from,
               valid_to
             ) VALUES (
               $1::text::uuid,
               $2::text::uuid,
               NULL,
               $3,
               $4,
               $5::jsonb,
               'active',
               now(),
               $6::timestamptz
             )
             RETURNING api_credential_id::text,
                       to_char(valid_from AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
                       to_char(valid_to AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"')"#,
            &[
                &order_id,
                &app.app_id,
                &api_key_hash,
                &upstream_mode,
                &quota_json,
                &payload.expire_at,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let api_credential_id: String = credential_row.get(0);
    let valid_from: String = credential_row.get(1);
    let valid_to: Option<String> = credential_row.get(2);

    let target_state = target_state_for_api_delivery(&sku_type);
    let layered_status = derive_api_delivery_layered_status(target_state, &payment_status);
    let trade_audit_action = if sku_type == "API_SUB" {
        "trade.order.api_sub.transition"
    } else {
        "trade.order.api_ppu.transition"
    };
    let reason_code = if sku_type == "API_SUB" {
        "delivery_api_sub_enabled"
    } else {
        "delivery_api_ppu_enabled"
    };

    let trust_boundary_patch = json!({
        "delivery_mode": "api_access",
        "api_delivery": {
            "app_id": app.app_id,
            "api_credential_id": api_credential_id,
            "endpoint_uri": endpoint.endpoint_uri,
            "rate_limit_profile": rate_limit_json,
            "quota": quota_json,
            "upstream_mode": upstream_mode,
            "api_endpoint_object_id": endpoint.asset_object_id,
        }
    });

    let committed_at: String = tx
        .query_one(
            "UPDATE delivery.delivery_record
             SET delivery_type = 'api_access',
                 delivery_route = $2,
                 status = 'committed',
                 delivery_commit_hash = $3,
                 trust_boundary_snapshot = trust_boundary_snapshot || $4::jsonb,
                 receipt_hash = $5,
                 committed_at = now(),
                 expires_at = $6::timestamptz,
                 updated_at = now()
             WHERE delivery_id = $1::text::uuid
             RETURNING to_char(committed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &prepared.delivery_id,
                &upstream_mode,
                &payload.delivery_commit_hash,
                &trust_boundary_patch,
                &payload.receipt_hash,
                &payload.expire_at,
            ],
        )
        .await
        .map_err(map_db_error)?
        .get(0);

    tx.execute(
        "UPDATE trade.order_main
         SET status = $2,
             delivery_status = $3,
             acceptance_status = $4,
             settlement_status = $5,
             dispute_status = $6,
             last_reason_code = $7,
             updated_at = now()
         WHERE order_id = $1::text::uuid",
        &[
            &order_id,
            &target_state,
            &layered_status.delivery_status,
            &layered_status.acceptance_status,
            &layered_status.settlement_status,
            &layered_status.dispute_status,
            &reason_code,
        ],
    )
    .await
    .map_err(map_db_error)?;

    write_trade_audit_event(
        &tx,
        "order",
        order_id,
        actor_role,
        trade_audit_action,
        "success",
        request_id,
        trace_id,
    )
    .await?;

    write_delivery_audit_event(
        &tx,
        "api_credential",
        &api_credential_id,
        actor_role,
        "delivery.api.enable",
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": order_id,
            "branch": "api",
            "sku_type": sku_type,
            "app_id": app.app_id,
            "api_credential_id": api_credential_id,
            "endpoint_uri": endpoint.endpoint_uri,
            "quota_json": quota_json,
            "rate_limit_json": rate_limit_json,
            "upstream_mode": upstream_mode,
            "api_endpoint_object_id": endpoint.asset_object_id,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(CommitOrderDeliveryResponseData {
        order_id: order_id.to_string(),
        delivery_id: prepared.delivery_id,
        branch: "api".to_string(),
        previous_state: current_state,
        current_state: target_state.to_string(),
        payment_status,
        delivery_status: layered_status.delivery_status,
        acceptance_status: layered_status.acceptance_status,
        settlement_status: layered_status.settlement_status,
        dispute_status: layered_status.dispute_status,
        object_id: None,
        envelope_id: None,
        ticket_id: None,
        bucket_name: None,
        object_key: None,
        expires_at: valid_to.clone(),
        download_limit: None,
        receipt_hash: payload.receipt_hash.clone(),
        delivery_commit_hash: payload.delivery_commit_hash.clone(),
        committed_at,
        app_id: Some(app.app_id),
        app_name: Some(app.app_name),
        app_type: Some(app.app_type),
        client_id: Some(app.client_id),
        api_credential_id: Some(api_credential_id),
        api_key: Some(api_key_plaintext),
        api_key_hint: Some(api_key_hint),
        quota_json: Some(quota_json),
        rate_limit_json: Some(rate_limit_json),
        upstream_mode: Some(upstream_mode),
        operation: Some(if current_binding.is_some() {
            "reissued".to_string()
        } else {
            "enabled".to_string()
        }),
        endpoint_uri: Some(endpoint.endpoint_uri),
        credential_status: Some("active".to_string()),
        report_artifact_id: None,
        report_type: None,
        report_version_no: None,
        report_status: None,
        report_hash: None,
    })
}

struct ApiEndpointBinding {
    asset_object_id: String,
    endpoint_uri: String,
    access_constraints: Value,
    metadata: Value,
}

struct ExistingApiBinding {
    delivery_id: String,
    api_credential_id: String,
    app_id: String,
    app_name: String,
    app_type: String,
    client_id: String,
    api_key_hash: String,
    quota_json: Value,
    status: String,
    valid_to: Option<String>,
    upstream_mode: String,
    committed_at: String,
    rate_limit_json: Value,
}

struct ApplicationBinding {
    app_id: String,
    app_name: String,
    app_type: String,
    client_id: String,
}

async fn resolve_api_endpoint_binding(
    client: &(impl GenericClient + Sync),
    asset_version_id: &str,
    asset_object_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<ApiEndpointBinding, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            "SELECT asset_object_id::text,
                    COALESCE(object_locator, ''),
                    access_constraints,
                    metadata
             FROM catalog.asset_object_binding
             WHERE asset_version_id = $1::text::uuid
               AND object_kind = 'api_endpoint'
               AND ($2::text IS NULL OR asset_object_id = $2::text::uuid)
             ORDER BY created_at DESC, asset_object_id DESC
             LIMIT 2",
            &[&asset_version_id, &asset_object_id],
        )
        .await
        .map_err(map_db_error)?;

    if rows.is_empty() {
        let message = if asset_object_id.is_some() {
            "API_DELIVERY_COMMIT_FORBIDDEN: specified api_endpoint object not found"
        } else {
            "API_DELIVERY_COMMIT_FORBIDDEN: api_endpoint object binding not found"
        };
        return Err(conflict(message, request_id));
    }
    if asset_object_id.is_none() && rows.len() > 1 {
        return Err(conflict(
            "API_DELIVERY_COMMIT_FORBIDDEN: asset_object_id is required when multiple api_endpoint bindings exist",
            request_id,
        ));
    }

    let row = &rows[0];
    let endpoint_uri: String = row.get(1);
    if endpoint_uri.trim().is_empty() {
        return Err(conflict(
            "API_DELIVERY_COMMIT_FORBIDDEN: api_endpoint object locator is empty",
            request_id,
        ));
    }
    Ok(ApiEndpointBinding {
        asset_object_id: row.get(0),
        endpoint_uri,
        access_constraints: row.get(2),
        metadata: row.get(3),
    })
}

async fn load_active_api_binding(
    client: &(impl GenericClient + Sync),
    order_id: &str,
) -> Result<Option<ExistingApiBinding>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT dr.delivery_id::text,
                    ac.api_credential_id::text,
                    ac.app_id::text,
                    app.app_name,
                    app.app_type,
                    app.client_id,
                    ac.api_key_hash,
                    ac.quota_json,
                    ac.status,
                    to_char(ac.valid_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    ac.upstream_mode,
                    to_char(dr.committed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                    COALESCE(app.metadata -> 'rate_limit_profile', '{}'::jsonb)
             FROM delivery.api_credential ac
             JOIN core.application app ON app.app_id = ac.app_id
             JOIN delivery.delivery_record dr
               ON dr.order_id = ac.order_id
              AND dr.delivery_type = 'api_access'
              AND dr.status = 'committed'
             WHERE ac.order_id = $1::text::uuid
               AND ac.status = 'active'
             ORDER BY dr.committed_at DESC NULLS LAST, ac.created_at DESC, ac.api_credential_id DESC
             LIMIT 1",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    Ok(row.map(|row| ExistingApiBinding {
        delivery_id: row.get(0),
        api_credential_id: row.get(1),
        app_id: row.get(2),
        app_name: row.get(3),
        app_type: row.get(4),
        client_id: row.get(5),
        api_key_hash: row.get(6),
        quota_json: row.get(7),
        status: row.get(8),
        valid_to: row.get(9),
        upstream_mode: row.get(10),
        committed_at: row.get(11),
        rate_limit_json: row.get(12),
    }))
}

async fn ensure_application_binding(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    buyer_org_id: &str,
    sku_type: &str,
    endpoint: &ApiEndpointBinding,
    payload: &CommitOrderDeliveryRequest,
    rate_limit_json: &Value,
    request_id: Option<&str>,
) -> Result<ApplicationBinding, (StatusCode, Json<ErrorResponse>)> {
    let metadata_patch = json!({
        "order_id": order_id,
        "sku_type": sku_type,
        "delivery_mode": "api_access",
        "api_endpoint_object_id": endpoint.asset_object_id,
        "endpoint_uri": endpoint.endpoint_uri,
        "rate_limit_profile": rate_limit_json,
        "client_secret_status": "managed_by_api_credential",
    });

    if let Some(app_id) = payload.app_id.as_deref() {
        let row = client
            .query_opt(
                "UPDATE core.application
                 SET metadata = metadata || $2::jsonb,
                     updated_at = now()
                 WHERE app_id = $1::text::uuid
                   AND org_id = $3::text::uuid
                   AND status = 'active'
                 RETURNING app_id::text, app_name, app_type, client_id",
                &[&app_id, &metadata_patch, &buyer_org_id],
            )
            .await
            .map_err(map_db_error)?;
        let Some(row) = row else {
            return Err(conflict(
                "API_DELIVERY_COMMIT_FORBIDDEN: app_id is not an active buyer application",
                request_id,
            ));
        };
        return Ok(ApplicationBinding {
            app_id: row.get(0),
            app_name: row.get(1),
            app_type: row.get(2),
            client_id: row.get(3),
        });
    }

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix epoch")
        .as_millis();
    let generated_client_id = payload
        .client_id
        .clone()
        .unwrap_or_else(|| format!("dlv007-{}-{suffix}", &order_id[..8.min(order_id.len())]));
    let generated_app_name = payload
        .app_name
        .clone()
        .unwrap_or_else(|| format!("api-access-{}", &order_id[..8.min(order_id.len())]));
    let generated_app_type = payload
        .app_type
        .clone()
        .unwrap_or_else(|| "api_client".to_string());

    let row = client
        .query_one(
            "INSERT INTO core.application (
               org_id,
               app_name,
               app_type,
               status,
               client_id,
               metadata
             ) VALUES (
               $1::text::uuid,
               $2,
               $3,
               'active',
               $4,
               $5::jsonb
             )
             RETURNING app_id::text, app_name, app_type, client_id",
            &[
                &buyer_org_id,
                &generated_app_name,
                &generated_app_type,
                &generated_client_id,
                &metadata_patch,
            ],
        )
        .await
        .map_err(map_db_error)?;

    Ok(ApplicationBinding {
        app_id: row.get(0),
        app_name: row.get(1),
        app_type: row.get(2),
        client_id: row.get(3),
    })
}

fn validate_api_commit_request(
    payload: &CommitOrderDeliveryRequest,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if payload.branch.trim().to_ascii_lowercase() != "api" {
        return Err(conflict(
            "API_DELIVERY_COMMIT_FORBIDDEN: only `api` branch is supported by DLV-007",
            request_id,
        ));
    }
    if payload
        .delivery_commit_hash
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("delivery_commit_hash is required", request_id));
    }
    if payload
        .receipt_hash
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("receipt_hash is required", request_id));
    }
    if payload
        .expire_at
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(bad_request("expire_at is required", request_id));
    }
    Ok(())
}

fn enforce_api_delivery_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    buyer_org_id: &str,
    seller_org_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if actor_role.starts_with("platform_") {
        return Ok(());
    }
    let allowed = match actor_role {
        "tenant_developer" => tenant_id == Some(buyer_org_id),
        "seller_operator" => tenant_id == Some(seller_org_id),
        "tenant_admin" => tenant_id == Some(buyer_org_id) || tenant_id == Some(seller_org_id),
        _ => false,
    };
    if allowed {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: "api delivery commit is forbidden for tenant scope".to_string(),
            request_id: request_id.map(str::to_string),
        }),
    ))
}

fn is_api_delivery_start_state_allowed(sku_type: &str, current_state: &str) -> bool {
    match sku_type {
        "API_SUB" => matches!(
            current_state,
            "buyer_locked" | "api_bound" | "api_key_issued"
        ),
        "API_PPU" => matches!(
            current_state,
            "buyer_locked" | "api_authorized" | "quota_ready"
        ),
        _ => false,
    }
}

fn is_idempotent_enabled_state(sku_type: &str, current_state: &str) -> bool {
    match sku_type {
        "API_SUB" => matches!(
            current_state,
            "api_key_issued" | "api_trial_active" | "active"
        ),
        "API_PPU" => matches!(current_state, "quota_ready" | "usage_active"),
        _ => false,
    }
}

fn target_state_for_api_delivery(sku_type: &str) -> &'static str {
    match sku_type {
        "API_SUB" => "api_key_issued",
        "API_PPU" => "quota_ready",
        _ => "buyer_locked",
    }
}

fn derive_api_delivery_layered_status(
    target_state: &str,
    payment_status: &str,
) -> LayeredOrderStatus {
    match target_state {
        "api_key_issued" | "quota_ready" | "api_authorized" | "api_bound" => LayeredOrderStatus {
            delivery_status: "in_progress".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: if payment_status == "paid" {
                "pending_settlement".to_string()
            } else {
                "not_started".to_string()
            },
            dispute_status: "none".to_string(),
        },
        _ => LayeredOrderStatus {
            delivery_status: "pending_delivery".to_string(),
            acceptance_status: "not_started".to_string(),
            settlement_status: "not_started".to_string(),
            dispute_status: "none".to_string(),
        },
    }
}

fn default_rate_limit_json() -> Value {
    json!({
        "requests_per_minute": 120,
        "burst": 30,
        "concurrency": 10,
        "ip_policy": "buyer_app_whitelist"
    })
}

fn default_quota_json(sku_type: &str, rate_limit_json: &Value) -> Value {
    match sku_type {
        "API_SUB" => json!({
            "billing_mode": "subscription",
            "period": "monthly",
            "included_calls": 10000,
            "overage_policy": "metered",
            "rate_limit_profile": rate_limit_json,
        }),
        "API_PPU" => json!({
            "billing_mode": "pay_per_use",
            "period": "per_call",
            "included_calls": 0,
            "overage_policy": "per_call",
            "rate_limit_profile": rate_limit_json,
        }),
        _ => json!({"rate_limit_profile": rate_limit_json}),
    }
}

fn generate_api_key(order_id: &str, client_id: &str) -> String {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix epoch")
        .as_nanos();
    format!(
        "datab_api_{}_{}_{}",
        &order_id[..8.min(order_id.len())],
        &client_id[..12.min(client_id.len())],
        suffix
    )
}

fn hash_token(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("mockhash:{:016x}", hasher.finish())
}

fn mask_plaintext_key(value: &str) -> String {
    let suffix = if value.len() > 6 {
        &value[value.len() - 6..]
    } else {
        value
    };
    format!("****{}", suffix)
}

fn mask_hash_hint(value: &str) -> String {
    let suffix = if value.len() > 6 {
        &value[value.len() - 6..]
    } else {
        value
    };
    format!("hash:{}", suffix)
}
