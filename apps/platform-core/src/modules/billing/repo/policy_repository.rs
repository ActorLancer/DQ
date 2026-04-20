use crate::modules::billing::db::map_db_error;
use crate::modules::billing::domain::{CorridorPolicy, JurisdictionProfile, PayoutPreference};
use crate::modules::billing::models::{
    CreateCorridorPolicyRequest, CreateJurisdictionProfileRequest, CreatePayoutPreferenceRequest,
};
use axum::Json;
use axum::http::StatusCode;
use db::{GenericClient, Row};
use kernel::ErrorResponse;
use serde_json::Value;

type HttpResult<T> = Result<T, (StatusCode, Json<ErrorResponse>)>;

pub async fn list_jurisdictions(
    client: &impl GenericClient,
) -> HttpResult<Vec<JurisdictionProfile>> {
    let rows = client
        .query(
            "SELECT
               jurisdiction_code,
               jurisdiction_name,
               regulator_name,
               launch_phase,
               supports_fiat_collection,
               supports_fiat_payout,
               supports_crypto_settlement,
               status,
               policy_snapshot
             FROM payment.jurisdiction_profile
             ORDER BY jurisdiction_code ASC",
            &[],
        )
        .await
        .map_err(map_db_error)?;
    rows.into_iter().map(parse_jurisdiction_row).collect()
}

pub async fn upsert_jurisdiction(
    client: &impl GenericClient,
    payload: &CreateJurisdictionProfileRequest,
) -> HttpResult<JurisdictionProfile> {
    let row = client
        .query_one(
            "INSERT INTO payment.jurisdiction_profile (
               jurisdiction_code,
               jurisdiction_name,
               regulator_name,
               launch_phase,
               supports_fiat_collection,
               supports_fiat_payout,
               supports_crypto_settlement,
               status,
               policy_snapshot
             ) VALUES (
               $1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb
             )
             ON CONFLICT (jurisdiction_code)
             DO UPDATE SET
               jurisdiction_name = EXCLUDED.jurisdiction_name,
               regulator_name = EXCLUDED.regulator_name,
               launch_phase = EXCLUDED.launch_phase,
               supports_fiat_collection = EXCLUDED.supports_fiat_collection,
               supports_fiat_payout = EXCLUDED.supports_fiat_payout,
               supports_crypto_settlement = EXCLUDED.supports_crypto_settlement,
               status = EXCLUDED.status,
               policy_snapshot = EXCLUDED.policy_snapshot,
               updated_at = now()
             RETURNING
               jurisdiction_code,
               jurisdiction_name,
               regulator_name,
               launch_phase,
               supports_fiat_collection,
               supports_fiat_payout,
               supports_crypto_settlement,
               status,
               policy_snapshot",
            &[
                &payload.jurisdiction_code,
                &payload.jurisdiction_name,
                &payload.regulator_name,
                &payload.launch_phase,
                &payload.supports_fiat_collection,
                &payload.supports_fiat_payout,
                &payload.supports_crypto_settlement,
                &payload.jurisdiction_status,
                &payload.policy_snapshot,
            ],
        )
        .await
        .map_err(map_db_error)?;
    parse_jurisdiction_row(row)
}

