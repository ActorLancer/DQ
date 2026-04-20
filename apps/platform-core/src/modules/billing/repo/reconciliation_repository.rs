use crate::modules::billing::db::{map_db_error, write_audit_event};
use crate::modules::billing::models::{
    CreateReconciliationImportRequest, ReconciliationDiffView, ReconciliationImportDiffInput,
    ReconciliationImportView, ReconciliationStatementView,
};
use axum::Json;
use axum::http::StatusCode;
use db::{Client, GenericClient, Row};
use kernel::{ErrorCode, ErrorResponse};
use sha2::{Digest, Sha256};

pub async fn import_reconciliation_statement(
    client: &Client,
    payload: &CreateReconciliationImportRequest,
    _actor_user_id: Option<&str>,
    actor_role: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<ReconciliationImportView, (StatusCode, Json<ErrorResponse>)> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    let file_hash = compute_file_hash(&payload.file_bytes).await?;

    if let Some(existing) = find_existing_statement(
        &tx,
        &payload.provider_key,
        &payload.provider_account_id,
        &payload.statement_date,
        &payload.statement_type,
        request_id,
    )
    .await?
    {
        if existing.statement.file_hash.as_deref() != Some(file_hash.as_str()) {
            return Err(billing_error(
                StatusCode::CONFLICT,
                "reconciliation statement already imported for provider/account/date/type with a different file hash",
                request_id,
            ));
        }
        write_audit_event(
            &tx,
            "billing",
            "reconciliation_statement",
            &existing.statement.reconciliation_statement_id,
            actor_role,
            "payment.reconciliation.import.idempotent_replay",
            "idempotent_replay",
            request_id,
            trace_id,
        )
        .await?;
        tx.commit().await.map_err(map_db_error)?;
        return Ok(ReconciliationImportView {
            idempotent_replay: true,
            ..existing
        });
    }

    ensure_provider_account(
        &tx,
        &payload.provider_key,
        &payload.provider_account_id,
        request_id,
    )
    .await?;

    let statement_status = derive_statement_status(&payload.diffs);
    let file_uri = build_placeholder_file_uri(
        &payload.provider_key,
        &payload.statement_date,
        &payload.statement_type,
        &payload.file_name,
    );
    let statement_row = tx
        .query_one(
            r#"INSERT INTO payment.reconciliation_statement (
               provider_key,
               provider_account_id,
               statement_date,
               statement_type,
               file_uri,
               file_hash,
               import_status
             ) VALUES (
               $1,
               $2::text::uuid,
               $3::date,
               $4,
               $5,
               $6,
               $7
             )
             RETURNING
               reconciliation_statement_id::text,
               provider_key,
               provider_account_id::text,
               statement_date::text,
               statement_type,
               file_uri,
               file_hash,
               import_status,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')"#,
            &[
                &payload.provider_key,
                &payload.provider_account_id,
                &payload.statement_date,
                &payload.statement_type,
                &file_uri,
                &file_hash,
                &statement_status,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let statement = map_statement_row(&statement_row);
    let statement_id = statement.reconciliation_statement_id.clone();

    let mut diffs = Vec::with_capacity(payload.diffs.len());
    for diff in &payload.diffs {
        diffs.push(insert_reconciliation_diff(&tx, &statement_id, diff).await?);
    }
    apply_reconcile_statuses(&tx, &diffs).await?;

    write_audit_event(
        &tx,
        "billing",
        "reconciliation_statement",
        &statement_id,
        actor_role,
        "payment.reconciliation.import",
        "success",
        request_id,
        trace_id,
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(ReconciliationImportView {
        statement,
        imported_diff_count: diffs.len(),
        open_diff_count: diffs
            .iter()
            .filter(|diff| !matches!(diff.diff_status.as_str(), "resolved" | "matched"))
            .count(),
        diffs,
        idempotent_replay: false,
        step_up_bound: true,
    })
}

async fn ensure_provider_account(
    client: &impl GenericClient,
    provider_key: &str,
    provider_account_id: &str,
    request_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            "SELECT status
             FROM payment.provider_account
             WHERE provider_account_id = $1::text::uuid
               AND provider_key = $2",
            &[&provider_account_id, &provider_key],
        )
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            billing_error(
                StatusCode::NOT_FOUND,
                "provider account not found for reconciliation import",
                request_id,
            )
        })?;
    let status: String = row.get(0);
    if status != "active" {
        return Err(billing_error(
            StatusCode::CONFLICT,
            &format!("provider account is not active: {status}"),
            request_id,
        ));
    }
    Ok(())
}

