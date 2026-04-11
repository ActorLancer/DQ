# 数据交易平台表关系总图 / ER 文本图

## 1. 说明

本文件基于以下输入统一整理：

- [正式PRD](/home/luna/Documents/DataB/正式PRD)
- [业务流程](/home/luna/Documents/DataB/业务流程)
- [页面说明书](/home/luna/Documents/DataB/页面说明书)
- [领域模型](/home/luna/Documents/DataB/领域模型)
- [用户角色说明](/home/luna/Documents/DataB/用户角色说明)
- [权限设计](/home/luna/Documents/DataB/权限设计)
- [数据库设计](/home/luna/Documents/DataB/数据库设计)
- [data_trading_blockchain_system_design_split](/home/luna/Documents/DataB/data_trading_blockchain_system_design_split)
- [原始PRD](/home/luna/Documents/DataB/原始PRD)

目标是给研发和 AI 提供一份可快速阅读的全量对象关系文本图。这里不是替代 SQL，而是把全局主干、版本增量和跨域关联集中描述出来。

## 2. 顶层域关系图

```text
core.organization
  ├─< core.department
  ├─< core.user_account
  ├─< core.application
  ├─< core.service_identity
  ├─< core.connector
  ├─< core.execution_environment
  ├─< catalog.storage_namespace
  └─< catalog.storage_policy_profile

authz.role_definition
  ├─< authz.role_permission >─ authz.permission_definition
  └─< authz.subject_role_binding >─ core.user_account / core.application / core.service_identity

catalog.data_asset
  ├─< catalog.asset_version
  │    └─1 catalog.storage_policy_profile
  │    └─< catalog.query_surface_definition
  ├─< catalog.raw_ingest_batch
  │    └─< catalog.raw_object_manifest
  │         ├─< catalog.format_detection_result
  │         └─< catalog.extraction_job
  ├─< catalog.asset_storage_binding
  ├─< catalog.asset_sample
  ├─< catalog.preview_artifact
  ├─< catalog.asset_field_definition
  ├─< catalog.asset_quality_report
  ├─< catalog.asset_processing_job
  │    └─< catalog.asset_processing_input
  ├─< catalog.asset_structured_dataset
  │    └─< catalog.asset_structured_row
  └─< catalog.product
        ├─< catalog.product_metadata_profile
        ├─< catalog.product_tag >─ catalog.tag
        ├─< catalog.product_sku
        ├─< contract.data_contract
        ├─< contract.template_binding >─ contract.template_definition
        └─< contract.policy_binding >─ contract.usage_policy

trade.inquiry
  └─< trade.order_main
        ├─< trade.order_line >─ catalog.product_sku
        ├─< trade.order_status_history
        ├─1 contract.digital_contract
        │    ├─1 contract.data_contract
        │    └─< contract.contract_signer
        ├─< trade.authorization_grant
        ├─< delivery.delivery_record
        │    ├─< delivery.delivery_ticket
        │    ├─< delivery.delivery_receipt
        │    ├─< delivery.key_envelope
        │    ├─< delivery.api_credential
        │    ├─< delivery.sandbox_workspace
        │    │    ├─< delivery.sandbox_session
        │    │    └─1 catalog.query_surface_definition
        │    ├─< delivery.template_query_grant
        │    │    └─1 catalog.query_surface_definition
        │    │    └─< delivery.query_template_definition
        │    │         └─< delivery.query_execution_run
        │    └─< delivery.report_artifact
        ├─< billing.billing_event
        ├─< billing.settlement_record
        ├─< billing.refund_record
        ├─< billing.compensation_record
        ├─< support.dispute_case
        │    ├─< support.dispute_status_history
        │    ├─< support.evidence_object
        │    └─< support.decision_record
        ├─< review.review_task
        │    └─< review.review_step
        └─< ops.approval_ticket
             └─< ops.approval_step

billing.wallet_account
  ├─< billing.account_ledger_entry
  ├─< billing.escrow_ledger
  ├─< billing.deposit_record
  ├─< billing.penalty_event
  └─< billing.invoice_request

risk.rating_record
  ├─< risk.reputation_snapshot
  └─< risk.blacklist_entry

audit.audit_event
  └─< audit.evidence_package

ops.outbox_event
  └─< ops.dead_letter_event

search.product_search_document
search.seller_search_document
search.search_signal_aggregate
search.ranking_profile
search.index_alias_binding
search.index_sync_task

developer.test_application
developer.test_wallet
developer.mock_provider_binding

chain.contract_event_projection
chain.chain_anchor
```

