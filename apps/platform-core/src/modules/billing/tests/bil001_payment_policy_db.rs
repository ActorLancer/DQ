#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn bil001_payment_policy_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis()
            .to_string();
        let tenant_org_id = seed_tenant_org(&client, &suffix).await;
        let provider_account_id = seed_provider_account(&client, &tenant_org_id, &suffix).await;
        let request_jurisdiction = format!("req-bil001-jurisdiction-{suffix}");
        let request_corridor = format!("req-bil001-corridor-{suffix}");
        let request_payout_manage = format!("req-bil001-payout-manage-{suffix}");
        let request_payout_read = format!("req-bil001-payout-read-{suffix}");

        let app = crate::with_live_test_state(router()).await;

        let jurisdiction_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payment-jurisdictions")
                    .header("x-role", "platform_admin")
                    .header("x-step-up-token", "bil001-stepup-jurisdiction")
                    .header("x-request-id", &request_jurisdiction)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"jurisdiction_code":"SG","jurisdiction_name":"Singapore","regulator_name":"MAS","launch_phase":"launch_active","supports_fiat_collection":true,"supports_fiat_payout":true,"supports_crypto_settlement":false,"jurisdiction_status":"active","policy_snapshot":{"launch_scope":"initial_production","price_currency":"USD"}}"#,
                    ))
                    .expect("jurisdiction request should build"),
            )
            .await
            .expect("jurisdiction response");
        let jurisdiction_status = jurisdiction_response.status();
        let jurisdiction_body = to_bytes(jurisdiction_response.into_body(), usize::MAX)
            .await
            .expect("jurisdiction body");
        assert_eq!(
            jurisdiction_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&jurisdiction_body)
        );
        let jurisdiction_json: Value =
            serde_json::from_slice(&jurisdiction_body).expect("jurisdiction json");
        assert_eq!(
            jurisdiction_json["data"]["jurisdiction_code"].as_str(),
            Some("SG")
        );
        assert_eq!(
            jurisdiction_json["data"]["policy_snapshot"]["price_currency"].as_str(),
            Some("USD")
        );

        let corridor_payload = format!(
            r#"{{"policy_name":"BIL001 Corridor {suffix}","payer_jurisdiction_code":"SG","payee_jurisdiction_code":"SG","product_scope":"general","price_currency_code":"USD","allowed_collection_currencies":["USD","SGD"],"allowed_payout_currencies":["USD","SGD"],"route_mode":"partner_routed","requires_manual_review":false,"allows_crypto":false,"corridor_status":"active","effective_from":"2026-04-08T00:00:00Z","policy_snapshot":{{"real_payment_enabled":true,"scenario":"bil001"}}}}"#
        );
        let corridor_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payment-corridors")
                    .header("x-role", "platform_admin")
                    .header("x-step-up-token", "bil001-stepup-corridor")
                    .header("x-request-id", &request_corridor)
                    .header("content-type", "application/json")
                    .body(Body::from(corridor_payload))
                    .expect("corridor request should build"),
            )
            .await
            .expect("corridor response");
        let corridor_status = corridor_response.status();
        let corridor_body = to_bytes(corridor_response.into_body(), usize::MAX)
            .await
            .expect("corridor body");
        assert_eq!(
            corridor_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&corridor_body)
        );
        let corridor_json: Value = serde_json::from_slice(&corridor_body).expect("corridor json");
        let corridor_id = corridor_json["data"]["corridor_policy_id"]
            .as_str()
            .expect("corridor id")
            .to_string();
        assert_eq!(
            corridor_json["data"]["product_scope"].as_str(),
            Some("general")
        );

        let payout_payload = format!(
            r#"{{"beneficiary_subject_type":"organization","beneficiary_subject_id":"{tenant_org_id}","destination_jurisdiction_code":"SG","preferred_currency_code":"SGD","payout_method":"bank_transfer","preferred_provider_key":"offline_bank","preferred_provider_account_id":"{provider_account_id}","beneficiary_snapshot":{{"org_name":"BIL001 Tenant {suffix}"}}}}"#
        );
        let payout_manage_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/payout-preferences")
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &tenant_org_id)
                    .header("x-request-id", &request_payout_manage)
                    .header("content-type", "application/json")
                    .body(Body::from(payout_payload))
                    .expect("payout manage request should build"),
            )
            .await
            .expect("payout manage response");
        let payout_manage_status = payout_manage_response.status();
        let payout_manage_body = to_bytes(payout_manage_response.into_body(), usize::MAX)
            .await
            .expect("payout manage body");
        assert_eq!(
            payout_manage_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&payout_manage_body)
        );
        let payout_manage_json: Value =
            serde_json::from_slice(&payout_manage_body).expect("payout manage json");
        let payout_preference_id = payout_manage_json["data"]["payout_preference_id"]
            .as_str()
            .expect("payout preference id")
            .to_string();
        assert_eq!(
            payout_manage_json["data"]["preferred_provider_key"].as_str(),
            Some("offline_bank")
        );

        let payout_read_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/payout-preferences?beneficiary_subject_type=organization&beneficiary_subject_id={tenant_org_id}"
                    ))
                    .header("x-role", "tenant_admin")
                    .header("x-tenant-id", &tenant_org_id)
                    .header("x-request-id", &request_payout_read)
                    .body(Body::empty())
                    .expect("payout read request should build"),
            )
            .await
            .expect("payout read response");
        let payout_read_status = payout_read_response.status();
        let payout_read_body = to_bytes(payout_read_response.into_body(), usize::MAX)
            .await
            .expect("payout read body");
        assert_eq!(
            payout_read_status,
            StatusCode::OK,
            "{}",
            String::from_utf8_lossy(&payout_read_body)
        );
        let payout_read_json: Value =
            serde_json::from_slice(&payout_read_body).expect("payout read json");
        let payout_items = payout_read_json["data"].as_array().expect("payout list");
        assert_eq!(payout_items.len(), 1);
        assert_eq!(
            payout_items[0]["beneficiary_subject_id"].as_str(),
            Some(tenant_org_id.as_str())
        );
        assert_eq!(payout_items[0]["is_default"].as_bool(), Some(true));

        let jurisdiction_db: (String, String) = {
            let row = client
                .query_one(
                    "SELECT status, COALESCE(policy_snapshot ->> 'price_currency', '')
                     FROM payment.jurisdiction_profile
                     WHERE jurisdiction_code = 'SG'",
                    &[],
                )
                .await
                .expect("query jurisdiction db");
            (row.get(0), row.get(1))
        };
        assert_eq!(jurisdiction_db.0, "active");
        assert_eq!(jurisdiction_db.1, "USD");

        let corridor_db: (String, String, bool) = {
            let row = client
                .query_one(
                    "SELECT status, product_scope, COALESCE((policy_snapshot ->> 'real_payment_enabled')::boolean, false)
                     FROM payment.corridor_policy
                     WHERE corridor_policy_id = $1::text::uuid",
                    &[&corridor_id],
                )
                .await
                .expect("query corridor db");
            (row.get(0), row.get(1), row.get(2))
        };
        assert_eq!(corridor_db.0, "active");
        assert_eq!(corridor_db.1, "general");
        assert!(corridor_db.2);

        let payout_db: (String, String, bool) = {
            let row = client
                .query_one(
                    "SELECT preferred_currency_code, preferred_provider_key, is_default
                     FROM payment.payout_preference
                     WHERE payout_preference_id = $1::text::uuid",
                    &[&payout_preference_id],
                )
                .await
                .expect("query payout db");
            (row.get(0), row.get(1), row.get(2))
        };
        assert_eq!(payout_db.0, "SGD");
        assert_eq!(payout_db.1, "offline_bank");
        assert!(payout_db.2);

        let jurisdiction_audit: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'payment.jurisdiction.manage'",
                &[&request_jurisdiction],
            )
            .await
            .expect("query jurisdiction audit")
            .get(0);
        assert!(jurisdiction_audit >= 1);

        let corridor_audit: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'payment.corridor.manage'
                   AND ref_id = $2::text::uuid",
                &[&request_corridor, &corridor_id],
            )
            .await
            .expect("query corridor audit")
            .get(0);
        assert!(corridor_audit >= 1);

        let payout_audit: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id IN ($1, $2)
                   AND action_name IN ('payment.payout_preference.manage', 'payment.payout_preference.read')",
                &[&request_payout_manage, &request_payout_read],
            )
            .await
            .expect("query payout audit")
            .get(0);
        assert!(payout_audit >= 2);

        cleanup_seed(
            &client,
            &tenant_org_id,
            &provider_account_id,
            &payout_preference_id,
            &corridor_id,
        )
        .await;
    }

    async fn seed_tenant_org(client: &db::Client, suffix: &str) -> String {
        client
            .query_one(
                "INSERT INTO core.organization (org_name, org_type, status, metadata)
                 VALUES ($1, 'enterprise', 'active', '{}'::jsonb)
                 RETURNING org_id::text",
                &[&format!("bil001-tenant-{suffix}")],
            )
            .await
            .expect("insert tenant org")
            .get(0)
    }

    async fn seed_provider_account(
        client: &db::Client,
        tenant_org_id: &str,
        suffix: &str,
    ) -> String {
        client
            .query_one(
                "INSERT INTO payment.provider_account (
                   provider_key,
                   account_scope,
                   account_scope_id,
                   account_name,
                   settlement_subject_type,
                   settlement_subject_id,
                   jurisdiction_code,
                   account_mode,
                   status,
                   config_json
                 ) VALUES (
                   'offline_bank', 'tenant', $1::text::uuid, $2,
                   'organization', $1::text::uuid, 'SG', 'sandbox', 'active', '{}'::jsonb
                 )
                 RETURNING provider_account_id::text",
                &[&tenant_org_id, &format!("bil001-offline-bank-{suffix}")],
            )
            .await
            .expect("insert provider account")
            .get(0)
    }

    async fn cleanup_seed(
        client: &db::Client,
        tenant_org_id: &str,
        provider_account_id: &str,
        payout_preference_id: &str,
        corridor_id: &str,
    ) {
        let _ = client
            .execute(
                "DELETE FROM payment.payout_preference WHERE payout_preference_id = $1::text::uuid",
                &[&payout_preference_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.provider_account WHERE provider_account_id = $1::text::uuid",
                &[&provider_account_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM payment.corridor_policy WHERE corridor_policy_id = $1::text::uuid",
                &[&corridor_id],
            )
            .await;
        let _ = client
            .execute(
                "DELETE FROM core.organization WHERE org_id = $1::text::uuid",
                &[&tenant_org_id],
            )
            .await;
    }
}
