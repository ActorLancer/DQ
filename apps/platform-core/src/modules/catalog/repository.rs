use crate::modules::catalog::domain::{
    AssetFieldDefinitionView, AssetQualityReportView, AssetVersionView,
    CreateAssetFieldDefinitionRequest, CreateAssetQualityReportRequest, CreateAssetVersionRequest,
    CreateDataProductRequest, CreateDataResourceRequest, CreateExtractionJobRequest,
    CreateFormatDetectionRequest, CreatePreviewArtifactRequest, CreateProductSkuRequest,
    CreateRawIngestBatchRequest, CreateRawObjectManifestRequest, DataProductView, DataResourceView,
    ExtractionJobView, FormatDetectionResultView, PatchDataProductRequest, PatchProductSkuRequest,
    PreviewArtifactView, ProductMetadataProfileView, ProductSkuView,
    PutProductMetadataProfileRequest, RawIngestBatchView, RawObjectManifestView,
};
use serde_json::{Value, json};
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

    pub async fn get_raw_object_manifest(
        client: &impl GenericClient,
        id: &str,
    ) -> Result<Option<RawObjectManifestView>, tokio_postgres::Error> {
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
    ) -> Result<FormatDetectionResultView, tokio_postgres::Error> {
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
    ) -> Result<ExtractionJobView, tokio_postgres::Error> {
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
    ) -> Result<PreviewArtifactView, tokio_postgres::Error> {
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

    pub async fn upsert_product_metadata_profile(
        client: &impl GenericClient,
        product_id: &str,
        asset_version_id: &str,
        payload: &PutProductMetadataProfileRequest,
    ) -> Result<ProductMetadataProfileView, tokio_postgres::Error> {
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
    ) -> Result<AssetFieldDefinitionView, tokio_postgres::Error> {
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
    ) -> Result<AssetQualityReportView, tokio_postgres::Error> {
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
