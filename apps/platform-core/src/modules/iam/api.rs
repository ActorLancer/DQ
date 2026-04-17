use crate::modules::iam::domain::{
    ApplicationView, ConnectorView, CreateApplicationRequest, CreateConnectorRequest,
    CreateDepartmentRequest, CreateExecutionEnvironmentRequest, CreateUserRequest, DepartmentView,
    ExecutionEnvironmentView, OrganizationAggregateView, PatchApplicationRequest,
    RegisterOrganizationRequest, SessionContextView, UserView,
};
use crate::modules::iam::service::{IamPermission, is_allowed};
use auth::{JwtParser, MockJwtParser, extract_bearer};
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, patch, post};
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
               org_id, app_name, app_type, status, client_id, client_secret_hash
             ) VALUES (
               $1::text::uuid, $2, $3, 'active', $4, $5
             )
             RETURNING app_id::text, org_id::text, app_name, app_type, status, client_id",
            &[
                &payload.org_id,
                &payload.app_name,
                &payload
                    .app_type
                    .clone()
                    .unwrap_or_else(|| "api_client".to_string()),
                &payload.client_id,
                &payload.client_secret_hash,
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
             RETURNING app_id::text, org_id::text, app_name, app_type, status, client_id",
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
            "SELECT app_id::text, org_id::text, app_name, app_type, status, client_id
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
    }))
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
}
