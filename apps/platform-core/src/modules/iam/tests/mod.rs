mod api_db;

#[cfg(test)]
mod tests {
    use super::super::api::*;
    use super::super::service::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::http::StatusCode;
    use tower::util::ServiceExt;

    #[test]
    fn role_matrix_for_org_register() {
        assert!(is_allowed("platform_admin", IamPermission::OrgRegister));
        assert!(is_allowed("tenant_admin", IamPermission::OrgRegister));
        assert!(!is_allowed("tenant_developer", IamPermission::OrgRegister));
    }

    #[test]
    fn role_matrix_for_identity_write() {
        assert!(is_allowed("platform_admin", IamPermission::IdentityWrite));
        assert!(is_allowed("tenant_admin", IamPermission::IdentityWrite));
        assert!(!is_allowed(
            "tenant_developer",
            IamPermission::IdentityWrite
        ));
    }

    #[test]
    fn role_matrix_for_session_read() {
        assert!(is_allowed("buyer_operator", IamPermission::SessionRead));
        assert!(is_allowed("seller_operator", IamPermission::SessionRead));
        assert!(is_allowed("tenant_developer", IamPermission::SessionRead));
        assert!(is_allowed("platform_reviewer", IamPermission::SessionRead));
        assert!(is_allowed(
            "platform_risk_settlement",
            IamPermission::SessionRead
        ));
        assert!(!is_allowed("guest", IamPermission::SessionRead));
    }

    #[test]
    fn role_matrix_for_platform_reviewer_subject_queue() {
        assert!(is_allowed("platform_reviewer", IamPermission::OrgRead));
        assert!(is_allowed("platform_reviewer", IamPermission::IdentityRead));
        assert!(is_allowed(
            "platform_reviewer",
            IamPermission::AccessPolicyRead
        ));
        assert!(!is_allowed(
            "platform_reviewer",
            IamPermission::IdentityWrite
        ));
    }

    #[test]
    fn role_matrix_for_step_up_write() {
        assert!(is_allowed("platform_admin", IamPermission::StepUpWrite));
        assert!(is_allowed("tenant_admin", IamPermission::StepUpWrite));
        assert!(!is_allowed(
            "platform_audit_security",
            IamPermission::StepUpWrite
        ));
    }

    #[test]
    fn all_high_risk_actions_require_step_up() {
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::ProductFreeze
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::CompensationPayout
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::EvidenceExport
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::EvidenceReplay
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::PermissionChange
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::FabricIdentityIssue
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::FabricIdentityRevoke
        ));
        assert!(high_risk_action_requires_step_up(
            HighRiskAction::CertificateRevoke
        ));
    }

    #[test]
    fn role_matrix_for_sso_and_fabric_permissions() {
        assert!(is_allowed("tenant_admin", IamPermission::SsoWrite));
        assert!(is_allowed("platform_admin", IamPermission::FabricWrite));
        assert!(is_allowed(
            "platform_audit_security",
            IamPermission::FabricRead
        ));
        assert!(is_allowed(
            "fabric_ca_admin",
            IamPermission::FabricIdentityIssue
        ));
        assert!(is_allowed(
            "platform_admin",
            IamPermission::CertificateRevoke
        ));
        assert!(!is_allowed("tenant_developer", IamPermission::SsoWrite));
    }

    #[tokio::test]
    async fn rejects_org_register_without_permission() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/orgs/register")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"org_name":"Acme","org_type":"enterprise"}"#))
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_auth_me_without_session_context_headers() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/auth/me")
                    .method("GET")
                    .header("x-role", "tenant_developer")
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_app_patch_for_tenant_developer() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/apps/10000000-0000-0000-0000-000000000401")
                    .method("PATCH")
                    .header("x-role", "tenant_developer")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"status":"disabled"}"#))
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_invitation_create_for_tenant_developer_role() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/invitations")
                    .method("POST")
                    .header("x-role", "tenant_developer")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"org_id":"10000000-0000-0000-0000-000000000001","invited_email":"new@acme.test"}"#,
                    ))
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_session_revoke_for_tenant_developer() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/sessions/10000000-0000-0000-0000-000000000401/revoke")
                    .method("POST")
                    .header("x-role", "tenant_developer")
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_step_up_check_without_user_id() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/step-up/check")
                    .method("POST")
                    .header("x-role", "tenant_admin")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"action_name":"risk.product.freeze","target_ref_type":"product"}"#,
                    ))
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_mfa_create_for_tenant_developer_role() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/mfa/authenticators")
                    .method("POST")
                    .header("x-role", "tenant_developer")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"user_id":"10000000-0000-0000-0000-000000000401","authenticator_type":"totp"}"#,
                    ))
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_sso_create_for_tenant_developer_role() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/sso/connections")
                    .method("POST")
                    .header("x-role", "tenant_developer")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"org_id":"10000000-0000-0000-0000-000000000001","connection_name":"corp-oidc"}"#,
                    ))
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_logout_for_tenant_developer() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/auth/logout")
                    .method("POST")
                    .header("x-role", "tenant_developer")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"session_id":"10000000-0000-0000-0000-000000000411"}"#,
                    ))
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_fabric_issue_without_step_up() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/fabric-identities/10000000-0000-0000-0000-000000000901/issue")
                    .method("POST")
                    .header("x-role", "platform_admin")
                    .header("x-user-id", "10000000-0000-0000-0000-000000000304")
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_certificate_revoke_without_step_up() {
        let app = crate::with_stub_test_state(router());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/certificates/10000000-0000-0000-0000-000000000902/revoke")
                    .method("POST")
                    .header("x-role", "platform_admin")
                    .header("x-user-id", "10000000-0000-0000-0000-000000000304")
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