## 3. V1 业务主链路 ER 图

```text
Organization(卖方/买方/平台)
  ├─ Department
  ├─ UserAccount
  ├─ Application
  ├─ StorageNamespace
  ├─ StoragePolicyProfile
  └─ WalletAccount

DataAsset
  ├─ RawIngestBatch
  │    └─ RawObjectManifest
  │         ├─ FormatDetectionResult
  │         └─ ExtractionJob
  ├─ AssetVersion
  │    ├─ PreviewArtifact
  │    ├─ AssetFieldDefinition
  │    ├─ AssetQualityReport
  │    └─ AssetProcessingJob

DataAsset
  ├─ AssetVersion
  ├─ AssetStorageBinding
  ├─ AssetSample
  ├─ AssetFieldDefinition
  ├─ AssetQualityReport
  ├─ AssetProcessingJob
  └─ Product
       ├─ ProductMetadataProfile
       ├─ ProductTag
       ├─ ProductSku
       ├─ DataContract
       ├─ ProductSearchDocument
       ├─ TemplateBinding
       └─ PolicyBinding

Organization(卖方)
  └─ SellerSearchDocument

Inquiry
  └─ OrderMain
       ├─ OrderLine
       ├─ DigitalContract
       │   └─ DataContract
       ├─ AuthorizationGrant
       ├─ DeliveryRecord
       ├─ BillingEvent
       ├─ SettlementRecord
       ├─ DisputeCase
       └─ ChainAnchor
```

## 4. V2 增量 ER 图

```text
catalog.product / catalog.product_sku
  └─ may reference model-oriented offerings

ml.model_asset
  ├─< ml.model_version
  ├─< ml.training_task
  │    ├─< ml.task_participant
  │    ├─< ml.training_round
  │    ├─< ml.model_update
  │    ├─< ml.proof_artifact
  │    └─< ml.task_status_history
  ├─< ml.compute_task
  │    ├─< ml.compute_result
  │    └─< ml.proof_artifact
  └─< search.model_search_document

ml.algorithm_artifact
  └─< ml.compute_task

billing.profit_share_rule
  ├─< billing.contribution_record
  ├─< billing.reward_record
  └─< billing.reward_pool

chain.public_anchor_batch
  └─< chain.credential_token
       └─< chain.credential_status_history

search.synonym_rule
```

## 5. V3 增量 ER 图

```text
crosschain.gateway_identity
  └─< crosschain.cross_chain_request
       ├─< crosschain.cross_chain_ack
       ├─< crosschain.witness_record
       ├─< crosschain.request_status_history
       └─< crosschain.compensation_task

ecosystem.partner
  ├─< ecosystem.connector_version
  └─< ecosystem.mutual_recognition
  └─< search.partner_search_document

risk.graph_node
  └─< risk.graph_edge

risk.risk_alert
  └─< risk.risk_case

risk.freeze_ticket
  └─< risk.governance_action_log

audit.regulator_query
  └─< audit.regulator_export_record
```

## 6. 分域详细对象关系

### 6.1 身份、组织与访问控制

```text
core.organization(id)
  ├─ core.department.organization_id
  ├─ core.user_account.organization_id
  ├─ core.application.organization_id
  ├─ core.service_identity.organization_id
  ├─ core.connector.organization_id
  └─ core.execution_environment.organization_id

core.department(id)
  ├─ self.parent_department_id
  ├─ core.user_account.department_id
  └─ authz.subject_role_binding.scope_department_id

core.user_account(id)
  ├─ core.did_binding.user_id
  ├─ contract.contract_signer.user_id
  ├─ authz.subject_role_binding.subject_id(type=user)
  ├─ developer.test_application.created_by
  └─ developer.test_wallet.created_by

core.application(id)
  ├─ delivery.api_credential.application_id
  ├─ authz.subject_role_binding.subject_id(type=application)
  └─ ml.compute_task.application_id

## 7. 存储与信任边界补充关系图

```text
catalog.data_asset
  ├─< catalog.asset_version
  ├─< catalog.asset_storage_binding
  ├─< catalog.asset_custody_profile
  └─< catalog.asset_trust_evidence

catalog.asset_version
  ├─< catalog.asset_storage_binding
  ├─< catalog.asset_custody_profile
  └─< catalog.asset_trust_evidence

