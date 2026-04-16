# V1 数据库表字典（自动导出）

- 导出时间（UTC）：2026-04-16 15:01:21
- 数据库：`luna_data_trading`
- 来源：PostgreSQL 系统目录（`pg_catalog`）
- 范围：`core/iam/authz/catalog/contract/trade/delivery/payment/billing/risk/audit/search/ops/support/developer/chain`

## Schema: `core`

- 对象职责：身份主体、组织、账号、连接器与执行环境
- 表数量：8

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `application` | `app_id` | `application_client_id_key(client_id)` | `org_id->core.organization(org_id)` | `application_client_id_key; application_pkey; idx_application_org_id` | 身份主体、组织、账号、连接器与执行环境 |
| `connector` | `connector_id` | `-` | `org_id->core.organization(org_id)` | `connector_pkey` | 身份主体、组织、账号、连接器与执行环境 |
| `department` | `department_id` | `-` | `org_id->core.organization(org_id); parent_department_id->core.department(department_id)` | `department_pkey; idx_department_org_id` | 身份主体、组织、账号、连接器与执行环境 |
| `did_binding` | `did_id` | `-` | `org_id->core.organization(org_id); user_id->core.user_account(user_id)` | `did_binding_pkey` | 身份主体、组织、账号、连接器与执行环境 |
| `execution_environment` | `environment_id` | `-` | `connector_id->core.connector(connector_id); org_id->core.organization(org_id)` | `execution_environment_pkey` | 身份主体、组织、账号、连接器与执行环境 |
| `organization` | `org_id` | `-` | `-` | `organization_pkey` | 身份主体、组织、账号、连接器与执行环境 |
| `service_identity` | `service_identity_id` | `service_identity_service_name_key(service_name)` | `-` | `service_identity_pkey; service_identity_service_name_key` | 身份主体、组织、账号、连接器与执行环境 |
| `user_account` | `user_id` | `user_account_login_id_key(login_id)` | `department_id->core.department(department_id); org_id->core.organization(org_id)` | `idx_user_account_org_id; user_account_login_id_key; user_account_pkey` | 身份主体、组织、账号、连接器与执行环境 |

## Schema: `iam`

- 对象职责：认证、会话、设备、证书、SSO 与身份凭证
- 表数量：14

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `auth_method_binding` | `auth_method_binding_id` | `-` | `-` | `auth_method_binding_pkey; idx_auth_method_binding_user_id` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `certificate_record` | `certificate_id` | `certificate_record_certificate_digest_key(certificate_digest); uq_certificate_serial_issuer(serial_number, issuer_dn)` | `fabric_ca_registry_id->iam.fabric_ca_registry(fabric_ca_registry_id)` | `certificate_record_certificate_digest_key; certificate_record_pkey; idx_certificate_record_registry_status; uq_certificate_serial_issuer` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `certificate_revocation_record` | `certificate_revocation_id` | `certificate_revocation_record_certificate_id_key(certificate_id)` | `certificate_id->iam.certificate_record(certificate_id)` | `certificate_revocation_record_certificate_id_key; certificate_revocation_record_pkey` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `external_identity_binding` | `external_identity_binding_id` | `uq_external_identity_binding_subject(sso_connection_id, external_subject)` | `sso_connection_id->iam.sso_connection(sso_connection_id)` | `external_identity_binding_pkey; idx_external_identity_binding_user_id; uq_external_identity_binding_subject` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `fabric_ca_registry` | `fabric_ca_registry_id` | `uq_fabric_ca_registry_name(msp_id, registry_name)` | `-` | `fabric_ca_registry_pkey; idx_fabric_ca_registry_org_id; uq_fabric_ca_registry_name` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `fabric_identity_binding` | `fabric_identity_binding_id` | `fabric_identity_binding_certificate_id_key(certificate_id); uq_fabric_identity_enrollment(fabric_ca_registry_id, enrollment_id)` | `certificate_id->iam.certificate_record(certificate_id); fabric_ca_registry_id->iam.fabric_ca_registry(fabric_ca_registry_id)` | `fabric_identity_binding_certificate_id_key; fabric_identity_binding_pkey; idx_fabric_identity_binding_service_id; idx_fabric_identity_binding_user_id; uq_fabric_identity_enrollment` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `identity_proof` | `identity_proof_id` | `-` | `-` | `identity_proof_pkey; idx_identity_proof_subject` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `invitation` | `invitation_id` | `invitation_token_hash_key(token_hash)` | `-` | `idx_invitation_org_id; invitation_pkey; invitation_token_hash_key` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `mfa_authenticator` | `mfa_authenticator_id` | `-` | `-` | `mfa_authenticator_pkey; uq_mfa_authenticator_credential` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `refresh_token_family` | `refresh_token_family_id` | `refresh_token_family_current_token_hash_key(current_token_hash)` | `trusted_device_id->iam.trusted_device(trusted_device_id)` | `idx_refresh_token_family_user_id; refresh_token_family_current_token_hash_key; refresh_token_family_pkey` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `sso_connection` | `sso_connection_id` | `uq_sso_connection_org_name(org_id, connection_name)` | `-` | `idx_sso_connection_org_id; sso_connection_pkey; uq_sso_connection_org_name` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `step_up_challenge` | `step_up_challenge_id` | `-` | `-` | `idx_step_up_challenge_user_status; step_up_challenge_pkey` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `trusted_device` | `trusted_device_id` | `-` | `-` | `trusted_device_pkey; uq_trusted_device_user_fingerprint` | 认证、会话、设备、证书、SSO 与身份凭证 |
| `user_session` | `session_id` | `-` | `refresh_token_family_id->iam.refresh_token_family(refresh_token_family_id); trusted_device_id->iam.trusted_device(trusted_device_id)` | `idx_user_session_user_status; user_session_pkey` | 认证、会话、设备、证书、SSO 与身份凭证 |

