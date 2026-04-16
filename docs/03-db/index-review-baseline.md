# 索引审查基线（DB-033）

## 范围

本基线聚焦四类高频路径：

1. 目录搜索回查（catalog/search）
2. 订单详情与交付回查（trade/delivery）
3. 审计联查（audit）
4. 运营任务列表（ops）

## 基线索引清单

| 场景 | 表 | 索引 |
| --- | --- | --- |
| 商品全文检索 | `search.product_search_document` | `idx_product_search_document_tsv` |
| 商品向量检索 | `search.product_search_document` | `idx_product_search_document_embedding` |
| 标签回查商品 | `catalog.product_tag` | `idx_product_tag_tag_id` |
| 订单列表 | `trade.order_main` | `idx_order_main_status_created_at` |
| 订单详情子项回查 | `trade.order_line` | `idx_order_line_order_id` |
| 订单授权回查 | `trade.authorization_grant` | `idx_authorization_grant_order_id` |
| 订单交付记录回查 | `delivery.delivery_record` | `idx_delivery_record_order_id` |
| 订单交付票据回查 | `delivery.delivery_ticket` | `idx_delivery_ticket_order_id` |
| 审计对象联查 | `audit.audit_event` | `idx_audit_event_ref` |
| 审计追踪联查 | `audit.audit_event` | `idx_audit_event_trace` |
| outbox 拉取队列 | `ops.outbox_event` | `idx_outbox_pending`, `idx_outbox_topic_pending` |
| 运维任务列表 | `ops.job_run` | `idx_job_run_status_started` |

## 自动审查入口

- 脚本：`db/scripts/review-index-baseline.sh`
- 机制：通过 `to_regclass(schema.index_name)` 断言索引存在。

## 变更原则

- 新增高频查询路径时，需先补充本文件，再补 migration 索引。
- 索引命名统一：`idx_<table>_<query-shape>`。
- 不允许只改业务 SQL、不补索引基线。