pub async fn list_corridors(client: &impl GenericClient) -> HttpResult<Vec<CorridorPolicy>> {
    let rows = client
        .query(
            "SELECT
               corridor_policy_id::text,
               policy_name,
               payer_jurisdiction_code,
               payee_jurisdiction_code,
               product_scope,
               price_currency_code,
               allowed_collection_currencies,
               allowed_payout_currencies,
               route_mode,
               requires_manual_review,
               allows_crypto,
               status,
               to_char(effective_from AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(effective_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               policy_snapshot
             FROM payment.corridor_policy
             ORDER BY policy_name ASC",
            &[],
        )
        .await
        .map_err(map_db_error)?;
    rows.into_iter().map(parse_corridor_row).collect()
}

pub async fn upsert_corridor(
    client: &impl GenericClient,
    payload: &CreateCorridorPolicyRequest,
) -> HttpResult<CorridorPolicy> {
    let row = client
        .query_one(
            "INSERT INTO payment.corridor_policy (
               policy_name,
               payer_jurisdiction_code,
               payee_jurisdiction_code,
               product_scope,
               price_currency_code,
               allowed_collection_currencies,
               allowed_payout_currencies,
               route_mode,
               requires_manual_review,
               allows_crypto,
               status,
               effective_from,
               effective_to,
               policy_snapshot
             ) VALUES (
               $1, $2, $3, $4, $5, $6::text[], $7::text[], $8, $9, $10, $11,
               $12::timestamptz, $13::timestamptz, $14::jsonb
             )
             ON CONFLICT (policy_name)
             DO UPDATE SET
               payer_jurisdiction_code = EXCLUDED.payer_jurisdiction_code,
               payee_jurisdiction_code = EXCLUDED.payee_jurisdiction_code,
               product_scope = EXCLUDED.product_scope,
               price_currency_code = EXCLUDED.price_currency_code,
               allowed_collection_currencies = EXCLUDED.allowed_collection_currencies,
               allowed_payout_currencies = EXCLUDED.allowed_payout_currencies,
               route_mode = EXCLUDED.route_mode,
               requires_manual_review = EXCLUDED.requires_manual_review,
               allows_crypto = EXCLUDED.allows_crypto,
               status = EXCLUDED.status,
               effective_from = EXCLUDED.effective_from,
               effective_to = EXCLUDED.effective_to,
               policy_snapshot = EXCLUDED.policy_snapshot,
               updated_at = now()
             RETURNING
               corridor_policy_id::text,
               policy_name,
               payer_jurisdiction_code,
               payee_jurisdiction_code,
               product_scope,
               price_currency_code,
               allowed_collection_currencies,
               allowed_payout_currencies,
               route_mode,
               requires_manual_review,
               allows_crypto,
               status,
               to_char(effective_from AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(effective_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               policy_snapshot",
            &[
                &payload.policy_name,
                &payload.payer_jurisdiction_code,
                &payload.payee_jurisdiction_code,
                &payload.product_scope,
                &payload.price_currency_code,
                &payload.allowed_collection_currencies,
                &payload.allowed_payout_currencies,
                &payload.route_mode,
                &payload.requires_manual_review,
                &payload.allows_crypto,
                &payload.corridor_status,
                &payload.effective_from,
                &payload.effective_to,
                &payload.policy_snapshot,
            ],
        )
        .await
        .map_err(map_db_error)?;
    parse_corridor_row(row)
}

pub async fn list_payout_preferences(
    client: &impl GenericClient,
    beneficiary_subject_type: Option<&str>,
    beneficiary_subject_id: Option<&str>,
) -> HttpResult<Vec<PayoutPreference>> {
    let rows = match (beneficiary_subject_type, beneficiary_subject_id) {
        (Some(subject_type), Some(subject_id)) => {
            client
                .query(
                    "SELECT
                       payout_preference_id::text,
                       beneficiary_subject_type,
                       beneficiary_subject_id::text,
                       destination_jurisdiction_code,
                       preferred_currency_code,
                       payout_method,
                       preferred_provider_key,
                       preferred_provider_account_id::text,
                       beneficiary_snapshot,
                       is_default,
                       status
                     FROM payment.payout_preference
                     WHERE beneficiary_subject_type = $1
                       AND beneficiary_subject_id = $2::text::uuid
                     ORDER BY is_default DESC, created_at DESC",
                    &[&subject_type, &subject_id],
                )
                .await
                .map_err(map_db_error)?
        }
        _ => {
            client
                .query(
                    "SELECT
                       payout_preference_id::text,
                       beneficiary_subject_type,
                       beneficiary_subject_id::text,
                       destination_jurisdiction_code,
                       preferred_currency_code,
                       payout_method,
                       preferred_provider_key,
                       preferred_provider_account_id::text,
                       beneficiary_snapshot,
                       is_default,
                       status
                     FROM payment.payout_preference
                     ORDER BY beneficiary_subject_type ASC, beneficiary_subject_id ASC, is_default DESC, created_at DESC",
                    &[],
                )
                .await
                .map_err(map_db_error)?
        }
    };

    rows.into_iter().map(parse_payout_row).collect()
}

pub async fn create_default_payout_preference(
    client: &db::Client,
    payload: &CreatePayoutPreferenceRequest,
) -> HttpResult<PayoutPreference> {
    let tx = client.transaction().await.map_err(map_db_error)?;
    tx.execute(
        "UPDATE payment.payout_preference
         SET is_default = false, updated_at = now()
         WHERE beneficiary_subject_type = $1
           AND beneficiary_subject_id = $2::text::uuid
           AND is_default = true",
        &[
            &payload.beneficiary_subject_type,
            &payload.beneficiary_subject_id,
        ],
    )
    .await
    .map_err(map_db_error)?;

    let row = tx
        .query_one(
            "INSERT INTO payment.payout_preference (
               beneficiary_subject_type,
               beneficiary_subject_id,
               destination_jurisdiction_code,
               preferred_currency_code,
               payout_method,
               preferred_provider_key,
               preferred_provider_account_id,
               beneficiary_snapshot,
               is_default,
               status
             ) VALUES (
               $1, $2::text::uuid, $3, $4, $5, $6, $7::text::uuid, $8::jsonb, true, 'active'
             )
             RETURNING
               payout_preference_id::text,
               beneficiary_subject_type,
               beneficiary_subject_id::text,
               destination_jurisdiction_code,
               preferred_currency_code,
               payout_method,
               preferred_provider_key,
               preferred_provider_account_id::text,
               beneficiary_snapshot,
               is_default,
               status",
            &[
                &payload.beneficiary_subject_type,
                &payload.beneficiary_subject_id,
                &payload.destination_jurisdiction_code,
                &payload.preferred_currency_code,
                &payload.payout_method,
                &payload.preferred_provider_key,
                &payload.preferred_provider_account_id,
                &payload.beneficiary_snapshot,
            ],
        )
        .await
        .map_err(map_db_error)?;
    tx.commit().await.map_err(map_db_error)?;
    parse_payout_row(row)
}

fn parse_jurisdiction_row(row: Row) -> HttpResult<JurisdictionProfile> {
    Ok(JurisdictionProfile {
        jurisdiction_code: row.get::<_, String>(0),
        jurisdiction_name: row.get::<_, String>(1),
        regulator_name: row.get::<_, Option<String>>(2),
        launch_phase: row.get::<_, String>(3),
        supports_fiat_collection: row.get::<_, bool>(4),
        supports_fiat_payout: row.get::<_, bool>(5),
        supports_crypto_settlement: row.get::<_, bool>(6),
        jurisdiction_status: row.get::<_, String>(7),
        policy_snapshot: row.get::<_, Value>(8),
    })
}

fn parse_corridor_row(row: Row) -> HttpResult<CorridorPolicy> {
    Ok(CorridorPolicy {
        corridor_policy_id: row.get::<_, String>(0),
        policy_name: row.get::<_, String>(1),
        payer_jurisdiction_code: row.get::<_, String>(2),
        payee_jurisdiction_code: row.get::<_, String>(3),
        product_scope: row.get::<_, String>(4),
        price_currency_code: row.get::<_, String>(5),
        allowed_collection_currencies: row.get::<_, Vec<String>>(6),
        allowed_payout_currencies: row.get::<_, Vec<String>>(7),
        route_mode: row.get::<_, String>(8),
        requires_manual_review: row.get::<_, bool>(9),
        allows_crypto: row.get::<_, bool>(10),
        corridor_status: row.get::<_, String>(11),
        effective_from: row.get::<_, Option<String>>(12),
        effective_to: row.get::<_, Option<String>>(13),
        policy_snapshot: row.get::<_, Value>(14),
    })
}

fn parse_payout_row(row: Row) -> HttpResult<PayoutPreference> {
    Ok(PayoutPreference {
        payout_preference_id: row.get::<_, String>(0),
        beneficiary_subject_type: row.get::<_, String>(1),
        beneficiary_subject_id: row.get::<_, String>(2),
        destination_jurisdiction_code: row.get::<_, String>(3),
        preferred_currency_code: row.get::<_, String>(4),
        payout_method: row.get::<_, String>(5),
        preferred_provider_key: row.get::<_, String>(6),
        preferred_provider_account_id: row.get::<_, Option<String>>(7),
        beneficiary_snapshot: row.get::<_, Value>(8),
        is_default: row.get::<_, bool>(9),
        preference_status: row.get::<_, String>(10),
    })
}
