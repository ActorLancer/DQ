use crate::modules::iam::api::router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{Client, Error, GenericClient, NoTls, connect};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("IAM_DB_SMOKE").ok().as_deref() == Some("1")
}

#[derive(Debug)]
struct SeedIds {
    org_id: String,
    registry_id: String,
    binding_id: String,
}

const AUD016_OPERATOR_ORG_ID: &str = "10000000-0000-0000-0000-000000000416";
const AUD016_OPERATOR_USER_ID: &str = "10000000-0000-0000-0000-000000000417";

async fn ensure_operator_user(client: &Client) -> Result<String, Error> {
    client
        .execute(
            "INSERT INTO core.organization (
               org_id, org_name, org_type, status, metadata
             ) VALUES (
               $1::text::uuid, 'aud016-platform-operators', 'platform', 'active',
               jsonb_build_object('source', 'aud016-fixture')
             )
             ON CONFLICT (org_id) DO UPDATE
             SET org_name = EXCLUDED.org_name,
                 org_type = EXCLUDED.org_type,
                 status = EXCLUDED.status,
                 metadata = core.organization.metadata || EXCLUDED.metadata",
            &[&AUD016_OPERATOR_ORG_ID],
        )
        .await?;
    client
        .execute(
            "INSERT INTO core.user_account (
               user_id, org_id, login_id, display_name, user_type, status, mfa_status, email, attrs
             ) VALUES (
               $1::text::uuid, $2::text::uuid, 'aud016-platform-admin@example.test',
               'AUD016 Platform Admin', 'human', 'active', 'verified',
               'aud016-platform-admin@example.test',
               jsonb_build_object('fixture', 'aud016', 'role', 'platform_admin')
             )
             ON CONFLICT (user_id) DO UPDATE
             SET org_id = EXCLUDED.org_id,
                 login_id = EXCLUDED.login_id,
                 display_name = EXCLUDED.display_name,
                 user_type = EXCLUDED.user_type,
                 status = EXCLUDED.status,
                 mfa_status = EXCLUDED.mfa_status,
                 email = EXCLUDED.email,
                 attrs = core.user_account.attrs || EXCLUDED.attrs",
            &[&AUD016_OPERATOR_USER_ID, &AUD016_OPERATOR_ORG_ID],
        )
        .await?;
    Ok(AUD016_OPERATOR_USER_ID.to_string())
}

async fn seed_fabric_identity(client: &Client, suffix: &str) -> Result<SeedIds, Error> {
    let org = client
        .query_one(
            "INSERT INTO core.organization (
               org_name, org_type, status, metadata
             ) VALUES (
               $1, 'enterprise', 'active', jsonb_build_object('source', 'aud016')
             )
             RETURNING org_id::text",
            &[&format!("aud016-org-{suffix}")],
        )
        .await?;
    let org_id: String = org.get(0);

    let registry = client
        .query_one(
            "INSERT INTO iam.fabric_ca_registry (
               org_id, registry_name, msp_id, ca_name, ca_url, ca_type,
               status, enrollment_profile, config_json
             ) VALUES (
               $1::text::uuid, $2, 'DATABMSP', 'fabric-ca', 'http://127.0.0.1:7054', 'fabric_ca',
               'active', 'default', jsonb_build_object('mode', 'mock')
             )
             RETURNING fabric_ca_registry_id::text",
            &[&org_id, &format!("aud016-registry-{suffix}")],
        )
        .await?;
    let registry_id: String = registry.get(0);

    let binding = client
        .query_one(
            "INSERT INTO iam.fabric_identity_binding (
               fabric_ca_registry_id, org_id, msp_id, affiliation, enrollment_id,
               identity_type, attrs_snapshot, status
             ) VALUES (
               $1::text::uuid, $2::text::uuid, 'DATABMSP', 'platform.security',
               $3, 'user', jsonb_build_object('scope', 'audit_admin'), 'approved'
             )
             RETURNING fabric_identity_binding_id::text",
            &[
                &registry_id,
                &org_id,
                &format!("aud016-enrollment-{suffix}"),
            ],
        )
        .await?;
    let binding_id: String = binding.get(0);

    Ok(SeedIds {
        org_id,
        registry_id,
        binding_id,
    })
}