## Schema: `authz`

- 对象职责：角色、权限、主体授权绑定
- 表数量：4

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `permission_definition` | `permission_code` | `-` | `-` | `permission_definition_pkey` | 角色、权限、主体授权绑定 |
| `role_definition` | `role_key` | `-` | `-` | `role_definition_pkey` | 角色、权限、主体授权绑定 |
| `role_permission` | `role_key, permission_code` | `-` | `permission_code->authz.permission_definition(permission_code); role_key->authz.role_definition(role_key)` | `role_permission_pkey` | 角色、权限、主体授权绑定 |
| `subject_role_binding` | `subject_role_binding_id` | `-` | `role_key->authz.role_definition(role_key)` | `idx_subject_role_binding_subject; subject_role_binding_pkey` | 角色、权限、主体授权绑定 |

## Schema: `catalog`

- 对象职责：数据资产、版本、商品、SKU、标签与元信息
- 表数量：28

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `asset_custody_profile` | `custody_profile_id` | `asset_custody_profile_asset_version_id_key(asset_version_id)` | `asset_id->catalog.data_asset(asset_id); asset_version_id->catalog.asset_version(asset_version_id)` | `asset_custody_profile_asset_version_id_key; asset_custody_profile_pkey; idx_asset_custody_profile_asset_id` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_field_definition` | `field_definition_id` | `asset_field_definition_asset_version_id_field_path_key(asset_version_id, field_path)` | `asset_version_id->catalog.asset_version(asset_version_id)` | `asset_field_definition_asset_version_id_field_path_key; asset_field_definition_pkey; idx_asset_field_definition_version` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_object_binding` | `asset_object_id` | `asset_object_binding_asset_version_id_object_kind_object_na_key(asset_version_id, object_kind, object_name)` | `asset_version_id->catalog.asset_version(asset_version_id)` | `asset_object_binding_asset_version_id_object_kind_object_na_key; asset_object_binding_pkey; idx_asset_object_binding_version` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_processing_input` | `processing_input_id` | `asset_processing_input_processing_job_id_input_asset_versio_key(processing_job_id, input_asset_version_id, input_role)` | `input_asset_version_id->catalog.asset_version(asset_version_id); processing_job_id->catalog.asset_processing_job(processing_job_id)` | `asset_processing_input_pkey; asset_processing_input_processing_job_id_input_asset_versio_key; idx_asset_processing_input_job` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_processing_job` | `processing_job_id` | `-` | `output_asset_version_id->catalog.asset_version(asset_version_id)` | `asset_processing_job_pkey; idx_asset_processing_job_output` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_quality_report` | `quality_report_id` | `asset_quality_report_asset_version_id_report_no_key(asset_version_id, report_no)` | `asset_version_id->catalog.asset_version(asset_version_id)` | `asset_quality_report_asset_version_id_report_no_key; asset_quality_report_pkey; idx_asset_quality_report_version` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_sample` | `asset_sample_id` | `-` | `asset_version_id->catalog.asset_version(asset_version_id)` | `asset_sample_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_storage_binding` | `asset_storage_binding_id` | `-` | `asset_version_id->catalog.asset_version(asset_version_id); storage_namespace_id->catalog.storage_namespace(storage_namespace_id)` | `asset_storage_binding_pkey; idx_asset_storage_binding_asset_version_id; idx_asset_storage_binding_namespace` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_structured_dataset` | `dataset_id` | `-` | `asset_version_id->catalog.asset_version(asset_version_id)` | `asset_structured_dataset_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_structured_row` | `row_id` | `asset_structured_row_dataset_id_row_no_key(dataset_id, row_no)` | `dataset_id->catalog.asset_structured_dataset(dataset_id)` | `asset_structured_row_dataset_id_row_no_key; asset_structured_row_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_trust_evidence` | `trust_evidence_id` | `-` | `asset_id->catalog.data_asset(asset_id); asset_version_id->catalog.asset_version(asset_version_id)` | `asset_trust_evidence_pkey; idx_asset_trust_evidence_asset_id` | 数据资产、版本、商品、SKU、标签与元信息 |
| `asset_version` | `asset_version_id` | `asset_version_asset_id_version_no_key(asset_id, version_no)` | `asset_id->catalog.data_asset(asset_id); storage_policy_id->catalog.storage_policy_profile(storage_policy_id)` | `asset_version_asset_id_version_no_key; asset_version_pkey; idx_asset_version_asset_id; idx_asset_version_storage_policy` | 数据资产、版本、商品、SKU、标签与元信息 |
| `data_asset` | `asset_id` | `-` | `-` | `data_asset_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `extraction_job` | `extraction_job_id` | `-` | `asset_version_id->catalog.asset_version(asset_version_id); raw_object_manifest_id->catalog.raw_object_manifest(raw_object_manifest_id)` | `extraction_job_pkey; idx_extraction_job_manifest; idx_extraction_job_version` | 数据资产、版本、商品、SKU、标签与元信息 |
| `format_detection_result` | `format_detection_result_id` | `-` | `raw_object_manifest_id->catalog.raw_object_manifest(raw_object_manifest_id)` | `format_detection_result_pkey; idx_format_detection_manifest` | 数据资产、版本、商品、SKU、标签与元信息 |
| `preview_artifact` | `preview_artifact_id` | `-` | `asset_version_id->catalog.asset_version(asset_version_id); raw_object_manifest_id->catalog.raw_object_manifest(raw_object_manifest_id)` | `idx_preview_artifact_version; preview_artifact_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `product` | `product_id` | `-` | `asset_id->catalog.data_asset(asset_id); asset_version_id->catalog.asset_version(asset_version_id)` | `idx_product_asset_version_id; idx_product_seller_org_id; idx_product_status_type; product_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `product_metadata_profile` | `product_metadata_profile_id` | `product_metadata_profile_product_id_metadata_version_no_key(product_id, metadata_version_no)` | `asset_version_id->catalog.asset_version(asset_version_id); product_id->catalog.product(product_id)` | `idx_product_metadata_profile_product; product_metadata_profile_pkey; product_metadata_profile_product_id_metadata_version_no_key` | 数据资产、版本、商品、SKU、标签与元信息 |
| `product_sku` | `sku_id` | `product_sku_product_id_sku_code_key(product_id, sku_code)` | `product_id->catalog.product(product_id)` | `idx_product_sku_product_id; product_sku_pkey; product_sku_product_id_sku_code_key` | 数据资产、版本、商品、SKU、标签与元信息 |
| `product_tag` | `product_id, tag_id` | `-` | `product_id->catalog.product(product_id); tag_id->catalog.tag(tag_id)` | `idx_product_tag_tag_id; product_tag_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `query_surface_definition` | `query_surface_id` | `-` | `asset_object_id->catalog.asset_object_binding(asset_object_id); asset_version_id->catalog.asset_version(asset_version_id)` | `idx_query_surface_asset_version; query_surface_definition_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `raw_ingest_batch` | `raw_ingest_batch_id` | `-` | `asset_id->catalog.data_asset(asset_id)` | `idx_raw_ingest_batch_asset; idx_raw_ingest_batch_owner_status; raw_ingest_batch_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `raw_object_manifest` | `raw_object_manifest_id` | `-` | `raw_ingest_batch_id->catalog.raw_ingest_batch(raw_ingest_batch_id); storage_binding_id->catalog.asset_storage_binding(asset_storage_binding_id)` | `idx_raw_object_manifest_batch; idx_raw_object_manifest_hash; raw_object_manifest_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `safe_preview_artifact` | `safe_preview_artifact_id` | `-` | `asset_version_id->catalog.asset_version(asset_version_id); product_id->catalog.product(product_id)` | `idx_safe_preview_artifact_asset_version; safe_preview_artifact_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `sensitive_handling_policy` | `sensitive_policy_id` | `sensitive_handling_policy_asset_version_id_key(asset_version_id)` | `asset_version_id->catalog.asset_version(asset_version_id); product_id->catalog.product(product_id)` | `idx_sensitive_handling_policy_asset_version; sensitive_handling_policy_asset_version_id_key; sensitive_handling_policy_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `storage_namespace` | `storage_namespace_id` | `storage_namespace_namespace_name_key(namespace_name)` | `-` | `idx_storage_namespace_owner_kind; storage_namespace_namespace_name_key; storage_namespace_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `storage_policy_profile` | `storage_policy_id` | `storage_policy_profile_owner_org_id_policy_name_key(owner_org_id, policy_name)` | `archive_namespace_id->catalog.storage_namespace(storage_namespace_id); curated_namespace_id->catalog.storage_namespace(storage_namespace_id); delivery_namespace_id->catalog.storage_namespace(storage_namespace_id); evidence_namespace_id->catalog.storage_namespace(storage_namespace_id); model_namespace_id->catalog.storage_namespace(storage_namespace_id); preview_namespace_id->catalog.storage_namespace(storage_namespace_id); product_namespace_id->catalog.storage_namespace(storage_namespace_id); raw_namespace_id->catalog.storage_namespace(storage_namespace_id)` | `idx_storage_policy_profile_owner; storage_policy_profile_owner_org_id_policy_name_key; storage_policy_profile_pkey` | 数据资产、版本、商品、SKU、标签与元信息 |
| `tag` | `tag_id` | `tag_tag_name_key(tag_name)` | `parent_tag_id->catalog.tag(tag_id)` | `idx_catalog_tag_group_status; tag_pkey; tag_tag_name_key; uq_catalog_tag_code` | 数据资产、版本、商品、SKU、标签与元信息 |

## Schema: `contract`

- 对象职责：模板、策略、数字合同与法律依据
- 表数量：8

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `contract_signer` | `contract_signer_id` | `-` | `contract_id->contract.digital_contract(contract_id)` | `contract_signer_pkey` | 模板、策略、数字合同与法律依据 |
| `data_contract` | `data_contract_id` | `-` | `-` | `data_contract_pkey; idx_data_contract_product; idx_data_contract_sku` | 模板、策略、数字合同与法律依据 |
| `digital_contract` | `contract_id` | `digital_contract_order_id_key(order_id)` | `contract_template_id->contract.template_definition(template_id); data_contract_id->contract.data_contract(data_contract_id)` | `digital_contract_order_id_key; digital_contract_pkey` | 模板、策略、数字合同与法律依据 |
| `legal_basis_evidence` | `legal_basis_evidence_id` | `-` | `-` | `idx_legal_basis_evidence_asset_version; legal_basis_evidence_pkey` | 模板、策略、数字合同与法律依据 |
| `policy_binding` | `policy_binding_id` | `-` | `policy_id->contract.usage_policy(policy_id)` | `policy_binding_pkey` | 模板、策略、数字合同与法律依据 |
| `template_binding` | `template_binding_id` | `template_binding_sku_id_template_id_binding_type_key(sku_id, template_id, binding_type)` | `template_id->contract.template_definition(template_id)` | `template_binding_pkey; template_binding_sku_id_template_id_binding_type_key` | 模板、策略、数字合同与法律依据 |
| `template_definition` | `template_id` | `-` | `-` | `template_definition_pkey` | 模板、策略、数字合同与法律依据 |
| `usage_policy` | `policy_id` | `-` | `-` | `usage_policy_pkey` | 模板、策略、数字合同与法律依据 |

## Schema: `trade`

- 对象职责：询单、订单、授权主链路
- 表数量：5

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `authorization_grant` | `authorization_grant_id` | `-` | `order_id->trade.order_main(order_id)` | `authorization_grant_pkey; idx_authorization_grant_order_id` | 询单、订单、授权主链路 |
| `inquiry` | `inquiry_id` | `-` | `-` | `inquiry_pkey` | 询单、订单、授权主链路 |
| `order_line` | `order_line_id` | `-` | `order_id->trade.order_main(order_id)` | `idx_order_line_order_id; order_line_pkey` | 询单、订单、授权主链路 |
| `order_main` | `order_id` | `order_main_idempotency_key_key(idempotency_key)` | `inquiry_id->trade.inquiry(inquiry_id)` | `idx_order_main_buyer_org_id; idx_order_main_reconcile; idx_order_main_seller_org_id; idx_order_main_status_created_at; order_main_idempotency_key_key; order_main_pkey` | 询单、订单、授权主链路 |
| `order_status_history` | `order_status_history_id` | `-` | `order_id->trade.order_main(order_id)` | `order_status_history_pkey` | 询单、订单、授权主链路 |

## Schema: `delivery`

- 对象职责：交付对象、票据、API、沙箱、报告、查询执行面
- 表数量：19

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `api_credential` | `api_credential_id` | `-` | `-` | `api_credential_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `api_usage_log` | `api_usage_log_id` | `-` | `api_credential_id->delivery.api_credential(api_credential_id)` | `api_usage_log_pkey; idx_api_usage_log_order_id` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `attestation_record` | `attestation_record_id` | `-` | `query_run_id->delivery.query_execution_run(query_run_id); sandbox_session_id->delivery.sandbox_session(sandbox_session_id)` | `attestation_record_pkey; idx_attestation_record_order` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `data_share_grant` | `data_share_grant_id` | `-` | `-` | `data_share_grant_pkey; idx_data_share_grant_order` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `delivery_receipt` | `receipt_id` | `-` | `delivery_id->delivery.delivery_record(delivery_id)` | `delivery_receipt_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `delivery_record` | `delivery_id` | `-` | `object_id->delivery.storage_object(object_id); envelope_id->delivery.key_envelope(envelope_id)` | `delivery_record_pkey; idx_delivery_record_order_id` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `delivery_ticket` | `ticket_id` | `-` | `-` | `delivery_ticket_pkey; idx_delivery_ticket_order_id` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `destruction_attestation` | `destruction_attestation_id` | `-` | `object_id->delivery.storage_object(object_id)` | `destruction_attestation_pkey; idx_destruction_attestation_order` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `key_envelope` | `envelope_id` | `-` | `-` | `key_envelope_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `query_execution_run` | `query_run_id` | `-` | `query_template_id->delivery.query_template_definition(query_template_id); result_object_id->delivery.storage_object(object_id); sandbox_session_id->delivery.sandbox_session(sandbox_session_id); template_query_grant_id->delivery.template_query_grant(template_query_grant_id)` | `idx_query_execution_run_order; idx_query_execution_run_surface; query_execution_run_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `query_template_definition` | `query_template_id` | `query_template_definition_query_surface_id_template_name_ve_key(query_surface_id, template_name, version_no)` | `-` | `idx_query_template_surface; query_template_definition_pkey; query_template_definition_query_surface_id_template_name_ve_key` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `report_artifact` | `report_artifact_id` | `-` | `object_id->delivery.storage_object(object_id)` | `idx_report_artifact_order_id; report_artifact_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `result_disclosure_review` | `result_disclosure_review_id` | `-` | `query_run_id->delivery.query_execution_run(query_run_id); report_artifact_id->delivery.report_artifact(report_artifact_id); result_object_id->delivery.storage_object(object_id)` | `idx_result_disclosure_review_order; result_disclosure_review_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `revision_subscription` | `revision_subscription_id` | `revision_subscription_order_id_key(order_id)` | `-` | `idx_revision_subscription_asset; idx_revision_subscription_order; revision_subscription_order_id_key; revision_subscription_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `sandbox_session` | `sandbox_session_id` | `-` | `sandbox_workspace_id->delivery.sandbox_workspace(sandbox_workspace_id)` | `sandbox_session_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `sandbox_workspace` | `sandbox_workspace_id` | `-` | `-` | `idx_sandbox_workspace_order_id; idx_sandbox_workspace_surface; sandbox_workspace_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `sensitive_execution_policy` | `sensitive_execution_policy_id` | `-` | `sandbox_workspace_id->delivery.sandbox_workspace(sandbox_workspace_id); template_query_grant_id->delivery.template_query_grant(template_query_grant_id)` | `idx_sensitive_execution_policy_order; sensitive_execution_policy_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `storage_object` | `object_id` | `-` | `-` | `idx_storage_object_namespace; storage_object_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |
| `template_query_grant` | `template_query_grant_id` | `-` | `sandbox_workspace_id->delivery.sandbox_workspace(sandbox_workspace_id)` | `idx_template_query_grant_order; idx_template_query_grant_surface; template_query_grant_pkey` | 交付对象、票据、API、沙箱、报告、查询执行面 |

