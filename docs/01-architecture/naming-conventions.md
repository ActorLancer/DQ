# 命名规范（BOOT-018）

## 代码与工程对象

- crate/package：使用 `kebab-case`，例如 `platform-core`, `domain-types`。
- Rust module/file：使用 `snake_case`。
- Service 目录：使用 `kebab-case`，与服务边界文档一致。
- 环境变量：使用 `UPPER_SNAKE_CASE`，按域前缀分组，例如 `IAM_*`, `TRADE_*`。

## API 与事件

- REST 路径：资源名复数 + `kebab-case`，例如 `/api/v1/orders/{id}`。
- JSON 字段：统一 `snake_case`。
- 事件 Topic：`domain.entity.action.v1`，例如 `trade.order.created.v1`。
- 事件名与错误码前缀必须与领域一致，不允许跨域复用歧义命名。

## 数据库对象

- 表名：`snake_case` 复数，按域前缀分组（如 `trade_orders`）。
- 主键：统一 `id`；外键使用 `{entity}_id`。
- 时间字段：`created_at`, `updated_at`, `deleted_at`。
- 唯一索引命名：`uk_{table}_{columns}`；普通索引命名：`idx_{table}_{columns}`。

## 文档与任务

- 架构文档文件名使用 `kebab-case`，与任务清单 `deliverable_path` 对齐。
- 批次编号统一 `BATCH-XXX`。
- 代码 TODO 必须使用 `TODO(V1-gap|V2-reserved|V3-reserved, TASK-ID): ...` 格式。