async fn compute_file_hash(bytes: &[u8]) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

async fn find_existing_statement(
    client: &impl GenericClient,
    provider_key: &str,
    provider_account_id: &str,
    statement_date: &str,
    statement_type: &str,
    request_id: Option<&str>,
) -> Result<Option<ReconciliationImportView>, (StatusCode, Json<ErrorResponse>)> {
    let row = client
        .query_opt(
            r#"SELECT
               reconciliation_statement_id::text,
               provider_key,
               provider_account_id::text,
               statement_date::text,
               statement_type,
               file_uri,
               file_hash,
               import_status,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM payment.reconciliation_statement
             WHERE provider_key = $1
               AND provider_account_id = $2::text::uuid
               AND statement_date = $3::date
               AND statement_type = $4"#,
            &[
                &provider_key,
                &provider_account_id,
                &statement_date,
                &statement_type,
            ],
        )
        .await
        .map_err(map_db_error)?;

    let Some(row) = row else {
        return Ok(None);
    };
    let statement = map_statement_row(&row);
    let diffs =
        load_diffs_for_statement(client, &statement.reconciliation_statement_id, request_id)
            .await?;
    Ok(Some(ReconciliationImportView {
        imported_diff_count: diffs.len(),
        open_diff_count: diffs
            .iter()
            .filter(|diff| !matches!(diff.diff_status.as_str(), "resolved" | "matched"))
            .count(),
        statement,
        diffs,
        idempotent_replay: false,
        step_up_bound: true,
    }))
}

fn map_statement_row(row: &Row) -> ReconciliationStatementView {
    ReconciliationStatementView {
        reconciliation_statement_id: row.get(0),
        provider_key: row.get(1),
        provider_account_id: row.get(2),
        statement_date: row.get(3),
        statement_type: row.get(4),
        file_uri: row.get(5),
        file_hash: row.get(6),
        import_status: row.get(7),
        created_at: row.get(8),
        updated_at: row.get(9),
    }
}

async fn load_diffs_for_statement(
    client: &impl GenericClient,
    statement_id: &str,
    _request_id: Option<&str>,
) -> Result<Vec<ReconciliationDiffView>, (StatusCode, Json<ErrorResponse>)> {
    let rows = client
        .query(
            r#"SELECT
               reconciliation_diff_id::text,
               reconciliation_statement_id::text,
               diff_type,
               ref_type,
               ref_id::text,
               provider_reference_no,
               internal_amount::text,
               provider_amount::text,
               diff_status,
               resolution_note,
               to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM payment.reconciliation_diff
             WHERE reconciliation_statement_id = $1::text::uuid
             ORDER BY created_at ASC, reconciliation_diff_id ASC"#,
            &[&statement_id],
        )
        .await
        .map_err(map_db_error)?;
    Ok(rows.into_iter().map(map_diff_row).collect())
}

