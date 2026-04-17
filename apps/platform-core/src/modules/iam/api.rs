use crate::modules::access;
use crate::modules::iam::domain::{
    AccessCheckRequest, AccessCheckView, AccessPermissionRuleView, ApplicationView, ConnectorView,
    CreateApplicationRequest, CreateConnectorRequest, CreateDepartmentRequest,
    CreateExecutionEnvironmentRequest, CreateInvitationRequest, CreateMfaAuthenticatorRequest,
    CreateUserRequest, DepartmentView, DeviceListQuery, DeviceView, ExecutionEnvironmentView,
    InvitationListQuery, InvitationView, MfaAuthenticatorView, OrganizationAggregateView,
    PatchApplicationRequest, RegisterOrganizationRequest, RotateApplicationSecretRequest,
    SessionContextView, SessionListQuery, SessionView, StepUpCheckRequest, StepUpCheckView,
    StepUpVerifyRequest, UserView,
};
use crate::modules::iam::service::{
    HighRiskAction, IamPermission, high_risk_action_requires_step_up, is_allowed, role_seeds,
};
use auth::{JwtParser, MockJwtParser, extract_bearer};
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse, new_external_readable_id};
use tokio_postgres::{NoTls, Row};
use tracing::info;

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/orgs/register", post(register_org))
        .route("/api/v1/iam/orgs/{id}", get(get_org))
        .route("/api/v1/iam/departments", post(create_department))
        .route("/api/v1/iam/departments/{id}", get(get_department))
        .route("/api/v1/iam/users", post(create_user))
        .route("/api/v1/iam/users/{id}", get(get_user))
        .route("/api/v1/apps", post(create_app))
        .route("/api/v1/apps/{id}", patch(patch_app).get(get_app))
        .route(
            "/api/v1/apps/{id}/credentials/rotate",
            post(rotate_app_secret),
        )
        .route(
            "/api/v1/apps/{id}/credentials/revoke",
            post(revoke_app_secret),
        )
        .route("/api/v1/users/invite", post(invite_user))
        .route(
            "/api/v1/iam/invitations",
            post(create_invitation).get(list_invitations),
        )
        .route(
            "/api/v1/iam/invitations/{id}/cancel",
            post(cancel_invitation),
        )
        .route("/api/v1/iam/sessions", get(list_sessions))
        .route("/api/v1/iam/sessions/{id}/revoke", post(revoke_session))
        .route("/api/v1/iam/devices", get(list_devices))
        .route("/api/v1/iam/devices/{id}/revoke", post(revoke_device))
        .route("/api/v1/iam/access/rules", get(list_access_rules))
        .route("/api/v1/iam/access/check", post(check_access_rule))
        .route("/api/v1/iam/rbac/seeds", get(get_rbac_seeds))
        .route("/api/v1/iam/step-up/check", post(check_step_up))
        .route(
            "/api/v1/iam/step-up/challenges/{id}/verify",
            post(verify_step_up),
        )
        .route(
            "/api/v1/iam/mfa/authenticators",
            get(list_mfa_authenticators).post(create_mfa_authenticator),
        )
        .route(
            "/api/v1/iam/mfa/authenticators/{id}",
            delete(delete_mfa_authenticator),
        )
        .route("/api/v1/iam/connectors", post(create_connector))
        .route("/api/v1/iam/connectors/{id}", get(get_connector))
        .route(
            "/api/v1/iam/execution-environments",
            post(create_execution_environment),
        )
        .route(
            "/api/v1/iam/execution-environments/{id}",
            get(get_execution_environment),
        )
        .route("/api/v1/auth/me", get(get_auth_me))
}

async fn register_org(
    headers: HeaderMap,
    Json(payload): Json<RegisterOrganizationRequest>,
) -> Result<Json<ApiResponse<OrganizationAggregateView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::OrgRegister, "org register")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let metadata = serde_json::json!({
        "certification_level": payload.certification_level,
        "risk_profile": payload.risk_profile,
        "whitelist_refs": payload.whitelist_refs,
        "graylist_refs": payload.graylist_refs,
        "blacklist_refs": payload.blacklist_refs,
    });

    let row = client
        .query_one(
            "INSERT INTO core.organization (
               org_name, org_type, status, compliance_level, country_code, metadata
             ) VALUES (
               $1, $2, 'pending_review', $3, $4, $5::jsonb
             )
             RETURNING
               org_id::text,
               org_name,
               org_type,
               status,
               country_code,
               compliance_level,
               metadata,
               to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &payload.org_name,
                &payload.org_type,
                &payload.compliance_level,
                &payload.jurisdiction_code,
                &metadata,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let view = parse_org_row(&row, false);
    write_audit_event(
        &client,
        "organization",
        &view.org_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.org.register",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "iam.org.register",
        org_id = %view.org_id,
        org_type = %view.org_type,
        "organization registered"
    );
    Ok(ApiResponse::ok(view))
}