## Schema: `payment`

- 对象职责：支付渠道、意图、webhook、对账与提现
- 表数量：12

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `corridor_policy` | `corridor_policy_id` | `corridor_policy_payer_jurisdiction_code_payee_jurisdiction__key(payer_jurisdiction_code, payee_jurisdiction_code, product_scope, price_currency_code, effective_from); corridor_policy_policy_name_key(policy_name)` | `payee_jurisdiction_code->payment.jurisdiction_profile(jurisdiction_code); payer_jurisdiction_code->payment.jurisdiction_profile(jurisdiction_code)` | `corridor_policy_payer_jurisdiction_code_payee_jurisdiction__key; corridor_policy_pkey; corridor_policy_policy_name_key; idx_corridor_policy_pair_status` | 支付渠道、意图、webhook、对账与提现 |
| `jurisdiction_profile` | `jurisdiction_code` | `-` | `-` | `jurisdiction_profile_pkey` | 支付渠道、意图、webhook、对账与提现 |
| `payment_intent` | `payment_intent_id` | `payment_intent_idempotency_key_key(idempotency_key)` | `corridor_policy_id->payment.corridor_policy(corridor_policy_id); launch_jurisdiction_code->payment.jurisdiction_profile(jurisdiction_code); payee_jurisdiction_code->payment.jurisdiction_profile(jurisdiction_code); payer_jurisdiction_code->payment.jurisdiction_profile(jurisdiction_code); provider_account_id->payment.provider_account(provider_account_id); provider_key->payment.provider(provider_key)` | `idx_payment_intent_corridor_policy_id; idx_payment_intent_order_id; idx_payment_intent_reconcile; idx_payment_intent_status; payment_intent_idempotency_key_key; payment_intent_pkey` | 支付渠道、意图、webhook、对账与提现 |
| `payment_transaction` | `payment_transaction_id` | `-` | `payment_intent_id->payment.payment_intent(payment_intent_id)` | `idx_payment_transaction_intent_id; payment_transaction_pkey` | 支付渠道、意图、webhook、对账与提现 |
| `payment_webhook_event` | `webhook_event_id` | `payment_webhook_event_provider_key_provider_event_id_key(provider_key, provider_event_id)` | `payment_intent_id->payment.payment_intent(payment_intent_id); payment_transaction_id->payment.payment_transaction(payment_transaction_id); provider_key->payment.provider(provider_key)` | `payment_webhook_event_pkey; payment_webhook_event_provider_key_provider_event_id_key` | 支付渠道、意图、webhook、对账与提现 |
| `payout_instruction` | `payout_instruction_id` | `payout_instruction_idempotency_key_key(idempotency_key)` | `destination_jurisdiction_code->payment.jurisdiction_profile(jurisdiction_code); payout_preference_id->payment.payout_preference(payout_preference_id); provider_account_id->payment.provider_account(provider_account_id); provider_key->payment.provider(provider_key)` | `idx_payout_instruction_settlement_id; payout_instruction_idempotency_key_key; payout_instruction_pkey` | 支付渠道、意图、webhook、对账与提现 |
| `payout_preference` | `payout_preference_id` | `-` | `destination_jurisdiction_code->payment.jurisdiction_profile(jurisdiction_code); preferred_provider_account_id->payment.provider_account(provider_account_id); preferred_provider_key->payment.provider(provider_key)` | `idx_payout_preference_beneficiary; payout_preference_pkey` | 支付渠道、意图、webhook、对账与提现 |
| `provider` | `provider_key` | `-` | `-` | `provider_pkey` | 支付渠道、意图、webhook、对账与提现 |
| `provider_account` | `provider_account_id` | `provider_account_provider_key_account_scope_account_scope_i_key(provider_key, account_scope, account_scope_id, account_name)` | `jurisdiction_code->payment.jurisdiction_profile(jurisdiction_code); provider_key->payment.provider(provider_key)` | `provider_account_pkey; provider_account_provider_key_account_scope_account_scope_i_key` | 支付渠道、意图、webhook、对账与提现 |
| `reconciliation_diff` | `reconciliation_diff_id` | `-` | `reconciliation_statement_id->payment.reconciliation_statement(reconciliation_statement_id)` | `reconciliation_diff_pkey` | 支付渠道、意图、webhook、对账与提现 |
| `reconciliation_statement` | `reconciliation_statement_id` | `reconciliation_statement_provider_key_provider_account_id_s_key(provider_key, provider_account_id, statement_date, statement_type)` | `provider_account_id->payment.provider_account(provider_account_id); provider_key->payment.provider(provider_key)` | `reconciliation_statement_pkey; reconciliation_statement_provider_key_provider_account_id_s_key` | 支付渠道、意图、webhook、对账与提现 |
| `refund_intent` | `refund_intent_id` | `-` | `payment_intent_id->payment.payment_intent(payment_intent_id); provider_account_id->payment.provider_account(provider_account_id); provider_key->payment.provider(provider_key)` | `refund_intent_pkey` | 支付渠道、意图、webhook、对账与提现 |

