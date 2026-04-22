use crate::modules::catalog::domain::{
    AssetFieldDefinitionView, AssetObjectView, AssetProcessingInputView, AssetProcessingJobView,
    AssetQualityReportView, AssetReleasePolicyView, AssetVersionView, BindTemplateRequest,
    CreateAssetFieldDefinitionRequest, CreateAssetObjectRequest, CreateAssetProcessingJobRequest,
    CreateAssetQualityReportRequest, CreateAssetVersionRequest, CreateDataContractRequest,
    CreateDataProductRequest, CreateDataResourceRequest, CreateExtractionJobRequest,
    CreateFormatDetectionRequest, CreatePreviewArtifactRequest, CreateProductSkuRequest,
    CreateRawIngestBatchRequest, CreateRawObjectManifestRequest, DataContractView, DataProductView,
    DataResourceView, ExtractionJobView, FormatDetectionResultView, PatchAssetReleasePolicyRequest,
    PatchDataProductRequest, PatchProductSkuRequest, PatchUsagePolicyRequest, PreviewArtifactView,
    ProductDetailView, ProductMetadataProfileView, ProductSkuView,
    PutProductMetadataProfileRequest, RawIngestBatchView, RawObjectManifestView,
    ReviewDecisionView, SellerProfileView, TemplateBindingView, UsagePolicyView,
};
use db::{Error, GenericClient, Row};
use serde_json::{Value, json};

pub struct PostgresCatalogRepository;

