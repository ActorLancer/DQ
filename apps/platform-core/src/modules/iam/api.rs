use crate::AppState;
use crate::modules::access;
use crate::modules::iam::domain::{
    AccessCheckRequest, AccessCheckView, AccessPermissionRuleView, ActionResultView,
    ApplicationListQuery, ApplicationView, CertificateView, ConnectorListQuery, ConnectorView,
    CreateApplicationRequest, CreateConnectorRequest, CreateDepartmentRequest,
    CreateExecutionEnvironmentRequest, CreateInvitationRequest, CreateMfaAuthenticatorRequest,
    CreateSsoConnectionRequest, CreateUserRequest, DepartmentListQuery, DepartmentView,
    DeviceListQuery, DeviceView, ExecutionEnvironmentListQuery, ExecutionEnvironmentView,
    FabricIdentityView, InvitationListQuery, InvitationView, LoginRequest, LoginView,
    LogoutRequest, MfaAuthenticatorView, OrganizationAggregateView, OrganizationListQuery,
    PatchApplicationRequest, PatchOrganizationLinkageRequest, PatchSsoConnectionRequest,
    RegisterOrganizationRequest, RotateApplicationSecretRequest, SessionContextView,
    SessionListQuery, SessionView, SsoConnectionListQuery, SsoConnectionView, StepUpCheckRequest,
    StepUpCheckView, StepUpVerifyRequest, UpdateUserRolesRequest, UserListQuery, UserView,
};
use crate::modules::iam::repository::PostgresIamRepository;
use crate::modules::iam::service::{
    HighRiskAction, IamPermission, high_risk_action_requires_step_up, is_allowed, role_seeds,
};
use auth::{JwtParser, KeycloakClaimsJwtParser, MockJwtParser, extract_bearer};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use db::{DbClientOps, DbRecord, Error};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse, new_external_readable_id};
use tracing::info;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/orgs/register", post(register_org))
        .route("/api/v1/iam/orgs", get(list_orgs))
        .route("/api/v1/iam/orgs/{id}", get(get_org))
        .route(
            "/api/v1/iam/orgs/{id}/party-review-linkage",
            patch(patch_org_party_review_linkage),
        )
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route(
            "/api/v1/iam/departments",
            post(create_department).get(list_departments),
        )
        .route("/api/v1/iam/departments/{id}", get(get_department))
        .route("/api/v1/iam/users", post(create_user).get(list_users))
        .route("/api/v1/iam/users/{id}", get(get_user))
        .route("/api/v1/iam/users/{id}/roles", post(update_user_roles))
        .route("/api/v1/apps", post(create_app).get(list_apps))
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
        .route("/api/v1/iam/step-up/challenges", post(check_step_up))
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
        .route(
            "/api/v1/iam/sso/connections",
            post(create_sso_connection).get(list_sso_connections),
        )
        .route(
            "/api/v1/iam/sso/connections/{id}",
            patch(patch_sso_connection),
        )
        .route("/api/v1/iam/fabric-identities", get(list_fabric_identities))
        .route(
            "/api/v1/iam/fabric-identities/{id}/issue",
            post(issue_fabric_identity),
        )
        .route(
            "/api/v1/iam/fabric-identities/{id}/revoke",
            post(revoke_fabric_identity),
        )
        .route("/api/v1/iam/certificates", get(list_certificates))
        .route(
            "/api/v1/iam/certificates/{id}/revoke",
            post(revoke_certificate),
        )
        .route(
            "/api/v1/iam/connectors",
            post(create_connector).get(list_connectors),
        )
        .route("/api/v1/iam/connectors/{id}", get(get_connector))
        .route(
            "/api/v1/iam/execution-environments",
            post(create_execution_environment).get(list_execution_environments),
        )
        .route(
            "/api/v1/iam/execution-environments/{id}",
            get(get_execution_environment),
        )
        .route("/api/v1/auth/me", get(get_auth_me))
}

