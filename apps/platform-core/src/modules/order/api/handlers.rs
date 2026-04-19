use crate::modules::order::dto::{
    ApiPpuTransitionRequest, ApiPpuTransitionResponse, ApiSubTransitionRequest,
    ApiSubTransitionResponse, CancelOrderResponse, ConfirmOrderContractRequest,
    ConfirmOrderContractResponse, CreateOrderRequest, CreateOrderResponse, CreateOrderResponseData,
    CreateTradePreRequestRequest, FileStdTransitionRequest, FileStdTransitionResponse,
    FileSubTransitionRequest, FileSubTransitionResponse, FreezeOrderPriceSnapshotResponse,
    FreezeOrderPriceSnapshotResponseData, GetOrderDetailResponse,
    OrderAuthorizationTransitionRequest, OrderAuthorizationTransitionResponse,
    QryLiteTransitionRequest, QryLiteTransitionResponse, RptStdTransitionRequest,
    RptStdTransitionResponse, SbxStdTransitionRequest, SbxStdTransitionResponse,
    ShareRoTransitionRequest, ShareRoTransitionResponse, TradePreRequestResponse,
    TradePreRequestResponseData,
};
use crate::modules::order::repo::{
    cancel_order_with_state_machine, confirm_order_contract, create_order_with_snapshot,
    find_order_by_idempotency, freeze_order_price_snapshot, insert_trade_pre_request,
    load_order_cancel_context, load_order_contract_confirm_context, load_order_detail,
    load_trade_pre_request, transition_api_ppu_order, transition_api_sub_order,
    transition_file_std_order, transition_file_sub_order, transition_order_authorization,
    transition_qry_lite_order, transition_rpt_std_order, transition_sbx_std_order,
    transition_share_ro_order, write_trade_audit_event,
};
use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use http::ApiResponse;
use kernel::{ErrorCode, ErrorResponse};
use tokio_postgres::NoTls;
use tracing::info;

pub async fn create_order_api(
    headers: HeaderMap,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<ApiResponse<CreateOrderResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, TradePermission::CreateOrder, "order create")?;
    enforce_order_create_scope(&headers, &payload.buyer_org_id)?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let request_id = header(&headers, "x-request-id");
    let trace_id = header(&headers, "x-trace-id");
    let idempotency_key = header(&headers, "x-idempotency-key");
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());

    if let Some(ref key) = idempotency_key {
        if let Some(existing) = find_order_by_idempotency(&client, key).await? {
            write_trade_audit_event(
                &client,
                "order",
                &existing.order_id,
                &actor_role,
                "trade.order.create.idempotent_replay",
                "success",
                request_id.as_deref(),
                trace_id.as_deref(),
            )
            .await?;
            return Ok(ApiResponse::ok(CreateOrderResponse { data: existing }));
        }
    }

    let created: CreateOrderResponseData = create_order_with_snapshot(
        &mut client,
        &payload,
        &actor_role,
        request_id.as_deref(),
        trace_id.as_deref(),
        idempotency_key.as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.create",
        order_id = %created.order_id,
        product_id = %created.product_id,
        sku_id = %created.sku_id,
        "order created"
    );
    Ok(ApiResponse::ok(CreateOrderResponse { data: created }))
}

pub async fn get_order_detail_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
) -> Result<Json<ApiResponse<GetOrderDetailResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, TradePermission::ReadOrder, "order read")?;
    let dsn = database_dsn()?;
    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let Some(order) = load_order_detail(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(&headers, &order.buyer_org_id, &order.seller_org_id)?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    write_trade_audit_event(
        &client,
        "order",
        &order.order_id,
        &actor_role,
        "trade.order.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "trade.order.read",
        order_id = %order.order_id,
        current_state = %order.current_state,
        payment_status = %order.payment_status,
        "order detail queried"
    );

    Ok(ApiResponse::ok(GetOrderDetailResponse { data: order }))
}

pub async fn cancel_order_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
) -> Result<Json<ApiResponse<CancelOrderResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(&headers, TradePermission::CancelOrder, "order cancel")?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let canceled = cancel_order_with_state_machine(
        &mut client,
        &order_id,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "trade.order.cancel",
        order_id = %canceled.order_id,
        previous_state = %canceled.previous_state,
        current_state = %canceled.current_state,
        refund_branch = %canceled.refund_branch,
        "order canceled"
    );
    Ok(ApiResponse::ok(CancelOrderResponse { data: canceled }))
}

