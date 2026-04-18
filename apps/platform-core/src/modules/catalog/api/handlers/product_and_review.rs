use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tracing::info;

use crate::modules::catalog::domain::{
    CreateDataProductRequest, DataProductView, PatchDataProductRequest, ProductLifecycleView,
    ProductMetadataProfileView, ProductSubmitView, PutProductMetadataProfileRequest,
    ReviewDecisionRequest, ReviewDecisionView, SubmitProductRequest, SuspendProductRequest,
};
use crate::modules::catalog::repository::PostgresCatalogRepository;
use crate::modules::catalog::service::{
    CatalogPermission, can_transition_listing_status, is_valid_listing_status,
};

use super::super::support::*;
use super::super::validators::*;

pub(in crate::modules::catalog) async fn create_product_draft(
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
    write_outbox_event(
        &tx,
        &headers,
        "product",
        &view.product_id,
        "search.product.changed",
        "dtp.outbox.domain-events",
        &serde_json::json!({
            "product_id": view.product_id,
            "seller_org_id": view.seller_org_id,
            "change_type": "create",
            "status": view.status
        }),
    )
    .await?;
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

pub(in crate::modules::catalog) async fn patch_product_draft(
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
    write_outbox_event(
        &tx,
        &headers,
        "product",
        &view.product_id,
        "search.product.changed",
        "dtp.outbox.domain-events",
        &serde_json::json!({
            "product_id": view.product_id,
            "seller_org_id": view.seller_org_id,
            "change_type": "patch",
            "status": view.status
        }),
    )
    .await?;
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

pub(in crate::modules::catalog) async fn put_product_metadata_profile(
    headers: HeaderMap,
    Path(product_id): Path<String>,
    Json(payload): Json<PutProductMetadataProfileRequest>,
) -> Result<Json<ApiResponse<ProductMetadataProfileView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductDraftWrite,
        "catalog product metadata profile put",
    )?;
    validate_put_product_metadata_profile_payload(&product_id, &payload, &headers)?;
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
                message: "only draft product can edit metadata profile".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::upsert_product_metadata_profile(
        &tx,
        &product_id,
        &product.asset_version_id,
        &payload,
    )
    .await
    .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "product_metadata_profile",
        &view.product_metadata_profile_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.product_metadata_profile.upsert",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.product_metadata_profile.upsert",
        product_id = %product_id,
        product_metadata_profile_id = %view.product_metadata_profile_id,
        "catalog product metadata profile upserted"
    );
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn submit_product(
    headers: HeaderMap,
    Path(product_id): Path<String>,
    Json(payload): Json<SubmitProductRequest>,
) -> Result<Json<ApiResponse<ProductSubmitView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ProductSubmit,
        "catalog product submit",
    )?;
    validate_submit_product_payload(&payload, &headers)?;
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
                message: "only draft product can be submitted".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }
    enforce_product_scope(&headers, &product.seller_org_id, "catalog product submit")?;
    let has_pending_task = PostgresCatalogRepository::has_pending_review_task(
        &client,
        "product_review",
        "product",
        &product_id,
    )
    .await
    .map_err(map_db_error)?;
    if has_pending_task {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "product already has a pending review task".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }
    ensure_product_submit_ready(&client, &product_id, &headers).await?;

    let tx = client.transaction().await.map_err(map_db_error)?;
    let updated = PostgresCatalogRepository::transition_product_status(
        &tx,
        &product_id,
        "draft",
        "pending_review",
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "product status changed concurrently".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;
    let review = PostgresCatalogRepository::create_review_task_with_initial_step(
        &tx,
        "product_review",
        "product",
        &product_id,
        "submit",
        payload.submission_note.as_deref(),
    )
    .await
    .map_err(map_db_error)?;
    write_outbox_event(
        &tx,
        &headers,
        "product",
        &product_id,
        "catalog.product.submitted",
        "dtp.outbox.domain-events",
        &serde_json::json!({
            "product_id": product_id,
            "status": "pending_review"
        }),
    )
    .await?;
    write_audit_event(
        &tx,
        "product",
        &product_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.product.submit",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    info!(
        action = "catalog.product.submit",
        product_id = %product_id,
        review_task_id = %review.review_task_id,
        status = %updated.status,
        "catalog product submitted"
    );
    Ok(ApiResponse::ok(ProductSubmitView {
        product_id,
        status: updated.status,
        review_task_id: review.review_task_id,
    }))
}