async fn get_org(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<OrganizationAggregateView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::OrgRead, "org read")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT
               o.org_id::text,
               o.org_name,
               o.org_type,
               o.status,
               o.country_code,
               o.compliance_level,
               o.metadata,
               to_char(o.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               to_char(o.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
               EXISTS (
                 SELECT 1 FROM risk.blacklist_entry b
                 WHERE b.subject_type = 'organization'
                   AND b.subject_id = o.org_id
                   AND b.status = 'active'
               ) AS blacklist_active
             FROM core.organization o
             WHERE o.org_id = $1::text::uuid",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;

    let row =
        row.ok_or_else(|| not_found("organization not found", header(&headers, "x-request-id")))?;
    let view = parse_org_row(&row, row.get::<_, bool>(9));
    write_audit_event(
        &client,
        "organization",
        &view.org_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.org.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn create_department(
    headers: HeaderMap,
    Json(payload): Json<CreateDepartmentRequest>,
) -> Result<Json<ApiResponse<DepartmentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "department create")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let row = client
        .query_one(
            "INSERT INTO core.department (
               org_id, department_name, parent_department_id
             ) VALUES (
               $1::text::uuid, $2, $3::text::uuid
             )
             RETURNING
               department_id::text,
               org_id::text,
               department_name,
               parent_department_id::text,
               status",
            &[
                &payload.org_id,
                &payload.department_name,
                &payload.parent_department_id,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let view = DepartmentView {
        department_id: row.get(0),
        org_id: row.get(1),
        department_name: row.get(2),
        parent_department_id: row.get(3),
        status: row.get(4),
    };
    write_audit_event(
        &client,
        "department",
        &view.department_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.department.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn get_department(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<DepartmentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "department read")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT
               department_id::text,
               org_id::text,
               department_name,
               parent_department_id::text,
               status
             FROM core.department
             WHERE department_id = $1::text::uuid",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("department not found", header(&headers, "x-request-id")))?;
    Ok(ApiResponse::ok(DepartmentView {
        department_id: row.get(0),
        org_id: row.get(1),
        department_name: row.get(2),
        parent_department_id: row.get(3),
        status: row.get(4),
    }))
}

async fn create_user(
    headers: HeaderMap,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "user create")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let row = client
        .query_one(
            "INSERT INTO core.user_account (
               org_id, department_id, login_id, display_name, user_type, email, phone, mfa_status
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3, $4, $5, $6, $7, 'pending'
             )
             RETURNING
               user_id::text,
               org_id::text,
               department_id::text,
               login_id::text,
               display_name,
               user_type,
               status,
               email::text,
               phone",
            &[
                &payload.org_id,
                &payload.department_id,
                &payload.login_id,
                &payload.display_name,
                &payload
                    .user_type
                    .clone()
                    .unwrap_or_else(|| "human".to_string()),
                &payload.email,
                &payload.phone,
            ],
        )
        .await
        .map_err(map_db_error)?;

    let view = UserView {
        user_id: row.get(0),
        org_id: row.get(1),
        department_id: row.get(2),
        login_id: row.get(3),
        display_name: row.get(4),
        user_type: row.get(5),
        status: row.get(6),
        email: row.get(7),
        phone: row.get(8),
    };
    write_audit_event(
        &client,
        "user",
        &view.user_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.user.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn get_user(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<UserView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "user read")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT
               user_id::text,
               org_id::text,
               department_id::text,
               login_id::text,
               display_name,
               user_type,
               status,
               email::text,
               phone
             FROM core.user_account
             WHERE user_id = $1::text::uuid",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| not_found("user not found", header(&headers, "x-request-id")))?;
    Ok(ApiResponse::ok(UserView {
        user_id: row.get(0),
        org_id: row.get(1),
        department_id: row.get(2),
        login_id: row.get(3),
        display_name: row.get(4),
        user_type: row.get(5),
        status: row.get(6),
        email: row.get(7),
        phone: row.get(8),
    }))
}

async fn create_app(
    headers: HeaderMap,
    Json(payload): Json<CreateApplicationRequest>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "app create")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_one(
            "INSERT INTO core.application (
               org_id, app_name, app_type, status, client_id, client_secret_hash, metadata
             ) VALUES (
               $1::text::uuid, $2, $3, 'active', $4, $5, $6::jsonb
             )
             RETURNING app_id::text, org_id::text, app_name, app_type, status, client_id, metadata",
            &[
                &payload.org_id,
                &payload.app_name,
                &payload
                    .app_type
                    .clone()
                    .unwrap_or_else(|| "api_client".to_string()),
                &payload.client_id,
                &payload.client_secret_hash,
                &serde_json::json!({
                    "client_secret_status": if payload.client_secret_hash.is_some() { "active" } else { "missing" }
                }),
            ],
        )
        .await
        .map_err(map_db_error)?;
    let view = ApplicationView {
        app_id: row.get(0),
        org_id: row.get(1),
        app_name: row.get(2),
        app_type: row.get(3),
        status: row.get(4),
        client_id: row.get(5),
        client_secret_status: row
            .get::<_, serde_json::Value>(6)
            .get("client_secret_status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
    };
    write_audit_event(
        &client,
        "application",
        &view.app_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.app.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn patch_app(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PatchApplicationRequest>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "app patch")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "UPDATE core.application
             SET
               app_name = COALESCE($2, app_name),
               status = COALESCE($3, status),
               updated_at = now()
             WHERE app_id = $1::text::uuid
             RETURNING app_id::text, org_id::text, app_name, app_type, status, client_id, metadata",
            &[&id, &payload.app_name, &payload.status],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("application not found", header(&headers, "x-request-id")))?;
    let view = ApplicationView {
        app_id: row.get(0),
        org_id: row.get(1),
        app_name: row.get(2),
        app_type: row.get(3),
        status: row.get(4),
        client_id: row.get(5),
        client_secret_status: row
            .get::<_, serde_json::Value>(6)
            .get("client_secret_status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
    };
    write_audit_event(
        &client,
        "application",
        &view.app_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.app.patch",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn get_app(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "app read")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT app_id::text, org_id::text, app_name, app_type, status, client_id, metadata
             FROM core.application
             WHERE app_id = $1::text::uuid",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("application not found", header(&headers, "x-request-id")))?;
    Ok(ApiResponse::ok(ApplicationView {
        app_id: row.get(0),
        org_id: row.get(1),
        app_name: row.get(2),
        app_type: row.get(3),
        status: row.get(4),
        client_id: row.get(5),
        client_secret_status: row
            .get::<_, serde_json::Value>(6)
            .get("client_secret_status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
    }))
}

async fn rotate_app_secret(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<RotateApplicationSecretRequest>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityWrite,
        "application credential rotate",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let secret_hash = payload
        .client_secret_hash
        .unwrap_or_else(|| new_external_readable_id("appsec"));
    let row = client
        .query_opt(
            "UPDATE core.application
             SET
               client_secret_hash = $2,
               metadata = jsonb_set(
                 jsonb_set(COALESCE(metadata, '{}'::jsonb), '{client_secret_status}', '\"active\"'::jsonb, true),
                 '{client_secret_rotated_at}', to_jsonb(now()), true
               ),
               updated_at = now()
             WHERE app_id = $1::text::uuid
             RETURNING app_id::text, org_id::text, app_name, app_type, status, client_id, metadata",
            &[&id, &secret_hash],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("application not found", header(&headers, "x-request-id")))?;
    let view = ApplicationView {
        app_id: row.get(0),
        org_id: row.get(1),
        app_name: row.get(2),
        app_type: row.get(3),
        status: row.get(4),
        client_id: row.get(5),
        client_secret_status: row
            .get::<_, serde_json::Value>(6)
            .get("client_secret_status")
            .and_then(|v| v.as_str())
            .unwrap_or("active")
            .to_string(),
    };
    write_audit_event(
        &client,
        "application",
        &view.app_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.app.secret.rotate",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn revoke_app_secret(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityWrite,
        "application credential revoke",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "UPDATE core.application
             SET
               client_secret_hash = NULL,
               metadata = jsonb_set(
                 jsonb_set(COALESCE(metadata, '{}'::jsonb), '{client_secret_status}', '\"revoked\"'::jsonb, true),
                 '{client_secret_revoked_at}', to_jsonb(now()), true
               ),
               updated_at = now()
             WHERE app_id = $1::text::uuid
             RETURNING app_id::text, org_id::text, app_name, app_type, status, client_id, metadata",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("application not found", header(&headers, "x-request-id")))?;
    let view = ApplicationView {
        app_id: row.get(0),
        org_id: row.get(1),
        app_name: row.get(2),
        app_type: row.get(3),
        status: row.get(4),
        client_id: row.get(5),
        client_secret_status: row
            .get::<_, serde_json::Value>(6)
            .get("client_secret_status")
            .and_then(|v| v.as_str())
            .unwrap_or("revoked")
            .to_string(),
    };
    write_audit_event(
        &client,
        "application",
        &view.app_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.app.secret.revoke",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn invite_user(
    headers: HeaderMap,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<Json<ApiResponse<InvitationView>>, (StatusCode, Json<ErrorResponse>)> {
    create_invitation_internal(headers, payload, "iam.user.invite").await
}

async fn create_invitation(
    headers: HeaderMap,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<Json<ApiResponse<InvitationView>>, (StatusCode, Json<ErrorResponse>)> {
    create_invitation_internal(headers, payload, "iam.invitation.create").await
}

async fn create_invitation_internal(
    headers: HeaderMap,
    payload: CreateInvitationRequest,
    audit_action: &str,
) -> Result<Json<ApiResponse<InvitationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "invitation create")?;
    if payload.invited_email.is_none() && payload.invited_phone.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "invited_email or invited_phone is required".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let expires_hours = payload.expires_in_hours.unwrap_or(72).max(1);
    let row = client
        .query_one(
            "INSERT INTO iam.invitation (
               org_id, invited_email, invited_phone, invited_role_snapshot, invitation_type,
               token_hash, expires_at, created_by_user_id
             ) VALUES (
               $1::text::uuid, $2::citext, $3, $4::jsonb, $5, $6, now() + ($7::bigint || ' hours')::interval, $8::text::uuid
             )
             RETURNING invitation_id::text, org_id::text, invited_email::text, invited_phone,
                       invitation_type, status,
                       to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[
                &payload.org_id,
                &payload.invited_email,
                &payload.invited_phone,
                &serde_json::Value::Array(
                    payload
                        .invited_roles
                        .iter()
                        .map(|r| serde_json::Value::String(r.clone()))
                        .collect(),
                ),
                &payload
                    .invitation_type
                    .clone()
                    .unwrap_or_else(|| "member".to_string()),
                &new_external_readable_id("invite"),
                &expires_hours,
                &header(&headers, "x-user-id"),
            ],
        )
        .await
        .map_err(map_db_error)?;
    let view = InvitationView {
        invitation_id: row.get(0),
        org_id: row.get(1),
        invited_email: row.get(2),
        invited_phone: row.get(3),
        invitation_type: row.get(4),
        status: row.get(5),
        expires_at: row.get(6),
    };
    write_audit_event(
        &client,
        "invitation",
        &view.invitation_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        audit_action,
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn list_invitations(
    headers: HeaderMap,
    Query(query): Query<InvitationListQuery>,
) -> Result<Json<ApiResponse<Vec<InvitationView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "invitation read")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let rows = client
        .query(
            "SELECT invitation_id::text, org_id::text, invited_email::text, invited_phone,
                    invitation_type, status,
                    to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM iam.invitation
             WHERE ($1::text IS NULL OR org_id = $1::text::uuid)
               AND ($2::text IS NULL OR status = $2)
             ORDER BY created_at DESC
             LIMIT 100",
            &[&query.org_id, &query.status],
        )
        .await
        .map_err(map_db_error)?;
    let views = rows
        .iter()
        .map(|row| InvitationView {
            invitation_id: row.get(0),
            org_id: row.get(1),
            invited_email: row.get(2),
            invited_phone: row.get(3),
            invitation_type: row.get(4),
            status: row.get(5),
            expires_at: row.get(6),
        })
        .collect();
    Ok(ApiResponse::ok(views))
}

async fn cancel_invitation(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<InvitationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "invitation cancel")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "UPDATE iam.invitation
             SET status = 'cancelled', updated_at = now()
             WHERE invitation_id = $1::text::uuid
             RETURNING invitation_id::text, org_id::text, invited_email::text, invited_phone,
                       invitation_type, status,
                       to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("invitation not found", header(&headers, "x-request-id")))?;
    let view = InvitationView {
        invitation_id: row.get(0),
        org_id: row.get(1),
        invited_email: row.get(2),
        invited_phone: row.get(3),
        invitation_type: row.get(4),
        status: row.get(5),
        expires_at: row.get(6),
    };
    write_audit_event(
        &client,
        "invitation",
        &view.invitation_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.invitation.cancel",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn list_sessions(
    headers: HeaderMap,
    Query(query): Query<SessionListQuery>,
) -> Result<Json<ApiResponse<Vec<SessionView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SessionRead, "session list")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let rows = client
        .query(
            "SELECT session_id::text, user_id::text, trusted_device_id::text, session_type,
                    auth_context_level, session_status,
                    to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM iam.user_session
             WHERE ($1::text IS NULL OR user_id = $1::text::uuid)
               AND ($2::text IS NULL OR session_status = $2)
             ORDER BY created_at DESC
             LIMIT 100",
            &[&query.user_id, &query.status],
        )
        .await
        .map_err(map_db_error)?;
    let views = rows
        .iter()
        .map(|row| SessionView {
            session_id: row.get(0),
            user_id: row.get(1),
            trusted_device_id: row.get(2),
            session_type: row.get(3),
            auth_context_level: row.get(4),
            session_status: row.get(5),
            expires_at: row.get(6),
        })
        .collect();
    Ok(ApiResponse::ok(views))
}

async fn revoke_session(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<SessionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "session revoke")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "UPDATE iam.user_session
             SET session_status = 'revoked', revoked_at = now(), updated_at = now()
             WHERE session_id = $1::text::uuid
             RETURNING session_id::text, user_id::text, trusted_device_id::text, session_type,
                       auth_context_level, session_status,
                       to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("session not found", header(&headers, "x-request-id")))?;
    let view = SessionView {
        session_id: row.get(0),
        user_id: row.get(1),
        trusted_device_id: row.get(2),
        session_type: row.get(3),
        auth_context_level: row.get(4),
        session_status: row.get(5),
        expires_at: row.get(6),
    };
    write_audit_event(
        &client,
        "session",
        &view.session_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.session.revoke",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn list_devices(
    headers: HeaderMap,
    Query(query): Query<DeviceListQuery>,
) -> Result<Json<ApiResponse<Vec<DeviceView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SessionRead, "device list")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let rows = client
        .query(
            "SELECT trusted_device_id::text, user_id::text, device_name, platform, browser,
                    trust_level, status,
                    to_char(last_seen_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
             FROM iam.trusted_device
             WHERE ($1::text IS NULL OR user_id = $1::text::uuid)
               AND ($2::text IS NULL OR status = $2)
             ORDER BY created_at DESC
             LIMIT 100",
            &[&query.user_id, &query.status],
        )
        .await
        .map_err(map_db_error)?;
    let views = rows
        .iter()
        .map(|row| DeviceView {
            trusted_device_id: row.get(0),
            user_id: row.get(1),
            device_name: row.get(2),
            platform: row.get(3),
            browser: row.get(4),
            trust_level: row.get(5),
            status: row.get(6),
            last_seen_at: row.get(7),
        })
        .collect();
    Ok(ApiResponse::ok(views))
}