impl PostgresCatalogRepository {
    pub async fn get_raw_ingest_batch(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<RawIngestBatchView>, Error> {
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
    ) -> Result<RawIngestBatchView, Error> {
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
    ) -> Result<RawObjectManifestView, Error> {
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

    pub async fn get_raw_object_manifest(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<RawObjectManifestView>, Error> {
        let row = client
            .query_opt(
                "SELECT
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
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM catalog.raw_object_manifest
                 WHERE raw_object_manifest_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| parse_raw_object_manifest_row(&row)))
    }

    pub async fn create_format_detection_result(
        client: &impl GenericClient,
        raw_object_manifest_id: &str,
        payload: &CreateFormatDetectionRequest,
    ) -> Result<FormatDetectionResultView, Error> {
        let row = client
            .query_one(
                "INSERT INTO catalog.format_detection_result (
                   raw_object_manifest_id, detected_object_family, detected_format, schema_hint_json,
                   recommended_processing_path, classification_confidence, detected_at, status
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4::jsonb, $5, $6::double precision::numeric(8,4), now(), $7
                 )
                 RETURNING
                   format_detection_result_id::text,
                   raw_object_manifest_id::text,
                   detected_object_family,
                   detected_format,
                   schema_hint_json,
                   recommended_processing_path,
                   classification_confidence::double precision,
                   to_char(detected_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &raw_object_manifest_id,
                    &payload.detected_object_family,
                    &payload.detected_format,
                    &payload.schema_hint_json,
                    &payload.recommended_processing_path,
                    &payload.classification_confidence,
                    &payload.status.clone().unwrap_or_else(|| "detected".to_string()),
                ],
            )
            .await?;
        Ok(parse_format_detection_result_row(&row))
    }

    pub async fn create_extraction_job(
        client: &impl GenericClient,
        raw_object_manifest_id: &str,
        payload: &CreateExtractionJobRequest,
    ) -> Result<ExtractionJobView, Error> {
        let row = client
            .query_one(
                "INSERT INTO catalog.extraction_job (
                   raw_object_manifest_id, asset_version_id, job_type, job_config_json,
                   result_summary_json, output_uri, output_hash, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, $4::jsonb, $5::jsonb, $6, $7, $8
                 )
                 RETURNING
                   extraction_job_id::text,
                   raw_object_manifest_id::text,
                   asset_version_id::text,
                   job_type,
                   job_config_json,
                   result_summary_json,
                   output_uri,
                   output_hash,
                   status,
                   to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &raw_object_manifest_id,
                    &payload.asset_version_id,
                    &payload.job_type,
                    &payload.job_config_json,
                    &payload.result_summary_json,
                    &payload.output_uri,
                    &payload.output_hash,
                    &payload
                        .status
                        .clone()
                        .unwrap_or_else(|| "draft".to_string()),
                ],
            )
            .await?;
        Ok(parse_extraction_job_row(&row))
    }

    pub async fn create_preview_artifact(
        client: &impl GenericClient,
        asset_version_id: &str,
        payload: &CreatePreviewArtifactRequest,
    ) -> Result<PreviewArtifactView, Error> {
        let row = client
            .query_one(
                "INSERT INTO catalog.preview_artifact (
                   asset_version_id, raw_object_manifest_id, preview_type, preview_uri, preview_hash,
                   preview_payload, preview_policy_json, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, $4, $5, $6::jsonb, $7::jsonb, $8
                 )
                 RETURNING
                   preview_artifact_id::text,
                   asset_version_id::text,
                   raw_object_manifest_id::text,
                   preview_type,
                   preview_uri,
                   preview_hash,
                   preview_payload,
                   preview_policy_json,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &asset_version_id,
                    &payload.raw_object_manifest_id,
                    &payload.preview_type,
                    &payload.preview_uri,
                    &payload.preview_hash,
                    &payload.preview_payload,
                    &payload.preview_policy_json,
                    &payload
                        .status
                        .clone()
                        .unwrap_or_else(|| "active".to_string()),
                ],
            )
            .await?;
        Ok(parse_preview_artifact_row(&row))
    }

    pub async fn create_data_resource(
        client: &impl GenericClient,
        payload: &CreateDataResourceRequest,
    ) -> Result<DataResourceView, Error> {
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
    ) -> Result<Option<DataResourceView>, Error> {
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
    ) -> Result<AssetVersionView, Error> {
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
    ) -> Result<Option<AssetVersionView>, Error> {
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
    ) -> Result<DataProductView, Error> {
        let metadata = compose_product_metadata(
            &payload.metadata,
            payload.subtitle.as_deref(),
            payload.industry.as_deref(),
            Some(&payload.use_cases),
            payload.data_classification.as_deref(),
            payload.quality_score.as_deref(),
        );
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
                    &metadata,
                ],
            )
            .await?;
        Ok(parse_data_product_row(&row))
    }

    pub async fn get_data_product(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<DataProductView>, Error> {
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

    pub async fn get_product_detail(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<ProductDetailView>, Error> {
        let row = client
            .query_opt(
                "SELECT
                   p.product_id::text,
                   p.asset_id::text,
                   p.asset_version_id::text,
                   p.seller_org_id::text,
                   p.title,
                   p.category,
                   p.product_type,
                   p.status,
                   p.description,
                   p.price_mode,
                   p.price::text,
                   p.currency_code,
                   p.delivery_type,
                   p.allowed_usage::text[],
                   p.searchable_text,
                   p.metadata,
                   COALESCE(spd.document_version, 0)::int,
                   COALESCE(spd.index_sync_status, 'pending'),
                   to_char(p.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(p.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM catalog.product p
                 LEFT JOIN search.product_search_document spd ON spd.product_id = p.product_id
                 WHERE p.product_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| {
            let metadata: Value = row.get(15);
            ProductDetailView {
                product_id: row.get(0),
                asset_id: row.get(1),
                asset_version_id: row.get(2),
                seller_org_id: row.get(3),
                title: row.get(4),
                category: row.get(5),
                product_type: row.get(6),
                status: row.get(7),
                description: row.get(8),
                price_mode: row.get(9),
                price: row.get(10),
                currency_code: row.get(11),
                delivery_type: row.get(12),
                allowed_usage: row.get(13),
                searchable_text: row.get(14),
                subtitle: metadata
                    .get("subtitle")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                industry: metadata
                    .get("industry")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                use_cases: metadata
                    .get("use_cases")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_str)
                            .map(ToString::to_string)
                            .collect()
                    })
                    .unwrap_or_default(),
                data_classification: metadata
                    .get("data_classification")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                quality_score: metadata
                    .get("quality_score")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                metadata,
                search_document_version: row.get(16),
                index_sync_status: row.get(17),
                skus: Vec::new(),
                created_at: row.get(18),
                updated_at: row.get(19),
            }
        }))
    }

    pub async fn patch_data_product(
        client: &impl GenericClient,
        id: &str,
        payload: &PatchDataProductRequest,
    ) -> Result<Option<DataProductView>, Error> {
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
                   metadata = jsonb_strip_nulls(
                     metadata || jsonb_build_object(
                       'subtitle', $12::text,
                       'industry', $13::text,
                       'use_cases', CASE
                         WHEN $14::text[] IS NULL THEN NULL
                         ELSE to_jsonb($14::text[])
                       END,
                       'data_classification', $15::text,
                       'quality_score', $16::text
                     )
                   ),
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
                    &payload.subtitle,
                    &payload.industry,
                    &payload.use_cases,
                    &payload.data_classification,
                    &payload.quality_score,
                ],
            )
            .await?;
        Ok(row.map(|row| parse_data_product_row(&row)))
    }

    pub async fn product_has_metadata_profile(
        client: &impl GenericClient,
        product_id: &str,
    ) -> Result<bool, Error> {
        let row = client
            .query_one(
                "SELECT EXISTS (
                   SELECT 1
                   FROM catalog.product_metadata_profile
                   WHERE product_id = $1::text::uuid
                 )",
                &[&product_id],
            )
            .await?;
        Ok(row.get(0))
    }

    pub async fn product_has_skus(
        client: &impl GenericClient,
        product_id: &str,
    ) -> Result<bool, Error> {
        let row = client
            .query_one(
                "SELECT EXISTS (
                   SELECT 1
                   FROM catalog.product_sku
                   WHERE product_id = $1::text::uuid
                 )",
                &[&product_id],
            )
            .await?;
        Ok(row.get(0))
    }

    pub async fn product_all_skus_have_template(
        client: &impl GenericClient,
        product_id: &str,
    ) -> Result<bool, Error> {
        let row = client
            .query_one(
                "SELECT NOT EXISTS (
                   SELECT 1
                   FROM catalog.product_sku sku
                   WHERE sku.product_id = $1::text::uuid
                     AND (
                       coalesce(nullif(sku.metadata->>'draft_template_id', ''), '') = ''
                       OR NOT (
                         (sku.metadata->>'draft_template_id') ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$'
                         AND EXISTS (
                           SELECT 1
                           FROM contract.template_definition t
                           WHERE t.template_id = (sku.metadata->>'draft_template_id')::uuid
                             AND t.status = 'active'
                             AND sku.sku_type = ANY(t.applicable_sku_types)
                         )
                       )
                     )
                 )",
                &[&product_id],
            )
            .await?;
        Ok(row.get(0))
    }

    pub async fn product_is_risk_blocked(
        client: &impl GenericClient,
        product_id: &str,
    ) -> Result<bool, Error> {
        let row = client
            .query_one(
                "SELECT
                   CASE
                     WHEN lower(coalesce(metadata->>'risk_blocked', 'false')) IN ('true', '1') THEN true
                     WHEN lower(coalesce(metadata#>>'{risk_flags,block_submit}', 'false')) IN ('true', '1') THEN true
                     ELSE false
                   END
                 FROM catalog.product
                 WHERE product_id = $1::text::uuid",
                &[&product_id],
            )
            .await?;
        Ok(row.get(0))
    }

    pub async fn transition_product_status(
        client: &impl GenericClient,
        product_id: &str,
        from_status: &str,
        to_status: &str,
    ) -> Result<Option<DataProductView>, Error> {
        let row = client
            .query_opt(
                "UPDATE catalog.product
                 SET status = $3,
                     updated_at = now()
                 WHERE product_id = $1::text::uuid
                   AND status = $2
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
                &[&product_id, &from_status, &to_status],
            )
            .await?;
        Ok(row.map(|row| parse_data_product_row(&row)))
    }

    pub async fn create_review_decision(
        client: &impl GenericClient,
        review_type: &str,
        ref_type: &str,
        ref_id: &str,
        action_name: &str,
        action_reason: Option<&str>,
        status: &str,
    ) -> Result<ReviewDecisionView, Error> {
        let task_row = client
            .query_one(
                "INSERT INTO review.review_task (
                   review_type, ref_type, ref_id, status
                 ) VALUES (
                   $1, $2, $3::text::uuid, $4
                 )
                 RETURNING review_task_id::text, review_type, ref_type, ref_id::text, status",
                &[&review_type, &ref_type, &ref_id, &status],
            )
            .await?;
        let review_task_id: String = task_row.get(0);

        client
            .query_one(
                "INSERT INTO review.review_step (
                   review_task_id, step_no, action_name, action_reason, action_at
                 ) VALUES (
                   $1::text::uuid, 1, $2, $3, now()
                 )
                 RETURNING review_step_id::text",
                &[&review_task_id, &action_name, &action_reason],
            )
            .await?;
        Ok(parse_review_decision_row(&task_row))
    }

    pub async fn organization_exists(
        client: &impl GenericClient,
        org_id: &str,
    ) -> Result<bool, Error> {
        let row = client
            .query_one(
                "SELECT EXISTS (
                   SELECT 1 FROM core.organization WHERE org_id = $1::text::uuid
                 )",
                &[&org_id],
            )
            .await?;
        Ok(row.get(0))
    }

    pub async fn get_seller_profile(
        client: &impl GenericClient,
        org_id: &str,
    ) -> Result<Option<SellerProfileView>, Error> {
        let row = client
            .query_opt(
                "SELECT
                   org.org_id::text,
                   org.org_name,
                   org.org_type,
                   org.status,
                   org.country_code,
                   org.region_code,
                   COALESCE(org.industry_tags, '{}')::text[],
                   COALESCE(ssd.certification_tags, '{}')::text[],
                   COALESCE(ssd.featured_products, '[]'::jsonb),
                   COALESCE(ssd.rating_summary, '{}'::jsonb),
                   COALESCE(rep.credit_level, org.credit_level, 0),
                   COALESCE(rep.risk_level, org.risk_level, 0),
                   COALESCE(rep.score, 0)::text,
                   COALESCE(listed.listing_product_count, 0)::bigint,
                   NULLIF(org.metadata ->> 'description', ''),
                   COALESCE(ssd.document_version, 0)::int,
                   COALESCE(ssd.index_sync_status, 'pending')
                 FROM core.organization org
                 LEFT JOIN LATERAL (
                   SELECT rs.score, rs.credit_level, rs.risk_level
                   FROM risk.reputation_snapshot rs
                   WHERE rs.subject_type = 'organization'
                     AND rs.subject_id = org.org_id
                   ORDER BY rs.effective_at DESC
                   LIMIT 1
                 ) AS rep ON true
                 LEFT JOIN LATERAL (
                   SELECT COUNT(*)::bigint AS listing_product_count
                   FROM catalog.product p
                   WHERE p.seller_org_id = org.org_id
                     AND p.status = 'listed'
                 ) AS listed ON true
                 LEFT JOIN search.seller_search_document ssd ON ssd.org_id = org.org_id
                 WHERE org.org_id = $1::text::uuid",
                &[&org_id],
            )
            .await?;
        Ok(row.map(|row| SellerProfileView {
            org_id: row.get(0),
            org_name: row.get(1),
            org_type: row.get(2),
            status: row.get(3),
            country_code: row.get(4),
            region_code: row.get(5),
            industry_tags: row.get(6),
            certification_tags: row.get(7),
            featured_products: row.get(8),
            rating_summary: row.get(9),
            credit_level: row.get(10),
            risk_level: row.get(11),
            reputation_score: row.get(12),
            listed_product_count: row.get(13),
            description: row.get(14),
            search_document_version: row.get(15),
            index_sync_status: row.get(16),
        }))
    }

    pub async fn create_review_task_with_initial_step(
        client: &impl GenericClient,
        review_type: &str,
        ref_type: &str,
        ref_id: &str,
        initial_action: &str,
        initial_reason: Option<&str>,
    ) -> Result<ReviewDecisionView, Error> {
        let task_row = client
            .query_one(
                "INSERT INTO review.review_task (
                   review_type, ref_type, ref_id, status
                 ) VALUES (
                   $1, $2, $3::text::uuid, 'pending'
                 )
                 RETURNING review_task_id::text, review_type, ref_type, ref_id::text, status",
                &[&review_type, &ref_type, &ref_id],
            )
            .await?;
        let review_task_id: String = task_row.get(0);
        client
            .query_one(
                "INSERT INTO review.review_step (
                   review_task_id, step_no, action_name, action_reason, action_at
                 ) VALUES (
                   $1::text::uuid, 1, $2, $3, now()
                 )
                 RETURNING review_step_id::text",
                &[&review_task_id, &initial_action, &initial_reason],
            )
            .await?;
        Ok(parse_review_decision_row(&task_row))
    }

    pub async fn has_pending_review_task(
        client: &impl GenericClient,
        review_type: &str,
        ref_type: &str,
        ref_id: &str,
    ) -> Result<bool, Error> {
        let row = client
            .query_one(
                "SELECT EXISTS (
                   SELECT 1
                   FROM review.review_task
                   WHERE review_type = $1
                     AND ref_type = $2
                     AND ref_id = $3::text::uuid
                     AND status = 'pending'
                 )",
                &[&review_type, &ref_type, &ref_id],
            )
            .await?;
        Ok(row.get(0))
    }

    pub async fn append_review_step_and_close_task(
        client: &impl GenericClient,
        review_type: &str,
        ref_type: &str,
        ref_id: &str,
        action_name: &str,
        action_reason: Option<&str>,
        next_status: &str,
    ) -> Result<Option<ReviewDecisionView>, Error> {
        let task_row = client
            .query_opt(
                "SELECT review_task_id::text
                 FROM review.review_task
                 WHERE review_type = $1
                   AND ref_type = $2
                   AND ref_id = $3::text::uuid
                   AND status = 'pending'
                 ORDER BY created_at DESC
                 LIMIT 1",
                &[&review_type, &ref_type, &ref_id],
            )
            .await?;
        let Some(task_row) = task_row else {
            return Ok(None);
        };
        let task_id: String = task_row.get(0);
        let inserted_step_row = client
            .query_one(
                "INSERT INTO review.review_step (
                   review_task_id, step_no, action_name, action_reason, action_at
                 ) VALUES (
                   $1::text::uuid,
                   COALESCE((
                     SELECT max(step_no) + 1
                     FROM review.review_step
                     WHERE review_task_id = $1::text::uuid
                   ), 1),
                   $2,
                   $3,
                   now()
                 )
                 RETURNING step_no",
                &[&task_id, &action_name, &action_reason],
            )
            .await?;
        let current_step_no: i32 = inserted_step_row.get(0);
        let updated_row = client
            .query_one(
                "UPDATE review.review_task
                 SET status = $2,
                     current_step_no = $3,
                     updated_at = now()
                 WHERE review_task_id = $1::text::uuid
                 RETURNING review_task_id::text, review_type, ref_type, ref_id::text, status",
                &[&task_id, &next_status, &current_step_no],
            )
            .await?;
        Ok(Some(parse_review_decision_row(&updated_row)))
    }

    pub async fn is_verified_step_up_challenge(
        client: &impl GenericClient,
        challenge_id: &str,
        user_id: &str,
        target_action: &str,
        target_ref_type: &str,
        target_ref_id: &str,
    ) -> Result<bool, Error> {
        let row = client
            .query_one(
                "SELECT EXISTS (
                   SELECT 1
                   FROM iam.step_up_challenge
                   WHERE step_up_challenge_id = $1::text::uuid
                     AND user_id = $2::text::uuid
                     AND target_action = $3
                     AND target_ref_type = $4
                     AND target_ref_id = $5::text::uuid
                     AND challenge_status = 'verified'
                     AND expires_at > now()
                 )",
                &[
                    &challenge_id,
                    &user_id,
                    &target_action,
                    &target_ref_type,
                    &target_ref_id,
                ],
            )
            .await?;
        Ok(row.get(0))
    }

    pub async fn upsert_product_metadata_profile(
        client: &impl GenericClient,
        product_id: &str,
        asset_version_id: &str,
        payload: &PutProductMetadataProfileRequest,
    ) -> Result<ProductMetadataProfileView, Error> {
        let metadata_version_no = payload.metadata_version_no.unwrap_or(1);
        let business_description_json = normalize_json_object(&payload.business_description_json);
        let data_content_json = normalize_json_object(&payload.data_content_json);
        let structure_description_json = normalize_json_object(&payload.structure_description_json);
        let quality_description_json = normalize_json_object(&payload.quality_description_json);
        let compliance_description_json =
            normalize_json_object(&payload.compliance_description_json);
        let delivery_description_json = normalize_json_object(&payload.delivery_description_json);
        let version_description_json = normalize_json_object(&payload.version_description_json);
        let authorization_description_json =
            normalize_json_object(&payload.authorization_description_json);
        let responsibility_description_json =
            normalize_json_object(&payload.responsibility_description_json);
        let processing_overview_json = normalize_json_object(&payload.processing_overview_json);
        let metadata = normalize_json_object(&payload.metadata);
        let row = client
            .query_one(
                "INSERT INTO catalog.product_metadata_profile (
                   product_id, asset_version_id, metadata_version_no,
                   business_description_json, data_content_json, structure_description_json,
                   quality_description_json, compliance_description_json, delivery_description_json,
                   version_description_json, authorization_description_json, responsibility_description_json,
                   processing_overview_json, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3,
                   $4::jsonb, $5::jsonb, $6::jsonb, $7::jsonb, $8::jsonb, $9::jsonb,
                   $10::jsonb, $11::jsonb, $12::jsonb, $13::jsonb, $14, $15::jsonb
                 )
                 ON CONFLICT (product_id, metadata_version_no) DO UPDATE SET
                   asset_version_id = EXCLUDED.asset_version_id,
                   business_description_json = EXCLUDED.business_description_json,
                   data_content_json = EXCLUDED.data_content_json,
                   structure_description_json = EXCLUDED.structure_description_json,
                   quality_description_json = EXCLUDED.quality_description_json,
                   compliance_description_json = EXCLUDED.compliance_description_json,
                   delivery_description_json = EXCLUDED.delivery_description_json,
                   version_description_json = EXCLUDED.version_description_json,
                   authorization_description_json = EXCLUDED.authorization_description_json,
                   responsibility_description_json = EXCLUDED.responsibility_description_json,
                   processing_overview_json = EXCLUDED.processing_overview_json,
                   status = EXCLUDED.status,
                   metadata = EXCLUDED.metadata
                 RETURNING
                   product_metadata_profile_id::text,
                   product_id::text,
                   asset_version_id::text,
                   metadata_version_no,
                   business_description_json,
                   data_content_json,
                   structure_description_json,
                   quality_description_json,
                   compliance_description_json,
                   delivery_description_json,
                   version_description_json,
                   authorization_description_json,
                   responsibility_description_json,
                   processing_overview_json,
                   status,
                   metadata,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &product_id,
                    &asset_version_id,
                    &metadata_version_no,
                    &business_description_json,
                    &data_content_json,
                    &structure_description_json,
                    &quality_description_json,
                    &compliance_description_json,
                    &delivery_description_json,
                    &version_description_json,
                    &authorization_description_json,
                    &responsibility_description_json,
                    &processing_overview_json,
                    &payload
                        .status
                        .clone()
                        .unwrap_or_else(|| "draft".to_string()),
                    &metadata,
                ],
            )
            .await?;
        Ok(parse_product_metadata_profile_row(&row))
    }

    pub async fn create_asset_field_definition(
        client: &impl GenericClient,
        asset_version_id: &str,
        payload: &CreateAssetFieldDefinitionRequest,
    ) -> Result<AssetFieldDefinitionView, Error> {
        let enum_values_json = normalize_json_array(&payload.enum_values_json);
        let row = client
            .query_one(
                "INSERT INTO catalog.asset_field_definition (
                   asset_version_id, object_name, field_name, field_path, field_type,
                   is_nullable, is_primary_key, is_partition_key, is_time_field, code_rule,
                   unit_text, enum_values_json, description
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb, $13
                 )
                 RETURNING
                   field_definition_id::text,
                   asset_version_id::text,
                   object_name,
                   field_name,
                   field_path,
                   field_type,
                   is_nullable,
                   is_primary_key,
                   is_partition_key,
                   is_time_field,
                   code_rule,
                   unit_text,
                   enum_values_json,
                   description,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &asset_version_id,
                    &payload.object_name,
                    &payload.field_name,
                    &payload.field_path,
                    &payload.field_type,
                    &payload.is_nullable.unwrap_or(true),
                    &payload.is_primary_key.unwrap_or(false),
                    &payload.is_partition_key.unwrap_or(false),
                    &payload.is_time_field.unwrap_or(false),
                    &payload.code_rule,
                    &payload.unit_text,
                    &enum_values_json,
                    &payload.description,
                ],
            )
            .await?;
        Ok(parse_asset_field_definition_row(&row))
    }

    pub async fn create_asset_quality_report(
        client: &impl GenericClient,
        asset_version_id: &str,
        payload: &CreateAssetQualityReportRequest,
    ) -> Result<AssetQualityReportView, Error> {
        let coverage_range_json = normalize_json_object(&payload.coverage_range_json);
        let freshness_json = normalize_json_object(&payload.freshness_json);
        let metrics_json = normalize_json_object(&payload.metrics_json);
        let metadata = normalize_json_object(&payload.metadata);
        let row = client
            .query_one(
                "INSERT INTO catalog.asset_quality_report (
                   asset_version_id, report_no, report_type, coverage_range_json, freshness_json,
                   missing_rate, duplicate_rate, anomaly_rate, sampling_method, assessed_at,
                   assessor_org_id, report_uri, report_hash, metrics_json, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4::jsonb, $5::jsonb,
                   $6::double precision::numeric(8,6), $7::double precision::numeric(8,6),
                   $8::double precision::numeric(8,6), $9, $10::text::timestamptz,
                   $11::text::uuid, $12, $13, $14::jsonb, $15, $16::jsonb
                 )
                 RETURNING
                   quality_report_id::text,
                   asset_version_id::text,
                   report_no,
                   report_type,
                   coverage_range_json,
                   freshness_json,
                   missing_rate::double precision,
                   duplicate_rate::double precision,
                   anomaly_rate::double precision,
                   sampling_method,
                   to_char(assessed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   assessor_org_id::text,
                   report_uri,
                   report_hash,
                   metrics_json,
                   status,
                   metadata,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &asset_version_id,
                    &payload.report_no.unwrap_or(1),
                    &payload
                        .report_type
                        .clone()
                        .unwrap_or_else(|| "seller_declared".to_string()),
                    &coverage_range_json,
                    &freshness_json,
                    &payload.missing_rate,
                    &payload.duplicate_rate,
                    &payload.anomaly_rate,
                    &payload.sampling_method,
                    &payload.assessed_at,
                    &payload.assessor_org_id,
                    &payload.report_uri,
                    &payload.report_hash,
                    &metrics_json,
                    &payload
                        .status
                        .clone()
                        .unwrap_or_else(|| "draft".to_string()),
                    &metadata,
                ],
            )
            .await?;
        Ok(parse_asset_quality_report_row(&row))
    }

    pub async fn create_asset_object(
        client: &impl GenericClient,
        asset_version_id: &str,
        payload: &CreateAssetObjectRequest,
    ) -> Result<AssetObjectView, Error> {
        let object_row = client
            .query_one(
                "INSERT INTO catalog.asset_object_binding (
                   asset_version_id, object_kind, object_name, object_locator, share_protocol,
                   schema_json, output_schema_json, freshness_json, access_constraints, metadata
                 ) VALUES (
                   $1::text::uuid, $2, $3, $4, $5, $6::jsonb, $7::jsonb, $8::jsonb, $9::jsonb, $10::jsonb
                 )
                 RETURNING
                   asset_object_id::text,
                   asset_version_id::text,
                   object_kind,
                   object_name,
                   object_locator,
                   share_protocol,
                   schema_json,
                   output_schema_json,
                   freshness_json,
                   access_constraints,
                   metadata,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &asset_version_id,
                    &payload.object_kind,
                    &payload.object_name,
                    &payload.object_locator,
                    &payload.share_protocol,
                    &normalize_object_json(&payload.schema_json),
                    &normalize_object_json(&payload.output_schema_json),
                    &normalize_object_json(&payload.freshness_json),
                    &normalize_object_json(&payload.access_constraints),
                    &normalize_object_json(&payload.metadata),
                ],
            )
            .await?;

        let storage_row = client
            .query_one(
                "INSERT INTO catalog.asset_storage_binding (
                   asset_version_id, storage_type, object_uri, payload_role, object_hash,
                   encryption_algo, worm_enabled, metadata, storage_zone, access_path_type
                 ) VALUES (
                   $1::text::uuid, $2, $3, 'primary_payload', $4, $5, $6, $7::jsonb, $8, $9
                 )
                 RETURNING
                   asset_storage_binding_id::text,
                   object_uri,
                   storage_type,
                   storage_zone,
                   access_path_type,
                   object_hash,
                   encryption_algo,
                   worm_enabled,
                   metadata",
                &[
                    &asset_version_id,
                    &payload
                        .storage_type
                        .clone()
                        .unwrap_or_else(|| "object_storage".to_string()),
                    &payload.object_uri,
                    &payload.object_hash,
                    &payload.encryption_algo,
                    &payload.worm_enabled.unwrap_or(false),
                    &normalize_object_json(&payload.metadata),
                    &payload
                        .storage_zone
                        .clone()
                        .unwrap_or_else(|| "product".to_string()),
                    &payload
                        .access_path_type
                        .clone()
                        .unwrap_or_else(|| "object_uri".to_string()),
                ],
            )
            .await?;

        Ok(parse_asset_object_row(&object_row, &storage_row))
    }

    pub async fn create_asset_processing_job(
        client: &impl GenericClient,
        output_asset_version_id: &str,
        payload: &CreateAssetProcessingJobRequest,
    ) -> Result<AssetProcessingJobView, Error> {
        let mut metadata = normalize_object_json(&payload.metadata);
        let processing_summary_json = normalize_object_json(&payload.processing_summary_json);
        if let Value::Object(ref mut metadata_map) = metadata {
            metadata_map.insert(
                "processing_summary_json".to_string(),
                processing_summary_json.clone(),
            );
        }
        let row = client
            .query_one(
                "INSERT INTO catalog.asset_processing_job (
                   output_asset_version_id, processing_mode, processor_org_id, executor_type, job_name,
                   transform_spec_version, desensitization_profile, standardization_profile, labeling_profile,
                   model_artifact_ref, evidence_uri, evidence_hash, started_at, completed_at, status, metadata
                 ) VALUES (
                   $1::text::uuid, $2, $3::text::uuid, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                   $13::text::timestamptz, $14::text::timestamptz, $15, $16::jsonb
                 )
                 RETURNING
                   processing_job_id::text,
                   output_asset_version_id::text,
                   processing_mode,
                   processor_org_id::text,
                   executor_type,
                   job_name,
                   transform_spec_version,
                   desensitization_profile,
                   standardization_profile,
                   labeling_profile,
                   model_artifact_ref,
                   evidence_uri,
                   evidence_hash,
                   to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   status,
                   metadata,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &output_asset_version_id,
                    &payload.processing_mode,
                    &payload.processor_org_id,
                    &payload
                        .executor_type
                        .clone()
                        .unwrap_or_else(|| "seller".to_string()),
                    &payload.job_name,
                    &payload.transform_spec_version,
                    &payload.desensitization_profile,
                    &payload.standardization_profile,
                    &payload.labeling_profile,
                    &payload.model_artifact_ref,
                    &payload.evidence_uri,
                    &payload.evidence_hash,
                    &payload.started_at,
                    &payload.completed_at,
                    &payload
                        .status
                        .clone()
                        .unwrap_or_else(|| "draft".to_string()),
                    &metadata,
                ],
            )
            .await?;

        let processing_job_id: String = row.get(0);
        let mut input_sources = Vec::with_capacity(payload.input_sources.len());
        for source in &payload.input_sources {
            let input_row = client
                .query_one(
                    "INSERT INTO catalog.asset_processing_input (
                       processing_job_id, input_asset_version_id, input_role
                     ) VALUES (
                       $1::text::uuid, $2::text::uuid, $3
                     )
                     RETURNING
                       processing_input_id::text,
                       input_asset_version_id::text,
                       input_role,
                       to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                    &[
                        &processing_job_id,
                        &source.input_asset_version_id,
                        &source
                            .input_role
                            .clone()
                            .unwrap_or_else(|| "primary_input".to_string()),
                    ],
                )
                .await?;
            input_sources.push(parse_asset_processing_input_row(&input_row));
        }

        Ok(parse_asset_processing_job_row(&row, input_sources))
    }

    pub async fn create_product_sku(
        client: &impl GenericClient,
        product_id: &str,
        payload: &CreateProductSkuRequest,
    ) -> Result<ProductSkuView, Error> {
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
    ) -> Result<Option<ProductSkuView>, Error> {
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
    ) -> Result<Option<ProductSkuView>, Error> {
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
    ) -> Result<Vec<ProductSkuView>, Error> {
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

    pub async fn set_product_default_template(
        client: &impl GenericClient,
        product_id: &str,
        template_id: &str,
    ) -> Result<(), Error> {
        client
            .execute(
                "UPDATE catalog.product
                 SET metadata = jsonb_set(metadata, '{draft_template_id}', to_jsonb($2::text), true),
                     updated_at = now()
                 WHERE product_id = $1::text::uuid",
                &[&product_id, &template_id],
            )
            .await?;
        Ok(())
    }

    pub async fn bind_template_to_sku(
        client: &impl GenericClient,
        sku_id: &str,
        payload: &BindTemplateRequest,
    ) -> Result<(), Error> {
        let binding_type = payload.binding_type.as_deref().unwrap_or("contract");
        client
            .execute(
                "INSERT INTO contract.template_binding (
                   sku_id, template_id, binding_type, status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, 'active'
                 )
                 ON CONFLICT (sku_id, template_id, binding_type)
                 DO UPDATE SET status = 'active'",
                &[&sku_id, &payload.template_id, &binding_type],
            )
            .await?;
        client
            .execute(
                "UPDATE catalog.product_sku
                 SET metadata = jsonb_set(metadata, '{draft_template_id}', to_jsonb($2::text), true),
                     updated_at = now()
                 WHERE sku_id = $1::text::uuid",
                &[&sku_id, &payload.template_id],
            )
            .await?;
        Ok(())
    }

    pub async fn build_template_binding_view(
        client: &impl GenericClient,
        binding_scope: &str,
        target_id: &str,
        payload: &BindTemplateRequest,
        bound_sku_count: i32,
    ) -> Result<TemplateBindingView, Error> {
        let row = client
            .query_one(
                "SELECT to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[],
            )
            .await?;
        Ok(TemplateBindingView {
            binding_scope: binding_scope.to_string(),
            target_id: target_id.to_string(),
            template_id: payload.template_id.clone(),
            binding_type: payload
                .binding_type
                .clone()
                .unwrap_or_else(|| "contract".to_string()),
            status: "active".to_string(),
            bound_sku_count,
            updated_at: row.get(0),
        })
    }

    pub async fn patch_usage_policy(
        client: &impl GenericClient,
        policy_id: &str,
        payload: &PatchUsagePolicyRequest,
    ) -> Result<Option<UsagePolicyView>, Error> {
        let row = client
            .query_opt(
                "UPDATE contract.usage_policy
                 SET
                   policy_name = COALESCE($2, policy_name),
                   subject_constraints = COALESCE($3::jsonb, subject_constraints),
                   usage_constraints = COALESCE($4::jsonb, usage_constraints),
                   time_constraints = COALESCE($5::jsonb, time_constraints),
                   region_constraints = COALESCE($6::jsonb, region_constraints),
                   output_constraints = COALESCE($7::jsonb, output_constraints),
                   exportable = COALESCE($8, exportable),
                   status = COALESCE($9, status),
                   updated_at = now()
                 WHERE policy_id = $1::text::uuid
                 RETURNING
                   policy_id::text,
                   owner_org_id::text,
                   policy_name,
                   stage_from,
                   subject_constraints,
                   usage_constraints,
                   time_constraints,
                   region_constraints,
                   output_constraints,
                   exportable,
                   status,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &policy_id,
                    &payload.policy_name,
                    &payload.subject_constraints,
                    &payload.usage_constraints,
                    &payload.time_constraints,
                    &payload.region_constraints,
                    &payload.output_constraints,
                    &payload.exportable,
                    &payload.status,
                ],
            )
            .await?;
        Ok(row.map(|row| parse_usage_policy_row(&row)))
    }

    pub async fn create_data_contract(
        client: &impl GenericClient,
        sku_id: &str,
        payload: &CreateDataContractRequest,
    ) -> Result<DataContractView, Error> {
        let row = client
            .query_one(
                "INSERT INTO contract.data_contract (
                   asset_version_id, product_id, sku_id, contract_name, version_no, contract_scope,
                   business_terms_json, structure_terms_json, quality_terms_json, compliance_terms_json,
                   delivery_terms_json, version_terms_json, acceptance_terms_json, rights_terms_json,
                   responsibility_terms_json, processing_terms_json, content_digest, status,
                   effective_from, effective_to, metadata
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3::text::uuid, $4, $5, $6,
                   $7::jsonb, $8::jsonb, $9::jsonb, $10::jsonb,
                   $11::jsonb, $12::jsonb, $13::jsonb, $14::jsonb,
                   $15::jsonb, $16::jsonb, $17, $18,
                   $19::text::timestamptz, $20::text::timestamptz, $21::jsonb
                 )
                 RETURNING
                   data_contract_id::text,
                   asset_version_id::text,
                   product_id::text,
                   sku_id::text,
                   contract_name,
                   version_no,
                   contract_scope,
                   business_terms_json,
                   structure_terms_json,
                   quality_terms_json,
                   compliance_terms_json,
                   delivery_terms_json,
                   version_terms_json,
                   acceptance_terms_json,
                   rights_terms_json,
                   responsibility_terms_json,
                   processing_terms_json,
                   content_digest,
                   status,
                   to_char(effective_from AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(effective_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   metadata,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')",
                &[
                    &payload.asset_version_id,
                    &payload.product_id,
                    &sku_id,
                    &payload.contract_name,
                    &payload.version_no.unwrap_or(1),
                    &payload
                        .contract_scope
                        .clone()
                        .unwrap_or_else(|| "sku".to_string()),
                    &normalize_object_json(&payload.business_terms_json),
                    &normalize_object_json(&payload.structure_terms_json),
                    &normalize_object_json(&payload.quality_terms_json),
                    &normalize_object_json(&payload.compliance_terms_json),
                    &normalize_object_json(&payload.delivery_terms_json),
                    &normalize_object_json(&payload.version_terms_json),
                    &normalize_object_json(&payload.acceptance_terms_json),
                    &normalize_object_json(&payload.rights_terms_json),
                    &normalize_object_json(&payload.responsibility_terms_json),
                    &normalize_object_json(&payload.processing_terms_json),
                    &payload.content_digest,
                    &payload
                        .status
                        .clone()
                        .unwrap_or_else(|| "draft".to_string()),
                    &payload.effective_from,
                    &payload.effective_to,
                    &normalize_object_json(&payload.metadata),
                ],
            )
            .await?;
        Ok(parse_data_contract_row(&row))
    }

    pub async fn get_data_contract(
        client: &impl GenericClient,
        sku_id: &str,
        data_contract_id: &str,
    ) -> Result<Option<DataContractView>, Error> {
        let row = client
            .query_opt(
                "SELECT
                   data_contract_id::text,
                   asset_version_id::text,
                   product_id::text,
                   sku_id::text,
                   contract_name,
                   version_no,
                   contract_scope,
                   business_terms_json,
                   structure_terms_json,
                   quality_terms_json,
                   compliance_terms_json,
                   delivery_terms_json,
                   version_terms_json,
                   acceptance_terms_json,
                   rights_terms_json,
                   responsibility_terms_json,
                   processing_terms_json,
                   content_digest,
                   status,
                   to_char(effective_from AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(effective_to AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   metadata,
                   to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),
                   to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')
                 FROM contract.data_contract
                 WHERE sku_id = $1::text::uuid
                   AND data_contract_id = $2::text::uuid",
                &[&sku_id, &data_contract_id],
            )
            .await?;
        Ok(row.map(|row| parse_data_contract_row(&row)))
    }

    pub async fn patch_asset_release_policy(
        client: &impl GenericClient,
        asset_id: &str,
        payload: &PatchAssetReleasePolicyRequest,
    ) -> Result<Option<AssetReleasePolicyView>, Error> {
        let updated_count_row = client
            .query_one(
                "WITH updated AS (
                   UPDATE catalog.asset_version
                      SET release_mode = COALESCE($2, release_mode),
                          is_revision_subscribable = COALESCE($3, is_revision_subscribable),
                          update_frequency = CASE
                            WHEN $4::text IS NULL THEN update_frequency
                            ELSE $4
                          END,
                          release_notes_json = CASE
                            WHEN $5::jsonb = '{}'::jsonb THEN release_notes_json
                            ELSE $5::jsonb
                          END,
                          updated_at = now()
                    WHERE asset_id = $1::text::uuid
                    RETURNING 1
                 )
                 SELECT count(*)::bigint FROM updated",
                &[
                    &asset_id,
                    &payload.release_mode,
                    &payload.is_revision_subscribable,
                    &payload.update_frequency,
                    &normalize_object_json(&payload.release_notes_json),
                ],
            )
            .await?;
        let updated_count: i64 = updated_count_row.get(0);
        if updated_count == 0 {
            return Ok(None);
        }
        let latest_row = client
            .query_one(
                "SELECT
                   asset_version_id::text,
                   version_no,
                   release_mode,
                   is_revision_subscribable,
                   update_frequency,
                   release_notes_json
                 FROM catalog.asset_version
                 WHERE asset_id = $1::text::uuid
                 ORDER BY version_no DESC
                 LIMIT 1",
                &[&asset_id],
            )
            .await?;

        Ok(Some(AssetReleasePolicyView {
            asset_id: asset_id.to_string(),
            release_mode: latest_row.get(2),
            is_revision_subscribable: latest_row.get(3),
            update_frequency: latest_row.get(4),
            release_notes_json: latest_row.get(5),
            applied_version_count: updated_count,
            latest_asset_version_id: latest_row.get(0),
            latest_version_no: latest_row.get(1),
        }))
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

fn compose_product_metadata(
    metadata: &Value,
    subtitle: Option<&str>,
    industry: Option<&str>,
    use_cases: Option<&Vec<String>>,
    data_classification: Option<&str>,
    quality_score: Option<&str>,
) -> Value {
    let mut merged = normalize_object_json(metadata);
    if let Some(subtitle) = subtitle {
        merged["subtitle"] = Value::String(subtitle.to_string());
    }
    if let Some(industry) = industry {
        merged["industry"] = Value::String(industry.to_string());
    }
    if let Some(use_cases) = use_cases
        && !use_cases.is_empty()
    {
        merged["use_cases"] = json!(use_cases);
    }
    if let Some(data_classification) = data_classification {
        merged["data_classification"] = Value::String(data_classification.to_string());
    }
    if let Some(quality_score) = quality_score {
        merged["quality_score"] = Value::String(quality_score.to_string());
    }
    merged
}

fn parse_usage_policy_row(row: &Row) -> UsagePolicyView {
    UsagePolicyView {
        policy_id: row.get(0),
        owner_org_id: row.get(1),
        policy_name: row.get(2),
        stage_from: row.get(3),
        subject_constraints: row.get(4),
        usage_constraints: row.get(5),
        time_constraints: row.get(6),
        region_constraints: row.get(7),
        output_constraints: row.get(8),
        exportable: row.get(9),
        status: row.get(10),
        created_at: row.get(11),
        updated_at: row.get(12),
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

fn parse_data_contract_row(row: &Row) -> DataContractView {
    DataContractView {
        data_contract_id: row.get(0),
        asset_version_id: row.get(1),
        product_id: row.get(2),
        sku_id: row.get(3),
        contract_name: row.get(4),
        version_no: row.get(5),
        contract_scope: row.get(6),
        business_terms_json: row.get(7),
        structure_terms_json: row.get(8),
        quality_terms_json: row.get(9),
        compliance_terms_json: row.get(10),
        delivery_terms_json: row.get(11),
        version_terms_json: row.get(12),
        acceptance_terms_json: row.get(13),
        rights_terms_json: row.get(14),
        responsibility_terms_json: row.get(15),
        processing_terms_json: row.get(16),
        content_digest: row.get(17),
        status: row.get(18),
        effective_from: row.get(19),
        effective_to: row.get(20),
        metadata: row.get(21),
        created_at: row.get(22),
        updated_at: row.get(23),
    }
}

fn parse_review_decision_row(row: &Row) -> ReviewDecisionView {
    ReviewDecisionView {
        review_task_id: row.get(0),
        review_type: row.get(1),
        ref_type: row.get(2),
        ref_id: row.get(3),
        status: row.get(4),
    }
}

fn parse_asset_object_row(object_row: &Row, storage_row: &Row) -> AssetObjectView {
    AssetObjectView {
        asset_object_id: object_row.get(0),
        asset_version_id: object_row.get(1),
        object_kind: object_row.get(2),
        object_name: object_row.get(3),
        object_locator: object_row.get(4),
        share_protocol: object_row.get(5),
        schema_json: object_row.get(6),
        output_schema_json: object_row.get(7),
        freshness_json: object_row.get(8),
        access_constraints: object_row.get(9),
        object_metadata: object_row.get(10),
        asset_storage_binding_id: storage_row.get(0),
        object_uri: storage_row.get(1),
        storage_type: storage_row.get(2),
        storage_zone: storage_row.get(3),
        access_path_type: storage_row.get(4),
        object_hash: storage_row.get(5),
        encryption_algo: storage_row.get(6),
        worm_enabled: storage_row.get(7),
        storage_metadata: storage_row.get(8),
        created_at: object_row.get(11),
        updated_at: object_row.get(12),
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

fn parse_format_detection_result_row(row: &Row) -> FormatDetectionResultView {
    FormatDetectionResultView {
        format_detection_result_id: row.get(0),
        raw_object_manifest_id: row.get(1),
        detected_object_family: row.get(2),
        detected_format: row.get(3),
        schema_hint_json: row.get(4),
        recommended_processing_path: row.get(5),
        classification_confidence: row.get(6),
        detected_at: row.get(7),
        status: row.get(8),
        created_at: row.get(9),
    }
}

fn parse_extraction_job_row(row: &Row) -> ExtractionJobView {
    ExtractionJobView {
        extraction_job_id: row.get(0),
        raw_object_manifest_id: row.get(1),
        asset_version_id: row.get(2),
        job_type: row.get(3),
        job_config_json: row.get(4),
        result_summary_json: row.get(5),
        output_uri: row.get(6),
        output_hash: row.get(7),
        status: row.get(8),
        started_at: row.get(9),
        completed_at: row.get(10),
        created_at: row.get(11),
        updated_at: row.get(12),
    }
}

fn parse_preview_artifact_row(row: &Row) -> PreviewArtifactView {
    PreviewArtifactView {
        preview_artifact_id: row.get(0),
        asset_version_id: row.get(1),
        raw_object_manifest_id: row.get(2),
        preview_type: row.get(3),
        preview_uri: row.get(4),
        preview_hash: row.get(5),
        preview_payload: row.get(6),
        preview_policy_json: row.get(7),
        status: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    }
}

fn parse_product_metadata_profile_row(row: &Row) -> ProductMetadataProfileView {
    ProductMetadataProfileView {
        product_metadata_profile_id: row.get(0),
        product_id: row.get(1),
        asset_version_id: row.get(2),
        metadata_version_no: row.get(3),
        business_description_json: row.get(4),
        data_content_json: row.get(5),
        structure_description_json: row.get(6),
        quality_description_json: row.get(7),
        compliance_description_json: row.get(8),
        delivery_description_json: row.get(9),
        version_description_json: row.get(10),
        authorization_description_json: row.get(11),
        responsibility_description_json: row.get(12),
        processing_overview_json: row.get(13),
        status: row.get(14),
        metadata: row.get(15),
        created_at: row.get(16),
        updated_at: row.get(17),
    }
}

fn normalize_json_object(value: &Value) -> Value {
    if value.is_object() {
        value.clone()
    } else {
        json!({})
    }
}

fn normalize_json_array(value: &Value) -> Value {
    if value.is_array() {
        value.clone()
    } else {
        json!([])
    }
}

fn parse_asset_field_definition_row(row: &Row) -> AssetFieldDefinitionView {
    AssetFieldDefinitionView {
        field_definition_id: row.get(0),
        asset_version_id: row.get(1),
        object_name: row.get(2),
        field_name: row.get(3),
        field_path: row.get(4),
        field_type: row.get(5),
        is_nullable: row.get(6),
        is_primary_key: row.get(7),
        is_partition_key: row.get(8),
        is_time_field: row.get(9),
        code_rule: row.get(10),
        unit_text: row.get(11),
        enum_values_json: row.get(12),
        description: row.get(13),
        created_at: row.get(14),
        updated_at: row.get(15),
    }
}

fn normalize_object_json(value: &Value) -> Value {
    if value.is_object() {
        return value.clone();
    }
    json!({})
}

fn parse_asset_quality_report_row(row: &Row) -> AssetQualityReportView {
    AssetQualityReportView {
        quality_report_id: row.get(0),
        asset_version_id: row.get(1),
        report_no: row.get(2),
        report_type: row.get(3),
        coverage_range_json: row.get(4),
        freshness_json: row.get(5),
        missing_rate: row.get(6),
        duplicate_rate: row.get(7),
        anomaly_rate: row.get(8),
        sampling_method: row.get(9),
        assessed_at: row.get(10),
        assessor_org_id: row.get(11),
        report_uri: row.get(12),
        report_hash: row.get(13),
        metrics_json: row.get(14),
        status: row.get(15),
        metadata: row.get(16),
        created_at: row.get(17),
        updated_at: row.get(18),
    }
}

fn parse_asset_processing_input_row(row: &Row) -> AssetProcessingInputView {
    AssetProcessingInputView {
        processing_input_id: row.get(0),
        input_asset_version_id: row.get(1),
        input_role: row.get(2),
        created_at: row.get(3),
    }
}

fn parse_asset_processing_job_row(
    row: &Row,
    input_sources: Vec<AssetProcessingInputView>,
) -> AssetProcessingJobView {
    let metadata: Value = row.get(16);
    let processing_summary_json = metadata
        .get("processing_summary_json")
        .cloned()
        .unwrap_or_else(|| json!({}));
    AssetProcessingJobView {
        processing_job_id: row.get(0),
        output_asset_version_id: row.get(1),
        processing_mode: row.get(2),
        processor_org_id: row.get(3),
        executor_type: row.get(4),
        job_name: row.get(5),
        transform_spec_version: row.get(6),
        desensitization_profile: row.get(7),
        standardization_profile: row.get(8),
        labeling_profile: row.get(9),
        model_artifact_ref: row.get(10),
        evidence_uri: row.get(11),
        evidence_hash: row.get(12),
        started_at: row.get(13),
        completed_at: row.get(14),
        input_sources,
        processing_summary_json,
        status: row.get(15),
        metadata,
        created_at: row.get(17),
        updated_at: row.get(18),
    }
}
