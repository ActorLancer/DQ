#[cfg(test)]
mod tests {
    use super::super::super::api::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use db::{GenericClient, NoTls, connect};
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn trade001_create_and_get_pre_request_db_smoke() {
        if std::env::var("TRADE_DB_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let dsn = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".into());
        let request_id = format!(
            "req-trade001-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix epoch")
                .as_secs()
        );

        let app = crate::with_live_test_state(router()).await;
        let create_resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/trade/pre-requests")
                    .header("x-role", "buyer_operator")
                    .header("x-request-id", &request_id)
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                          "buyer_org_id":"10000000-0000-0000-0000-000000000102",
                          "product_id":"20000000-0000-0000-0000-000000000301",
                          "created_by":"10000000-0000-0000-0000-000000000302",
                          "request_kind":"sample_request",
                          "details":{
                            "title":"Need sample",
                            "description":"Need sample rows for validation",
                            "sample_field_scope":["field_a","field_b"]
                          },
                          "contract_expectation":{
                            "expected_term_days":30,
                            "expected_sla_tier":"gold"
                          },
                          "authorization_expectation":{
                            "grant_scope":"sample_preview",
                            "access_mode":"read_only",
                            "export_allowed":false,
                            "requires_poc_environment":false
                          }
                        }"#,
                    ))
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(create_resp.status(), StatusCode::OK);
        let create_body = axum::body::to_bytes(create_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let create_json: Value = serde_json::from_slice(&create_body).expect("json");
        assert_eq!(create_json["code"].as_str(), Some("OK"));
        assert_eq!(create_json["message"].as_str(), Some("success"));
        assert_eq!(
            create_json["request_id"].as_str(),
            Some(request_id.as_str())
        );
        let inquiry_id = create_json["data"]["inquiry_id"]
            .as_str()
            .expect("inquiry id")
            .to_string();
        assert_eq!(
            create_json["data"]["request_kind"].as_str(),
            Some("sample_request")
        );
        assert_eq!(create_json["data"]["current_state"].as_str(), Some("open"));

        let get_resp = crate::with_live_test_state(router())
            .await
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/trade/pre-requests/{inquiry_id}"))
                    .header("x-role", "buyer_operator")
                    .header("x-request-id", format!("{request_id}-read"))
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("response");
        assert_eq!(get_resp.status(), StatusCode::OK);

        let (client, connection) = connect(&dsn, NoTls).await.expect("connect db");
        tokio::spawn(async move {
            let _ = connection.await;
        });
        let row = client
            .query_one(
                "SELECT status, message_text FROM trade.inquiry WHERE inquiry_id = $1::text::uuid",
                &[&inquiry_id],
            )
            .await
            .expect("query inquiry");
        let status: String = row.get(0);
        let message_text: Option<String> = row.get(1);
        assert_eq!(status, "open");
        let payload: Value =
            serde_json::from_str(message_text.as_deref().unwrap_or("{}")).expect("payload json");
        assert_eq!(payload["request_kind"].as_str(), Some("sample_request"));

        let audit_count: i64 = client
            .query_one(
                "SELECT COUNT(*)::bigint FROM audit.audit_event WHERE request_id = $1",
                &[&request_id],
            )
            .await
            .expect("query audit count")
            .get(0);
        assert!(audit_count >= 1);
    }
}
