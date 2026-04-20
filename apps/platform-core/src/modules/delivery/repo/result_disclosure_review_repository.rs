use crate::modules::delivery::dto::{
    ResultDisclosureReviewResponseData, ReviewResultDisclosureRequest,
};
use crate::modules::delivery::repo::file_delivery_repository::{
    bad_request, conflict, write_delivery_audit_event,
};
use crate::modules::order::repo::map_db_error;
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient};
use kernel::ErrorResponse;
use serde_json::{Map, Value, json};

const DELIVERY_RESULT_DISCLOSURE_REVIEW_EVENT: &str = "delivery.result_disclosure.review";

pub async fn review_result_disclosure(
    client: &mut Client,
    query_run_id: &str,
    tenant_id: Option<&str>,
    payload: &ReviewResultDisclosureRequest,
    actor_role: &str,
    header_user_id: Option<&str>,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ResultDisclosureReviewResponseData, (StatusCode, Json<ErrorResponse>)> {
    validate_request(payload, request_id)?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    enforce_platform_scope(actor_role, tenant_id, request_id)?;

    let context = load_query_run_context(&tx, query_run_id, request_id).await?;
    enforce_context(&context, request_id)?;

    let review_status = resolve_review_status(payload.review_status.as_deref(), request_id)?;
    let masking_level = resolve_masking_level(
        payload.masking_level.as_deref(),
        &context.masked_level,
        request_id,
    )?;
    let export_scope = resolve_export_scope(
        payload.export_scope.as_deref(),
        &context.export_scope,
        request_id,
    )?;
    let requires_disclosure_review = extract_requires_disclosure_review(&context);
    let output_boundary_json = extract_output_boundary_json(&context.sensitive_policy_snapshot);
    let approval_ticket_id = resolve_approval_ticket_id(
        payload.approval_ticket_id.as_deref(),
        context.approval_ticket_id.as_deref(),
        &review_status,
        &export_scope,
        requires_disclosure_review,
        request_id,
    )?;
    validate_approval_ticket(&tx, approval_ticket_id.as_deref(), request_id).await?;
    let reviewer_user_id = resolve_reviewer_user_id(
        &tx,
        payload.reviewer_user_id.as_deref().or(header_user_id),
        &review_status,
        request_id,
    )
    .await?;
    let review_notes = normalize_review_notes(payload.review_notes.as_deref());
    let decision_snapshot = build_decision_snapshot(
        &context,
        &review_status,
        &masking_level,
        &export_scope,
        reviewer_user_id.as_deref(),
        approval_ticket_id.as_deref(),
        review_notes.as_deref(),
        requires_disclosure_review,
        &output_boundary_json,
        payload.decision_snapshot.as_ref(),
    )?;

    let existing_review_id = load_existing_review_id(
        &tx,
        query_run_id,
        payload.result_disclosure_review_id.as_deref(),
        request_id,
    )
    .await?;

    let (row, operation) = if let Some(result_disclosure_review_id) = existing_review_id.as_deref()
    {
        (
            tx.query_one(
                "UPDATE delivery.result_disclosure_review
                 SET result_object_id = $2::text::uuid,
                     review_status = $3,
                     masking_level = $4,
                     export_scope = $5,
                     reviewer_user_id = $6::text::uuid,
                     approval_ticket_id = $7::text::uuid,
                     review_notes = $8,
                     decision_snapshot = $9::jsonb,
                     reviewed_at = CASE WHEN $3 = 'pending' THEN NULL ELSE now() END,
                     updated_at = now()
                 WHERE result_disclosure_review_id = $1::text::uuid
                 RETURNING result_disclosure_review_id::text,
                           order_id::text,
                           query_run_id::text,
                           result_object_id::text,
                           review_status,
                           masking_level,
                           export_scope,
                           reviewer_user_id::text,
                           approval_ticket_id::text,
                           review_notes,
                           decision_snapshot,
                           to_char(reviewed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &result_disclosure_review_id,
                    &context.result_object_id,
                    &review_status,
                    &masking_level,
                    &export_scope,
                    &reviewer_user_id,
                    &approval_ticket_id,
                    &review_notes,
                    &decision_snapshot,
                ],
            )
            .await
            .map_err(map_db_error)?,
            "updated",
        )
    } else {
        (
            tx.query_one(
                "INSERT INTO delivery.result_disclosure_review (
                   order_id,
                   query_run_id,
                   result_object_id,
                   review_status,
                   masking_level,
                   export_scope,
                   reviewer_user_id,
                   approval_ticket_id,
                   review_notes,
                   decision_snapshot,
                   reviewed_at
                 ) VALUES (
                   $1::text::uuid,
                   $2::text::uuid,
                   $3::text::uuid,
                   $4,
                   $5,
                   $6,
                   $7::text::uuid,
                   $8::text::uuid,
                   $9,
                   $10::jsonb,
                   CASE WHEN $4 = 'pending' THEN NULL ELSE now() END
                 )
                 RETURNING result_disclosure_review_id::text,
                           order_id::text,
                           query_run_id::text,
                           result_object_id::text,
                           review_status,
                           masking_level,
                           export_scope,
                           reviewer_user_id::text,
                           approval_ticket_id::text,
                           review_notes,
                           decision_snapshot,
                           to_char(reviewed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                           to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &context.order_id,
                    &query_run_id,
                    &context.result_object_id,
                    &review_status,
                    &masking_level,
                    &export_scope,
                    &reviewer_user_id,
                    &approval_ticket_id,
                    &review_notes,
                    &decision_snapshot,
                ],
            )
            .await
            .map_err(map_db_error)?,
            "created",
        )
    };

    let response = ResultDisclosureReviewResponseData {
        result_disclosure_review_id: row.get(0),
        order_id: row.get(1),
        query_run_id: row.get(2),
        result_object_id: row.get(3),
        review_status: row.get(4),
        masking_level: row.get(5),
        export_scope: row.get(6),
        reviewer_user_id: row.get(7),
        approval_ticket_id: row.get(8),
        review_notes: row.get(9),
        decision_snapshot: row.get(10),
        reviewed_at: row.get(11),
        created_at: row.get(12),
        updated_at: row.get(13),
        requires_disclosure_review,
        output_boundary_json: output_boundary_json.clone(),
        operation: operation.to_string(),
        current_state: context.current_state.clone(),
        payment_status: context.payment_status.clone(),
        delivery_status: context.delivery_status.clone(),
    };

    sync_query_run_summary(
        &tx,
        query_run_id,
        &response.result_disclosure_review_id,
        &response.review_status,
        response.reviewed_at.as_deref(),
        response.approval_ticket_id.as_deref(),
        response.requires_disclosure_review,
    )
    .await?;
    sync_delivery_record_review_status(&tx, &response.order_id, &response.review_status).await?;

    write_delivery_audit_event(
        &tx,
        "query_run",
        query_run_id,
        actor_role,
        DELIVERY_RESULT_DISCLOSURE_REVIEW_EVENT,
        "success",
        request_id,
        trace_id,
        json!({
            "order_id": response.order_id,
            "query_run_id": response.query_run_id,
            "result_object_id": response.result_object_id,
            "result_disclosure_review_id": response.result_disclosure_review_id,
            "review_status": response.review_status,
            "masking_level": response.masking_level,
            "export_scope": response.export_scope,
            "approval_ticket_id": response.approval_ticket_id,
            "requires_disclosure_review": response.requires_disclosure_review,
            "operation": response.operation,
            "output_boundary_json": response.output_boundary_json,
        }),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(response)
}

#[derive(Debug)]
struct QueryRunContext {
    order_id: String,
    current_state: String,
    payment_status: String,
    delivery_status: String,
    query_run_status: String,
    result_object_id: String,
    masked_level: String,
    export_scope: String,
    approval_ticket_id: Option<String>,
    sensitive_policy_snapshot: Value,
    result_summary_json: Value,
}

fn validate_request(
    payload: &ReviewResultDisclosureRequest,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(decision_snapshot) = payload.decision_snapshot.as_ref()
        && !decision_snapshot.is_object()
    {
        return Err(bad_request(
            "decision_snapshot must be a JSON object",
            request_id,
        ));
    }
    Ok(())
}

fn enforce_platform_scope(
    actor_role: &str,
    tenant_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let _ = tenant_id;
    if matches!(
        actor_role,
        "platform_admin"
            | "platform_risk_settlement"
            | "platform_audit_security"
            | "platform_reviewer"
            | "compliance_reviewer"
            | "audit_admin"
    ) {
        return Ok(());
    }
    Err(conflict(
        "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: platform reviewer scope is required",
        request_id,
    ))
}

async fn load_query_run_context(
    client: &(impl GenericClient + Sync),
    query_run_id: &str,
    request_id: Option<&str>,
) -> Result<QueryRunContext, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT r.order_id::text,
                    o.status,
                    o.payment_status,
                    o.delivery_status,
                    r.status,
                    r.result_object_id::text,
                    r.masked_level,
                    r.export_scope,
                    r.approval_ticket_id::text,
                    COALESCE(r.sensitive_policy_snapshot, '{}'::jsonb),
                    COALESCE(r.result_summary_json, '{}'::jsonb)
             FROM delivery.query_execution_run r
             JOIN trade.order_main o ON o.order_id = r.order_id
             WHERE r.query_run_id = $1::text::uuid",
            &[&query_run_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err(query_run_not_found(query_run_id, request_id));
    };
    let Some(result_object_id) = row.get::<_, Option<String>>(5) else {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: query run result object is missing",
            request_id,
        ));
    };
    Ok(QueryRunContext {
        order_id: row.get(0),
        current_state: row.get(1),
        payment_status: row.get(2),
        delivery_status: row.get(3),
        query_run_status: row.get(4),
        result_object_id,
        masked_level: row.get(6),
        export_scope: row.get(7),
        approval_ticket_id: row.get(8),
        sensitive_policy_snapshot: row.get(9),
        result_summary_json: row.get(10),
    })
}

