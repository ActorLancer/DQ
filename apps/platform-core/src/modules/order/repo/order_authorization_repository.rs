use crate::modules::authorization::domain::{
    AuthorizationModelSnapshot, build_authorization_model_snapshot,
    extract_or_build_authorization_model, normalize_policy_snapshot,
};
use crate::modules::order::dto::{
    OrderAuthorizationTransitionRequest, OrderAuthorizationTransitionResponseData,
};
use crate::modules::order::repo::pre_request_repository::{map_db_error, write_trade_audit_event};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, Error, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use serde_json::{Value, json};

struct OrderScope {
    buyer_org_id: String,
    policy_id: Option<String>,
    sku_type: String,
    product_id: String,
    sku_id: String,
    scenario_sku_snapshot: Option<Value>,
}

struct PolicyInfo {
    policy_id: String,
    policy_name: String,
    policy_status: String,
    subject_constraints: Value,
    usage_constraints: Value,
    time_constraints: Value,
    region_constraints: Value,
    output_constraints: Value,
    exportable: bool,
}

struct AuthorizationRow {
    authorization_id: String,
    status: String,
    grant_type: String,
    granted_to_type: String,
    granted_to_id: String,
    authorization_model: AuthorizationModelSnapshot,
    policy_snapshot: Value,
    valid_from: String,
    valid_to: Option<String>,
    transitioned_at: String,
}