async fn register_org(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<RegisterOrganizationRequest>,
) -> Result<Json<ApiResponse<OrganizationAggregateView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::OrgRegister, "org register")?;
    let client = state.db.client().map_err(map_db_error)?;

    let metadata = serde_json::json!({
        "certification_level": payload.certification_level,
        "risk_profile": payload.risk_profile,
        "whitelist_refs": payload.whitelist_refs,
        "graylist_refs": payload.graylist_refs,
        "blacklist_refs": payload.blacklist_refs,
    });

    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "organization",
        &view.org_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.org.register",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<OrganizationAggregateView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::OrgRead, "org read")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "organization",
        &view.org_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.org.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn list_orgs(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<OrganizationListQuery>,
) -> Result<Json<ApiResponse<Vec<OrganizationAggregateView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::OrgRead, "org list")?;
    let client = state.db.client().map_err(map_db_error)?;
    let rows = client
        .query(
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
             WHERE ($1::text IS NULL OR o.status = $1)
               AND ($2::text IS NULL OR o.org_type = $2)
             ORDER BY o.created_at DESC
             LIMIT 100",
            &[&query.status, &query.org_type],
        )
        .await
        .map_err(map_db_error)?;
    let views = rows
        .iter()
        .map(|row| parse_org_row(row, row.get::<_, bool>(9)))
        .collect();
    Ok(ApiResponse::ok(views))
}

async fn patch_org_party_review_linkage(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PatchOrganizationLinkageRequest>,
) -> Result<Json<ApiResponse<OrganizationAggregateView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityWrite,
        "party review linkage patch",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "UPDATE core.organization
             SET metadata = jsonb_strip_nulls(
               COALESCE(metadata, '{}'::jsonb)
               || jsonb_build_object(
                    'review_status', $2::text,
                    'risk_status', $3::text,
                    'sellable_status', $4::text,
                    'freeze_reason', $5::text
                  )
             ),
             updated_at = now()
             WHERE org_id = $1::text::uuid
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
                &id,
                &payload.review_status,
                &payload.risk_status,
                &payload.sellable_status,
                &payload.freeze_reason,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("organization not found", header(&headers, "x-request-id")))?;
    let view = parse_org_row(&row, false);
    write_audit_event(
        &tx,
        "organization",
        &view.org_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.org.party_review_linkage.patch",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn login(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SessionWrite, "login")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "SELECT user_id::text, org_id::text
             FROM core.user_account
             WHERE login_id = $1::citext",
            &[&payload.login_id],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "invalid login id".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;
    let user_id: String = row.get(0);
    let org_id: String = row.get(1);
    let row = tx
        .query_one(
            "INSERT INTO iam.user_session (
               user_id, login_method, auth_context_level, session_type, current_ip, current_country_code,
               session_status, expires_at, metadata
             ) VALUES (
               $1::text::uuid, 'password', 'aal1', 'web', NULL, NULL, 'active', now() + interval '24 hours', $2::jsonb
             )
             RETURNING session_id::text, user_id::text, session_status",
            &[
                &user_id,
                &serde_json::json!({
                    "login_id": payload.login_id,
                    "mode": std::env::var("IAM_LOGIN_MODE").unwrap_or_else(|_| "local_mock".to_string())
                }),
            ],
        )
        .await
        .map_err(map_db_error)?;
    let view = LoginView {
        session_id: row.get(0),
        user_id: row.get(1),
        org_id,
        session_status: row.get(2),
    };
    write_audit_event(
        &tx,
        "session",
        &view.session_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.session.login",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn logout(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<Json<ApiResponse<ActionResultView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SessionWrite, "logout")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "UPDATE iam.user_session
             SET session_status = 'revoked', revoked_at = now(), updated_at = now()
             WHERE session_id = $1::text::uuid
             RETURNING session_id::text, session_status",
            &[&payload.session_id],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("session not found", header(&headers, "x-request-id")))?;
    let result = ActionResultView {
        target_id: row.get(0),
        status: row.get(1),
    };
    write_audit_event(
        &tx,
        "session",
        &result.target_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.session.logout",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(result))
}