pub async fn confirm_order_contract_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<ConfirmOrderContractRequest>,
) -> Result<Json<ApiResponse<ConfirmOrderContractResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::ConfirmContract,
        "order contract confirm",
    )?;
    validate_signer_role(&payload.signer_role)?;
    let signer_id = header(&headers, "x-user-id").ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "contract confirm requires x-user-id".to_string(),
                request_id: header(&headers, "x-request-id"),
            }),
        )
    })?;

    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let Some(order_scope) = load_order_contract_confirm_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_create_scope(&headers, &order_scope.buyer_org_id)?;

    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let confirmed = confirm_order_contract(
        &mut client,
        &order_id,
        &payload,
        &signer_id,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.contract.confirm",
        order_id = %confirmed.order_id,
        contract_id = %confirmed.contract_id,
        signer_role = %confirmed.signer_role,
        "order contract confirmed"
    );
    Ok(ApiResponse::ok(ConfirmOrderContractResponse {
        data: confirmed,
    }))
}

pub async fn transition_order_authorization_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<OrderAuthorizationTransitionRequest>,
) -> Result<
    Json<ApiResponse<OrderAuthorizationTransitionResponse>>,
    (StatusCode, Json<ErrorResponse>),
> {
    require_permission(
        &headers,
        TradePermission::TransitionAuthorization,
        "order authorization transition",
    )?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let transitioned = transition_order_authorization(
        &mut client,
        &order_id,
        &payload,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.authorization.transition",
        order_id = %transitioned.order_id,
        authorization_id = %transitioned.authorization_id,
        transition_action = %transitioned.action,
        previous_status = ?transitioned.previous_status,
        current_status = %transitioned.current_status,
        "order authorization transitioned"
    );
    Ok(ApiResponse::ok(OrderAuthorizationTransitionResponse {
        data: transitioned,
    }))
}

pub async fn transition_file_std_order_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<FileStdTransitionRequest>,
) -> Result<Json<ApiResponse<FileStdTransitionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::TransitionFileStd,
        "file std state transition",
    )?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let transitioned = transition_file_std_order(
        &mut client,
        &order_id,
        &payload,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.file_std.transition",
        order_id = %transitioned.order_id,
        transition_action = %transitioned.action,
        previous_state = %transitioned.previous_state,
        current_state = %transitioned.current_state,
        "file std order state transitioned"
    );
    Ok(ApiResponse::ok(FileStdTransitionResponse {
        data: transitioned,
    }))
}

pub async fn transition_file_sub_order_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<FileSubTransitionRequest>,
) -> Result<Json<ApiResponse<FileSubTransitionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::TransitionFileSub,
        "file sub state transition",
    )?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let transitioned = transition_file_sub_order(
        &mut client,
        &order_id,
        &payload,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.file_sub.transition",
        order_id = %transitioned.order_id,
        transition_action = %transitioned.action,
        previous_state = %transitioned.previous_state,
        current_state = %transitioned.current_state,
        "file sub order state transitioned"
    );
    Ok(ApiResponse::ok(FileSubTransitionResponse {
        data: transitioned,
    }))
}

pub async fn transition_api_sub_order_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<ApiSubTransitionRequest>,
) -> Result<Json<ApiResponse<ApiSubTransitionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::TransitionApiSub,
        "api sub state transition",
    )?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let transitioned = transition_api_sub_order(
        &mut client,
        &order_id,
        &payload,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.api_sub.transition",
        order_id = %transitioned.order_id,
        transition_action = %transitioned.action,
        previous_state = %transitioned.previous_state,
        current_state = %transitioned.current_state,
        "api sub order state transitioned"
    );
    Ok(ApiResponse::ok(ApiSubTransitionResponse {
        data: transitioned,
    }))
}

pub async fn transition_api_ppu_order_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<ApiPpuTransitionRequest>,
) -> Result<Json<ApiResponse<ApiPpuTransitionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::TransitionApiPpu,
        "api ppu state transition",
    )?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let transitioned = transition_api_ppu_order(
        &mut client,
        &order_id,
        &payload,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.api_ppu.transition",
        order_id = %transitioned.order_id,
        transition_action = %transitioned.action,
        previous_state = %transitioned.previous_state,
        current_state = %transitioned.current_state,
        "api ppu order state transitioned"
    );
    Ok(ApiResponse::ok(ApiPpuTransitionResponse {
        data: transitioned,
    }))
}

pub async fn transition_share_ro_order_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<ShareRoTransitionRequest>,
) -> Result<Json<ApiResponse<ShareRoTransitionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::TransitionShareRo,
        "share ro state transition",
    )?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let transitioned = transition_share_ro_order(
        &mut client,
        &order_id,
        &payload,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.share_ro.transition",
        order_id = %transitioned.order_id,
        transition_action = %transitioned.action,
        previous_state = %transitioned.previous_state,
        current_state = %transitioned.current_state,
        "share ro order state transitioned"
    );
    Ok(ApiResponse::ok(ShareRoTransitionResponse {
        data: transitioned,
    }))
}

