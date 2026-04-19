use crate::modules::catalog::router::router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use db::{GenericClient, NoTls, connect};
use serde_json::Value;
use tower::ServiceExt;

fn live_db_enabled() -> bool {
    std::env::var("CATALOG_DB_SMOKE").ok().as_deref() == Some("1")
}

#[tokio::test]
async fn cat023_standard_scenarios_endpoint_db_smoke() {
    if !live_db_enabled() {
        return;
    }
    let Ok(dsn) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (client, connection) = connect(&dsn, NoTls).await.expect("connect database");
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
    let request_id = format!("req-cat023-standard-scenarios-{suffix}");

    let outcome: Result<(), String> = async {
        let app = crate::with_live_test_state(router()).await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/catalog/standard-scenarios")
                    .header("x-role", "tenant_admin")
                    .header("x-request-id", &request_id)
                    .body(Body::empty())
                    .map_err(|err| format!("build request: {err}"))?,
            )
            .await
            .map_err(|err| format!("call standard scenarios endpoint: {err}"))?;
        if resp.status() != StatusCode::OK {
            let status = resp.status();
            let body = to_bytes(resp.into_body(), usize::MAX)
                .await
                .map_err(|err| format!("read error body: {err}"))?;
            return Err(format!(
                "status mismatch: expected 200, got {status}; body={}",
                String::from_utf8_lossy(&body)
            ));
        }
        let body = to_bytes(resp.into_body(), usize::MAX)
            .await
            .map_err(|err| format!("read response body: {err}"))?;
        let json: Value =
            serde_json::from_slice(&body).map_err(|err| format!("decode response json: {err}"))?;
        let scenarios = json["data"]
            .as_array()
            .ok_or_else(|| "response data is not array".to_string())?;
        if scenarios.len() != 5 {
            return Err(format!(
                "standard scenario count mismatch: expected 5, got {}",
                scenarios.len()
            ));
        }
        let codes: std::collections::BTreeSet<String> = scenarios
            .iter()
            .filter_map(|item| item["scenario_code"].as_str().map(ToString::to_string))
            .collect();
        let expected = ["S1", "S2", "S3", "S4", "S5"]
            .into_iter()
            .map(ToString::to_string)
            .collect::<std::collections::BTreeSet<_>>();
        if codes != expected {
            return Err(format!("scenario code set mismatch: got {:?}", codes));
        }
        let audit_row = client
            .query_one(
                "SELECT count(*)::bigint
                 FROM audit.audit_event
                 WHERE request_id = $1
                   AND action_name = 'catalog.standard.scenarios.read'
                   AND ref_type = 'catalog_standard_scenarios'
                   AND ref_id = $2::text::uuid",
                &[&request_id, &"00000000-0000-0000-0000-000000000023"],
            )
            .await
            .map_err(|err| format!("query standard scenario audit: {err}"))?;
        let audit_count: i64 = audit_row.get(0);
        if audit_count < 1 {
            return Err("catalog.standard.scenarios.read audit event missing".to_string());
        }
        Ok(())
    }
    .await;

    let _ = client
        .execute(
            "DELETE FROM audit.audit_event WHERE request_id = $1",
            &[&request_id],
        )
        .await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
}