fn enforce_context(
    context: &QueryRunContext,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if context.query_run_status != "completed" {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: query run status must be `completed`",
            request_id,
        ));
    }
    if context.payment_status != "paid" {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: order payment_status must be `paid`",
            request_id,
        ));
    }
    Ok(())
}

fn resolve_review_status(
    requested: Option<&str>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let review_status = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("pending")
        .to_ascii_lowercase();
    if !matches!(review_status.as_str(), "pending" | "approved" | "rejected") {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: review_status must be pending/approved/rejected",
            request_id,
        ));
    }
    Ok(review_status)
}

fn resolve_masking_level(
    requested: Option<&str>,
    query_run_masked_level: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let masking_level = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(query_run_masked_level)
        .to_ascii_lowercase();
    if !matches!(masking_level.as_str(), "masked" | "summary" | "restricted") {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: masking_level must be masked/summary/restricted",
            request_id,
        ));
    }
    if masking_level != query_run_masked_level {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: masking_level must match query run masked_level in V1 placeholder",
            request_id,
        ));
    }
    Ok(masking_level)
}

fn resolve_export_scope(
    requested: Option<&str>,
    query_run_export_scope: &str,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let export_scope = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(query_run_export_scope)
        .to_ascii_lowercase();
    if !matches!(
        export_scope.as_str(),
        "none" | "summary" | "restricted_object"
    ) {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: export_scope must be none/summary/restricted_object",
            request_id,
        ));
    }
    if export_scope != query_run_export_scope {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: export_scope must match query run export_scope in V1 placeholder",
            request_id,
        ));
    }
    Ok(export_scope)
}

