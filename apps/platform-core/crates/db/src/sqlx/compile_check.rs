use sqlx::postgres::PgPool;
use uuid::Uuid;

pub const ORGANIZATION_STATUS_COMPILE_CHECK_SQL: &str = r#"
SELECT org_id, org_name, status
FROM core.organization
WHERE org_id = $1
"#;

pub const PRODUCT_STATUS_COMPILE_CHECK_SQL: &str = r#"
SELECT product_id, seller_org_id, title, status, delivery_type
FROM catalog.product
WHERE product_id = $1
"#;

pub const ORDER_SUMMARY_COMPILE_CHECK_SQL: &str = r#"
SELECT order_id, buyer_org_id, seller_org_id, status, payment_status
FROM trade.order_main
WHERE order_id = $1
"#;

pub const PAYMENT_INTENT_STATUS_COMPILE_CHECK_SQL: &str = r#"
SELECT payment_intent_id, order_id, provider_key, status
FROM payment.payment_intent
WHERE payment_intent_id = $1
"#;

#[derive(Debug)]
pub struct CompileCheckedOrganizationStatus {
    pub org_id: Uuid,
    pub org_name: String,
    pub status: String,
}

#[derive(Debug)]
pub struct CompileCheckedProductStatus {
    pub product_id: Uuid,
    pub seller_org_id: Uuid,
    pub title: String,
    pub status: String,
    pub delivery_type: String,
}

#[derive(Debug)]
pub struct CompileCheckedOrderSummary {
    pub order_id: Uuid,
    pub buyer_org_id: Uuid,
    pub seller_org_id: Uuid,
    pub status: String,
    pub payment_status: String,
}

#[derive(Debug)]
pub struct CompileCheckedPaymentIntentStatus {
    pub payment_intent_id: Uuid,
    pub order_id: Option<Uuid>,
    pub provider_key: String,
    pub status: String,
}

#[allow(dead_code)]
pub async fn load_organization_status(
    pool: &PgPool,
    org_id: Uuid,
) -> Result<CompileCheckedOrganizationStatus, sqlx::Error> {
    sqlx::query_as!(
        CompileCheckedOrganizationStatus,
        r#"
        SELECT org_id, org_name, status
        FROM core.organization
        WHERE org_id = $1
        "#,
        org_id
    )
    .fetch_one(pool)
    .await
}

#[allow(dead_code)]
pub async fn load_product_status(
    pool: &PgPool,
    product_id: Uuid,
) -> Result<CompileCheckedProductStatus, sqlx::Error> {
    sqlx::query_as!(
        CompileCheckedProductStatus,
        r#"
        SELECT product_id, seller_org_id, title, status, delivery_type
        FROM catalog.product
        WHERE product_id = $1
        "#,
        product_id
    )
    .fetch_one(pool)
    .await
}

#[allow(dead_code)]
pub async fn load_order_summary(
    pool: &PgPool,
    order_id: Uuid,
) -> Result<CompileCheckedOrderSummary, sqlx::Error> {
    sqlx::query_as!(
        CompileCheckedOrderSummary,
        r#"
        SELECT order_id, buyer_org_id, seller_org_id, status, payment_status
        FROM trade.order_main
        WHERE order_id = $1
        "#,
        order_id
    )
    .fetch_one(pool)
    .await
}

#[allow(dead_code)]
pub async fn load_payment_intent_status(
    pool: &PgPool,
    payment_intent_id: Uuid,
) -> Result<CompileCheckedPaymentIntentStatus, sqlx::Error> {
    sqlx::query_as!(
        CompileCheckedPaymentIntentStatus,
        r#"
        SELECT payment_intent_id, order_id, provider_key, status
        FROM payment.payment_intent
        WHERE payment_intent_id = $1
        "#,
        payment_intent_id
    )
    .fetch_one(pool)
    .await
}
