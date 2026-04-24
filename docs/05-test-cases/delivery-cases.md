# Delivery Cases (DLV-028)

`TEST-012` 的正式回归入口为 `ENV_FILE=infra/docker/.env.local ./scripts/check-delivery-revocation.sh`。该 checker 会复用本文件 `4.4 撤权后访问` 的冻结口径，并继续验证旧下载 token 失效、share/API/sandbox 正式入口被拒绝，以及 `Redis / PostgreSQL / audit` 的同步断权证据。

## 1. 目标与范围

本矩阵用于冻结 `Delivery / Storage Gateway / Query Execution` 子域在 V1 阶段的五类关键异常/边界用例，并把现有自动化 smoke、仓储保护逻辑和真实接口验证映射到统一回归基线：

- 交付超时
- 重复开通
- 票据过期
- 撤权后访问
- 验收失败

范围覆盖：

- `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / SBX_STD / RPT_STD / QRY_LITE`
- 交付接口、下载票据/下载接口、验收接口、自动断权逻辑
- PostgreSQL、MinIO、Redis、Outbox/Kafka 联动验证证据

不包含：

- 独立调度器或 cron 型“超时补偿任务”设计
- V2/V3 预留的敏感交付强化策略
- 非标准 SKU 或跨阶段编排扩展

## 2. 关联冻结约束

- 业务流程：`docs/业务流程/业务流程图-V1-完整版.md`（4.4 交付、验真与验收主流程）
- 测试策略：`docs/data_trading_blockchain_system_design_split/15-测试策略、验收标准与实施里程碑.md`（15.1 测试策略）
- 页面覆盖：`docs/页面说明书/页面说明书-V1-完整版.md`（14. 页面覆盖校验）
- 实现路由：`apps/platform-core/src/modules/delivery/api/mod.rs`
- 自动化证据：
  - `apps/platform-core/src/modules/delivery/tests/dlv004_download_validation_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv017_report_delivery_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv018_acceptance_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv021_auto_cutoff_resources_db.rs`
  - `apps/platform-core/src/modules/delivery/tests/dlv025_delivery_integration_db.rs`

## 3. 自动化证据总览

| 用例 | 当前实现口径 | 关键接口 / 触发点 | 预期结果 | 自动化证据 |
| --- | --- | --- | --- | --- |
| 交付超时 | 当前代码基线用“访问窗口到期/资源自动断权”表达交付超时 | `POST /api/v1/orders/{id}/share-ro/transition` `expire_share`；`POST /api/v1/orders/{id}/sbx-std/transition` `expire_sandbox` | 资源变为 `expired`，关联 `delivery_record` 同步到期，访问入口不再可用 | `dlv021_auto_cutoff_resources_db.rs` |
| 重复开通 | 交付提交、报告交付、验收都必须幂等 | `POST /api/v1/orders/{id}/deliver`；`POST /api/v1/orders/{id}/accept` | 返回 `already_committed` / `already_accepted`，不产生重复资源 | `dlv017_report_delivery_db.rs`、`dlv018_acceptance_db.rs` |
| 票据过期 | 下载票据依赖 Redis 缓存 + DB `expire_at` 双重保护 | `GET /api/v1/orders/{id}/download-ticket`；`GET /api/v1/orders/{id}/download` | 过期或失效返回 `409`，消息含 `ticket expired` 或 `ticket cache not found or expired` | `download_ticket_repository.rs`、`download_file_repository.rs`、`dlv021_auto_cutoff_resources_db.rs`、`dlv004_download_validation_db.rs` |
| 撤权后访问 | 退款、到期、争议、风控冻结后必须切断下载/API/共享/沙箱资源 | 文件退款、`expire_share`、`interrupt_dispute`、`disable_access`、`expire_sandbox` | 下载票据失效；共享/API/沙箱资源变为 `revoked / expired / suspended` | `dlv021_auto_cutoff_resources_db.rs` |
| 验收失败 | 买方拒收必须阻断结算并打开争议 | `POST /api/v1/orders/{id}/reject` | 订单到 `rejected`，`settlement_status=blocked`，`dispute_status=open` | `dlv018_acceptance_db.rs`、`dlv025_delivery_integration_db.rs` |

## 4. 用例明细

### 4.1 交付超时

当前 V1 代码基线没有单独的“后台超时工单”对象；交付超时通过“资源访问窗口到期后自动断权”来落实。

覆盖口径：

- `SHARE_RO`：`expire_share` 后共享授权与 `delivery_record` 同步进入 `expired`
- `SBX_STD`：`expire_sandbox` 后工作区、会话、`delivery_record` 同步进入 `expired`
- 文件下载资源：退款/关闭后下载票据缓存即时失效，买方不能继续访问