pub(in crate::modules::catalog) async fn review_subject(
    headers: HeaderMap,
    Path(subject_id): Path<String>,
    Json(payload): Json<ReviewDecisionRequest>,
) -> Result<Json<ApiResponse<ReviewDecisionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ReviewWrite,
        "catalog subject review",
    )?;
    validate_review_decision_payload(&payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let org_exists = PostgresCatalogRepository::organization_exists(&client, &subject_id)
        .await
        .map_err(map_db_error)?;
    if !org_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "subject does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }
    enforce_subject_scope(&headers, &subject_id, "catalog subject review")?;
    let review_status = decision_to_review_status(&payload.action_name);
    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_review_decision(
        &tx,
        "subject_review",
        "subject",
        &subject_id,
        &payload.action_name,
        payload.action_reason.as_deref(),
        review_status,
    )
    .await
    .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "subject",
        &subject_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.review.subject",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn review_product(
    headers: HeaderMap,
    Path(product_id): Path<String>,
    Json(payload): Json<ReviewDecisionRequest>,
) -> Result<Json<ApiResponse<ProductSubmitView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ReviewWrite,
        "catalog product review",
    )?;
    validate_review_decision_payload(&payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let current = PostgresCatalogRepository::get_data_product(&client, &product_id)
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
    enforce_product_scope(&headers, &current.seller_org_id, "catalog product review")?;
    if current.status != "pending_review" {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "only pending_review product can be reviewed".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }
    let next_status = match payload.action_name.as_str() {
        "approve" => "listed",
        "reject" => "draft",
        _ => "pending_review",
    };
    if !is_valid_listing_status(next_status) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "target listing status is invalid".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }
    if !can_transition_listing_status(&current.status, next_status) {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "review action is not allowed for current product status".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }

    let tx = client.transaction().await.map_err(map_db_error)?;
    let updated = PostgresCatalogRepository::transition_product_status(
        &tx,
        &product_id,
        &current.status,
        next_status,
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "product status changed concurrently".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;
    let review = PostgresCatalogRepository::append_review_step_and_close_task(
        &tx,
        "product_review",
        "product",
        &product_id,
        &payload.action_name,
        payload.action_reason.as_deref(),
        decision_to_review_status(&payload.action_name),
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "pending review task does not exist".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;
    write_outbox_event(
        &tx,
        &headers,
        "product",
        &product_id,
        "catalog.product.status.changed",
        "dtp.outbox.domain-events",
        &serde_json::json!({
            "product_id": product_id,
            "status": updated.status,
            "review_action": payload.action_name
        }),
    )
    .await?;
    write_audit_event(
        &tx,
        "product",
        &product_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.review.product",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(ProductSubmitView {
        product_id,
        status: updated.status,
        review_task_id: review.review_task_id,
    }))
}