async fn revoke_device(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<DeviceView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "device revoke")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "UPDATE iam.trusted_device
             SET status = 'revoked', updated_at = now()
             WHERE trusted_device_id = $1::text::uuid
             RETURNING trusted_device_id::text, user_id::text, device_name, platform, browser,
                       trust_level, status,
                       to_char(last_seen_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| not_found("device not found", header(&headers, "x-request-id")))?;
    let view = DeviceView {
        trusted_device_id: row.get(0),
        user_id: row.get(1),
        device_name: row.get(2),
        platform: row.get(3),
        browser: row.get(4),
        trust_level: row.get(5),
        status: row.get(6),
        last_seen_at: row.get(7),
    };
    write_audit_event(
        &client,
        "trusted_device",
        &view.trusted_device_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.device.revoke",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn create_connector(
    headers: HeaderMap,
    Json(payload): Json<CreateConnectorRequest>,
) -> Result<Json<ApiResponse<ConnectorView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "connector create")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_one(
            "INSERT INTO core.connector (
               org_id, connector_name, connector_type, status, endpoint_ref
             ) VALUES (
               $1::text::uuid, $2, $3, 'draft', $4
             )
             RETURNING connector_id::text, org_id::text, connector_name, connector_type, status, endpoint_ref",
            &[
                &payload.org_id,
                &payload.connector_name,
                &payload.connector_type,
                &payload.endpoint_ref,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let view = ConnectorView {
        connector_id: row.get(0),
        org_id: row.get(1),
        connector_name: row.get(2),
        connector_type: row.get(3),
        status: row.get(4),
        endpoint_ref: row.get(5),
    };
    write_audit_event(
        &client,
        "connector",
        &view.connector_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.connector.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn get_connector(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ConnectorView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "connector read")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT connector_id::text, org_id::text, connector_name, connector_type, status, endpoint_ref
             FROM core.connector
             WHERE connector_id = $1::text::uuid",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("connector not found", header(&headers, "x-request-id")))?;
    Ok(ApiResponse::ok(ConnectorView {
        connector_id: row.get(0),
        org_id: row.get(1),
        connector_name: row.get(2),
        connector_type: row.get(3),
        status: row.get(4),
        endpoint_ref: row.get(5),
    }))
}

async fn create_execution_environment(
    headers: HeaderMap,
    Json(payload): Json<CreateExecutionEnvironmentRequest>,
) -> Result<Json<ApiResponse<ExecutionEnvironmentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityWrite,
        "execution environment create",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_one(
            "INSERT INTO core.execution_environment (
               org_id, connector_id, environment_name, environment_type, status, region_code
             ) VALUES (
               $1::text::uuid, $2::text::uuid, $3, $4, 'draft', $5
             )
             RETURNING environment_id::text, org_id::text, connector_id::text, environment_name, environment_type, status, region_code",
            &[
                &payload.org_id,
                &payload.connector_id,
                &payload.environment_name,
                &payload.environment_type,
                &payload.region_code,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let view = ExecutionEnvironmentView {
        environment_id: row.get(0),
        org_id: row.get(1),
        connector_id: row.get(2),
        environment_name: row.get(3),
        environment_type: row.get(4),
        status: row.get(5),
        region_code: row.get(6),
    };
    write_audit_event(
        &client,
        "execution_environment",
        &view.environment_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.execution_environment.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn get_execution_environment(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ExecutionEnvironmentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityRead,
        "execution environment read",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT environment_id::text, org_id::text, connector_id::text, environment_name, environment_type, status, region_code
             FROM core.execution_environment
             WHERE environment_id = $1::text::uuid",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| {
        not_found(
            "execution environment not found",
            header(&headers, "x-request-id"),
        )
    })?;
    Ok(ApiResponse::ok(ExecutionEnvironmentView {
        environment_id: row.get(0),
        org_id: row.get(1),
        connector_id: row.get(2),
        environment_name: row.get(3),
        environment_type: row.get(4),
        status: row.get(5),
        region_code: row.get(6),
    }))
}

async fn get_auth_me(
    headers: HeaderMap,
) -> Result<Json<ApiResponse<SessionContextView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SessionRead, "session read")?;
    if let Some(token) = extract_bearer(&headers) {
        let parser = MockJwtParser;
        let subject = parser.parse_subject(&token).map_err(|err| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: err.to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
        let roles = headers
            .get("x-role")
            .and_then(|v| v.to_str().ok())
            .map(|role| vec![role.to_string()])
            .unwrap_or(subject.roles);
        return Ok(ApiResponse::ok(SessionContextView {
            mode: "jwt_mirror".to_string(),
            user_id: Some(subject.user_id),
            org_id: None,
            login_id: None,
            display_name: None,
            tenant_id: Some(subject.tenant_id),
            roles,
            auth_context_level: "aal1".to_string(),
        }));
    }

    let Some(login_id) = header(&headers, "x-login-id") else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "missing bearer token or x-login-id for auth context".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };

    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT user_id::text, org_id::text, login_id::text, display_name
             FROM core.user_account
             WHERE login_id = $1::citext",
            &[&login_id],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "local test user not found".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;

    let roles = headers
        .get("x-role")
        .and_then(|v| v.to_str().ok())
        .map(|role| vec![role.to_string()])
        .unwrap_or_else(|| vec!["tenant_admin".to_string()]);
    let user_id: String = row.get(0);
    write_audit_event(
        &client,
        "session",
        &user_id,
        roles.first().map(|s| s.as_str()).unwrap_or("unknown"),
        "iam.session.context.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(SessionContextView {
        mode: "local_test_user".to_string(),
        user_id: Some(user_id),
        org_id: Some(row.get(1)),
        login_id: Some(row.get(2)),
        display_name: Some(row.get(3)),
        tenant_id: None,
        roles,
        auth_context_level: "aal1".to_string(),
    }))
}

