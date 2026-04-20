use crate::AppState;
use crate::modules::billing::handlers::{
    header, map_db_connect, require_permission, require_step_up_placeholder,
};
use crate::modules::billing::models::{
    CreateReconciliationImportRequest, ReconciliationImportDiffInput, ReconciliationImportView,
};
use crate::modules::billing::repo::reconciliation_repository::import_reconciliation_statement as import_reconciliation_statement_repo;
use crate::modules::billing::service::BillingPermission;
use axum::Json;
use axum::extract::{Multipart, State};
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

pub async fn import_reconciliation_statement(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<ReconciliationImportView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        BillingPermission::ReconciliationImport,
        "reconciliation import",
    )?;
    require_step_up_placeholder(&headers, "reconciliation import")?;

    let request_id = header(&headers, "x-request-id");
    let role = header(&headers, "x-role").unwrap_or_default();
    let actor_user_id = header(&headers, "x-user-id");

    let mut provider_key: Option<String> = None;
    let mut provider_account_id: Option<String> = None;
    let mut statement_date: Option<String> = None;
    let mut statement_type: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut file_content_type: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut diffs: Vec<ReconciliationImportDiffInput> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| multipart_error(&err, request_id.as_deref()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "provider_key" => {
                provider_key =
                    Some(non_empty(field.text().await.map_err(|err| {
                        multipart_error(&err, request_id.as_deref())
                    })?));
            }
            "provider_account_id" => {
                provider_account_id =
                    Some(non_empty(field.text().await.map_err(|err| {
                        multipart_error(&err, request_id.as_deref())
                    })?));
            }
            "statement_date" => {
                statement_date =
                    Some(non_empty(field.text().await.map_err(|err| {
                        multipart_error(&err, request_id.as_deref())
                    })?));
            }
            "statement_type" => {
                statement_type =
                    Some(non_empty(field.text().await.map_err(|err| {
                        multipart_error(&err, request_id.as_deref())
                    })?));
            }
            "diffs_json" => {
                let raw = field
                    .text()
                    .await
                    .map_err(|err| multipart_error(&err, request_id.as_deref()))?;
                if !raw.trim().is_empty() {
                    diffs = serde_json::from_str::<Vec<ReconciliationImportDiffInput>>(&raw)
                        .map_err(|err| {
                            bad_request(
                                &format!("diffs_json must be a valid JSON array: {err}"),
                                request_id.as_deref(),
                            )
                        })?;
                }
            }
            "file" => {
                file_name = field.file_name().map(str::to_string);
                file_content_type = field.content_type().map(str::to_string);
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|err| multipart_error(&err, request_id.as_deref()))?
                        .to_vec(),
                );
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let payload = CreateReconciliationImportRequest {
        provider_key: required_field("provider_key", provider_key, request_id.as_deref())?,
        provider_account_id: required_field(
            "provider_account_id",
            provider_account_id,
            request_id.as_deref(),
        )?,
        statement_date: required_field("statement_date", statement_date, request_id.as_deref())?,
        statement_type: required_field("statement_type", statement_type, request_id.as_deref())?,
        file_name: required_field("file", file_name, request_id.as_deref())?,
        file_content_type,
        file_bytes: file_bytes
            .filter(|bytes| !bytes.is_empty())
            .ok_or_else(|| {
                bad_request(
                    "file is required for reconciliation import",
                    request_id.as_deref(),
                )
            })?,
        diffs,
    };

    let client = state.db.client().map_err(map_db_connect)?;
    let imported = import_reconciliation_statement_repo(
        &client,
        &payload,
        actor_user_id.as_deref(),
        role.as_str(),
        request_id.as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = if imported.idempotent_replay {
            "payment.reconciliation.import.idempotent_replay"
        } else {
            "payment.reconciliation.import"
        },
        reconciliation_statement_id = %imported.statement.reconciliation_statement_id,
        provider_key = %imported.statement.provider_key,
        statement_date = %imported.statement.statement_date,
        imported_diff_count = imported.imported_diff_count,
        open_diff_count = imported.open_diff_count,
        "reconciliation statement imported"
    );

    Ok(ApiResponse::ok(imported))
}

fn required_field(
    field_name: &str,
    value: Option<String>,
    request_id: Option<&str>,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    value
        .map(non_empty)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            bad_request(
                &format!("{field_name} is required for reconciliation import"),
                request_id,
            )
        })
}

fn non_empty(value: String) -> String {
    value.trim().to_string()
}

fn multipart_error(
    err: &impl std::fmt::Display,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    bad_request(
        &format!("reconciliation import multipart parse failed: {err}"),
        request_id,
    )
}

fn bad_request(message: &str, request_id: Option<&str>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