## Schema: `billing`

- 对象职责：计费规则、账务、结算、退款、赔付
- 表数量：14

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `account_ledger_entry` | `account_ledger_entry_id` | `-` | `wallet_account_id->billing.wallet_account(wallet_account_id)` | `account_ledger_entry_pkey` | 计费规则、账务、结算、退款、赔付 |
| `billing_event` | `billing_event_id` | `-` | `-` | `billing_event_pkey; idx_billing_event_order_id` | 计费规则、账务、结算、退款、赔付 |
| `compensation_record` | `compensation_id` | `-` | `-` | `compensation_record_pkey` | 计费规则、账务、结算、退款、赔付 |
| `deposit_record` | `deposit_id` | `-` | `token_code->billing.token_asset(token_code); wallet_account_id->billing.wallet_account(wallet_account_id)` | `deposit_record_pkey` | 计费规则、账务、结算、退款、赔付 |
| `escrow_ledger` | `escrow_ledger_id` | `-` | `token_code->billing.token_asset(token_code)` | `escrow_ledger_pkey` | 计费规则、账务、结算、退款、赔付 |
| `fee_preview` | `fee_preview_id` | `-` | `fee_rule_id->billing.fee_rule(fee_rule_id)` | `fee_preview_pkey; idx_fee_preview_order_id` | 计费规则、账务、结算、退款、赔付 |
| `fee_rule` | `fee_rule_id` | `-` | `-` | `fee_rule_pkey` | 计费规则、账务、结算、退款、赔付 |
| `fee_rule_version` | `fee_rule_version_id` | `fee_rule_version_fee_rule_id_version_no_key(fee_rule_id, version_no)` | `fee_rule_id->billing.fee_rule(fee_rule_id)` | `fee_rule_version_fee_rule_id_version_no_key; fee_rule_version_pkey` | 计费规则、账务、结算、退款、赔付 |
| `invoice_request` | `invoice_request_id` | `-` | `settlement_id->billing.settlement_record(settlement_id)` | `invoice_request_pkey` | 计费规则、账务、结算、退款、赔付 |
| `penalty_event` | `penalty_event_id` | `-` | `token_code->billing.token_asset(token_code)` | `penalty_event_pkey` | 计费规则、账务、结算、退款、赔付 |
| `refund_record` | `refund_id` | `-` | `-` | `refund_record_pkey` | 计费规则、账务、结算、退款、赔付 |
| `settlement_record` | `settlement_id` | `-` | `-` | `settlement_record_pkey` | 计费规则、账务、结算、退款、赔付 |
| `token_asset` | `token_code` | `-` | `-` | `token_asset_pkey` | 计费规则、账务、结算、退款、赔付 |
| `wallet_account` | `wallet_account_id` | `wallet_account_subject_type_subject_id_token_code_key(subject_type, subject_id, token_code)` | `token_code->billing.token_asset(token_code)` | `wallet_account_pkey; wallet_account_subject_type_subject_id_token_code_key` | 计费规则、账务、结算、退款、赔付 |