async fn update_user_roles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserRolesRequest>,
) -> Result<Json<ApiResponse<UserView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::RoleChangeWrite,
        "user role change write",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let roles = serde_json::Value::Array(
        payload
            .roles
            .iter()
            .map(|r| serde_json::Value::String(r.clone()))
            .collect(),
    );
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "UPDATE core.user_account
             SET attrs = jsonb_set(COALESCE(attrs, '{}'::jsonb), '{roles}', $2::jsonb, true),
                 updated_at = now()
             WHERE user_id = $1::text::uuid
             RETURNING user_id::text, org_id::text, department_id::text, login_id::text,
                       display_name, user_type, status, email::text, phone",
            &[&id, &roles],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| not_found("user not found", header(&headers, "x-request-id")))?;
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
        &tx,
        "user",
        &view.user_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.user.role.change",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn create_department(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateDepartmentRequest>,
) -> Result<Json<ApiResponse<DepartmentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "department create")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresIamRepository::create_department(&tx, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "department",
        &view.department_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.department.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn get_department(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<DepartmentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "department read")?;
    let client = state.db.client().map_err(map_db_error)?;
    let view = PostgresIamRepository::get_department(&client, &id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| not_found("department not found", header(&headers, "x-request-id")))?;
    Ok(ApiResponse::ok(view))
}

async fn list_departments(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<DepartmentListQuery>,
) -> Result<Json<ApiResponse<Vec<DepartmentView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "department list")?;
    let client = state.db.client().map_err(map_db_error)?;
    let views = PostgresIamRepository::list_departments(&client, &query)
        .await
        .map_err(map_db_error)?;
    Ok(ApiResponse::ok(views))
}

async fn create_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "user create")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresIamRepository::create_user(&tx, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "user",
        &view.user_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.user.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn get_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<UserView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "user read")?;
    let client = state.db.client().map_err(map_db_error)?;
    let view = PostgresIamRepository::get_user(&client, &id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| not_found("user not found", header(&headers, "x-request-id")))?;
    Ok(ApiResponse::ok(view))
}

async fn list_users(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<UserListQuery>,
) -> Result<Json<ApiResponse<Vec<UserView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "user list")?;
    let client = state.db.client().map_err(map_db_error)?;
    let views = PostgresIamRepository::list_users(&client, &query)
        .await
        .map_err(map_db_error)?;
    Ok(ApiResponse::ok(views))
}

async fn create_app(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateApplicationRequest>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "app create")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresIamRepository::create_app(&tx, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "application",
        &view.app_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.app.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn patch_app(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PatchApplicationRequest>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "app patch")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresIamRepository::patch_app(&tx, &id, &payload)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| not_found("application not found", header(&headers, "x-request-id")))?;
    write_audit_event(
        &tx,
        "application",
        &view.app_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.app.patch",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn get_app(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "app read")?;
    let client = state.db.client().map_err(map_db_error)?;
    let view = PostgresIamRepository::get_app(&client, &id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| not_found("application not found", header(&headers, "x-request-id")))?;
    Ok(ApiResponse::ok(view))
}

async fn list_apps(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<ApplicationListQuery>,
) -> Result<Json<ApiResponse<Vec<ApplicationView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "application list")?;
    let client = state.db.client().map_err(map_db_error)?;
    let views = PostgresIamRepository::list_apps(&client, &query)
        .await
        .map_err(map_db_error)?;
    Ok(ApiResponse::ok(views))
}

async fn rotate_app_secret(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<RotateApplicationSecretRequest>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityWrite,
        "application credential rotate",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let secret_hash = payload
        .client_secret_hash
        .unwrap_or_else(|| new_external_readable_id("appsec"));
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "application",
        &view.app_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.app.secret.rotate",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn revoke_app_secret(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ApplicationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityWrite,
        "application credential revoke",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "application",
        &view.app_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.app.secret.revoke",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn invite_user(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<Json<ApiResponse<InvitationView>>, (StatusCode, Json<ErrorResponse>)> {
    create_invitation_internal(headers, state, payload, "iam.user.invite").await
}