async fn cleanup_seed(
    client: &Client,
    seed: &SeedIds,
    certificate_id: Option<&str>,
    request_ids: &[&str],
    challenge_ids: &[&str],
) -> Result<(), Error> {
    for request_id in request_ids {
        client
            .execute(
                "DELETE FROM ops.external_fact_receipt WHERE request_id = $1",
                &[request_id],
            )
            .await?;
    }
    if let Some(certificate_id) = certificate_id {
        client
            .execute(
                "DELETE FROM iam.certificate_revocation_record WHERE certificate_id = $1::text::uuid",
                &[&certificate_id],
            )
            .await?;
        client
            .execute(
                "DELETE FROM iam.certificate_record WHERE certificate_id = $1::text::uuid",
                &[&certificate_id],
            )
            .await?;
    }
    for challenge_id in challenge_ids {
        client
            .execute(
                "DELETE FROM iam.step_up_challenge WHERE step_up_challenge_id = $1::text::uuid",
                &[challenge_id],
            )
            .await?;
    }
    client
        .execute(
            "DELETE FROM iam.fabric_identity_binding WHERE fabric_identity_binding_id = $1::text::uuid",
            &[&seed.binding_id],
        )
        .await?;
    client
        .execute(
            "DELETE FROM iam.fabric_ca_registry WHERE fabric_ca_registry_id = $1::text::uuid",
            &[&seed.registry_id],
        )
        .await?;
    client
        .execute(
            "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
            &[&seed.org_id],
        )
        .await?;
    Ok(())
}

async fn fabric_ca_service_is_live() -> bool {
    let base_url = std::env::var("FABRIC_CA_ADMIN_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:18112".to_string());
    reqwest::get(format!("{}/healthz", base_url.trim_end_matches('/')))
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}

async fn parse_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    serde_json::from_slice(&body).expect("response JSON")
}