## Schema: `risk`

- 对象职责：风险评级、风控事件、公平性事件
- 表数量：4

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `blacklist_entry` | `blacklist_entry_id` | `-` | `-` | `blacklist_entry_pkey` | 风险评级、风控事件、公平性事件 |
| `fairness_incident` | `fairness_incident_id` | `-` | `-` | `fairness_incident_pkey; idx_fairness_incident_order; idx_fairness_incident_ref; idx_fairness_incident_trace` | 风险评级、风控事件、公平性事件 |
| `rating_record` | `rating_id` | `-` | `-` | `rating_record_pkey` | 风险评级、风控事件、公平性事件 |
| `reputation_snapshot` | `reputation_snapshot_id` | `-` | `-` | `idx_reputation_snapshot_subject; reputation_snapshot_pkey` | 风险评级、风控事件、公平性事件 |

## Schema: `support`

- 对象职责：工单、争议、客服协同
- 表数量：4

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `decision_record` | `decision_id` | `decision_record_case_id_key(case_id)` | `case_id->support.dispute_case(case_id)` | `decision_record_case_id_key; decision_record_pkey` | 工单、争议、客服协同 |
| `dispute_case` | `case_id` | `-` | `-` | `dispute_case_pkey; idx_dispute_case_order_id` | 工单、争议、客服协同 |
| `dispute_status_history` | `dispute_status_history_id` | `-` | `case_id->support.dispute_case(case_id)` | `dispute_status_history_pkey` | 工单、争议、客服协同 |
| `evidence_object` | `evidence_id` | `-` | `case_id->support.dispute_case(case_id)` | `evidence_object_pkey` | 工单、争议、客服协同 |