pub async fn transition_qry_lite_order_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<QryLiteTransitionRequest>,
) -> Result<Json<ApiResponse<QryLiteTransitionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::TransitionQryLite,
        "qry lite state transition",
    )?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let transitioned = transition_qry_lite_order(
        &mut client,
        &order_id,
        &payload,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.qry_lite.transition",
        order_id = %transitioned.order_id,
        transition_action = %transitioned.action,
        previous_state = %transitioned.previous_state,
        current_state = %transitioned.current_state,
        "qry lite order state transitioned"
    );
    Ok(ApiResponse::ok(QryLiteTransitionResponse {
        data: transitioned,
    }))
}

pub async fn transition_sbx_std_order_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<SbxStdTransitionRequest>,
) -> Result<Json<ApiResponse<SbxStdTransitionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::TransitionSbxStd,
        "sbx std state transition",
    )?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let transitioned = transition_sbx_std_order(
        &mut client,
        &order_id,
        &payload,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.sbx_std.transition",
        order_id = %transitioned.order_id,
        transition_action = %transitioned.action,
        previous_state = %transitioned.previous_state,
        current_state = %transitioned.current_state,
        "sbx std order state transitioned"
    );
    Ok(ApiResponse::ok(SbxStdTransitionResponse {
        data: transitioned,
    }))
}

pub async fn transition_rpt_std_order_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<RptStdTransitionRequest>,
) -> Result<Json<ApiResponse<RptStdTransitionResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::TransitionRptStd,
        "rpt std state transition",
    )?;
    let dsn = database_dsn()?;
    let (mut client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let Some(order_scope) = load_order_cancel_context(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    enforce_order_read_scope(
        &headers,
        &order_scope.buyer_org_id,
        &order_scope.seller_org_id,
    )?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    let transitioned = transition_rpt_std_order(
        &mut client,
        &order_id,
        &payload,
        &actor_role,
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;

    info!(
        action = "trade.order.rpt_std.transition",
        order_id = %transitioned.order_id,
        transition_action = %transitioned.action,
        previous_state = %transitioned.previous_state,
        current_state = %transitioned.current_state,
        "rpt std order state transitioned"
    );
    Ok(ApiResponse::ok(RptStdTransitionResponse {
        data: transitioned,
    }))
}

pub async fn create_trade_pre_request(
    headers: HeaderMap,
    Json(payload): Json<CreateTradePreRequestRequest>,
) -> Result<Json<ApiResponse<TradePreRequestResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::CreatePreRequest,
        "trade pre-request create",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let created = insert_trade_pre_request(&client, &payload).await?;
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    write_trade_audit_event(
        &client,
        "inquiry",
        &created.inquiry_id,
        &actor_role,
        "trade.pre_request.create",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "trade.pre_request.create",
        inquiry_id = %created.inquiry_id,
        request_kind = %created.request_payload.request_kind.as_str(),
        "trade pre-request created"
    );

    Ok(ApiResponse::ok(TradePreRequestResponse {
        data: TradePreRequestResponseData::from(created),
    }))
}

pub async fn get_trade_pre_request(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<TradePreRequestResponse>>, (StatusCode, Json<ErrorResponse>)> {
    require_permission(
        &headers,
        TradePermission::ReadPreRequest,
        "trade pre-request read",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let Some(found) = load_trade_pre_request(&client, &id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("trade pre-request not found: {id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    write_trade_audit_event(
        &client,
        "inquiry",
        &found.inquiry_id,
        &actor_role,
        "trade.pre_request.read",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "trade.pre_request.read",
        inquiry_id = %found.inquiry_id,
        "trade pre-request queried"
    );

    Ok(ApiResponse::ok(TradePreRequestResponse {
        data: TradePreRequestResponseData::from(found),
    }))
}

pub async fn freeze_order_price_snapshot_api(
    headers: HeaderMap,
    Path(order_id): Path<String>,
) -> Result<Json<ApiResponse<FreezeOrderPriceSnapshotResponse>>, (StatusCode, Json<ErrorResponse>)>
{
    require_permission(
        &headers,
        TradePermission::CreatePreRequest,
        "order price snapshot freeze",
    )?;
    let dsn = database_dsn()?;
    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(map_db_connect)?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let Some(snapshot) = freeze_order_price_snapshot(&client, &order_id).await? else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: ErrorCode::TrdStateConflict.as_str().to_string(),
                message: format!("order not found for snapshot freeze: {order_id}"),
                request_id: header(&headers, "x-request-id"),
            }),
        ));
    };
    let actor_role = header(&headers, "x-role").unwrap_or_else(|| "unknown".to_string());
    write_trade_audit_event(
        &client,
        "order",
        &order_id,
        &actor_role,
        "trade.order.price_snapshot.freeze",
        "success",
        header(&headers, "x-request-id").as_deref(),
        header(&headers, "x-trace-id").as_deref(),
    )
    .await?;
    info!(
        action = "trade.order.price_snapshot.freeze",
        order_id = %order_id,
        pricing_mode = %snapshot.pricing_mode,
        billing_mode = %snapshot.billing_mode,
        "order price snapshot frozen"
    );
    Ok(ApiResponse::ok(FreezeOrderPriceSnapshotResponse {
        data: FreezeOrderPriceSnapshotResponseData { order_id, snapshot },
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TradePermission {
    CreateOrder,
    ReadOrder,
    CancelOrder,
    ConfirmContract,
    TransitionAuthorization,
    TransitionFileStd,
    TransitionFileSub,
    TransitionApiSub,
    TransitionApiPpu,
    TransitionShareRo,
    TransitionQryLite,
    TransitionSbxStd,
    TransitionRptStd,
    CreatePreRequest,
    ReadPreRequest,
}

fn is_allowed(role: &str, permission: TradePermission) -> bool {
    match permission {
        TradePermission::CreateOrder => matches!(
            role,
            "buyer_operator" | "tenant_admin" | "platform_admin" | "platform_risk_settlement"
        ),
        TradePermission::ReadOrder => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "auditor"
                | "platform_admin"
                | "platform_audit_security"
                | "platform_risk_settlement"
        ),
        TradePermission::CancelOrder => matches!(
            role,
            "buyer_operator" | "tenant_admin" | "platform_admin" | "platform_risk_settlement"
        ),
        TradePermission::ConfirmContract => {
            matches!(role, "buyer_operator" | "tenant_admin" | "platform_admin")
        }
        TradePermission::TransitionAuthorization => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        TradePermission::TransitionFileStd => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        TradePermission::TransitionFileSub => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        TradePermission::TransitionApiSub => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        TradePermission::TransitionApiPpu => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        TradePermission::TransitionShareRo => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        TradePermission::TransitionQryLite => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        TradePermission::TransitionSbxStd => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        TradePermission::TransitionRptStd => matches!(
            role,
            "buyer_operator"
                | "seller_operator"
                | "tenant_admin"
                | "platform_admin"
                | "platform_risk_settlement"
        ),
        TradePermission::CreatePreRequest => matches!(role, "buyer_operator" | "tenant_admin"),
        TradePermission::ReadPreRequest => matches!(
            role,
            "buyer_operator" | "seller_operator" | "tenant_admin" | "auditor"
        ),
    }
}

fn validate_signer_role(signer_role: &str) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if matches!(
        signer_role,
        "buyer_signatory" | "buyer_operator" | "legal_reviewer"
    ) {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: ErrorCode::TrdStateConflict.as_str().to_string(),
            message: format!("unsupported signer_role: {signer_role}"),
            request_id: None,
        }),
    ))
}

