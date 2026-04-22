use base64::Engine;

mod search_api_db;

pub(super) fn authorization_header(user_id: &str, tenant_id: &str, roles: &[&str]) -> String {
    let header =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
        serde_json::json!({
            "sub": user_id,
            "tenant_id": tenant_id,
            "realm_access": {
                "roles": roles,
            },
        })
        .to_string(),
    );
    format!("Bearer {header}.{payload}.sig")
}

#[cfg(test)]
mod route_tests {
    use super::authorization_header;
    use crate::modules::search::api::router;
    use crate::modules::search::service::{SearchPermission, is_allowed, needs_step_up};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn rejects_catalog_search_without_bearer() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/catalog/search?q=test")
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_catalog_search_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/catalog/search?q=test")
            .header(
                "authorization",
                authorization_header(
                    "11111111-1111-1111-1111-111111111111",
                    "22222222-2222-2222-2222-222222222222",
                    &["developer"],
                ),
            )
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_catalog_search_with_invalid_entity_scope() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/catalog/search?q=test&entity_scope=unknown")
            .header(
                "authorization",
                authorization_header(
                    "11111111-1111-1111-1111-111111111111",
                    "22222222-2222-2222-2222-222222222222",
                    &["buyer_operator"],
                ),
            )
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_catalog_search_with_invalid_sort() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/catalog/search?q=test&sort=unknown")
            .header(
                "authorization",
                authorization_header(
                    "11111111-1111-1111-1111-111111111111",
                    "22222222-2222-2222-2222-222222222222",
                    &["buyer_operator"],
                ),
            )
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_search_ops_without_permission() {
        let app = crate::with_stub_test_state(router());
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/ops/search/sync")
            .header(
                "authorization",
                authorization_header(
                    "11111111-1111-1111-1111-111111111111",
                    "22222222-2222-2222-2222-222222222222",
                    &["buyer_operator"],
                ),
            )
            .body(Body::empty())
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn search_ops_matrix_matches_formal_roles() {
        let admin_roles = vec!["platform_admin".to_string()];
        let audit_roles = vec!["platform_audit_security".to_string()];

        assert!(is_allowed(&admin_roles, SearchPermission::SyncRead));
        assert!(is_allowed(&admin_roles, SearchPermission::ReindexExecute));
        assert!(is_allowed(&audit_roles, SearchPermission::SyncRead));
        assert!(is_allowed(&audit_roles, SearchPermission::CacheInvalidate));
        assert!(!is_allowed(&audit_roles, SearchPermission::ReindexExecute));
        assert!(!is_allowed(&audit_roles, SearchPermission::AliasManage));
        assert!(!needs_step_up(SearchPermission::CacheInvalidate));
        assert!(needs_step_up(SearchPermission::ReindexExecute));
    }
}