trade.order_main
  └─ stores:
       storage_mode_snapshot
       delivery_route_snapshot
       trust_boundary_snapshot

delivery.delivery_record
  ├─< catalog.asset_storage_binding (source_binding_id)
  └─ stores:
       executor_type
       delivery_route
       trust_boundary_snapshot

ml.model_asset
  └─ stores:
       artifact_storage_mode
       weight_custody_mode

ml.training_task / ml.compute_task / ml.compute_result
  └─ stores:
       training_data_residency_mode
       execution_boundary_json
       result_export_policy

ecosystem.partner
  └─ stores:
       partner_storage_capability
       partner_key_governance_capability
       partner_execution_boundary_capability

crosschain.cross_chain_request
  └─ stores:
       trust_scope_snapshot
       retention_obligation_snapshot
```

authz.role_definition(id)
  ├─ authz.role_permission.role_id
  └─ authz.subject_role_binding.role_id

authz.permission_definition(id)
  └─ authz.role_permission.permission_id
```

### 6.2 数据资产、商品与策略

```text
catalog.data_asset(id)
  ├─ catalog.asset_version.asset_id
  ├─ catalog.asset_storage_binding.asset_id
  ├─ catalog.asset_sample.asset_id
  ├─ catalog.asset_structured_dataset.asset_id
  └─ catalog.product.asset_id

catalog.asset_structured_dataset(id)
  └─ catalog.asset_structured_row.dataset_id

catalog.product(id)
  ├─ catalog.product_sku.product_id
  ├─ catalog.product_tag.product_id
  ├─ contract.template_binding.product_id
  ├─ contract.policy_binding.product_id
  ├─ trade.order_line.product_id
  ├─ review.review_task.product_id
  ├─ chain.chain_anchor.product_id
  ├─ search.product_search_document.product_id
  └─ billing.profit_share_rule.product_id

catalog.tag(id)
  └─ catalog.product_tag.tag_id

contract.template_definition(id)
  └─ contract.template_binding.template_id

contract.usage_policy(id)
  └─ contract.policy_binding.policy_id
```

### 6.3 交易、合同、交付

```text
trade.inquiry(id)
  └─ trade.order_main.inquiry_id

trade.order_main(id)
  ├─ trade.order_line.order_id
  ├─ trade.order_status_history.order_id
  ├─ contract.digital_contract.order_id
  ├─ trade.authorization_grant.order_id
  ├─ delivery.delivery_record.order_id
  ├─ billing.billing_event.order_id
  ├─ billing.settlement_record.order_id
  ├─ billing.refund_record.order_id
  ├─ billing.compensation_record.order_id
  ├─ support.dispute_case.order_id
  ├─ audit.audit_event.order_id
  ├─ chain.chain_anchor.order_id
  └─ ops.outbox_event.aggregate_id(order/order_main)

contract.digital_contract(id)
  ├─ contract.contract_signer.contract_id
  └─ trade.order_main.contract_id

delivery.storage_object(id)
  ├─ delivery.delivery_record.storage_object_id
  ├─ delivery.key_envelope.storage_object_id
  ├─ support.evidence_object.storage_object_id
  └─ audit.evidence_package.storage_object_id

delivery.delivery_record(id)
  ├─ delivery.delivery_ticket.delivery_id
  ├─ delivery.delivery_receipt.delivery_id
  ├─ delivery.key_envelope.delivery_id
  ├─ delivery.api_credential.delivery_id
  ├─ delivery.sandbox_workspace.delivery_id
  └─ delivery.report_artifact.delivery_id

delivery.sandbox_workspace(id)
  └─ delivery.sandbox_session.workspace_id

catalog.query_surface_definition(id)
  ├─ delivery.sandbox_workspace.query_surface_id
  ├─ delivery.template_query_grant.query_surface_id
  ├─ delivery.query_template_definition.query_surface_id
  └─ delivery.query_execution_run.query_surface_id
```

### 6.4 计费、托管、保证金、惩罚、奖励、分润

