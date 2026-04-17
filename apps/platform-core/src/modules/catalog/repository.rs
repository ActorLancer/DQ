use crate::modules::catalog::domain::{
    AssetVersionView, CreateAssetVersionRequest, CreateDataProductRequest,
    CreateDataResourceRequest, CreateProductSkuRequest, DataProductView, DataResourceView,
    ProductSkuView,
};
use tokio_postgres::{GenericClient, Row};

pub struct PostgresCatalogRepository;

impl PostgresCatalogRepository {
    pub async fn create_data_resource(
        client: &impl GenericClient,
        payload: &CreateDataResourceRequest,
    ) -> Result<DataResourceView, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO catalog.data_asset (
                   owner_org_id, title, category, sensitivity_level, status, description, metadata
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4, 'draft', $5, $6::jsonb
                 )
                 RETURNING
                   asset_id::text,
                   owner_org_id::text,
                   title,
                   category,
                   sensitivity_level,
                   status,
                   description,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &payload.owner_org_id,
                    &payload.title,
                    &payload.category,
                    &payload
                        .sensitivity_level
                        .clone()
                        .unwrap_or_else(|| "internal".to_string()),
                    &payload.description,
                    &payload.metadata,
                ],
            )
            .await?;
        Ok(parse_data_resource_row(&row))
    }

    pub async fn get_data_resource(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<DataResourceView>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                "SELECT
                   asset_id::text,
                   owner_org_id::text,
                   title,
                   category,
                   sensitivity_level,
                   status,
                   description,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM catalog.data_asset
                 WHERE asset_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| parse_data_resource_row(&row)))
    }

    pub async fn create_asset_version(
        client: &impl GenericClient,
        payload: &CreateAssetVersionRequest,
    ) -> Result<AssetVersionView, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO catalog.asset_version (
                   asset_id, version_no, schema_version, schema_hash, sample_hash, full_hash,
                   data_size_bytes, origin_region, allowed_region, requires_controlled_execution,
                   trust_boundary_snapshot, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4, $5, $6,
                   $7, $8, $9::text[], $10, $11::jsonb, 'draft', $12::jsonb
                 )
                 RETURNING
                   asset_version_id::text,
                   asset_id::text,
                   version_no,
                   schema_version,
                   schema_hash,
                   sample_hash,
                   full_hash,
                   data_size_bytes,
                   origin_region,
                   allowed_region::text[],
                   requires_controlled_execution,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &payload.asset_id,
                    &payload.version_no,
                    &payload.schema_version,
                    &payload.schema_hash,
                    &payload.sample_hash,
                    &payload.full_hash,
                    &payload.data_size_bytes,
                    &payload.origin_region,
                    &payload.allowed_region,
                    &payload.requires_controlled_execution.unwrap_or(false),
                    &payload.trust_boundary_snapshot,
                    &payload.metadata,
                ],
            )
            .await?;
        Ok(parse_asset_version_row(&row))
    }

    pub async fn get_asset_version(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<AssetVersionView>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                "SELECT
                   asset_version_id::text,
                   asset_id::text,
                   version_no,
                   schema_version,
                   schema_hash,
                   sample_hash,
                   full_hash,
                   data_size_bytes,
                   origin_region,
                   allowed_region::text[],
                   requires_controlled_execution,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM catalog.asset_version
                 WHERE asset_version_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| parse_asset_version_row(&row)))
    }

    pub async fn create_data_product(
        client: &impl GenericClient,
        payload: &CreateDataProductRequest,
    ) -> Result<DataProductView, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO catalog.product (
                   asset_id, asset_version_id, seller_org_id, title, category, product_type, description,
                   status, price_mode, price, currency_code, delivery_type, allowed_usage, searchable_text, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, $5, $6, $7,
                   'draft', $8, $9::text::numeric, $10, $11, $12::text[], $13, $14::jsonb
                 )
                 RETURNING
                   product_id::text,
                   asset_id::text,
                   asset_version_id::text,
                   seller_org_id::text,
                   title,
                   category,
                   product_type,
                   status,
                   price_mode,
                   price::text,
                   currency_code,
                   delivery_type,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &payload.asset_id,
                    &payload.asset_version_id,
                    &payload.seller_org_id,
                    &payload.title,
                    &payload.category,
                    &payload.product_type,
                    &payload.description,
                    &payload
                        .price_mode
                        .clone()
                        .unwrap_or_else(|| "one_time".to_string()),
                    &payload.price.clone().unwrap_or_else(|| "0".to_string()),
                    &payload
                        .currency_code
                        .clone()
                        .unwrap_or_else(|| "CNY".to_string()),
                    &payload.delivery_type,
                    &payload.allowed_usage,
                    &payload.searchable_text,
                    &payload.metadata,
                ],
            )
            .await?;
        Ok(parse_data_product_row(&row))
    }

    pub async fn get_data_product(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<DataProductView>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                "SELECT
                   product_id::text,
                   asset_id::text,
                   asset_version_id::text,
                   seller_org_id::text,
                   title,
                   category,
                   product_type,
                   status,
                   price_mode,
                   price::text,
                   currency_code,
                   delivery_type,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM catalog.product
                 WHERE product_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| parse_data_product_row(&row)))
    }

    pub async fn create_product_sku(
        client: &impl GenericClient,
        payload: &CreateProductSkuRequest,
    ) -> Result<ProductSkuView, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, acceptance_mode,
                   refund_mode, sla_json, quota_json, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4, $5, $6,
                   $7, $8::jsonb, $9::jsonb, 'draft', $10::jsonb
                 )
                 RETURNING
                   sku_id::text,
                   product_id::text,
                   sku_code,
                   sku_type,
                   unit_name,
                   billing_mode,
                   acceptance_mode,
                   refund_mode,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &payload.product_id,
                    &payload.sku_code,
                    &payload.sku_type,
                    &payload.unit_name,
                    &payload.billing_mode,
                    &payload.acceptance_mode,
                    &payload.refund_mode,
                    &payload.sla_json,
                    &payload.quota_json,
                    &payload.metadata,
                ],
            )
            .await?;
        Ok(parse_product_sku_row(&row))
    }

    pub async fn get_product_sku(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<ProductSkuView>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                "SELECT
                   sku_id::text,
                   product_id::text,
                   sku_code,
                   sku_type,
                   unit_name,
                   billing_mode,
                   acceptance_mode,
                   refund_mode,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM catalog.product_sku
                 WHERE sku_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| parse_product_sku_row(&row)))
    }

    pub async fn list_product_skus(
        client: &impl GenericClient,
        product_id: &str,
    ) -> Result<Vec<ProductSkuView>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT
                   sku_id::text,
                   product_id::text,
                   sku_code,
                   sku_type,
                   unit_name,
                   billing_mode,
                   acceptance_mode,
                   refund_mode,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM catalog.product_sku
                 WHERE product_id = $1::text::uuid
                 ORDER BY created_at DESC",
                &[&product_id],
            )
            .await?;
        Ok(rows.iter().map(parse_product_sku_row).collect())
    }
}