## Schema: `audit`

- 对象职责：审计事件、证据、锚定、回放、保全
- 表数量：13

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `access_audit` | `access_audit_id` | `-` | `-` | `access_audit_pkey; idx_access_audit_target` | 审计事件、证据、锚定、回放、保全 |
| `anchor_batch` | `anchor_batch_id` | `-` | `-` | `anchor_batch_pkey; idx_anchor_batch_status` | 审计事件、证据、锚定、回放、保全 |
| `anchor_item` | `anchor_item_id` | `-` | `anchor_batch_id->audit.anchor_batch(anchor_batch_id); evidence_manifest_id->audit.evidence_manifest(evidence_manifest_id)` | `anchor_item_pkey` | 审计事件、证据、锚定、回放、保全 |
| `audit_event` | `audit_id, event_time` | `-` | `evidence_manifest_id->audit.evidence_manifest(evidence_manifest_id); evidence_manifest_id->audit.evidence_manifest(evidence_manifest_id)` | `audit_event_pk; idx_audit_event_manifest; idx_audit_event_ref; idx_audit_event_request; idx_audit_event_trace; idx_audit_event_tx` | 审计事件、证据、锚定、回放、保全 |
| `audit_event_default` | `audit_id, event_time` | `-` | `evidence_manifest_id->audit.evidence_manifest(evidence_manifest_id); evidence_manifest_id->audit.evidence_manifest(evidence_manifest_id)` | `audit_event_default_evidence_manifest_id_event_time_idx; audit_event_default_pkey; audit_event_default_ref_type_ref_id_event_time_idx; audit_event_default_request_id_event_time_idx; audit_event_default_trace_id_event_time_idx; audit_event_default_tx_hash_event_time_idx` | 审计事件、证据、锚定、回放、保全 |
| `evidence_item` | `evidence_item_id` | `-` | `retention_policy_id->audit.retention_policy(retention_policy_id)` | `evidence_item_pkey; idx_evidence_item_ref` | 审计事件、证据、锚定、回放、保全 |
| `evidence_manifest` | `evidence_manifest_id` | `-` | `-` | `evidence_manifest_pkey` | 审计事件、证据、锚定、回放、保全 |
| `evidence_manifest_item` | `evidence_manifest_item_id` | `uq_evidence_manifest_item(evidence_manifest_id, evidence_item_id); uq_evidence_manifest_ordinal(evidence_manifest_id, ordinal_no)` | `evidence_item_id->audit.evidence_item(evidence_item_id); evidence_manifest_id->audit.evidence_manifest(evidence_manifest_id)` | `evidence_manifest_item_pkey; uq_evidence_manifest_item; uq_evidence_manifest_ordinal` | 审计事件、证据、锚定、回放、保全 |
| `evidence_package` | `evidence_package_id` | `-` | `evidence_manifest_id->audit.evidence_manifest(evidence_manifest_id)` | `evidence_package_pkey` | 审计事件、证据、锚定、回放、保全 |
| `legal_hold` | `legal_hold_id` | `-` | `retention_policy_id->audit.retention_policy(retention_policy_id)` | `legal_hold_pkey` | 审计事件、证据、锚定、回放、保全 |
| `replay_job` | `replay_job_id` | `-` | `-` | `idx_replay_job_ref; replay_job_pkey` | 审计事件、证据、锚定、回放、保全 |
| `replay_result` | `replay_result_id` | `-` | `replay_job_id->audit.replay_job(replay_job_id)` | `replay_result_pkey` | 审计事件、证据、锚定、回放、保全 |
| `retention_policy` | `retention_policy_id` | `retention_policy_policy_key_key(policy_key)` | `-` | `retention_policy_pkey; retention_policy_policy_key_key` | 审计事件、证据、锚定、回放、保全 |