pub async fn transition_order_authorization(
    client: &mut Client,
    order_id: &str,
    payload: &OrderAuthorizationTransitionRequest,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<OrderAuthorizationTransitionResponseData, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let scope = load_order_scope(&tx, order_id, request_id).await?;
    let action = payload.action.trim().to_lowercase();
    let existing = load_latest_authorization(&tx, order_id).await?;

    let (result, reason_code, audit_action, policy, previous_status) = match action.as_str() {
        "grant" => {
            if existing.as_ref().is_some_and(|row| row.status == "active") {
                return Err(conflict(
                    "AUTHORIZATION_TRANSITION_FORBIDDEN: active authorization already exists",
                    request_id,
                ));
            }
            if existing
                .as_ref()
                .is_some_and(|row| row.status == "suspended")
            {
                return Err(conflict(
                    "AUTHORIZATION_TRANSITION_FORBIDDEN: suspended authorization must use recover",
                    request_id,
                ));
            }

            let policy =
                resolve_policy(&tx, &scope, payload.policy_id.as_deref(), request_id).await?;
            let grant_type = payload
                .grant_type
                .clone()
                .unwrap_or_else(|| default_grant_type(&scope.sku_type).to_string());
            let granted_to_type = payload
                .granted_to_type
                .clone()
                .unwrap_or_else(|| "org".to_string());
            let granted_to_id = payload
                .granted_to_id
                .clone()
                .unwrap_or_else(|| scope.buyer_org_id.clone());

            let raw_policy_snapshot = payload.policy_snapshot.clone().unwrap_or_else(|| {
                build_policy_snapshot(&policy, scope.scenario_sku_snapshot.as_ref())
            });
            let authorization_model = build_authorization_model_snapshot(
                order_id,
                &scope.product_id,
                &scope.sku_id,
                &scope.sku_type,
                &policy.policy_id,
                &granted_to_type,
                &granted_to_id,
                &grant_type,
                &policy.subject_constraints,
                &policy.usage_constraints,
                policy.exportable,
            );
            let policy_snapshot = attach_scenario_sku_snapshot(
                normalize_policy_snapshot(raw_policy_snapshot, &authorization_model),
                scope.scenario_sku_snapshot.as_ref(),
            );

            tx.execute(
                "UPDATE trade.order_main
                 SET policy_id = $2::text::uuid,
                     updated_at = now()
                 WHERE order_id = $1::text::uuid",
                &[&order_id, &policy.policy_id],
            )
            .await
            .map_err(map_db_error)?;

            let row = tx
                .query_one(
                    "INSERT INTO trade.authorization_grant (
                       order_id,
                       grant_type,
                       granted_to_type,
                       granted_to_id,
                       policy_snapshot,
                       valid_from,
                       valid_to,
                       status
                     ) VALUES (
                       $1::text::uuid,
                       $2,
                       $3,
                       $4::text::uuid,
                       $5::jsonb,
                       now(),
                       $6::text::timestamptz,
                       'active'
                     )
                     RETURNING
                       authorization_grant_id::text,
                       status,
                       grant_type,
                       granted_to_type,
                       granted_to_id::text,
                       policy_snapshot,
                       to_char(valid_from AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(valid_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       order_id::text,
                       $7::text,
                       $8::text,
                       $9::text",
                    &[
                        &order_id,
                        &grant_type,
                        &granted_to_type,
                        &granted_to_id,
                        &policy_snapshot,
                        &payload.valid_to,
                        &scope.product_id,
                        &scope.sku_id,
                        &scope.sku_type,
                    ],
                )
                .await
                .map_err(map_db_error)?;
            (
                to_authorization_row(row),
                "trade017_authorization_granted",
                "trade.authorization.grant",
                policy,
                existing.as_ref().map(|r| r.status.clone()),
            )
        }
        "revoke" | "expire" | "suspend" | "recover" => {
            let Some(current) = existing.as_ref() else {
                return Err(conflict(
                    "AUTHORIZATION_TRANSITION_FORBIDDEN: authorization not found for order",
                    request_id,
                ));
            };
            let target_status = derive_target_status(&action, &current.status, request_id)?;
            let policy = resolve_policy_for_existing(
                &tx,
                &scope,
                payload.policy_id.as_deref(),
                &current.policy_snapshot,
                request_id,
            )
            .await?;

            let row = tx
                .query_one(
                    "UPDATE trade.authorization_grant
                     SET status = $3,
                         valid_to = CASE WHEN $3 IN ('revoked', 'expired') THEN coalesce(valid_to, now()) ELSE valid_to END,
                         updated_at = now()
                     WHERE authorization_grant_id = $1::text::uuid
                       AND order_id = $2::text::uuid
                     RETURNING
                       authorization_grant_id::text,
                       status,
                       grant_type,
                       granted_to_type,
                       granted_to_id::text,
                       policy_snapshot,
                       to_char(valid_from AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(valid_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                       order_id::text,
                       $4::text,
                       $5::text,
                       $6::text",
                    &[
                        &current.authorization_id,
                        &order_id,
                        &target_status,
                        &scope.product_id,
                        &scope.sku_id,
                        &scope.sku_type,
                    ],
                )
                .await
                .map_err(map_db_error)?;

            let (reason_code, audit_action) = match action.as_str() {
                "revoke" => (
                    "trade017_authorization_revoked",
                    "trade.authorization.revoke",
                ),
                "expire" => (
                    "trade017_authorization_expired",
                    "trade.authorization.expire",
                ),
                "suspend" => (
                    "trade017_authorization_suspended",
                    "trade.authorization.suspend",
                ),
                _ => (
                    "trade017_authorization_recovered",
                    "trade.authorization.recover",
                ),
            };

            (
                to_authorization_row(row),
                reason_code,
                audit_action,
                policy,
                Some(current.status.clone()),
            )
        }
        _ => {
            return Err(conflict(
                "AUTHORIZATION_TRANSITION_FORBIDDEN: action must be one of grant/revoke/expire/suspend/recover",
                request_id,
            ));
        }
    };

    write_trade_audit_event(
        &tx,
        "authorization",
        &result.authorization_id,
        actor_role,
        audit_action,
        "success",
        request_id,
        trace_id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;

    Ok(OrderAuthorizationTransitionResponseData {
        order_id: order_id.to_string(),
        authorization_id: result.authorization_id,
        action,
        previous_status,
        current_status: result.status,
        policy_id: policy.policy_id,
        policy_name: policy.policy_name,
        policy_status: policy.policy_status,
        grant_type: result.grant_type,
        granted_to_type: result.granted_to_type,
        granted_to_id: result.granted_to_id,
        valid_from: result.valid_from,
        valid_to: result.valid_to,
        reason_code: reason_code.to_string(),
        authorization_model: result.authorization_model,
        policy_snapshot: result.policy_snapshot,
        transitioned_at: result.transitioned_at,
    })
}

fn to_authorization_row(row: Row) -> AuthorizationRow {
    let policy_snapshot: Value = row.get(5);
    let fallback = build_authorization_model_snapshot(
        row.get::<_, String>(9).as_str(),
        row.get::<_, String>(10).as_str(),
        row.get::<_, String>(11).as_str(),
        row.get::<_, String>(12).as_str(),
        policy_snapshot
            .get("policy_id")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        row.get::<_, String>(3).as_str(),
        row.get::<_, String>(4).as_str(),
        row.get::<_, String>(2).as_str(),
        policy_snapshot
            .get("subject_constraints")
            .unwrap_or(&Value::Null),
        policy_snapshot
            .get("usage_constraints")
            .unwrap_or(&Value::Null),
        policy_snapshot
            .get("exportable")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    );
    AuthorizationRow {
        authorization_id: row.get(0),
        status: row.get(1),
        grant_type: row.get(2),
        granted_to_type: row.get(3),
        granted_to_id: row.get(4),
        authorization_model: extract_or_build_authorization_model(&policy_snapshot, fallback),
        policy_snapshot,
        valid_from: row.get(6),
        valid_to: row.get(7),
        transitioned_at: row.get(8),
    }
}

fn derive_target_status(
    action: &str,
    current_status: &str,
    request_id: Option<&str>,
) -> Result<&'static str, (StatusCode, Json<ErrorResponse>)> {
    match (action, current_status) {
        ("revoke", "active" | "suspended") => Ok("revoked"),
        ("expire", "active" | "suspended") => Ok("expired"),
        ("suspend", "active") => Ok("suspended"),
        ("recover", "suspended") => Ok("active"),
        _ => Err(conflict(
            &format!(
                "AUTHORIZATION_TRANSITION_FORBIDDEN: action `{action}` cannot apply on status `{current_status}`"
            ),
            request_id,
        )),
    }
}

async fn load_order_scope(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    request_id: Option<&str>,
) -> Result<OrderScope, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               buyer_org_id::text,
               policy_id::text,
               s.sku_type,
               o.product_id::text,
               o.sku_id::text,
               o.price_snapshot_json
             FROM trade.order_main o
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE o.order_id = $1::text::uuid
             FOR UPDATE",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };

    Ok(OrderScope {
        buyer_org_id: row.get(0),
        policy_id: row.get(1),
        sku_type: row.get(2),
        product_id: row.get(3),
        sku_id: row.get(4),
        scenario_sku_snapshot: row.get::<_, Value>(5).get("scenario_snapshot").cloned(),
    })
}

async fn load_latest_authorization(
    client: &(impl GenericClient + Sync),
    order_id: &str,
) -> Result<Option<AuthorizationRow>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               authorization_grant_id::text,
               ag.status,
               grant_type,
               granted_to_type,
               granted_to_id::text,
               policy_snapshot,
               to_char(valid_from AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(valid_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(ag.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               ag.order_id::text,
               o.product_id::text,
               o.sku_id::text,
               s.sku_type
             FROM trade.authorization_grant ag
             JOIN trade.order_main o ON o.order_id = ag.order_id
             JOIN catalog.product_sku s ON s.sku_id = o.sku_id
             WHERE ag.order_id = $1::text::uuid
             ORDER BY ag.created_at DESC, ag.updated_at DESC
             LIMIT 1
             FOR UPDATE",
            &[&order_id],
        )
        .await
        .map_err(map_db_error)?;

    Ok(row.map(to_authorization_row))
}

async fn resolve_policy(
    client: &(impl GenericClient + Sync),
    scope: &OrderScope,
    requested_policy_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<PolicyInfo, (StatusCode, Json<ErrorResponse>)> {
    if let Some(policy_id) = requested_policy_id {
        return load_policy_by_id(client, policy_id, request_id).await;
    }
    if let Some(policy_id) = scope.policy_id.as_deref() {
        return load_policy_by_id(client, policy_id, request_id).await;
    }

    let row = client
        .query_opt(
            "SELECT p.policy_id::text
             FROM contract.policy_binding pb
             JOIN contract.usage_policy p ON p.policy_id = pb.policy_id
             WHERE (pb.sku_id = $1::text::uuid OR pb.product_id = $2::text::uuid)
             ORDER BY CASE WHEN pb.sku_id IS NOT NULL THEN 0 ELSE 1 END, pb.created_at DESC
             LIMIT 1",
            &[&scope.sku_id, &scope.product_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err(conflict(
            "AUTHORIZATION_TRANSITION_FORBIDDEN: usage policy is not bound to order product/sku",
            request_id,
        ));
    };
    let policy_id: String = row.get(0);
    load_policy_by_id(client, &policy_id, request_id).await
}

async fn resolve_policy_for_existing(
    client: &(impl GenericClient + Sync),
    scope: &OrderScope,
    requested_policy_id: Option<&str>,
    policy_snapshot: &Value,
    request_id: Option<&str>,
) -> Result<PolicyInfo, (StatusCode, Json<ErrorResponse>)> {
    if let Some(policy_id) = requested_policy_id {
        return load_policy_by_id(client, policy_id, request_id).await;
    }
    if let Some(policy_id) = scope.policy_id.as_deref() {
        return load_policy_by_id(client, policy_id, request_id).await;
    }
    if let Some(policy_id) = policy_snapshot.get("policy_id").and_then(Value::as_str) {
        return load_policy_by_id(client, policy_id, request_id).await;
    }
    Err(conflict(
        "AUTHORIZATION_TRANSITION_FORBIDDEN: policy context is missing",
        request_id,
    ))
}

async fn load_policy_by_id(
    client: &(impl GenericClient + Sync),
    policy_id: &str,
    request_id: Option<&str>,
) -> Result<PolicyInfo, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT
               policy_id::text,
               policy_name,
               status,
               subject_constraints,
               usage_constraints,
               time_constraints,
               region_constraints,
               output_constraints,
               exportable
             FROM contract.usage_policy
             WHERE policy_id = $1::text::uuid",
            &[&policy_id],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("usage policy not found: {policy_id}"),
                request_id: request_id.map(str::to_string),
            }),
        ));
    };
    let status: String = row.get(2);
    if status != "active" {
        return Err(conflict(
            "AUTHORIZATION_TRANSITION_FORBIDDEN: usage policy is not active",
            request_id,
        ));
    }

    Ok(PolicyInfo {
        policy_id: row.get(0),
        policy_name: row.get(1),
        policy_status: status,
        subject_constraints: row.get(3),
        usage_constraints: row.get(4),
        time_constraints: row.get(5),
        region_constraints: row.get(6),
        output_constraints: row.get(7),
        exportable: row.get(8),
    })
}