fn extract_requires_disclosure_review(context: &QueryRunContext) -> bool {
    context
        .sensitive_policy_snapshot
        .get("output_boundary_json")
        .and_then(|value| value.get("requires_disclosure_review"))
        .and_then(Value::as_bool)
        .or_else(|| {
            context
                .result_summary_json
                .get("requires_disclosure_review")
                .and_then(Value::as_bool)
        })
        .unwrap_or(false)
}

fn extract_output_boundary_json(sensitive_policy_snapshot: &Value) -> Value {
    sensitive_policy_snapshot
        .get("output_boundary_json")
        .cloned()
        .unwrap_or_else(|| json!({}))
}

fn resolve_approval_ticket_id(
    requested: Option<&str>,
    query_run_approval_ticket_id: Option<&str>,
    review_status: &str,
    export_scope: &str,
    requires_disclosure_review: bool,
    request_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let approval_ticket_id = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| query_run_approval_ticket_id.map(str::to_string));
    if review_status == "approved"
        && (requires_disclosure_review || export_scope == "restricted_object")
        && approval_ticket_id.is_none()
    {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: approval_ticket_id is required before approving disclosure review",
            request_id,
        ));
    }
    Ok(approval_ticket_id)
}

async fn validate_approval_ticket(
    client: &(impl GenericClient + Sync),
    approval_ticket_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(approval_ticket_id) = approval_ticket_id else {
        return Ok(());
    };
    let exists = client
        .query_opt(
            "SELECT approval_ticket_id::text
             FROM ops.approval_ticket
             WHERE approval_ticket_id = $1::text::uuid",
            &[&approval_ticket_id],
        )
        .await
        .map_err(map_db_error)?
        .is_some();
    if exists {
        return Ok(());
    }
    Err(conflict(
        "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: approval_ticket_id does not exist",
        request_id,
    ))
}