async fn list_access_rules(
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<AccessPermissionRuleView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::AccessPolicyRead,
        "access policy read",
    )?;
    Ok(ApiResponse::ok(default_access_rules()))
}

async fn check_access_rule(
    headers: HeaderMap,
    Json(payload): Json<AccessCheckRequest>,
) -> Result<Json<ApiResponse<AccessCheckView>>, (StatusCode, Json<ErrorResponse>)> {
    let permission = permission_from_code(&payload.permission_code)?;
    let role = header(&headers, "x-role").unwrap_or_default();
    let mut allowed = is_allowed(&role, permission);
    if allowed {
        if let Some(scope) = payload.scope.as_deref() {
            allowed = allowed && validate_scope(&role, scope);
        }
        if let Some(api) = payload.api.as_deref() {
            allowed = allowed && validate_api_pattern(&payload.permission_code, api);
        }
        if let Some(button_key) = payload.button_key.as_deref() {
            allowed = allowed && validate_button_key(&payload.permission_code, button_key);
        }
    }
    Ok(ApiResponse::ok(AccessCheckView {
        allowed,
        permission_code: payload.permission_code,
        matched_role: if allowed { Some(role) } else { None },
    }))
}

async fn get_rbac_seeds(
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::AccessPolicyRead, "rbac seed read")?;
    let seeds = role_seeds()
        .iter()
        .map(|seed| {
            serde_json::json!({
                "role": seed.role,
                "domain": format!("{:?}", seed.domain).to_lowercase(),
                "permissions": seed.permissions.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    Ok(ApiResponse::ok(seeds))
}

async fn check_step_up(
    headers: HeaderMap,
    Json(payload): Json<StepUpCheckRequest>,
) -> Result<Json<ApiResponse<StepUpCheckView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::StepUpRead, "step-up check")?;
    let action = high_risk_action_from_name(&payload.action_name)?;
    let requires_step_up = high_risk_action_requires_step_up(action);
    let status = if requires_step_up {
        "challenge_required"
    } else {
        "not_required"
    };
    let user_id = header(&headers, "x-user-id").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "x-user-id is required for step-up challenge".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;

    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_one(
            "INSERT INTO iam.step_up_challenge (
               user_id, challenge_type, target_action, target_ref_type, target_ref_id,
               challenge_status, expires_at, metadata
             ) VALUES (
               $1::text::uuid, 'mock_otp', $2, $3, $4::text::uuid, $5, now() + interval '10 minutes', $6::jsonb
             )
             RETURNING step_up_challenge_id::text",
            &[
                &user_id,
                &payload.action_name,
                &payload.target_ref_type,
                &payload.target_ref_id,
                &status,
                &serde_json::json!({
                    "mode": std::env::var("IAM_STEP_UP_MODE").unwrap_or_else(|_| "mock".to_string())
                }),
            ],
        )
        .await
        .map_err(map_db_error)?;
    let challenge_id: String = row.get(0);

    write_audit_event(
        &client,
        "step_up_challenge",
        &challenge_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.step_up.check",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(StepUpCheckView {
        challenge_id,
        action_name: payload.action_name,
        requires_step_up,
        status: status.to_string(),
    }))
}