fn default_grant_type(sku_type: &str) -> &'static str {
    match sku_type {
        "FILE_STD" | "FILE_SUB" => "file_access",
        "API_SUB" | "API_PPU" => "api_access",
        "SHARE_RO" => "share_grant",
        "QRY_LITE" => "template_grant",
        "SBX_STD" => "sandbox_grant",
        "RPT_STD" => "report_delivery",
        _ => "order_access",
    }
}

fn build_policy_snapshot(policy: &PolicyInfo, scenario_sku_snapshot: Option<&Value>) -> Value {
    let mut snapshot = json!({
        "policy_id": policy.policy_id,
        "policy_name": policy.policy_name,
        "policy_status": policy.policy_status,
        "subject_constraints": policy.subject_constraints,
        "usage_constraints": policy.usage_constraints,
        "time_constraints": policy.time_constraints,
        "region_constraints": policy.region_constraints,
        "output_constraints": policy.output_constraints,
        "exportable": policy.exportable
    });
    if let (Some(snapshot_obj), Some(scenario_snapshot)) =
        (snapshot.as_object_mut(), scenario_sku_snapshot)
    {
        snapshot_obj.insert(
            "scenario_sku_snapshot".to_string(),
            scenario_snapshot.clone(),
        );
    }
    snapshot
}

fn attach_scenario_sku_snapshot(
    policy_snapshot: Value,
    scenario_sku_snapshot: Option<&Value>,
) -> Value {
    let Some(scenario_sku_snapshot) = scenario_sku_snapshot else {
        return policy_snapshot;
    };
    let mut snapshot = match policy_snapshot {
        Value::Object(map) => map,
        other => {
            let mut map = serde_json::Map::new();
            map.insert("legacy_policy_snapshot".to_string(), other);
            map
        }
    };
    snapshot.insert(
        "scenario_sku_snapshot".to_string(),
        scenario_sku_snapshot.clone(),
    );
    Value::Object(snapshot)
}

fn conflict(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            code: "AUTHORIZATION_TRANSITION_FORBIDDEN".to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
