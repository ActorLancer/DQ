# 数据交易平台数据库设计包

## 1. 目标

本目录基于当前工作区全部核心文档生成，覆盖：

- [正式PRD](/home/luna/Documents/DataB/正式PRD)
- [业务流程](/home/luna/Documents/DataB/业务流程)
- [领域模型](/home/luna/Documents/DataB/领域模型)
- [页面说明书](/home/luna/Documents/DataB/页面说明书)
- [权限设计](/home/luna/Documents/DataB/权限设计)
- [用户角色说明](/home/luna/Documents/DataB/用户角色说明)
- [data_trading_blockchain_system_design_split](/home/luna/Documents/DataB/data_trading_blockchain_system_design_split)
- [原始PRD](/home/luna/Documents/DataB/原始PRD)

目标输出：

- 一套面向 PostgreSQL 最新稳定版的全量数据库设计
- 按 `V1 / V2 / V3` 分阶段演进的迁移脚本
- 每个版本同时提供 `upgrade` 和 `downgrade`
- 一份 AI 易于阅读的全量表关系 / ER 文本图
- 一份正式版数据库表字典
- 同时覆盖：
  - 业务数据表
  - 用户/角色/权限数据表
  - 审计与日志数据表
  - 搜索、倒排、向量检索相关表
  - 模型、训练、证明、分润、跨链、监管、生态互联相关表

## 2. 目标数据库版本

- 目标数据库：`PostgreSQL 18.3`
- 依赖扩展：
  - `pgcrypto`
  - `citext`
  - `pg_trgm`
  - `btree_gist`
  - `vector`

说明：

- `vector` 用于向量检索
- PostgreSQL 自带全文检索与 `GIN/GiST` 用于倒排检索
- PostgreSQL 同时承担商品/卖方/模型搜索投影主表，OpenSearch 作为搜索读模型
- Redis 只承担搜索缓存，不承担权威状态
- 大对象、原始密文、模型二进制、证据原文默认放对象存储，数据库只保存元数据、摘要、索引、审计和小型结构化数据
- 数据库显式建模 `平台托管 / 卖方自持 / 受控执行 / 第三方可信存储` 的信任边界，不把“已登记”误等同于“平台可读明文”

## 3. 目录结构

```text
数据库设计/
  README.md
  数据库设计总说明.md
  表关系总图-ER文本图.md
  数据库表字典正式版.md
  V1/
    upgrade/
    downgrade/
  V2/
    upgrade/
    downgrade/
  V3/
    upgrade/
    downgrade/
```

## 4. 迁移策略

- `V1`：基础身份、商品、订单、交付、账单、争议、审计、搜索、开发者支持
  - 身份部分显式包含：主体、成员、应用、角色权限、邀请、SSO、MFA、设备、会话、Fabric 身份与证书治理
  - 数据对象部分显式包含：可交付对象、版本订阅、只读共享授权、模板查询授权、元信息档案、字段结构说明、质量报告、加工责任链、数据契约
  - 搜索部分显式包含：商品/服务搜索投影、卖方搜索投影、排序配置、别名绑定、索引同步任务
- `V2`：模型、算法、受控计算、联邦协作、证明、分润、公链增强
  - 搜索增量显式包含：模型搜索投影、同义词规则
- `V3`：跨链、图风控、监管穿透、治理冻结、连接器互联、合作伙伴与互认
  - 搜索增量显式包含：伙伴/生态搜索投影

规则：

- `V2` 默认基于 `V1` 已完成升级
- `V3` 默认基于 `V2` 已完成升级
- 每个升级脚本都有对应降级脚本
- 降级按同版本脚本逆序执行

## 5. 新增信任边界落库要点