async fn verify_step_up(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<StepUpVerifyRequest>,
) -> Result<Json<ApiResponse<StepUpCheckView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::StepUpWrite, "step-up verify")?;
    let mode = std::env::var("IAM_STEP_UP_MODE").unwrap_or_else(|_| "mock".to_string());
    let expected_code =
        std::env::var("IAM_STEP_UP_MOCK_CODE").unwrap_or_else(|_| "000000".to_string());
    let verification_passed = mode == "mock" && payload.verification_code == expected_code;
    let next_status = if verification_passed {
        "verified"
    } else {
        "rejected"
    };

    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "UPDATE iam.step_up_challenge
             SET challenge_status = $2, completed_at = CASE WHEN $2 = 'verified' THEN now() ELSE completed_at END, updated_at = now()
             WHERE step_up_challenge_id = $1::text::uuid
             RETURNING step_up_challenge_id::text, target_action",
            &[&id, &next_status],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| {
        not_found(
            "step-up challenge not found",
            header(&headers, "x-request-id"),
        )
    })?;

    write_audit_event(
        &client,
        "step_up_challenge",
        &id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.step_up.verify",
        if verification_passed {
            "success"
        } else {
            "failed_verification"
        },
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(StepUpCheckView {
        challenge_id: row.get(0),
        action_name: row.get(1),
        requires_step_up: true,
        status: next_status.to_string(),
    }))
}