## Schema: `search`

- 对象职责：搜索投影、索引同步、排序配置
- 表数量：6

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `index_alias_binding` | `alias_binding_id` | `index_alias_binding_entity_scope_backend_type_key(entity_scope, backend_type)` | `-` | `index_alias_binding_entity_scope_backend_type_key; index_alias_binding_pkey` | 搜索投影、索引同步、排序配置 |
| `index_sync_task` | `index_sync_task_id` | `-` | `-` | `idx_index_sync_task_scope_entity; idx_index_sync_task_status; index_sync_task_pkey` | 搜索投影、索引同步、排序配置 |
| `product_search_document` | `product_id` | `-` | `-` | `idx_product_search_document_embedding; idx_product_search_document_listing_status; idx_product_search_document_org_id; idx_product_search_document_price; idx_product_search_document_seller_industry_tags_gin; idx_product_search_document_sync_status; idx_product_search_document_tags_gin; idx_product_search_document_tsv; idx_product_search_document_use_cases_gin; product_search_document_pkey` | 搜索投影、索引同步、排序配置 |
| `ranking_profile` | `ranking_profile_id` | `ranking_profile_profile_key_key(profile_key)` | `-` | `ranking_profile_pkey; ranking_profile_profile_key_key` | 搜索投影、索引同步、排序配置 |
| `search_signal_aggregate` | `entity_scope, entity_id` | `-` | `-` | `search_signal_aggregate_pkey` | 搜索投影、索引同步、排序配置 |
| `seller_search_document` | `org_id` | `-` | `-` | `idx_seller_search_document_embedding; idx_seller_search_document_sync_status; idx_seller_search_document_tsv; seller_search_document_pkey` | 搜索投影、索引同步、排序配置 |

## Schema: `ops`