- `catalog.asset_custody_profile`：定义资产或版本的托管模式、明文边界、交付路由、保留/销毁策略
- `catalog.asset_trust_evidence`：定义卖方自持、第三方托管、执行边界等信任证据
- `catalog.asset_object_binding`：定义某个版本下真正可交付、可共享、可查询的逻辑对象
- `trade.order_main` / `delivery.delivery_record`：快照 `storage_mode`、`delivery_route`、`trust_boundary`
- `delivery.data_share_grant` / `delivery.revision_subscription` / `delivery.template_query_grant`：分别承接零拷贝共享、版本订阅和模板查询授权
- `ml.*`：显式记录模型权重托管方式、训练数据驻留方式、结果导出策略
- `ecosystem.partner` / `crosschain.cross_chain_request`：显式记录跨平台存储能力与责任边界快照

## 6. 新增支付与清结算落库要点

- 新增独立 `payment` schema，承载支付渠道、支付意图、支付交易、Webhook、打款、对账与清结算对象
- `V1` 额外落库起步司法辖区、新加坡走廊策略和卖方收款偏好
- `billing` 继续承载费用规则、费用快照、账单事件、保证金、退款、赔付、分润
- `trade.order_main` 额外快照支付状态、支付方式和费用快照
- `developer` 额外承载 `mock payment` 调试对象，支持开发演练
- `V2` 继续扩展自动打款、渠道分账与周期扣费
- `V3` 再扩展多币种、跨境、数字资产/交易所结算路由

## 7. 新增身份认证与会话落库要点

- 新增独立 `iam` schema，承载邀请、认证方式、MFA、设备、会话、SSO、step-up、Fabric 身份与证书治理
- 平台 IAM 身份与 Fabric 链上身份分层建模，不把链上证书直接作为普通网页登录主凭证
- `V1` 即落主体注册、成员邀请、企业 OIDC、MFA、会话与设备治理骨架
- `V2` 增量补齐 `SAML/SCIM`、更完整的企业身份联邦
- `V3` 再扩展跨平台身份联邦与自适应认证策略

## 8. 新增审计、证据链与回放落库要点

- 在 `audit` schema 下补齐 `EvidenceItem`、`EvidenceManifest`、`AnchorBatch`、`ReplayJob`、`LegalHold`、`AuditAccessRecord`
- `audit.audit_event` 扩展 `event_hash`、`previous_event_hash`、`before/after_state_digest`、`retention_class`、`legal_hold_status`
- `ops.system_log` 扩展 `traceparent`、`log_hash` 和保留状态
- `V1` 通过 `055_audit_hardening.sql` 落强审计基础结构

## 9. 新增双层权威模型与一致性落库要点

- `V1` 通过 `056_dual_authority_consistency.sql` 补齐关键业务对象的 `proof_commit_state / external_fact_status / reconcile_status`
- `ops.outbox_event` 扩展 Kafka 路由、幂等、重试与发布元数据
- 新增 `ops.event_route_policy`、`ops.outbox_publish_attempt`、`ops.consumer_idempotency_record`
- `V2` 通过 `004_dual_authority_consistency.sql` 把训练、计算、分润对象纳入同一模型
- `V3` 通过 `004_dual_authority_consistency.sql` 把跨链请求和数字资产结算对象纳入同一模型

## 10. 新增搜索与索引同步落库要点

- `search.product_search_document` 扩展为商品/服务搜索投影主表
- `search.seller_search_document` 支撑卖方主体搜索与卖方主页
- `search.search_signal_aggregate` 聚合热度、点击、成交等排序特征
- `search.ranking_profile`、`search.index_alias_binding`、`search.index_sync_task` 用于排序治理、别名切换和重建修复
- `V2` 补 `search.model_search_document` 同步字段和 `search.synonym_rule`
- `V3` 补 `search.partner_search_document`

## 10A. 新增数据商品元信息与数据契约落库要点

- `catalog.product_metadata_profile`：统一承接十大元信息域档案
- `catalog.asset_field_definition`：承接字段结构、主键/时间字段、编码规则摘要
- `catalog.asset_quality_report`：承接缺失率、覆盖范围、采样方式、异常率和质量评分
- `catalog.asset_processing_job` / `catalog.asset_processing_input`：承接清洗、脱敏、标注、标准化和结果加工责任链
- `contract.data_contract`：承接交付义务、验收标准、授权边界、责任边界和争议口径
- `contract.digital_contract`：继续作为订单签约合同，但必须引用签约时刻绑定的数据契约摘要