async fn resolve_reviewer_user_id(
    client: &(impl GenericClient + Sync),
    requested: Option<&str>,
    review_status: &str,
    request_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    let reviewer_user_id = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if review_status != "pending" && reviewer_user_id.is_none() {
        return Err(conflict(
            "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: reviewer_user_id is required for approved/rejected review",
            request_id,
        ));
    }
    let Some(reviewer_user_id) = reviewer_user_id else {
        return Ok(None);
    };
    let exists = client
        .query_opt(
            "SELECT user_id::text
             FROM core.user_account
             WHERE user_id = $1::text::uuid",
            &[&reviewer_user_id],
        )
        .await
        .map_err(map_db_error)?
        .is_some();
    if exists {
        return Ok(Some(reviewer_user_id));
    }
    Err(conflict(
        "RESULT_DISCLOSURE_REVIEW_FORBIDDEN: reviewer_user_id does not exist",
        request_id,
    ))
}

fn normalize_review_notes(review_notes: Option<&str>) -> Option<String> {
    review_notes
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn build_decision_snapshot(
    context: &QueryRunContext,
    review_status: &str,
    masking_level: &str,
    export_scope: &str,
    reviewer_user_id: Option<&str>,
    approval_ticket_id: Option<&str>,
    review_notes: Option<&str>,
    requires_disclosure_review: bool,
    output_boundary_json: &Value,
    client_snapshot: Option<&Value>,
) -> Result<Value, (StatusCode, Json<ErrorResponse>)> {
    let mut review = Map::new();
    review.insert(
        "review_status".to_string(),
        Value::String(review_status.to_string()),
    );
    review.insert(
        "masking_level".to_string(),
        Value::String(masking_level.to_string()),
    );
    review.insert(
        "export_scope".to_string(),
        Value::String(export_scope.to_string()),
    );
    review.insert(
        "reviewer_user_id".to_string(),
        reviewer_user_id.map_or(Value::Null, |value| Value::String(value.to_string())),
    );
    review.insert(
        "approval_ticket_id".to_string(),
        approval_ticket_id.map_or(Value::Null, |value| Value::String(value.to_string())),
    );
    review.insert(
        "review_notes".to_string(),
        review_notes.map_or(Value::Null, |value| Value::String(value.to_string())),
    );

    let mut object = Map::new();
    object.insert("placeholder_v1".to_string(), Value::Bool(true));
    object.insert(
        "requires_disclosure_review".to_string(),
        Value::Bool(requires_disclosure_review),
    );
    object.insert(
        "query_run".to_string(),
        json!({
            "order_id": context.order_id,
            "query_run_status": context.query_run_status,
            "masked_level": context.masked_level,
            "export_scope": context.export_scope,
            "result_object_id": context.result_object_id,
        }),
    );
    object.insert(
        "output_boundary_json".to_string(),
        output_boundary_json.clone(),
    );
    object.insert("review".to_string(), Value::Object(review));
    object.insert(
        "query_run_sensitive_policy_snapshot".to_string(),
        context.sensitive_policy_snapshot.clone(),
    );
    if let Some(snapshot) = client_snapshot {
        if !snapshot.is_object() {
            return Err(bad_request("decision_snapshot must be a JSON object", None));
        }
        object.insert("client_snapshot".to_string(), snapshot.clone());
    }
    Ok(Value::Object(object))
}

async fn load_existing_review_id(
    client: &(impl GenericClient + Sync),
    query_run_id: &str,
    requested_id: Option<&str>,
    request_id: Option<&str>,
) -> Result<Option<String>, (StatusCode, Json<ErrorResponse>)> {
    if let Some(result_disclosure_review_id) = requested_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let row = client
            .query_opt(
                "SELECT result_disclosure_review_id::text
                 FROM delivery.result_disclosure_review
                 WHERE result_disclosure_review_id = $1::text::uuid
                   AND query_run_id = $2::text::uuid",
                &[&result_disclosure_review_id, &query_run_id],
            )
            .await
            .map_err(map_db_error)?;
        return row
            .map(|row| row.get(0))
            .ok_or_else(|| query_run_not_found(result_disclosure_review_id, request_id))
            .map(Some);
    }

    Ok(client
        .query_opt(
            "SELECT result_disclosure_review_id::text
             FROM delivery.result_disclosure_review
             WHERE query_run_id = $1::text::uuid
             ORDER BY updated_at DESC, result_disclosure_review_id DESC
             LIMIT 1",
            &[&query_run_id],
        )
        .await
        .map_err(map_db_error)?
        .map(|row| row.get(0)))
}