```text
billing.token_asset(id)
  ├─ billing.wallet_account.token_asset_id
  ├─ billing.account_ledger_entry.token_asset_id
  ├─ billing.escrow_ledger.token_asset_id
  ├─ billing.deposit_record.token_asset_id
  ├─ billing.reward_pool.token_asset_id
  └─ billing.reward_record.token_asset_id

billing.wallet_account(id)
  ├─ billing.account_ledger_entry.wallet_account_id
  ├─ billing.deposit_record.wallet_account_id
  └─ billing.invoice_request.wallet_account_id

trade.order_main(id)
  ├─ billing.billing_event.order_id
  ├─ billing.settlement_record.order_id
  ├─ billing.penalty_event.order_id
  ├─ billing.refund_record.order_id
  ├─ billing.compensation_record.order_id
  ├─ billing.escrow_ledger.order_id
  └─ billing.reward_record.order_id

billing.profit_share_rule(id)
  ├─ billing.contribution_record.rule_id
  ├─ billing.reward_record.rule_id
  └─ billing.reward_pool.rule_id
```

### 6.5 合规、审核、争议、风险

```text
review.review_task(id)
  ├─ review.review_step.review_task_id
  └─ ops.approval_ticket.review_task_id

ops.approval_ticket(id)
  └─ ops.approval_step.ticket_id

support.dispute_case(id)
  ├─ support.dispute_status_history.dispute_case_id
  ├─ support.evidence_object.dispute_case_id
  ├─ support.decision_record.dispute_case_id
  └─ ops.outbox_event.aggregate_id(dispute_case)

risk.rating_record(id)
  └─ risk.reputation_snapshot.rating_record_id

risk.risk_alert(id)
  └─ risk.risk_case.alert_id

risk.graph_node(id)
  ├─ risk.graph_edge.from_node_id
  └─ risk.graph_edge.to_node_id

risk.freeze_ticket(id)
  └─ risk.governance_action_log.freeze_ticket_id
```

### 6.6 审计、日志、开发者、链上

```text
audit.audit_event(id)
  ├─ audit.evidence_package.audit_event_id
  ├─ chain.contract_event_projection.audit_event_id
  ├─ chain.chain_anchor.audit_event_id
  └─ audit.regulator_export_record.audit_event_id

ops.outbox_event(id)
  ├─ ops.dead_letter_event.outbox_event_id
  ├─ chain.chain_anchor.outbox_event_id
  └─ crosschain.cross_chain_request.outbox_event_id

developer.test_application(id)
  ├─ developer.mock_provider_binding.test_application_id
  └─ delivery.api_credential.test_application_id

developer.test_wallet(id)
  ├─ billing.wallet_account.test_wallet_id
  └─ chain.credential_token.test_wallet_id
```

### 6.7 模型、训练、受控计算、证明

```text
ml.model_asset(id)
  ├─ ml.model_version.model_asset_id
  ├─ ml.training_task.model_asset_id
  ├─ ml.compute_task.model_asset_id
  └─ search.model_search_document.model_asset_id

ml.algorithm_artifact(id)
  └─ ml.compute_task.algorithm_artifact_id

ml.training_task(id)
  ├─ ml.task_participant.training_task_id
  ├─ ml.training_round.training_task_id
  ├─ ml.model_update.training_task_id
  ├─ ml.proof_artifact.training_task_id
  └─ ml.task_status_history.training_task_id

ml.compute_task(id)
  ├─ ml.compute_result.compute_task_id
  ├─ ml.proof_artifact.compute_task_id
  └─ ml.task_status_history.compute_task_id
```

### 6.8 跨链、生态互联、监管

```text
crosschain.gateway_identity(id)
  └─ crosschain.cross_chain_request.gateway_identity_id

crosschain.cross_chain_request(id)
  ├─ crosschain.cross_chain_ack.request_id
  ├─ crosschain.witness_record.request_id
  ├─ crosschain.request_status_history.request_id
  ├─ crosschain.compensation_task.request_id
  └─ audit.audit_event.cross_chain_request_id

ecosystem.partner(id)
  ├─ ecosystem.connector_version.partner_id
  └─ ecosystem.mutual_recognition.partner_id

audit.regulator_query(id)
  └─ audit.regulator_export_record.regulator_query_id
```

### 6.9 支付、资金流与轻结算

