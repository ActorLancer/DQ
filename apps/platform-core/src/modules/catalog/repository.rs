use crate::modules::catalog::domain::{
    AssetVersionView, CreateAssetVersionRequest, CreateDataProductRequest,
    CreateDataResourceRequest, CreateProductSkuRequest, CreateRawIngestBatchRequest,
    CreateRawObjectManifestRequest, DataProductView, DataResourceView, PatchDataProductRequest,
    PatchProductSkuRequest, ProductSkuView, RawIngestBatchView, RawObjectManifestView,
};
use tokio_postgres::{GenericClient, Row};

pub struct PostgresCatalogRepository;

impl PostgresCatalogRepository {
    pub async fn get_raw_ingest_batch(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<RawIngestBatchView>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                "SELECT
                   raw_ingest_batch_id::text,
                   owner_org_id::text,
                   asset_id::text,
                   ingest_source_type,
                   declared_object_family,
                   source_declared_rights_json,
                   ingest_policy_json,
                   status,
                   created_by::text,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM catalog.raw_ingest_batch
                 WHERE raw_ingest_batch_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| parse_raw_ingest_batch_row(&row)))
    }

    pub async fn create_raw_ingest_batch(
        client: &impl GenericClient,
        asset_id: &str,
        payload: &CreateRawIngestBatchRequest,
        created_by: Option<&str>,
    ) -> Result<RawIngestBatchView, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO catalog.raw_ingest_batch (
                   owner_org_id, asset_id, ingest_source_type, declared_object_family,
                   source_declared_rights_json, ingest_policy_json, status, created_by
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, $4, $5::jsonb, $6::jsonb, 'draft', $7::text::uuid
                 )
                 RETURNING
                   raw_ingest_batch_id::text,
                   owner_org_id::text,
                   asset_id::text,
                   ingest_source_type,
                   declared_object_family,
                   source_declared_rights_json,
                   ingest_policy_json,
                   status,
                   created_by::text,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &payload.owner_org_id,
                    &asset_id,
                    &payload.ingest_source_type,
                    &payload.declared_object_family,
                    &payload.source_declared_rights_json,
                    &payload.ingest_policy_json,
                    &created_by,
                ],
            )
            .await?;
        Ok(parse_raw_ingest_batch_row(&row))
    }

    pub async fn create_raw_object_manifest(
        client: &impl GenericClient,
        raw_ingest_batch_id: &str,
        payload: &CreateRawObjectManifestRequest,
    ) -> Result<RawObjectManifestView, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO catalog.raw_object_manifest (
                   raw_ingest_batch_id, storage_binding_id, object_name, object_uri, mime_type,
                   container_type, byte_size, object_hash, source_time_range_json, manifest_json, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, $4, $5, $6, $7, $8, $9::jsonb, $10::jsonb, 'registered'
                 )
                 RETURNING
                   raw_object_manifest_id::text,
                   raw_ingest_batch_id::text,
                   storage_binding_id::text,
                   object_name,
                   object_uri,
                   mime_type,
                   container_type,
                   byte_size,
                   object_hash,
                   source_time_range_json,
                   manifest_json,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &raw_ingest_batch_id,
                    &payload.storage_binding_id,
                    &payload.object_name,
                    &payload.object_uri,
                    &payload.mime_type,
                    &payload.container_type,
                    &payload.byte_size,
                    &payload.object_hash,
                    &payload.source_time_range_json,
                    &payload.manifest_json,
                ],
            )
            .await?;
        Ok(parse_raw_object_manifest_row(&row))
    }

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

    pub async fn patch_data_product(
        client: &impl GenericClient,
        id: &str,
        payload: &PatchDataProductRequest,
    ) -> Result<Option<DataProductView>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                "UPDATE catalog.product
                 SET
                   title = COALESCE($2, title),
                   category = COALESCE($3, category),
                   product_type = COALESCE($4, product_type),
                   description = COALESCE($5, description),
                   price_mode = COALESCE($6, price_mode),
                   price = COALESCE($7::text::numeric, price),
                   currency_code = COALESCE($8, currency_code),
                   delivery_type = COALESCE($9, delivery_type),
                   searchable_text = COALESCE($10, searchable_text),
                   status = COALESCE($11, status),
                   updated_at = now()
                 WHERE product_id = $1::text::uuid
                   AND status = 'draft'
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
                    &id,
                    &payload.title,
                    &payload.category,
                    &payload.product_type,
                    &payload.description,
                    &payload.price_mode,
                    &payload.price,
                    &payload.currency_code,
                    &payload.delivery_type,
                    &payload.searchable_text,
                    &payload.status,
                ],
            )
            .await?;
        Ok(row.map(|row| parse_data_product_row(&row)))
    }

    pub async fn create_product_sku(
        client: &impl GenericClient,
        product_id: &str,
        payload: &CreateProductSkuRequest,
    ) -> Result<ProductSkuView, tokio_postgres::Error> {
        let mut metadata = payload.metadata.clone();
        if let Some(template_id) = &payload.template_id {
            metadata["draft_template_id"] = serde_json::Value::String(template_id.clone());
        }
        let row = client
            .query_one(
                "INSERT INTO catalog.product_sku (
                   product_id, sku_code, sku_type, unit_name, billing_mode, trade_mode,
                   delivery_object_kind, subscription_cadence, share_protocol, result_form,
                   acceptance_mode, refund_mode, sla_json, quota_json, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4, $5, $6,
                   $7, $8, $9, $10,
                   $11, $12, $13::jsonb, $14::jsonb, 'draft', $15::jsonb
                 )
                 RETURNING
                   sku_id::text,
                   product_id::text,
                   sku_code,
                   sku_type,
                   unit_name,
                   billing_mode,
                   trade_mode,
                   acceptance_mode,
                   refund_mode,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &product_id,
                    &payload.sku_code,
                    &payload.sku_type,
                    &payload.unit_name,
                    &payload.billing_mode,
                    &payload.trade_mode,
                    &payload.delivery_object_kind,
                    &payload.subscription_cadence,
                    &payload.share_protocol,
                    &payload.result_form,
                    &payload.acceptance_mode,
                    &payload.refund_mode,
                    &payload.sla_json,
                    &payload.quota_json,
                    &metadata,
                ],
            )
            .await?;
        Ok(parse_product_sku_row(&row))
    }

    pub async fn patch_product_sku(
        client: &impl GenericClient,
        id: &str,
        payload: &PatchProductSkuRequest,
    ) -> Result<Option<ProductSkuView>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                "UPDATE catalog.product_sku
                 SET
                   sku_code = COALESCE($2, sku_code),
                   sku_type = COALESCE($3, sku_type),
                   unit_name = COALESCE($4, unit_name),
                   billing_mode = COALESCE($5, billing_mode),
                   trade_mode = COALESCE($6, trade_mode),
                   delivery_object_kind = COALESCE($7, delivery_object_kind),
                   subscription_cadence = COALESCE($8, subscription_cadence),
                   share_protocol = COALESCE($9, share_protocol),
                   result_form = COALESCE($10, result_form),
                   acceptance_mode = COALESCE($11, acceptance_mode),
                   refund_mode = COALESCE($12, refund_mode),
                   status = COALESCE($13, status),
                   metadata = CASE
                     WHEN $14::text IS NULL THEN metadata
                     ELSE jsonb_set(metadata, '{draft_template_id}', to_jsonb($14::text), true)
                   END,
                   updated_at = now()
                 WHERE sku_id = $1::text::uuid
                   AND status = 'draft'
                 RETURNING
                   sku_id::text,
                   product_id::text,
                   sku_code,
                   sku_type,
                   unit_name,
                   billing_mode,
                   trade_mode,
                   acceptance_mode,
                   refund_mode,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &id,
                    &payload.sku_code,
                    &payload.sku_type,
                    &payload.unit_name,
                    &payload.billing_mode,
                    &payload.trade_mode,
                    &payload.delivery_object_kind,
                    &payload.subscription_cadence,
                    &payload.share_protocol,
                    &payload.result_form,
                    &payload.acceptance_mode,
                    &payload.refund_mode,
                    &payload.status,
                    &payload.template_id,
                ],
            )
            .await?;
        Ok(row.map(|row| parse_product_sku_row(&row)))
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
                   trade_mode,
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
                   trade_mode,
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
        trade_mode: row.get(6),
        acceptance_mode: row.get(7),
        refund_mode: row.get(8),
        status: row.get(9),
        created_at: row.get(10),
        updated_at: row.get(11),
    }
}

fn parse_raw_ingest_batch_row(row: &Row) -> RawIngestBatchView {
    RawIngestBatchView {
        raw_ingest_batch_id: row.get(0),
        owner_org_id: row.get(1),
        asset_id: row.get(2),
        ingest_source_type: row.get(3),
        declared_object_family: row.get(4),
        source_declared_rights_json: row.get(5),
        ingest_policy_json: row.get(6),
        status: row.get(7),
        created_by: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    }
}

fn parse_raw_object_manifest_row(row: &Row) -> RawObjectManifestView {
    RawObjectManifestView {
        raw_object_manifest_id: row.get(0),
        raw_ingest_batch_id: row.get(1),
        storage_binding_id: row.get(2),
        object_name: row.get(3),
        object_uri: row.get(4),
        mime_type: row.get(5),
        container_type: row.get(6),
        byte_size: row.get(7),
        object_hash: row.get(8),
        source_time_range_json: row.get(9),
        manifest_json: row.get(10),
        status: row.get(11),
        created_at: row.get(12),
        updated_at: row.get(13),
    }
}