async fn sync_query_run_summary(
    client: &(impl GenericClient + Sync),
    query_run_id: &str,
    result_disclosure_review_id: &str,
    review_status: &str,
    reviewed_at: Option<&str>,
    approval_ticket_id: Option<&str>,
    requires_disclosure_review: bool,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    client
        .execute(
            "UPDATE delivery.query_execution_run
             SET result_summary_json = COALESCE(result_summary_json, '{}'::jsonb)
                    || jsonb_build_object(
                         'disclosure_review_status', $2,
                         'result_disclosure_review_id', $3,
                         'reviewed_at', $4,
                         'approval_ticket_id', $5,
                         'requires_disclosure_review', $6
                       ),
                 updated_at = now()
             WHERE query_run_id = $1::text::uuid",
            &[
                &query_run_id,
                &review_status,
                &result_disclosure_review_id,
                &reviewed_at,
                &approval_ticket_id,
                &requires_disclosure_review,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

async fn sync_delivery_record_review_status(
    client: &(impl GenericClient + Sync),
    order_id: &str,
    review_status: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    client
        .execute(
            "WITH latest AS (
               SELECT delivery_id
               FROM delivery.delivery_record
               WHERE order_id = $1::text::uuid
                 AND delivery_route = 'template_query'
               ORDER BY updated_at DESC, delivery_id DESC
               LIMIT 1
             )
             UPDATE delivery.delivery_record dr
             SET disclosure_review_status = $2,
                 updated_at = now()
             FROM latest
             WHERE dr.delivery_id = latest.delivery_id",
            &[&order_id, &review_status],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

fn query_run_not_found(
    query_run_id: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: kernel::ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("query run not found: {query_run_id}"),
            request_id: request_id.map(str::to_string),
        }),
    )
}