async fn insert_reconciliation_diff(
    client: &impl GenericClient,
    statement_id: &str,
    diff: &ReconciliationImportDiffInput,
) -> Result<ReconciliationDiffView, (StatusCode, Json<ErrorResponse>)> {
    let diff_type = non_empty(diff.diff_type.as_str());
    let ref_type = diff.ref_type.as_deref().map(non_empty_owned);
    let diff_status = diff
        .diff_status
        .as_deref()
        .map(non_empty_owned)
        .filter(|status| !status.is_empty())
        .unwrap_or_else(|| "open".to_string());
    let row = client
        .query_one(
            r#"INSERT INTO payment.reconciliation_diff (
               reconciliation_statement_id,
               diff_type,
               ref_type,
               ref_id,
               provider_reference_no,
               internal_amount,
               provider_amount,
               diff_status,
               resolution_note,
               resolved_at
             ) VALUES (
               $1::text::uuid,
               $2,
               $3,
               $4::text::uuid,
               $5,
               $6::text::numeric,
               $7::text::numeric,
               $8,
               $9,
               $10::timestamptz
             )
             RETURNING
               reconciliation_diff_id::text,
               reconciliation_statement_id::text,
               diff_type,
               ref_type,
               ref_id::text,
               provider_reference_no,
               internal_amount::text,
               provider_amount::text,
               diff_status,
               resolution_note,
               to_char(resolved_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')"#,
            &[
                &statement_id,
                &diff_type,
                &ref_type,
                &diff.ref_id,
                &diff.provider_reference_no,
                &diff.internal_amount,
                &diff.provider_amount,
                &diff_status,
                &diff.resolution_note,
                &diff.resolved_at,
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(map_diff_row(row))
}

fn map_diff_row(row: Row) -> ReconciliationDiffView {
    ReconciliationDiffView {
        reconciliation_diff_id: row.get(0),
        reconciliation_statement_id: row.get(1),
        diff_type: row.get(2),
        ref_type: row.get(3),
        ref_id: row.get(4),
        provider_reference_no: row.get(5),
        internal_amount: row.get(6),
        provider_amount: row.get(7),
        diff_status: row.get(8),
        resolution_note: row.get(9),
        resolved_at: row.get(10),
        created_at: row.get(11),
        updated_at: row.get(12),
    }
}

async fn apply_reconcile_statuses(
    client: &impl GenericClient,
    diffs: &[ReconciliationDiffView],
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    for diff in diffs {
        let Some(ref_type) = diff.ref_type.as_deref() else {
            continue;
        };
        let Some(ref_id) = diff.ref_id.as_deref() else {
            continue;
        };
        let Some((table, id_column)) = reconcile_target(ref_type) else {
            continue;
        };
        let target_status = map_diff_status_to_reconcile_status(&diff.diff_status);
        let statement = format!(
            "UPDATE {table}
             SET reconcile_status = $2,
                 last_reconciled_at = now()
             WHERE {id_column} = $1::text::uuid"
        );
        let _ = client
            .execute(statement.as_str(), &[&ref_id, &target_status])
            .await
            .map_err(map_db_error)?;
    }
    Ok(())
}

fn reconcile_target(ref_type: &str) -> Option<(&'static str, &'static str)> {
    match ref_type {
        "payment_intent" => Some(("payment.payment_intent", "payment_intent_id")),
        "order" | "order_main" => Some(("trade.order_main", "order_id")),
        "settlement" | "settlement_record" => Some(("billing.settlement_record", "settlement_id")),
        "payout_instruction" | "payout" => {
            Some(("payment.payout_instruction", "payout_instruction_id"))
        }
        "refund_intent" | "refund" => Some(("payment.refund_intent", "refund_intent_id")),
        _ => None,
    }
}

fn map_diff_status_to_reconcile_status(diff_status: &str) -> &'static str {
    match diff_status {
        "matched" => "matched",
        "resolved" | "closed" => "resolved",
        _ => "mismatched",
    }
}

fn derive_statement_status(diffs: &[ReconciliationImportDiffInput]) -> String {
    if diffs.is_empty() {
        return "matched".to_string();
    }
    if diffs.iter().all(|diff| {
        matches!(
            diff.diff_status.as_deref().map(str::trim),
            Some("resolved") | Some("matched")
        )
    }) {
        return "resolved".to_string();
    }
    "mismatched".to_string()
}

fn build_placeholder_file_uri(
    provider_key: &str,
    statement_date: &str,
    statement_type: &str,
    file_name: &str,
) -> String {
    let safe_name = file_name
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ' ' => '_',
            other => other,
        })
        .collect::<String>();
    format!(
        "upload://payment-reconciliation/{provider_key}/{statement_date}/{statement_type}/{safe_name}"
    )
}

fn non_empty(value: &str) -> &str {
    value.trim()
}

fn non_empty_owned(value: &str) -> String {
    value.trim().to_string()
}

fn billing_error(
    status: StatusCode,
    message: &str,
    request_id: Option<&str>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            code: ErrorCode::BilProviderFailed.as_str().to_string(),
            message: message.to_string(),
            request_id: request_id.map(str::to_string),
        }),
    )
}