fn parse_data_resource_row(row: &Row) -> DataResourceView {
    DataResourceView {
        asset_id: row.get(0),
        owner_org_id: row.get(1),
        title: row.get(2),
        category: row.get(3),
        sensitivity_level: row.get(4),
        status: row.get(5),
        description: row.get(6),
        created_at: row.get(7),
        updated_at: row.get(8),
    }
}

fn parse_asset_version_row(row: &Row) -> AssetVersionView {
    AssetVersionView {
        asset_version_id: row.get(0),
        asset_id: row.get(1),
        version_no: row.get(2),
        schema_version: row.get(3),
        schema_hash: row.get(4),
        sample_hash: row.get(5),
        full_hash: row.get(6),
        data_size_bytes: row.get(7),
        origin_region: row.get(8),
        allowed_region: row.get(9),
        requires_controlled_execution: row.get(10),
        status: row.get(11),
        created_at: row.get(12),
        updated_at: row.get(13),
    }
}

fn parse_data_product_row(row: &Row) -> DataProductView {
    DataProductView {
        product_id: row.get(0),
        asset_id: row.get(1),
        asset_version_id: row.get(2),
        seller_org_id: row.get(3),
        title: row.get(4),
        category: row.get(5),
        product_type: row.get(6),
        status: row.get(7),
        price_mode: row.get(8),
        price: row.get(9),
        currency_code: row.get(10),
        delivery_type: row.get(11),
        created_at: row.get(12),
        updated_at: row.get(13),
    }
}

fn parse_product_sku_row(row: &Row) -> ProductSkuView {
    ProductSkuView {
        sku_id: row.get(0),
        product_id: row.get(1),
        sku_code: row.get(2),
        sku_type: row.get(3),
        unit_name: row.get(4),
        billing_mode: row.get(5),
        acceptance_mode: row.get(6),
        refund_mode: row.get(7),
        status: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    }
}