pub(in crate::modules::catalog) async fn review_compliance(
    headers: HeaderMap,
    Path(compliance_id): Path<String>,
    Json(payload): Json<ReviewDecisionRequest>,
) -> Result<Json<ApiResponse<ReviewDecisionView>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        CatalogPermission::ReviewWrite,
        "catalog compliance review",
    )?;
    validate_review_decision_payload(&payload, &headers)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let product = PostgresCatalogRepository::get_data_product(&client, &compliance_id)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: ErrorCode::CatValidationFailed.as_str().to_string(),
                    message: "compliance target product does not exist".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    enforce_product_scope(
        &headers,
        &product.seller_org_id,
        "catalog compliance review",
    )?;
    let review_status = decision_to_review_status(&payload.action_name);
    let tx = client.transaction().await.map_err(map_db_error)?;
    let view = PostgresCatalogRepository::create_review_decision(
        &tx,
        "compliance_review",
        "compliance",
        &compliance_id,
        &payload.action_name,
        payload.action_reason.as_deref(),
        review_status,
    )
    .await
    .map_err(map_db_error)?;
    write_audit_event(
        &tx,
        "compliance",
        &compliance_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.review.compliance",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(view))
}

pub(in crate::modules::catalog) async fn suspend_product(
    headers: HeaderMap,
    Path(product_id): Path<String>,
    Json(payload): Json<SuspendProductRequest>,
) -> Result<Json<ApiResponse<ProductLifecycleView>>, (StatusCode, Json<ErrorResponse>)> {
    validate_suspend_payload(&payload, &headers)?;
    if payload.suspend_mode == "freeze" {
        require_any_permission(
            &headers,
            &[
                CatalogPermission::ProductSuspend,
                CatalogPermission::RiskProductFreeze,
            ],
            "catalog product freeze",
        )?;
        let _ = header(&headers, "x-step-up-challenge-id").ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: "x-step-up-challenge-id is required for freeze action".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
        let _ = header(&headers, "x-user-id").ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: "x-user-id is required for freeze action".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            )
        })?;
    } else {
        require_permission(
            &headers,
            CatalogPermission::ProductSuspend,
            "catalog product suspend",
        )?;
    }

    let dsn = database_dsn()?;
    let (mut client, connection) = connect_db(&dsn).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let existing = PostgresCatalogRepository::get_data_product(&client, &product_id)
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
    enforce_product_scope(&headers, &existing.seller_org_id, "catalog product suspend")?;
    if payload.suspend_mode == "freeze" {
        let challenge_id = header(&headers, "x-step-up-challenge-id")
            .expect("x-step-up-challenge-id checked before db access");
        let user_id = header(&headers, "x-user-id").expect("x-user-id checked before db access");
        let step_up_ok = PostgresCatalogRepository::is_verified_step_up_challenge(
            &client,
            &challenge_id,
            &user_id,
            "risk.product.freeze",
            "product",
            &product_id,
        )
        .await
        .map_err(map_db_error)?;
        if !step_up_ok {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    code: ErrorCode::IamUnauthorized.as_str().to_string(),
                    message: "verified step-up challenge is required for freeze action".to_string(),
                    request_id: header(&headers, "x-request-id"),
                }),
            ));
        }
    }
    let next_status = if payload.suspend_mode == "freeze" {
        "frozen"
    } else {
        "delisted"
    };
    if !is_valid_listing_status(next_status) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: ErrorCode::CatValidationFailed.as_str().to_string(),
                message: "target listing status is invalid".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }
    if !can_transition_listing_status(&existing.status, next_status) {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "suspend action is not allowed for current product status".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    }
    let previous_status = existing.status.clone();
    let tx = client.transaction().await.map_err(map_db_error)?;
    let updated = PostgresCatalogRepository::transition_product_status(
        &tx,
        &product_id,
        &existing.status,
        next_status,
    )
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: "product status changed concurrently".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;
    write_audit_event(
        &tx,
        "product",
        &product_id,
        header(&headers, "x-role").as_deref().unwrap_or("unknown"),
        "catalog.product.suspend",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    write_outbox_event(
        &tx,
        &headers,
        "product",
        &product_id,
        "catalog.product.status.changed",
        "dtp.outbox.domain-events",
        &serde_json::json!({
            "product_id": product_id,
            "status": updated.status,
            "suspend_mode": payload.suspend_mode
        }),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(ApiResponse::ok(ProductLifecycleView {
        product_id,
        previous_status,
        status: updated.status,
    }))
}