```text
billing.fee_rule(id)
  ├─ billing.fee_rule_version.fee_rule_id
  └─ billing.fee_preview.fee_rule_id (logical snapshot)

trade.order_main(id)
  ├─ billing.fee_preview.order_id
  ├─ payment.payment_intent.order_id
  └─ billing.settlement_record.order_id

payment.provider(provider_key)
  ├─ payment.provider_account.provider_key
  ├─ payment.payment_intent.provider_key
  ├─ payment.payment_webhook_event.provider_key
  ├─ payment.refund_intent.provider_key
  ├─ payment.payout_instruction.provider_key
  └─ payment.reconciliation_statement.provider_key

payment.jurisdiction_profile(jurisdiction_code)
  ├─ payment.provider_account.jurisdiction_code
  ├─ payment.corridor_policy.payer_jurisdiction_code
  ├─ payment.corridor_policy.payee_jurisdiction_code
  ├─ payment.payout_preference.destination_jurisdiction_code
  ├─ payment.payment_intent.payer_jurisdiction_code
  ├─ payment.payment_intent.payee_jurisdiction_code
  ├─ payment.payment_intent.launch_jurisdiction_code
  └─ payment.settlement_route.source_jurisdiction_code / target_jurisdiction_code

payment.provider_account(provider_account_id)
  ├─ payment.payment_intent.provider_account_id
  ├─ payment.payout_preference.preferred_provider_account_id
  ├─ payment.refund_intent.provider_account_id
  ├─ payment.payout_instruction.provider_account_id
  └─ payment.reconciliation_statement.provider_account_id

payment.corridor_policy(corridor_policy_id)
  ├─ payment.payment_intent.corridor_policy_id
  └─ payment.settlement_route.corridor_policy_id

payment.payout_preference(payout_preference_id)
  └─ payment.payout_instruction.payout_preference_id

payment.settlement_route(settlement_route_id)
  ├─ payment.fx_quote.settlement_route_id
  └─ payment.crypto_settlement_transfer.settlement_route_id

payment.payment_intent(payment_intent_id)
  ├─ payment.payment_transaction.payment_intent_id
  ├─ payment.payment_webhook_event.payment_intent_id
  ├─ payment.refund_intent.payment_intent_id
  └─ developer.mock_payment_case.payment_intent_id

payment.reconciliation_statement(reconciliation_statement_id)
  └─ payment.reconciliation_diff.reconciliation_statement_id
```

## 7. 版本增量与预埋关系

### 7.1 V1 必落主表

- 身份与权限：`core.*`、`iam.*`、`authz.*`
- 商品与策略：`catalog.*`、`contract.template_*`、`contract.usage_policy`
- 交易链路：`trade.*`、`delivery.*`
- 账单与争议：`billing.*`、`support.*`
- 支付与对账：`payment.provider*`、`payment.jurisdiction_profile`、`payment.corridor_policy`、`payment.payout_preference`、`payment.payment_*`、`payment.reconciliation_*`
- 审核与审批：`review.*`、`ops.approval_*`
- 审计、搜索、开发者：`audit.*`、`search.product_search_document`、`search.seller_search_document`、`search.search_signal_aggregate`、`search.ranking_profile`、`search.index_alias_binding`、`search.index_sync_task`、`developer.*`
- 链下主状态与联盟链锚定：`chain.contract_event_projection`、`chain.chain_anchor`

### 7.2 V1 预埋对象

- `core.connector`
- `core.execution_environment`
- `catalog.asset_structured_row.embedding`
- `search.product_search_document.embedding`

### 7.3 V2 增量对象

- `ml.*`
- `search.model_search_document`
- `search.synonym_rule`
- `billing.profit_share_rule`
- `billing.reward_pool`
- `billing.contribution_record`
- `billing.reward_record`
- `payment.sub_merchant_binding`
- `payment.split_instruction`
- `payment.recurring_charge_plan`
- `chain.public_anchor_batch`
- `chain.credential_token`
- `chain.credential_status_history`

### 7.4 V3 增量对象

- `crosschain.*`
- `ecosystem.*`
- `search.partner_search_document`
- `risk.risk_alert`
- `risk.risk_case`
- `risk.graph_node`
- `risk.graph_edge`
- `risk.freeze_ticket`
- `risk.governance_action_log`
- `audit.regulator_query`
- `audit.regulator_export_record`
- `payment.settlement_route`
- `payment.fx_quote`
- `payment.crypto_settlement_transfer`

## 8. 关键触发器与联动

