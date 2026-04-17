use crate::modules::catalog::domain::{
    CreateDataProductRequest, CreateProductSkuRequest, DataProductView, PatchDataProductRequest,
    PatchProductSkuRequest, ProductSkuView, default_trade_mode_for_sku_type, is_standard_sku_type,
};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::{
    CatalogPermission, is_allowed, is_valid_sku_trade_mode_pair,
};
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
        .route("/api/v1/products/{id}/skus", post(create_product_sku))
        .route("/api/v1/skus/{id}", patch(patch_product_sku))
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

async fn create_product_sku(
    headers: HeaderMap,
    Path(product_id): Path<String>,
    Json(payload): Json<CreateProductSkuRequest>,
) -> Result<Json<ApiResponse<ProductSkuView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog product sku create",
    )?;
    validate_create_sku_payload(&product_id, &payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let product = PostgresCatalogRepository::get_data_product(&client, &product_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "product does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    if product.status != "draft" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "only draft product can add sku".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let trade_mode = payload
        .trade_mode
        .clone()
        .or_else(|| default_trade_mode_for_sku_type(&payload.sku_type).map(str::to_string))
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "cannot derive trade_mode from sku_type".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;

    validate_template_compatibility(
        &client,
        payload.template_id.as_deref(),
        &payload.sku_type,
        &headers,
    )
    .await?;

    let mut normalized_payload = payload.clone();
    normalized_payload.trade_mode = Some(trade_mode);

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_product_sku(&tx, &product_id, &normalized_payload)
        .await
        .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "sku",
        &view.sku_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.sku.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.sku.create",
        sku_id = %view.sku_id,
        product_id = %view.product_id,
        "catalog sku created"
    );
    Ok(ApiResponse::ok(view))
}

async fn patch_product_sku(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<PatchProductSkuRequest>,
) -> Result<Json<ApiResponse<ProductSkuView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog product sku patch",
    )?;
    validate_patch_sku_payload(&payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let existing = PostgresCatalogRepository::get_product_sku(&client, &id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "sku does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    if existing.status != "draft" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "only draft sku can be edited".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let next_sku_type = payload
        .sku_type
        .as_deref()
        .unwrap_or(existing.sku_type.as_str());
    let next_trade_mode = payload
        .trade_mode
        .as_deref()
        .unwrap_or(existing.trade_mode.as_str());

    if !is_standard_sku_type(next_sku_type)
        || !is_valid_sku_trade_mode_pair(next_sku_type, next_trade_mode)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "sku_type and trade_mode are incompatible".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    validate_template_compatibility(
        &client,
        payload.template_id.as_deref(),
        next_sku_type,
        &headers,
    )
    .await?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::patch_product_sku(&tx, &id, &payload)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    code: ErrorCode::TrdStateConflict.as_str().to_string(),
                    message: "sku is no longer editable".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    write_audit_event(
        &tx,
        "sku",
        &view.sku_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.sku.patch",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.sku.patch",
        sku_id = %view.sku_id,
        "catalog sku patched"
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

fn validate_create_sku_payload(
    product_id_from_path: &str,
    payload: &CreateProductSkuRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if let Some(product_id_from_body) = payload.product_id.as_deref()
        && product_id_from_body != product_id_from_path
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "product_id in body does not match path".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    let invalid = payload.sku_code.trim().is_empty()
        || payload.sku_type.trim().is_empty()
        || payload.billing_mode.trim().is_empty()
        || payload.acceptance_mode.trim().is_empty()
        || payload.refund_mode.trim().is_empty();
    if invalid {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "required sku fields must not be empty".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    if !is_standard_sku_type(&payload.sku_type) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "sku_type is not in V1 standard truth set".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    let trade_mode = payload
        .trade_mode
        .as_deref()
        .or_else(|| default_trade_mode_for_sku_type(&payload.sku_type));
    let Some(trade_mode) = trade_mode else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "trade_mode is required or derivable".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    };
    if !is_valid_sku_trade_mode_pair(&payload.sku_type, trade_mode) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "sku_type and trade_mode are incompatible".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    }
    Ok(())
}

fn validate_patch_sku_payload(
    payload: &PatchProductSkuRequest,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let has_change = payload.sku_code.is_some()
        || payload.sku_type.is_some()
        || payload.unit_name.is_some()
        || payload.billing_mode.is_some()
        || payload.trade_mode.is_some()
        || payload.delivery_object_kind.is_some()
        || payload.subscription_cadence.is_some()
        || payload.share_protocol.is_some()
        || payload.result_form.is_some()
        || payload.template_id.is_some()
        || payload.acceptance_mode.is_some()
        || payload.refund_mode.is_some()
        || payload.status.is_some();
    if has_change {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: "at least one sku patch field is required".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

async fn validate_template_compatibility(
    client: &impl GenericClient,
    template_id: Option<&str>,
    sku_type: &str,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(template_id) = template_id else {
        return Ok(());
    };
    let row = client
        .query_opt(
            "SELECT applicable_sku_types::text[]
             FROM contract.template_definition
             WHERE template_id = $1::text::uuid
               AND status = 'active'",
            &[&template_id],
        )
        .await
        .map_err(map_db_error)?;
    let Some(row) = row else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "template does not exist or is inactive".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        ));
    };
    let applicable_sku_types: Vec<String> = row.get(0);
    if applicable_sku_types.iter().any(|v| v == sku_type) {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::CatValidationFailed.as_str().to_string(),
            message: "template is not compatible with sku_type".to_string(),
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

    #[tokio::test]
    async fn rejects_create_sku_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/products/00000000-0000-0000-0000-000000000001/skus")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(
                r#"{
                  "sku_code":"SKU-001",
                  "sku_type":"FILE_STD",
                  "billing_mode":"one_time",
                  "acceptance_mode":"manual_accept",
                  "refund_mode":"manual_refund"
                }"#,
            ))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_patch_sku_without_permission() {
        let app = router();
        let req = Request::builder()
            .method("PATCH")
            .uri("/api/v1/skus/00000000-0000-0000-0000-000000000001")
            .header("content-type", "application/json")
            .header("x-role", "developer")
            .body(Body::from(r#"{"trade_mode":"snapshot_sale"}"#))
            .expect("request");
        let resp = app.oneshot(req).await.expect("response");
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