预期断言：

- 主状态或交付状态命中 `expired`
- 资源本体状态与 `delivery.delivery_record.status` 同步
- 审计命中：
  - `delivery.share.auto_cutoff.expired`
  - `delivery.sandbox.auto_cutoff.expired`
  - 文件链路命中对应 cutoff 审计

自动化证据：

- `dlv021_auto_cutoff_resources_db.rs`

### 4.2 重复开通

重复开通不是“允许重复创建资源”，而是“必须幂等返回已存在结果”。

覆盖口径：

- 报告交付重复提交：返回 `operation=already_committed`
- 验收重复通过：返回 `operation=already_accepted`

预期断言：

- 不新增第二条交付资源
- 返回第一次创建的稳定资源标识
- 审计不重复膨胀为多次成功开通

自动化证据：

- `dlv017_report_delivery_db.rs`
- `dlv018_acceptance_db.rs`

### 4.3 票据过期

票据校验分两层：

- Redis 缓存层：用于快速校验令牌是否仍在有效窗口
- PostgreSQL 层：最终以 `delivery.delivery_ticket.expire_at / status / download_count` 为准

当前已覆盖两类失败路径：

- 票据缓存失效或被断权删除：返回 `DOWNLOAD_TICKET_FORBIDDEN: ticket cache not found or expired`
- 票据在数据库中已经过期：返回 `DOWNLOAD_TICKET_FORBIDDEN: ticket expired`

相关保护：

- 到期或下载次数耗尽时会把 `delivery_ticket.status` 更新为 `expired / exhausted`
- Redis 缓存同步删除，避免脏票据继续可用

自动化证据：

- `dlv021_auto_cutoff_resources_db.rs`
- `dlv004_download_validation_db.rs`
- `apps/platform-core/src/modules/delivery/repo/download_ticket_repository.rs`
- `apps/platform-core/src/modules/delivery/repo/download_file_repository.rs`

### 4.4 撤权后访问

撤权后访问是 DLV 阶段最关键的安全边界之一。当前代码基线已覆盖四类资源：

- 文件下载票据：退款或关闭后立刻 `revoked`
- 共享授权：到期变 `expired`，争议中断变 `suspended`
- API credential：风控冻结后变 `suspended`
- 沙箱工作区/会话：到期后变 `expired`

预期断言：

- 原访问凭证不再可用
- 关联 `delivery_record` 同步为 `revoked / expired / suspended`
- 下载/API/共享/沙箱入口再次访问返回冲突或失效

自动化证据：

- `dlv021_auto_cutoff_resources_db.rs`

### 4.5 验收失败

验收失败当前以报告链路最清晰，拒收后会同时推进订单与交付快照：

- `current_state = rejected`
- `acceptance_status = rejected`
- `settlement_status = blocked`
- `dispute_status = open`

预期断言：

- `delivery.delivery_record.status = rejected`
- `trust_boundary_snapshot.acceptance.reason_code` 被稳定记录
- 审计命中 `delivery.reject`

自动化证据：

- `dlv018_acceptance_db.rs`
- `dlv025_delivery_integration_db.rs`

## 5. 建议执行顺序

1. 先跑交付安全边界：`dlv021_auto_cutoff_resources_db`
2. 再跑下载票据校验：`dlv004_download_validation_db`
3. 再跑重复幂等：`dlv017_report_delivery_db`、`dlv018_acceptance_db`
4. 最后跑交付集成总链路：`dlv025_delivery_integration_db`

## 6. 最小联调模板

### 6.1 票据过期

```text
1. 建立 FILE_STD 已交付订单
2. 调用 GET /api/v1/orders/{id}/download-ticket
3. 记录 ticket_id / download_token
4. 将 delivery.delivery_ticket.expire_at 改为过去时间
5. 调用 GET /api/v1/orders/{id}/download?ticket=...
6. 期望 HTTP 409，消息包含 ticket expired
7. 回查 delivery.delivery_ticket.status = expired
```

### 6.2 验收失败

```text
1. 建立 RPT_STD 已 report_delivered 订单
2. 调用 POST /api/v1/orders/{id}/reject
3. 期望 HTTP 200
4. 回查 trade.order_main = rejected / blocked / open
5. 回查 delivery.delivery_record.status = rejected
6. 回查 audit.audit_event 命中 delivery.reject
```

## 7. 页面与回归引用

本矩阵直接支撑以下页面与联调场景：

- 订单详情页
- 交付页
- 验收页
- 开发者调试页
- 审计联查页

后续任务如 `DLV-029` 的交付任务自动创建器、`DLV-031` 的按 SKU 验收触发矩阵，应优先复用本文件中的异常/边界断言，而不是重新定义失败语义。