```text
common.tg_set_updated_at
  -> all major tables

common.tg_order_status_history
  -> trade.order_main status update
  -> trade.order_status_history insert

common.tg_dispute_status_history
  -> support.dispute_case status update
  -> support.dispute_status_history insert

search.tg_refresh_product_search_document
  -> catalog.product insert/update
  -> search.product_search_document upsert

search.refresh_seller_search_document_by_id
  -> core.organization / risk.reputation_snapshot / search.search_signal_aggregate changes
  -> search.seller_search_document upsert

search.refresh_model_search_document_by_id
  -> ml.model_asset / ml.model_version changes
  -> search.model_search_document upsert

search.refresh_partner_search_document_by_id
  -> ecosystem.partner / ecosystem.mutual_recognition changes
  -> search.partner_search_document upsert

common.tg_write_outbox
  -> catalog.product / trade.order_main / support.dispute_case / payment.payment_intent / payment.payout_instruction
  -> ops.outbox_event insert
```

## 9. 说明结论

这套 ER 文本图的目标不是把所有字段逐项搬运，而是把：

- 哪些对象是聚合根
- 哪些对象是明细/历史/投影
- 哪些关系跨版本扩展
- 哪些对象虽然 V1 暂不完整启用但必须预留

统一描述清楚。实际落地时，以 [数据库设计](/home/luna/Documents/DataB/数据库设计) 下的 SQL 迁移脚本为最终执行基线。

## 10. 身份认证、注册登录与会话管理关系补充

```text
core.organization(org_id)
  ├─ core.user_account.org_id
  ├─ iam.invitation.org_id
  ├─ iam.sso_connection.org_id
  └─ iam.fabric_ca_registry.org_id

core.user_account(user_id)
  ├─ iam.invitation.accepted_by_user_id
  ├─ iam.auth_method_binding.user_id
  ├─ iam.mfa_authenticator.user_id
  ├─ iam.trusted_device.user_id
  ├─ iam.refresh_token_family.user_id
  ├─ iam.user_session.user_id
  ├─ iam.external_identity_binding.user_id
  ├─ iam.step_up_challenge.user_id
  ├─ iam.fabric_identity_binding.user_id
  └─ iam.certificate_revocation_record.revoked_by_user_id

iam.trusted_device(trusted_device_id)
  ├─ iam.refresh_token_family.trusted_device_id
  └─ iam.user_session.trusted_device_id

iam.refresh_token_family(refresh_token_family_id)
  └─ iam.user_session.refresh_token_family_id

iam.sso_connection(sso_connection_id)
  ├─ iam.external_identity_binding.sso_connection_id
  └─ iam.provisioning_job.sso_connection_id

iam.fabric_ca_registry(fabric_ca_registry_id)
  ├─ iam.certificate_record.fabric_ca_registry_id
  └─ iam.fabric_identity_binding.fabric_ca_registry_id

iam.certificate_record(certificate_id)
  ├─ iam.fabric_identity_binding.certificate_id
  └─ iam.certificate_revocation_record.certificate_id

core.service_identity(service_identity_id)
  └─ iam.fabric_identity_binding.service_identity_id

ecosystem.partner(partner_id)
  ├─ iam.external_identity_binding.partner_id
  └─ iam.risk_auth_policy.partner_id
```

## 11. 审计、证据链与回放关系补充

```text
audit.audit_event(audit_id)
  ├─ audit.anchor_item.audit_id
  └─ audit.access_audit.target_id (当 target_type=audit_event)

audit.retention_policy(retention_policy_id)
  ├─ audit.evidence_item.retention_policy_id
  └─ audit.legal_hold.retention_policy_id

audit.evidence_manifest(evidence_manifest_id)
  ├─ audit.evidence_manifest_item.evidence_manifest_id
  ├─ audit.evidence_package.evidence_manifest_id
  └─ audit.anchor_item.evidence_manifest_id

audit.evidence_item(evidence_item_id)
  └─ audit.evidence_manifest_item.evidence_item_id

audit.anchor_batch(anchor_batch_id)
  └─ audit.anchor_item.anchor_batch_id

audit.replay_job(replay_job_id)
  └─ audit.replay_result.replay_job_id
```

## 12. 双层权威模型与一致性关系补充

```text
trade.order_main(order_id)
  ├─ ops.outbox_event.aggregate_id(order_id)
  └─ proof_commit_state / external_fact_status / reconcile_status

payment.payment_intent(payment_intent_id)
  ├─ ops.outbox_event.aggregate_id(payment_intent_id)
  └─ proof_commit_state / external_fact_status / reconcile_status

ops.event_route_policy(event_route_policy_id)
  └─ defines target_bus / target_topic / proof_commit_policy

ops.outbox_event(outbox_event_id)
  ├─ ops.outbox_publish_attempt.outbox_event_id
  └─ ops.consumer_idempotency_record.event_id
```