async fn create_invitation(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<Json<ApiResponse<InvitationView>>, (StatusCode, Json<ErrorResponse>)> {
    create_invitation_internal(headers, state, payload, "iam.invitation.create").await
}

async fn create_invitation_internal(
    headers: HeaderMap,
    state: AppState,
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

    let client = state.db.client().map_err(map_db_error)?;
    let expires_hours = payload.expires_in_hours.unwrap_or(72).max(1);
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "invitation",
        &view.invitation_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        audit_action,
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn list_invitations(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<InvitationListQuery>,
) -> Result<Json<ApiResponse<Vec<InvitationView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "invitation read")?;
    let client = state.db.client().map_err(map_db_error)?;
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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<InvitationView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "invitation cancel")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "invitation",
        &view.invitation_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.invitation.cancel",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn list_sessions(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<SessionListQuery>,
) -> Result<Json<ApiResponse<Vec<SessionView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SessionRead, "session list")?;
    let client = state.db.client().map_err(map_db_error)?;
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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<SessionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "session revoke")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "session",
        &view.session_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.session.revoke",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn list_devices(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<DeviceListQuery>,
) -> Result<Json<ApiResponse<Vec<DeviceView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SessionRead, "device list")?;
    let client = state.db.client().map_err(map_db_error)?;
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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<DeviceView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "device revoke")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "trusted_device",
        &view.trusted_device_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.device.revoke",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn create_connector(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateConnectorRequest>,
) -> Result<Json<ApiResponse<ConnectorView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityWrite, "connector create")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresIamRepository::create_connector(&tx, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "connector",
        &view.connector_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.connector.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn get_connector(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ConnectorView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "connector read")?;
    let client = state.db.client().map_err(map_db_error)?;
    let view = PostgresIamRepository::get_connector(&client, &id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| not_found("connector not found", header(&headers, "x-request-id")))?;
    Ok(ApiResponse::ok(view))
}

async fn list_connectors(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<ConnectorListQuery>,
) -> Result<Json<ApiResponse<Vec<ConnectorView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::IdentityRead, "connector list")?;
    let client = state.db.client().map_err(map_db_error)?;
    let views = PostgresIamRepository::list_connectors(&client, &query)
        .await
        .map_err(map_db_error)?;
    Ok(ApiResponse::ok(views))
}

async fn create_execution_environment(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateExecutionEnvironmentRequest>,
) -> Result<Json<ApiResponse<ExecutionEnvironmentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityWrite,
        "execution environment create",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresIamRepository::create_execution_environment(&tx, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "execution_environment",
        &view.environment_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.execution_environment.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn get_execution_environment(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ExecutionEnvironmentView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityRead,
        "execution environment read",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let view = PostgresIamRepository::get_execution_environment(&client, &id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            not_found(
                "execution environment not found",
                header(&headers, "x-request-id"),
            )
        })?;
    Ok(ApiResponse::ok(view))
}

async fn list_execution_environments(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<ExecutionEnvironmentListQuery>,
) -> Result<Json<ApiResponse<Vec<ExecutionEnvironmentView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::IdentityRead,
        "execution environment list",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let views = PostgresIamRepository::list_execution_environments(&client, &query)
        .await
        .map_err(map_db_error)?;
    Ok(ApiResponse::ok(views))
}

async fn get_auth_me(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SessionContextView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SessionRead, "session read")?;
    if let Some(token) = extract_bearer(&headers) {
        let parser = parser_from_env();
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

    let client = state.db.client().map_err(map_db_error)?;
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

fn parser_from_env() -> Box<dyn JwtParser> {
    match std::env::var("IAM_JWT_PARSER")
        .unwrap_or_else(|_| "keycloak_claims".to_string())
        .as_str()
    {
        "mock" => Box::new(MockJwtParser),
        _ => Box::new(KeycloakClaimsJwtParser),
    }
}

async fn list_access_rules(
    headers: HeaderMap,
    State(_state): State<AppState>,
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
    State(_state): State<AppState>,
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
    State(_state): State<AppState>,
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
    State(state): State<AppState>,
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

    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "step_up_challenge",
        &challenge_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.step_up.check",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(StepUpCheckView {
        challenge_id,
        action_name: payload.action_name,
        requires_step_up,
        status: status.to_string(),
    }))
}

async fn verify_step_up(
    headers: HeaderMap,
    State(state): State<AppState>,
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

    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
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
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(StepUpCheckView {
        challenge_id: row.get(0),
        action_name: row.get(1),
        requires_step_up: true,
        status: next_status.to_string(),
    }))
}

