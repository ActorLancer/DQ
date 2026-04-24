#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn trade002_freeze_price_snapshot_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let order_id = "30000000-0000-0000-0000-000000000101";
        let request_id = format!(
            "req-trade002-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix epoch")
                .as_secs()
        );

        let app = crate::with_live_test_state(router()).await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/trade/orders/{order_id}/price-snapshot/freeze"
                    ))
                    .header("x-role", "buyer_operator")
                    .header("x-request-id", &request_id)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let json: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(
            json["data"]["snapshot"]["billing_mode"].as_str(),
            Some("one_time")
        );

        let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });
        let row = client
            .query_one(
                "SELECT price_snapshot_json, fee_preview_snapshot
                 FROM trade.order_main
                 WHERE order_id = $1::text::uuid",
                &[&order_id],
            )
            .await
            .expect("query order");
        let price_snapshot: Value = row.get(0);
        let fee_preview: Value = row.get(1);
        assert_eq!(price_snapshot["billing_mode"].as_str(), Some("one_time"));
        assert!(fee_preview.get("pricing_mode").is_some());

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'trade.order.price_snapshot.freeze'",
                &[&request_id],
            )
            .await
            .expect("query audit count")
            .get(0);
        assert!(audit_count >= 1);
    }
}