## 10B. 新增数据原样处理与产品化加工落库要点

- `catalog.raw_ingest_batch`：原始接入批次，承接来源、权利声明、接入策略和批次状态
- `catalog.raw_object_manifest`：原始对象清单，承接对象 URI、hash、大小、sidecar manifest 与来源时间范围
- `catalog.format_detection_result`：对象族识别、格式识别与推荐加工路径
- `catalog.extraction_job`：schema 抽取、OCR、ASR、转码、摘要抽取、预览生成等前置任务
- `catalog.preview_artifact`：样例、文本预览、图像预览、query preview 等展示工件
- `catalog.asset_version.processing_stage / standardization_status`：显式表达资产从原始登记到标准化完成的加工阶段

## 10C. 新增数据商品存储与分层存储落库要点

- `catalog.storage_namespace`：抽象 bucket / prefix / namespace 与 `raw / curated / product / preview / delivery / archive / evidence / model` 分层区域
- `catalog.storage_policy_profile`：定义不同对象族默认落在哪些分层区域、采用何种生命周期和查询面
- `catalog.asset_version.storage_policy_id / query_surface_type`：把版本和正式存储策略、查询暴露方式绑定
- `catalog.asset_storage_binding` 增补分层区域、命名空间、存储类与保留截止时间
- `delivery.storage_object` 增补命名空间、分层区域与交付临时对象的保留截止时间

## 10D. 新增数据商品查询与执行面落库要点

- `catalog.query_surface_definition`：定义某个资产版本正式暴露的查询面类型、执行环境、读取范围、输出边界与配额策略
- `delivery.query_template_definition`：定义查询模板版本、参数 schema、analysis rule、结果 schema 与导出策略
- `delivery.query_execution_run`：记录每次模板查询或沙箱查询的请求摘要、结果摘要、结果对象、计费单位和状态
- `delivery.template_query_grant` 增补 `query_surface_id / allowed_template_ids / execution_rule_snapshot`
- `delivery.sandbox_workspace` 增补 `query_surface_id / clean_room_mode`

## 11. 新增推荐与个性化发现落库要点

- 新增独立 `recommend` schema，承载推荐位、推荐排序配置、行为事件、画像快照、推荐请求与推荐结果
- `V1` 通过 `058_recommendation_module.sql` 落：
  - `recommend.placement_definition`
  - `recommend.ranking_profile`
  - `recommend.behavior_event`
  - `recommend.subject_profile_snapshot`
  - `recommend.cohort_definition`
  - `recommend.cohort_popularity`
  - `recommend.entity_similarity`
  - `recommend.bundle_relation`
  - `recommend.recommendation_request`
  - `recommend.recommendation_result`
  - `recommend.recommendation_result_item`
- `V2` 通过 `006_recommendation_module.sql` 补：
  - `recommend.model_registry`
  - `recommend.experiment_assignment`
  - `recommend.model_inference_log`
- `V3` 通过 `006_recommendation_module.sql` 补：
  - `recommend.page_optimization_profile`
  - `recommend.ecosystem_affinity`
- 推荐域与搜索域共享候选基础数据，但 PostgreSQL 继续是推荐行为、结果和配置的权威源

## 12. 新增日志、可观测性与告警落库要点

- `V1` 通过 `059_logging_observability.sql` 落：
  - `ops.observability_backend`
  - `ops.log_retention_policy`
  - `ops.trace_index`
  - `ops.alert_rule`
  - `ops.alert_event`
  - `ops.incident_ticket`
  - `ops.incident_event`
  - `ops.slo_definition`
  - `ops.slo_snapshot`
- `ops.system_log` 扩展为关键日志镜像，不承载 Loki 原始全文日志权威存储。
- 指标、原始日志与 trace 继续分别归属于 `Prometheus`、`Loki`、`Tempo`；PostgreSQL 保存配置、索引、工单和审计联动数据。