async fn list_mfa_authenticators(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<MfaAuthenticatorView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::MfaRead, "mfa authenticator read")?;
    let client = state.db.client().map_err(map_db_error)?;
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
    State(state): State<AppState>,
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

    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "mfa_authenticator",
        &view.authenticator_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.mfa.authenticator.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn delete_mfa_authenticator(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<MfaAuthenticatorView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::MfaWrite,
        "mfa authenticator delete",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
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
        &tx,
        "mfa_authenticator",
        &view.authenticator_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.mfa.authenticator.delete",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn create_sso_connection(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateSsoConnectionRequest>,
) -> Result<Json<ApiResponse<SsoConnectionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SsoWrite, "sso connection create")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_one(
            "INSERT INTO iam.sso_connection (
               org_id, connection_name, protocol_type, issuer, client_id, client_secret_ref,
               metadata_url, redirect_uri, jit_provisioning, status, metadata
             ) VALUES (
               $1::text::uuid, $2, COALESCE($3, 'oidc'), $4, $5, $6, $7, $8, COALESCE($9, false), 'draft', $10::jsonb
             )
             RETURNING sso_connection_id::text, org_id::text, connection_name, protocol_type, issuer, status",
            &[
                &payload.org_id,
                &payload.connection_name,
                &payload.protocol_type,
                &payload.issuer,
                &payload.client_id,
                &payload.client_secret_ref,
                &payload.metadata_url,
                &payload.redirect_uri,
                &payload.jit_provisioning,
                &serde_json::json!({"placeholder_mode":"local"}),
            ],
        )
        .await
        .map_err(map_db_error)?;
    let view = SsoConnectionView {
        sso_connection_id: row.get(0),
        org_id: row.get(1),
        connection_name: row.get(2),
        protocol_type: row.get(3),
        issuer: row.get(4),
        status: row.get(5),
    };
    write_audit_event(
        &tx,
        "sso_connection",
        &view.sso_connection_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.sso.connection.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn list_sso_connections(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<SsoConnectionListQuery>,
) -> Result<Json<ApiResponse<Vec<SsoConnectionView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SsoRead, "sso connection read")?;
    let client = state.db.client().map_err(map_db_error)?;
    let rows = client
        .query(
            "SELECT sso_connection_id::text, org_id::text, connection_name, protocol_type, issuer, status
             FROM iam.sso_connection
             WHERE ($1::text IS NULL OR org_id = $1::text::uuid)
             ORDER BY created_at DESC
             LIMIT 100",
            &[&query.org_id],
        )
        .await
        .map_err(map_db_error)?;
    let views = rows
        .iter()
        .map(|row| SsoConnectionView {
            sso_connection_id: row.get(0),
            org_id: row.get(1),
            connection_name: row.get(2),
            protocol_type: row.get(3),
            issuer: row.get(4),
            status: row.get(5),
        })
        .collect();
    Ok(ApiResponse::ok(views))
}

async fn patch_sso_connection(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PatchSsoConnectionRequest>,
) -> Result<Json<ApiResponse<SsoConnectionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::SsoWrite, "sso connection patch")?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "UPDATE iam.sso_connection
             SET issuer = COALESCE($2, issuer),
                 metadata_url = COALESCE($3, metadata_url),
                 redirect_uri = COALESCE($4, redirect_uri),
                 status = COALESCE($5, status),
                 updated_at = now()
             WHERE sso_connection_id = $1::text::uuid
             RETURNING sso_connection_id::text, org_id::text, connection_name, protocol_type, issuer, status",
            &[
                &id,
                &payload.issuer,
                &payload.metadata_url,
                &payload.redirect_uri,
                &payload.status,
            ],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("sso connection not found", header(&headers, "x-request-id")))?;
    let view = SsoConnectionView {
        sso_connection_id: row.get(0),
        org_id: row.get(1),
        connection_name: row.get(2),
        protocol_type: row.get(3),
        issuer: row.get(4),
        status: row.get(5),
    };
    write_audit_event(
        &tx,
        "sso_connection",
        &view.sso_connection_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.sso.connection.patch",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn list_fabric_identities(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<FabricIdentityView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::FabricRead, "fabric identity read")?;
    let client = state.db.client().map_err(map_db_error)?;
    let rows = client
        .query(
            "SELECT fabric_identity_binding_id::text, msp_id, enrollment_id, identity_type, status
             FROM iam.fabric_identity_binding
             ORDER BY created_at DESC
             LIMIT 100",
            &[],
        )
        .await
        .map_err(map_db_error)?;
    let views = rows
        .iter()
        .map(|row| FabricIdentityView {
            fabric_identity_binding_id: row.get(0),
            msp_id: row.get(1),
            enrollment_id: row.get(2),
            identity_type: row.get(3),
            status: row.get(4),
        })
        .collect();
    Ok(ApiResponse::ok(views))
}