fn require_permission(
    headers: &HeaderMap,
    permission: TradePermission,
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

fn map_db_connect(err: tokio_postgres::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            code: ErrorCode::OpsInternal.as_str().to_string(),
            message: format!("database connection failed: {err}"),
            request_id: None,
        }),
    )
}

fn header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}

fn enforce_order_create_scope(
    headers: &HeaderMap,
    buyer_org_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = header(headers, "x-role").unwrap_or_default();
    if role.starts_with("platform_") {
        return Ok(());
    }
    let tenant_id = header(headers, "x-tenant-id").ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "order create requires x-tenant-id".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })?;
    if tenant_id == buyer_org_id {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: "order create is forbidden for tenant scope".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}

fn enforce_order_read_scope(
    headers: &HeaderMap,
    buyer_org_id: &str,
    seller_org_id: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = header(headers, "x-role").unwrap_or_default();
    if role.starts_with("platform_") {
        return Ok(());
    }
    let tenant_id = header(headers, "x-tenant-id").ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                code: ErrorCode::IamUnauthorized.as_str().to_string(),
                message: "order read requires x-tenant-id".to_string(),
                request_id: header(headers, "x-request-id"),
            }),
        )
    })?;
    if tenant_id == buyer_org_id || tenant_id == seller_org_id {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            code: ErrorCode::IamUnauthorized.as_str().to_string(),
            message: "order read is forbidden for tenant scope".to_string(),
            request_id: header(headers, "x-request-id"),
        }),
    ))
}
