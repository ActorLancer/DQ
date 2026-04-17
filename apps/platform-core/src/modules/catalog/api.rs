use crate::modules::catalog::domain::{
    CreateDataProductRequest, DataProductView, PatchDataProductRequest,
};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::{CatalogPermission, is_allowed};
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{patch, post};
use axum::{Json, Router};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse, new_external_readable_id};
use tokio_postgres::{Client, GenericClient, NoTls};
use tracing::info;

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/products", post(create_product_draft))
        .route("/api/v1/products/{id}", patch(patch_product_draft))
}

async fn create_product_draft(
    headers: HeaderMap,
    Json(payload): Json<CreateDataProductRequest>,
) -> Result<Json<ApiResponse<DataProductView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog product draft create",
    )?;
    validate_create_product_payload(&payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_data_product(&tx, &payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "product",
        &view.product_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.product.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.product.create",
        product_id = %view.product_id,
        "catalog product draft created"
    );
    Ok(ApiResponse::ok(view))
}

async fn patch_product_draft(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PatchDataProductRequest>,
) -> Result<Json<ApiResponse<DataProductView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog product draft patch",
    )?;
    validate_patch_product_payload(&payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing = PostgresCatalogRepository::get_data_product(&client, &id)
        .await
        .map_err(map_db_error)?;
    let existing = match existing {
        Some(v) => v,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "product does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            ));
        }
    };
    if existing.status != "draft" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "only draft product can be edited".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::patch_data_product(&tx, &id, &payload)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    code: ErrorCode::TrdStateConflict.as_str().to_string(),
                    message: "product is no longer editable".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    write_audit_event(
        &tx,
        "product",
        &view.product_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.product.patch",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.product.patch",
        product_id = %view.product_id,
        "catalog product draft patched"
    );
    Ok(ApiResponse::ok(view))
}

fn validate_create_product_payload(
    payload: &CreateDataProductRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let invalid = payload.asset_id.trim().is_empty()
        || payload.asset_version_id.trim().is_empty()
        || payload.seller_org_id.trim().is_empty()
        || payload.title.trim().is_empty()
        || payload.category.trim().is_empty()
        || payload.product_type.trim().is_empty()
        || payload.delivery_type.trim().is_empty();
    if !invalid {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: "required product fields must not be empty".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn validate_patch_product_payload(
    payload: &PatchDataProductRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let has_change = payload.title.is_some()
        || payload.category.is_some()
        || payload.product_type.is_some()
        || payload.description.is_some()
        || payload.price_mode.is_some()
        || payload.price.is_some()
        || payload.currency_code.is_some()
        || payload.delivery_type.is_some()
        || payload.searchable_text.is_some()
        || payload.status.is_some();
    if has_change {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: "at least one patch field is required".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

async fn write_audit_event(
    client: &(impl GenericClient + Sync),
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
               'catalog', $1, $2::text::uuid, 'role', NULL, $3, $4, $5, $6, $7::jsonb
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
                    "event_id": new_external_readable_id("cat"),
                }),
            ],
        )
        .await
        .map_err(map_db_error)?;
    Ok(())
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

fn require_permission(
    headers: &HeaderMap,
    permission: CatalogPermission,
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
        Client,
        tokio_postgres::Connection<tokio_postgres::Socket, tokio_postgres::tls::NoTlsStream>,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    tokio_postgres::connect(dsn, NoTls)
        .await
        .map_err(map_db_error)
}

fn map_db_error(err: tokio_postgres::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database operation failed: {err}"),
            request_id: None,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn rejects_create_product_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/products")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "asset_id":"00000000-0000-0000-0000-000000000001",
                  "asset_version_id":"00000000-0000-0000-0000-000000000002",
                  "seller_org_id":"00000000-0000-0000-0000-000000000003",
                  "title":"p",
                  "category":"c",
                  "product_type":"data_product",
                  "delivery_type":"file_download"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_patch_product_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("PATCH")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000100")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(r#"{"title":"updated"}"#))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