async fn list_mfa_authenticators(
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<MfaAuthenticatorView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::MfaRead, "mfa authenticator read")?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let user_id = header(&headers, "x-user-id");
    let rows = client
        .query(
            "SELECT mfa_authenticator_id::text, user_id::text, authenticator_type, device_label, status
             FROM iam.mfa_authenticator
             WHERE ($1::text IS NULL OR user_id = $1::text::uuid)
             ORDER BY created_at DESC
             LIMIT 100",
            &[&user_id],
        )
        .await
        .map_err(map_db_error)?;
    let views = rows
        .iter()
        .map(|row| MfaAuthenticatorView {
            authenticator_id: row.get(0),
            user_id: row.get(1),
            authenticator_type: row.get(2),
            device_label: row.get(3),
            status: row.get(4),
        })
        .collect();
    Ok(ApiResponse::ok(views))
}

async fn create_mfa_authenticator(
    headers: HeaderMap,
    Json(payload): Json<CreateMfaAuthenticatorRequest>,
) -> Result<Json<ApiResponse<MfaAuthenticatorView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::MfaWrite,
        "mfa authenticator create",
    )?;
    let mode = std::env::var("IAM_MFA_MODE").unwrap_or_else(|_| "mock".to_string());
    if mode != "mock" && mode != "disabled" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "IAM_MFA_MODE must be mock or disabled in local".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_one(
            "INSERT INTO iam.mfa_authenticator (
               user_id, authenticator_type, device_label, status, metadata
             ) VALUES (
               $1::text::uuid, $2, $3, 'active', $4::jsonb
             )
             RETURNING mfa_authenticator_id::text, user_id::text, authenticator_type, device_label, status",
            &[
                &payload.user_id,
                &payload.authenticator_type,
                &payload.device_label,
                &serde_json::json!({"mode": mode}),
            ],
        )
        .await
        .map_err(map_db_error)?;
    let view = MfaAuthenticatorView {
        authenticator_id: row.get(0),
        user_id: row.get(1),
        authenticator_type: row.get(2),
        device_label: row.get(3),
        status: row.get(4),
    };
    write_audit_event(
        &client,
        "mfa_authenticator",
        &view.authenticator_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.mfa.authenticator.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

async fn delete_mfa_authenticator(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<MfaAuthenticatorView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::MfaWrite,
        "mfa authenticator delete",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "UPDATE iam.mfa_authenticator
             SET status = 'revoked', updated_at = now()
             WHERE mfa_authenticator_id = $1::text::uuid
             RETURNING mfa_authenticator_id::text, user_id::text, authenticator_type, device_label, status",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| {
        not_found(
            "mfa authenticator not found",
            header(&headers, "x-request-id"),
        )
    })?;
    let view = MfaAuthenticatorView {
        authenticator_id: row.get(0),
        user_id: row.get(1),
        authenticator_type: row.get(2),
        device_label: row.get(3),
        status: row.get(4),
    };
    write_audit_event(
        &client,
        "mfa_authenticator",
        &view.authenticator_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.mfa.authenticator.delete",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    Ok(ApiResponse::ok(view))
}