async fn issue_fabric_identity(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ActionResultView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::FabricWrite,
        "fabric identity issue placeholder",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "UPDATE iam.fabric_identity_binding
             SET status = 'issued', issued_at = now(), updated_at = now()
             WHERE fabric_identity_binding_id = $1::text::uuid
             RETURNING fabric_identity_binding_id::text, status",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| {
        not_found(
            "fabric identity not found",
            header(&headers, "x-request-id"),
        )
    })?;
    let view = ActionResultView {
        target_id: row.get(0),
        status: row.get(1),
    };
    write_audit_event(
        &tx,
        "fabric_identity_binding",
        &view.target_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.fabric.identity.issue",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn revoke_fabric_identity(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ActionResultView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::FabricWrite,
        "fabric identity revoke placeholder",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "UPDATE iam.fabric_identity_binding
             SET status = 'revoked', revoked_at = now(), updated_at = now()
             WHERE fabric_identity_binding_id = $1::text::uuid
             RETURNING fabric_identity_binding_id::text, status",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row = row.ok_or_else(|| {
        not_found(
            "fabric identity not found",
            header(&headers, "x-request-id"),
        )
    })?;
    let view = ActionResultView {
        target_id: row.get(0),
        status: row.get(1),
    };
    write_audit_event(
        &tx,
        "fabric_identity_binding",
        &view.target_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.fabric.identity.revoke",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

async fn list_certificates(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CertificateView>>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, IamPermission::FabricRead, "certificate read")?;
    let client = state.db.client().map_err(map_db_error)?;
    let rows = client
        .query(
            "SELECT certificate_id::text, serial_number, status
             FROM iam.certificate_record
             ORDER BY created_at DESC
             LIMIT 100",
            &[],
        )
        .await
        .map_err(map_db_error)?;
    let views = rows
        .iter()
        .map(|row| CertificateView {
            certificate_id: row.get(0),
            serial_number: row.get(1),
            status: row.get(2),
        })
        .collect();
    Ok(ApiResponse::ok(views))
}

async fn revoke_certificate(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ActionResultView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        IamPermission::FabricWrite,
        "certificate revoke placeholder",
    )?;
    let client = state.db.client().map_err(map_db_error)?;
    let tx = client.transaction().await.map_err(map_db_error)?;
    let row = tx
        .query_opt(
            "UPDATE iam.certificate_record
             SET status = 'revoked', updated_at = now()
             WHERE certificate_id = $1::text::uuid
             RETURNING certificate_id::text, status",
            &[&id],
        )
        .await
        .map_err(map_db_error)?;
    let row =
        row.ok_or_else(|| not_found("certificate not found", header(&headers, "x-request-id")))?;
    let result = ActionResultView {
        target_id: row.get(0),
        status: row.get(1),
    };
    write_audit_event(
        &tx,
        "certificate_record",
        &result.target_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "iam.certificate.revoke",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(result))
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
        "iam.sso.read" => Ok(IamPermission::SsoRead),
        "iam.sso.write" => Ok(IamPermission::SsoWrite),
        "iam.fabric.read" => Ok(IamPermission::FabricRead),
        "iam.fabric.write" => Ok(IamPermission::FabricWrite),
        "iam.session.write" => Ok(IamPermission::SessionWrite),
        "iam.user.role.change" => Ok(IamPermission::RoleChangeWrite),
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

fn parse_org_row(row: &DbRecord, blacklist_active: bool) -> OrganizationAggregateView {
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
        review_status: metadata
            .get("review_status")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        risk_status: metadata
            .get("risk_status")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        sellable_status: metadata
            .get("sellable_status")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        freeze_reason: metadata
            .get("freeze_reason")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        blacklist_active,
        created_at: row.get(7),
        updated_at: row.get(8),
    }
}

async fn write_audit_event(
    client: &(impl DbClientOps + Sync),
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

fn map_db_error(err: Error) -> (StatusCode, Json<ErrorResponse>) {
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