- 对象职责：outbox、DLQ、监控、告警、任务与系统日志
- 表数量：23

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `alert_event` | `alert_event_id` | `-` | `alert_rule_id->ops.alert_rule(alert_rule_id); source_backend_key->ops.observability_backend(backend_key)` | `alert_event_pkey; idx_alert_event_status; idx_alert_event_trace` | outbox、DLQ、监控、告警、任务与系统日志 |
| `alert_rule` | `alert_rule_id` | `alert_rule_rule_key_key(rule_key)` | `source_backend_key->ops.observability_backend(backend_key)` | `alert_rule_pkey; alert_rule_rule_key_key` | outbox、DLQ、监控、告警、任务与系统日志 |
| `approval_step` | `approval_step_id` | `approval_step_approval_ticket_id_step_no_key(approval_ticket_id, step_no)` | `approval_ticket_id->ops.approval_ticket(approval_ticket_id)` | `approval_step_approval_ticket_id_step_no_key; approval_step_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `approval_ticket` | `approval_ticket_id` | `-` | `-` | `approval_ticket_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `chain_projection_gap` | `chain_projection_gap_id` | `-` | `outbox_event_id->ops.outbox_event(outbox_event_id)` | `chain_projection_gap_pkey; idx_chain_projection_gap_order; idx_chain_projection_gap_outbox; idx_chain_projection_gap_status` | outbox、DLQ、监控、告警、任务与系统日志 |
| `consumer_idempotency_record` | `consumer_idempotency_record_id` | `uq_consumer_event(consumer_name, event_id)` | `-` | `consumer_idempotency_record_pkey; idx_consumer_idempotency_consumer; uq_consumer_event` | outbox、DLQ、监控、告警、任务与系统日志 |
| `dead_letter_event` | `dead_letter_event_id` | `-` | `-` | `dead_letter_event_pkey; idx_dead_letter_reprocess` | outbox、DLQ、监控、告警、任务与系统日志 |
| `event_route_policy` | `event_route_policy_id` | `uq_event_route_policy(aggregate_type, event_type, target_bus, target_topic)` | `-` | `event_route_policy_pkey; uq_event_route_policy` | outbox、DLQ、监控、告警、任务与系统日志 |
| `external_fact_receipt` | `external_fact_receipt_id` | `-` | `-` | `external_fact_receipt_pkey; idx_external_fact_receipt_order; idx_external_fact_receipt_ref; idx_external_fact_receipt_trace` | outbox、DLQ、监控、告警、任务与系统日志 |
| `incident_event` | `incident_event_id` | `-` | `incident_ticket_id->ops.incident_ticket(incident_ticket_id)` | `incident_event_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `incident_ticket` | `incident_ticket_id` | `incident_ticket_incident_key_key(incident_key)` | `source_alert_event_id->ops.alert_event(alert_event_id)` | `idx_incident_ticket_status; incident_ticket_incident_key_key; incident_ticket_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `job_run` | `job_run_id` | `-` | `-` | `idx_job_run_status_started; job_run_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `log_retention_policy` | `log_retention_policy_id` | `log_retention_policy_policy_key_key(policy_key)` | `storage_backend_key->ops.observability_backend(backend_key)` | `log_retention_policy_pkey; log_retention_policy_policy_key_key` | outbox、DLQ、监控、告警、任务与系统日志 |
| `monitoring_policy_profile` | `monitoring_policy_profile_id` | `monitoring_policy_profile_profile_key_key(profile_key)` | `-` | `monitoring_policy_profile_pkey; monitoring_policy_profile_profile_key_key` | outbox、DLQ、监控、告警、任务与系统日志 |
| `observability_backend` | `observability_backend_id` | `observability_backend_backend_key_key(backend_key)` | `-` | `observability_backend_backend_key_key; observability_backend_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `outbox_event` | `outbox_event_id` | `-` | `-` | `idx_outbox_pending; idx_outbox_topic_pending; idx_outbox_trace; outbox_event_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `outbox_publish_attempt` | `outbox_publish_attempt_id` | `-` | `outbox_event_id->ops.outbox_event(outbox_event_id)` | `idx_publish_attempt_event; outbox_publish_attempt_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `slo_definition` | `slo_definition_id` | `slo_definition_slo_key_key(slo_key)` | `alert_rule_id->ops.alert_rule(alert_rule_id); source_backend_key->ops.observability_backend(backend_key)` | `slo_definition_pkey; slo_definition_slo_key_key` | outbox、DLQ、监控、告警、任务与系统日志 |
| `slo_snapshot` | `slo_snapshot_id` | `-` | `slo_definition_id->ops.slo_definition(slo_definition_id); source_backend_key->ops.observability_backend(backend_key)` | `idx_slo_snapshot_def; slo_snapshot_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `system_log` | `system_log_id, created_at` | `-` | `-` | `idx_system_log_object; idx_system_log_request; idx_system_log_service_level; idx_system_log_traceparent; system_log_pk` | outbox、DLQ、监控、告警、任务与系统日志 |
| `system_log_default` | `system_log_id, created_at` | `-` | `-` | `system_log_default_object_type_object_id_created_at_idx; system_log_default_pkey; system_log_default_request_id_created_at_idx; system_log_default_service_name_log_level_created_at_idx; system_log_default_traceparent_created_at_idx` | outbox、DLQ、监控、告警、任务与系统日志 |
| `trace_index` | `trace_index_id` | `-` | `backend_key->ops.observability_backend(backend_key)` | `idx_trace_index_ref; idx_trace_index_trace; trace_index_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |
| `trade_lifecycle_checkpoint` | `trade_lifecycle_checkpoint_id` | `-` | `monitoring_policy_profile_id->ops.monitoring_policy_profile(monitoring_policy_profile_id)` | `idx_trade_lifecycle_checkpoint_order; idx_trade_lifecycle_checkpoint_ref; idx_trade_lifecycle_checkpoint_trace; trade_lifecycle_checkpoint_pkey` | outbox、DLQ、监控、告警、任务与系统日志 |

## Schema: `developer`

- 对象职责：开发者测试资产、模拟绑定、演示支付案例
- 表数量：4

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `mock_payment_case` | `mock_payment_case_id` | `-` | `-` | `idx_mock_payment_case_status; mock_payment_case_pkey` | 开发者测试资产、模拟绑定、演示支付案例 |
| `mock_provider_binding` | `mock_provider_binding_id` | `-` | `-` | `mock_provider_binding_pkey` | 开发者测试资产、模拟绑定、演示支付案例 |
| `test_application` | `test_application_id` | `-` | `-` | `test_application_pkey` | 开发者测试资产、模拟绑定、演示支付案例 |
| `test_wallet` | `test_wallet_id` | `test_wallet_address_key(address)` | `-` | `test_wallet_address_key; test_wallet_pkey` | 开发者测试资产、模拟绑定、演示支付案例 |

## Schema: `chain`

- 对象职责：链上事件投影与锚定记录
- 表数量：2

| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |
| --- | --- | --- | --- | --- | --- |
| `chain_anchor` | `chain_anchor_id` | `-` | `-` | `chain_anchor_pkey; idx_chain_anchor_status_created` | 链上事件投影与锚定记录 |
| `contract_event_projection` | `contract_event_projection_id` | `-` | `-` | `contract_event_projection_pkey; idx_contract_event_projection_ref` | 链上事件投影与锚定记录 |