#[tokio::test]
async fn iam_fabric_ca_admin_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    assert!(
        fabric_ca_service_is_live().await,
        "fabric-ca-admin must be running before IAM_DB_SMOKE=1 smoke"
    );

    let dsn = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string());
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
    tokio::spawn(async move {
        if let Err(err) = connection.await {
            panic!("db connection error: {err}");
        }
    });

    let seed_suffix = format!(
        "db-smoke-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    );
    let seed = seed_fabric_identity(&client, &seed_suffix)
        .await
        .expect("seed fabric identity");
    let operator_user_id = ensure_operator_user(&client)
        .await
        .expect("ensure operator user");
    let issue_request_id = format!("req-aud016-issue-{seed_suffix}");
    let revoke_request_id = format!("req-aud016-revoke-{seed_suffix}");
    let app = crate::with_live_test_state(router()).await;

    let issue_challenge_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/iam/step-up/check")
                .header("x-role", "platform_admin")
                .header("x-user-id", &operator_user_id)
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"action_name\":\"iam.fabric.identity.issue\",\"target_ref_type\":\"fabric_identity_binding\",\"target_ref_id\":\"{}\"}}",
                    seed.binding_id
                )))
                .expect("request"),
        )
        .await
        .expect("step-up check response");
    assert_eq!(issue_challenge_response.status(), StatusCode::OK);
    let issue_challenge_json = parse_json(issue_challenge_response).await;
    let issue_challenge_id = issue_challenge_json["data"]["challenge_id"]
        .as_str()
        .expect("issue challenge id")
        .to_string();

    let issue_verify_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/iam/step-up/challenges/{}/verify",
                    issue_challenge_id
                ))
                .header("x-role", "platform_admin")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"verification_code":"000000"}"#))
                .expect("request"),
        )
        .await
        .expect("step-up verify response");
    assert_eq!(issue_verify_response.status(), StatusCode::OK);

    let issue_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/iam/fabric-identities/{}/issue",
                    seed.binding_id
                ))
                .header("x-role", "platform_admin")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &issue_request_id)
                .header("x-trace-id", "trace-aud016-issue")
                .header("x-step-up-challenge-id", &issue_challenge_id)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("issue response");
    assert_eq!(issue_response.status(), StatusCode::OK);
    let issue_json = parse_json(issue_response).await;
    assert_eq!(
        issue_json["data"]["target_id"].as_str(),
        Some(seed.binding_id.as_str())
    );
    assert_eq!(issue_json["data"]["status"].as_str(), Some("issued"));

    let binding_row = client
        .query_one(
            "SELECT status, certificate_id::text
             FROM iam.fabric_identity_binding
             WHERE fabric_identity_binding_id = $1::text::uuid",
            &[&seed.binding_id],
        )
        .await
        .expect("query issued binding");
    let issued_status: String = binding_row.get(0);
    let certificate_id: String = binding_row.get(1);
    assert_eq!(issued_status, "issued");

    let certificate_row = client
        .query_one(
            "SELECT status
             FROM iam.certificate_record
             WHERE certificate_id = $1::text::uuid",
            &[&certificate_id],
        )
        .await
        .expect("query issued certificate");
    let certificate_status: String = certificate_row.get(0);
    assert_eq!(certificate_status, "active");

    let issue_receipt_row = client
        .query_one(
            "SELECT receipt_status, metadata ->> 'event_type'
             FROM ops.external_fact_receipt
             WHERE request_id = $1
               AND ref_type = 'fabric_identity_binding'
               AND fact_type = 'certificate_issue_receipt'
             ORDER BY created_at DESC
             LIMIT 1",
            &[&issue_request_id],
        )
        .await
        .expect("query issue receipt");
    let issue_receipt_status: String = issue_receipt_row.get(0);
    let issue_event_type: String = issue_receipt_row.get(1);
    assert_eq!(issue_receipt_status, "confirmed");
    assert_eq!(issue_event_type, "ca.certificate_issued");

    let audit_issue_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE action_name = 'iam.fabric.identity.issue'
               AND request_id = $1
               AND actor_id = $2::text::uuid",
            &[&issue_request_id, &operator_user_id],
        )
        .await
        .expect("query issue audit count")
        .get(0);
    assert_eq!(audit_issue_count, 1);

    let issue_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = $1
               AND message_text = 'fabric ca admin issued identity'",
            &[&issue_request_id],
        )
        .await
        .expect("query issue system log count")
        .get(0);
    assert_eq!(issue_log_count, 1);

    let revoke_challenge_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/iam/step-up/check")
                .header("x-role", "platform_admin")
                .header("x-user-id", &operator_user_id)
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"action_name\":\"iam.certificate.revoke\",\"target_ref_type\":\"certificate_record\",\"target_ref_id\":\"{}\"}}",
                    certificate_id
                )))
                .expect("request"),
        )
        .await
        .expect("revoke check response");
    assert_eq!(revoke_challenge_response.status(), StatusCode::OK);
    let revoke_challenge_json = parse_json(revoke_challenge_response).await;
    let revoke_challenge_id = revoke_challenge_json["data"]["challenge_id"]
        .as_str()
        .expect("revoke challenge id")
        .to_string();

    let revoke_verify_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/iam/step-up/challenges/{}/verify",
                    revoke_challenge_id
                ))
                .header("x-role", "platform_admin")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"verification_code":"000000"}"#))
                .expect("request"),
        )
        .await
        .expect("revoke verify response");
    assert_eq!(revoke_verify_response.status(), StatusCode::OK);

    let revoke_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/iam/certificates/{}/revoke",
                    certificate_id
                ))
                .header("x-role", "platform_admin")
                .header("x-user-id", &operator_user_id)
                .header("x-request-id", &revoke_request_id)
                .header("x-trace-id", "trace-aud016-revoke")
                .header("x-step-up-challenge-id", &revoke_challenge_id)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("revoke response");
    assert_eq!(revoke_response.status(), StatusCode::OK);
    let revoke_json = parse_json(revoke_response).await;
    assert_eq!(
        revoke_json["data"]["target_id"].as_str(),
        Some(certificate_id.as_str())
    );
    assert_eq!(revoke_json["data"]["status"].as_str(), Some("revoked"));

    let revoked_certificate_status: String = client
        .query_one(
            "SELECT status
             FROM iam.certificate_record
             WHERE certificate_id = $1::text::uuid",
            &[&certificate_id],
        )
        .await
        .expect("query revoked certificate")
        .get(0);
    assert_eq!(revoked_certificate_status, "revoked");

    let revoked_binding_status: String = client
        .query_one(
            "SELECT status
             FROM iam.fabric_identity_binding
             WHERE fabric_identity_binding_id = $1::text::uuid",
            &[&seed.binding_id],
        )
        .await
        .expect("query revoked binding")
        .get(0);
    assert_eq!(revoked_binding_status, "revoked");

    let revocation_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM iam.certificate_revocation_record
             WHERE certificate_id = $1::text::uuid",
            &[&certificate_id],
        )
        .await
        .expect("query revocation count")
        .get(0);
    assert_eq!(revocation_count, 1);

    let revoke_receipt_row = client
        .query_one(
            "SELECT receipt_status, metadata ->> 'event_type'
             FROM ops.external_fact_receipt
             WHERE request_id = $1
               AND ref_type = 'certificate_record'
               AND fact_type = 'certificate_revocation_receipt'
             ORDER BY created_at DESC
             LIMIT 1",
            &[&revoke_request_id],
        )
        .await
        .expect("query revoke receipt");
    let revoke_receipt_status: String = revoke_receipt_row.get(0);
    let revoke_event_type: String = revoke_receipt_row.get(1);
    assert_eq!(revoke_receipt_status, "confirmed");
    assert_eq!(revoke_event_type, "ca.certificate_revoked");

    let audit_revoke_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM audit.audit_event
             WHERE action_name = 'iam.certificate.revoke'
               AND request_id = $1
               AND actor_id = $2::text::uuid",
            &[&revoke_request_id, &operator_user_id],
        )
        .await
        .expect("query revoke audit count")
        .get(0);
    assert_eq!(audit_revoke_count, 1);

    let revoke_log_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::bigint
             FROM ops.system_log
             WHERE request_id = $1
               AND message_text = 'fabric ca admin revoked certificate'",
            &[&revoke_request_id],
        )
        .await
        .expect("query revoke system log count")
        .get(0);
    assert_eq!(revoke_log_count, 1);

    cleanup_seed(
        &client,
        &seed,
        Some(&certificate_id),
        &[&issue_request_id, &revoke_request_id],
        &[&issue_challenge_id, &revoke_challenge_id],
    )
    .await
    .expect("cleanup aud016 seed");
}
