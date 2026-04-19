#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tokio_postgres::NoTls;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn trade023_order_templates_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
            .await
            .expect("connect database");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let suffix = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_millis()
        );
        let request_id = format!("req-trade023-order-templates-{suffix}");

        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/orders/standard-templates")
                    .header("x-role", "tenant_admin")
                    .header("x-request-id", &request_id)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        let templates = json["data"].as_array().expect("template list");
        assert_eq!(templates.len(), 5);

        let codes = templates
            .iter()
            .filter_map(|item| item["scenario_code"].as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(codes, ["S1", "S2", "S3", "S4", "S5"].into_iter().collect());

        let sku_set = templates
            .iter()
            .fold(std::collections::BTreeSet::new(), |mut acc, item| {
                if let Some(primary) = item["primary_sku"].as_str() {
                    acc.insert(primary.to_string());
                }
                if let Some(extra) = item["supplementary_skus"].as_array() {
                    for sku in extra.iter().filter_map(|v| v.as_str()) {
                        acc.insert(sku.to_string());
                    }
                }
                acc
            });
        let expected = [
            "API_SUB", "API_PPU", "FILE_STD", "FILE_SUB", "SBX_STD", "SHARE_RO", "QRY_LITE",
            "RPT_STD",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(sku_set, expected);

        assert_eq!(
            templates[0]["order_draft"]["per_sku_snapshot_required"].as_bool(),
            Some(true)
        );

        let audit_count: i64 = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'trade.order.templates.read'
                   AND ref_type = 'trade_order_templates'
                   AND ref_id = $2::text::uuid",
                &[&request_id, &"00000000-0000-0000-0000-000000000123"],
            )
            .await
            .expect("query order template audit")
            .get(0);
        assert!(audit_count >= 1);
    }
}