fn default_access_rules() -> Vec<AccessPermissionRuleView> {
    access::ACCESS_RULES
        .iter()
        .map(|rule| AccessPermissionRuleView {
            permission_code: rule.permission_code.to_string(),
            scopes: rule.scopes.iter().map(|s| (*s).to_string()).collect(),
            api_patterns: rule.api_patterns.iter().map(|s| format!("{s}*")).collect(),
            button_keys: rule.button_keys.iter().map(|s| (*s).to_string()).collect(),
        })
        .collect()
}

fn permission_from_code(
    permission_code: &str,
) -> Result<IamPermission, (StatusCode, Json<ErrorResponse>)> {
    match permission_code {
        "iam.org.register" => Ok(IamPermission::OrgRegister),
        "iam.org.read" => Ok(IamPermission::OrgRead),
        "iam.identity.write" => Ok(IamPermission::IdentityWrite),
        "iam.identity.read" => Ok(IamPermission::IdentityRead),
        "iam.session.read" => Ok(IamPermission::SessionRead),
        "iam.stepup.write" => Ok(IamPermission::StepUpWrite),
        "iam.stepup.read" => Ok(IamPermission::StepUpRead),
        "iam.mfa.read" => Ok(IamPermission::MfaRead),
        "iam.mfa.write" => Ok(IamPermission::MfaWrite),
        "iam.access.policy.read" => Ok(IamPermission::AccessPolicyRead),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: format!("unknown permission code: {permission_code}"),
                request_id: None,
            }),
        )),
    }
}