## 13. 推荐与个性化发现关系补充

```text
recommend.placement_definition(placement_code)
  ├─ recommend.behavior_event.placement_code
  └─ recommend.recommendation_request.placement_code

recommend.ranking_profile(recommendation_ranking_profile_id)
  ├─ recommend.recommendation_request.ranking_profile_id
  └─ recommend.recommendation_result.ranking_profile_id

recommend.recommendation_request(recommendation_request_id)
  ├─ recommend.recommendation_result.recommendation_request_id
  ├─ recommend.behavior_event.recommendation_request_id
  └─ recommend.model_inference_log.recommendation_request_id (V2)

recommend.recommendation_result(recommendation_result_id)
  ├─ recommend.recommendation_result_item.recommendation_result_id
  ├─ recommend.behavior_event.recommendation_result_id
  └─ recommend.model_inference_log.recommendation_result_id (V2)

core.organization(org_id)
  ├─ recommend.behavior_event.subject_org_id
  ├─ recommend.subject_profile_snapshot.org_id
  └─ recommend.recommendation_request.subject_org_id

core.user_account(user_id)
  ├─ recommend.behavior_event.subject_user_id
  ├─ recommend.subject_profile_snapshot.user_id
  └─ recommend.recommendation_request.subject_user_id

recommend.cohort_definition(cohort_key)
  └─ recommend.cohort_popularity.cohort_key

ecosystem.partner(partner_id)
  └─ recommend.ecosystem_affinity.target_partner_id (V3)
```

## 14. 日志、可观测性与告警关系补充

```text
ops.observability_backend(backend_key)
  ├─ ops.log_retention_policy.storage_backend_key
  ├─ ops.trace_index.backend_key
  ├─ ops.alert_rule.source_backend_key
  ├─ ops.alert_event.source_backend_key
  ├─ ops.slo_definition.source_backend_key
  └─ ops.slo_snapshot.source_backend_key

ops.alert_rule(alert_rule_id)
  ├─ ops.alert_event.alert_rule_id
  └─ ops.slo_definition.alert_rule_id

ops.incident_ticket(incident_ticket_id)
  ├─ ops.alert_event.incident_ticket_id
  └─ ops.incident_event.incident_ticket_id

ops.trace_index(trace_index_id)
  └─ links to trade/payment/audit/ops objects by ref_type + ref_id + trace_id

ops.system_log(system_log_id)
  └─ links to business objects by request_id / trace_id / object_type + object_id
```

## 16. 敏感数据治理增量 ER 图

```text
catalog.asset_version
  ├─1 catalog.sensitive_handling_policy
  ├─< contract.legal_basis_evidence
  └─< catalog.safe_preview_artifact

trade.order_main
  ├─< delivery.sensitive_execution_policy
  ├─< delivery.attestation_record
  ├─< delivery.result_disclosure_review
  └─< delivery.destruction_attestation

delivery.query_execution_run
  ├─< delivery.attestation_record
  ├─< delivery.result_disclosure_review
  └─< delivery.privacy_budget_ledger

ml.compute_task
  └─< delivery.privacy_budget_ledger
```

## 17. 交易链监控、公平性与信任安全关系补充

```text
ops.monitoring_policy_profile(monitoring_policy_profile_id)
  └─< ops.trade_lifecycle_checkpoint

trade.order_main(order_id)
  ├─< ops.trade_lifecycle_checkpoint
  ├─< ops.external_fact_receipt
  ├─< risk.fairness_incident
  └─< ops.chain_projection_gap

ops.trade_lifecycle_checkpoint(trade_lifecycle_checkpoint_id)
  └─< risk.fairness_incident.source_checkpoint_id

ops.external_fact_receipt(external_fact_receipt_id)
  └─< risk.fairness_incident.source_receipt_id

ops.outbox_event(outbox_event_id)
  └─< ops.chain_projection_gap

chain.chain_anchor(chain_anchor_id)
  └─< ops.chain_projection_gap

crosschain.cross_chain_request(cross_chain_request_id)
  ├─< ops.trade_lifecycle_checkpoint (V3)
  ├─< ops.external_fact_receipt (V3)
  └─< risk.fairness_incident (V3)

ml.compute_task(task_id)
  ├─< ops.trade_lifecycle_checkpoint (V2)
  ├─< ops.external_fact_receipt (V2)
  └─< risk.fairness_incident (V2)
```
