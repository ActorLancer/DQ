# Service Runtime Map（CORE-046）

本文件冻结 `V1-Core` 运行时拓扑，明确共享 crate、业务模块与外围进程的所有权及同步/异步边界。

## 运行时总览

- 主应用：`apps/platform-core`（模块化单体，承载 V1 主交易闭环）
- 外围独立进程：
  - `apps/fabric-adapter`（链写入适配与回执回写）
  - `workers/outbox-publisher`（outbox 事件发布）
  - `apps/search-indexer`（搜索索引构建）
  - `services/notification-service`（通知下发）
- 中间件：PostgreSQL、Redis、Kafka、MinIO、OpenSearch、Keycloak

## 共享 Crate 所有权与边界

| crate | 所有权 | 同步边界 | 异步边界 |
| --- | --- | --- | --- |
| `crates/kernel` | `platform-core` | 共享类型、ID/时间、错误模型、模块生命周期、容器 | 仅提供进程内事件总线，不承载跨进程投递 |
| `crates/config` | `platform-core` | 运行模式、provider 模式、特性开关与配置装载 | 无 |
| `crates/http` | `platform-core` | HTTP 入口、请求中间件链、标准响应、健康/运行态端点 | 无 |
| `crates/db` | `platform-core` | DB 连接、事务模板、仓储 trait | 跨进程副作用通过 outbox 间接触发 |
| `crates/auth` | `platform-core` | 会话解析、权限门面、step-up 网关占位 | 外部 IAM 调用可异步审计，但不在本 crate 内实现 |
| `crates/audit-kit` | `platform-core` | 审计事件模型与写入接口 | 审计锚定由异步链路执行 |
| `crates/outbox-kit` | `platform-core` | outbox envelope、幂等键、重试状态模型 | 由 `outbox-publisher` 消费并发布 Kafka |
| `crates/provider-kit` | `platform-core` | KYC/签章/支付/通知/Fabric provider trait 与 mock/real 入口 | 真正外部调用由实现方执行，主链路不阻塞跨进程结果 |

## 业务模块所有权与边界

| 模块组 | 模块 | 所有权 | 同步边界 | 异步边界 |
| --- | --- | --- | --- | --- |
| 身份与主体 | `iam` / `party` / `access` | `platform-core` | 身份、主体、权限状态写入 PostgreSQL | 审计/通知事件异步分发 |
| 供给侧 | `catalog` / `contract_meta` / `listing` / `review` | `platform-core` | 商品、模板、上架、审核状态写入 PostgreSQL | 搜索/推荐投影事件异步分发 |
| 交易主链路 | `order` / `contract` / `authorization` / `delivery` | `platform-core` | 下单、签约、授权、交付状态推进 | 链写入、通知、索引同步异步执行 |
| 账务与争议 | `billing` / `dispute` | `platform-core` | 账务、退款、争议状态推进 | 支付回调与后续广播异步处理 |
| 治理与一致性 | `audit` / `consistency` / `fairness` | `platform-core` | 审计落库、一致性登记 | 审计锚定、告警异步执行 |
| 搜索推荐与运维 | `search` / `recommendation` / `developer` / `ops` | `platform-core` | 查询最终放行与运维治理操作 | 索引构建、推荐行为处理异步执行 |

## 请求级与运行态端点归属

- 请求级中间件链由 `crates/http` 收敛：
  - `request_id`
  - `trace`
  - `tenant`
  - `idempotency`
  - `access-log`
- 健康与运行态端点由 `crates/http` 收敛：
  - `/health/live`
  - `/health/ready`
  - `/health/deps`
  - `/internal/runtime`

## 强约束

- `PostgreSQL` 是业务主状态权威；`Kafka` 不是业务真值。
- 跨进程副作用必须通过 `outbox_event -> Kafka -> worker/adapter` 主路径触发。
- 搜索与推荐结果在放行前必须回 PostgreSQL 复核可见性与状态。
- `V2/V3` 能力仅允许接口/trait 预留，不在本文件映射为正式运行时职责。