fn validate_scope(role: &str, scope: &str) -> bool {
    match scope {
        "tenant" => matches!(role, "tenant_admin" | "tenant_operator"),
        "platform" => matches!(
            role,
            "platform_admin" | "platform_auditor" | "platform_finance_operator"
        ),
        "audit" => role == "platform_auditor",
        "developer" => role == "developer",
        _ => false,
    }
}

fn validate_api_pattern(permission_code: &str, api: &str) -> bool {
    access::find_rule(permission_code)
        .map(|rule| {
            rule.api_patterns
                .iter()
                .any(|pattern| api.starts_with(pattern))
        })
        .unwrap_or(false)
}

fn validate_button_key(permission_code: &str, button_key: &str) -> bool {
    access::find_rule(permission_code)
        .map(|rule| rule.button_keys.iter().any(|key| key == &button_key))
        .unwrap_or(false)
}

fn high_risk_action_from_name(
    action_name: &str,
) -> Result<HighRiskAction, (StatusCode, Json<ErrorResponse>)> {
    match action_name {
        "risk.product.freeze" => Ok(HighRiskAction::ProductFreeze),
        "billing.compensation.payout" => Ok(HighRiskAction::CompensationPayout),
        "audit.evidence.export" => Ok(HighRiskAction::EvidenceExport),
        "audit.evidence.replay" => Ok(HighRiskAction::EvidenceReplay),
        "iam.permission.change" => Ok(HighRiskAction::PermissionChange),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: format!("unsupported high-risk action: {action_name}"),
                request_id: None,
            }),
        )),
    }
}

fn parse_org_row(row: &Row, blacklist_active: bool) -> OrganizationAggregateView {
    let metadata: serde_json::Value = row.get(6);
    OrganizationAggregateView {
        org_id: row.get(0),
        org_name: row.get(1),
        org_type: row.get(2),
        org_status: row.get(3),
        jurisdiction_code: row.get(4),
        compliance_level: row.get(5),
        certification_level: metadata
            .get("certification_level")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        whitelist_refs: metadata
            .get("whitelist_refs")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        graylist_refs: metadata
            .get("graylist_refs")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        blacklist_refs: metadata
            .get("blacklist_refs")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        blacklist_active,
        created_at: row.get(7),
        updated_at: row.get(8),
    }
}

async fn write_audit_event(
    client: &tokio_postgres::Client,
    ref_type: &str,
    ref_id: &str,
    actor_role: &str,
    action_name: &str,
    result_code: &str,
    request_id: Option<&str>,
    trace_id: Option<&str>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    client
        .query_one(
            "INSERT INTO audit.audit_event (
               domain_name, ref_type, ref_id, actor_type, actor_id, action_name, result_code,
               request_id, trace_id, metadata
             ) VALUES (
               'iam', $1, $2::text::uuid, 'role', NULL, $3, $4, $5, $6, $7::jsonb
             )
             RETURNING audit_id::text",
            &[
                &ref_type,
                &ref_id,
                &action_name,
                &result_code,
                &request_id,
                &trace_id,
                &serde_json::json!({
                    "actor_role": actor_role,
                    "event_id": new_external_readable_id("iam"),
                }),
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

fn require_permission(
    headers: &HeaderMap,
    permission: IamPermission,
    action: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = headers
        .get("x-role")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if is_allowed(role, permission) {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: format!("{action} is forbidden for current role"),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn database_dsn() -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    std::env::var("DATABASE_URL").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: ErrorCode::OpsInternal.as_str().to_string(),
                message: "DATABASE_URL is not configured".to_string(),
                request_id: None,
            }),
        )
    })
}

async fn connect_db(
    dsn: &str,
) -> Result<
    (
        tokio_postgres::Client,
        tokio_postgres::Connection<tokio_postgres::Socket, tokio_postgres::tls::NoTlsStream>,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    tokio_postgres::connect(dsn, NoTls).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: ErrorCode::OpsInternal.as_str().to_string(),
                message: format!("database connection failed: {err}"),
                request_id: None,
            }),
        )
    })
}

fn not_found(message: &str, request_id: Option<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: message.to_string(),
            request_id,
        }),
    )
}

fn map_db_error(err: tokio_postgres::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("iam persistence failed: {err}"),
            request_id: None,
        }),
    )
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn rejects_org_register_without_permission() {
        let app = router();
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
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/auth/me")
                    .method("GET")
                    .header("x-role", "tenant_admin")
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_app_patch_for_tenant_operator() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/apps/10000000-0000-0000-0000-000000000401")
                    .method("PATCH")
                    .header("x-role", "tenant_operator")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"status":"disabled"}"#))
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_invitation_create_for_developer_role() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/invitations")
                    .method("POST")
                    .header("x-role", "developer")
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
    async fn rejects_session_revoke_for_tenant_operator() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/sessions/10000000-0000-0000-0000-000000000401/revoke")
                    .method("POST")
                    .header("x-role", "tenant_operator")
                    .body(Body::empty())
                    .expect("request build"),
            )
            .await
            .expect("router response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_step_up_check_without_user_id() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/step-up/check")
                    .method("POST")
                    .header("x-role", "tenant_operator")
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
    async fn rejects_mfa_create_for_developer_role() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/iam/mfa/authenticators")
                    .method("POST")
                    .header("x-role", "developer")
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
}
