# AUD-023 观测总览 / 日志导出 / Trace 联查

正式接口：

- `GET /api/v1/ops/observability/overview`
- `GET /api/v1/ops/logs/query`
- `GET /api/v1/ops/logs`（兼容别名）
- `POST /api/v1/ops/logs/export`
- `GET /api/v1/ops/traces/{traceId}`
- `GET /api/v1/ops/alerts`
- `GET /api/v1/ops/incidents`
- `GET /api/v1/ops/slos`

正式目标：

- `ops.observability_backend / ops.alert_event / ops.incident_ticket / ops.trace_index / ops.system_log / ops.slo_*` 作为正式读模型来源
- `overview` 必须真实探测 `Prometheus / Alertmanager / Grafana / Loki / Tempo / OTel Collector`
- `logs/export` 必须要求 `ops.log.export + step-up`，并把导出对象真实写入 `MinIO(report-results)`
- 查询与导出动作必须真实写入 `audit.access_audit`；导出动作还必须写入 `audit.audit_event`
- 所有接口都必须写入 `ops.system_log`

边界：

- 观测域只负责运行态联查与治理，不替代 `audit.*` 证据域
- `logs/query` 与 `logs/export` 的正式权威对象是 PostgreSQL 镜像 `ops.system_log`
- `Tempo / Grafana / Loki / Alertmanager / Prometheus` 在本任务中必须真实可探测，但不是主权威源

## 权限

- `GET /api/v1/ops/observability/overview`
  - `ops.observability.read`
- `GET /api/v1/ops/logs/query` / `GET /api/v1/ops/logs`
  - `ops.log.query`
- `POST /api/v1/ops/logs/export`
  - `ops.log.export`
  - 必须带 `x-step-up-token` 或 verified `x-step-up-challenge-id`
- `GET /api/v1/ops/traces/{traceId}`
  - `ops.trace.read`
- `GET /api/v1/ops/alerts`
  - `ops.alert.read`
- `GET /api/v1/ops/incidents`
  - `ops.incident.read`
- `GET /api/v1/ops/slos`
  - `ops.slo.read`

当前已落地平台角色：

- `platform_admin`
- `platform_audit_security`

## 宿主机启动

先启动本地观测栈：

```bash
set -a
source infra/docker/.env.local
set +a

docker compose --env-file infra/docker/.env.local -f infra/docker/docker-compose.local.yml --profile observability up -d
./scripts/check-observability-stack.sh
```

再启动 `platform-core`：

```bash
APP_PORT=18080 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
cargo run -p platform-core-bin
```

确认宿主机指标端点已经暴露：

```bash
curl -fsS http://127.0.0.1:18080/metrics | rg 'platform_core_http_(requests_total|request_duration_seconds)'
curl -fsS http://127.0.0.1:8089/metrics | rg 'mock_payment_provider_up'
curl -fsS 'http://127.0.0.1:9090/api/v1/query?query=up{job="platform-core"}'
curl -fsS 'http://127.0.0.1:9090/api/v1/query?query=up{job="mock-payment-provider"}'
```

如需让 overview 中的关键服务状态返回 `up`，还应同时启动：

- `notification-worker`
- `outbox-publisher`

## 快速真实验证

优先执行本批 live smoke：

```bash
AUD_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core observability_api_db_smoke -- --nocapture
```

该 smoke 会真实：

- 写入 `ops.system_log / ops.trace_index / ops.alert_event / ops.incident_ticket / ops.slo_*`
- 调用 `overview / logs/query / logs/export / traces/{traceId} / alerts / incidents / slos`
- 回查 `MinIO(report-results)` 导出对象、`audit.audit_event / audit.access_audit / ops.system_log`
- 校验 `trace_id / request_id / object_id` 可串联

## 手工调用

### 1. 观测总览

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/observability/overview" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud023-overview' \
  -H 'x-trace-id: trace-aud023-overview'
```

重点确认：

- `backend_statuses[*].backend.backend_key` 至少覆盖 `prometheus_main / alertmanager_main / grafana_main / loki_main / tempo_main`
- `key_services[*].service_name` 返回 `platform-core / notification-worker / outbox-publisher`
- `slo_summary.items[*]` 来自正式 `ops.slo_definition + ops.slo_snapshot`

### 2. 日志镜像查询

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/logs/query?trace_id=<trace_id>&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud023-logs' \
  -H 'x-trace-id: trace-aud023-logs'
```

兼容别名：

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/logs?trace_id=<trace_id>&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud023-logs-alias'
```

### 3. 原始日志导出

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/ops/logs/export" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud023-export' \
  -H 'x-trace-id: <trace_id>' \
  -H 'x-step-up-challenge-id: <log_export_step_up_id>' \
  -d '{
    "reason": "incident triage export",
    "trace_id": "<trace_id>"
  }'
```

重点确认：

- `bucket_name=report-results`
- `object_key` 指向 `ops/log-exports/<export_id>.json`
- `step_up_bound=true`

### 4. Trace 联查

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/traces/<trace_id>" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud023-trace'
```

重点确认：

- `trace.backend_key=tempo_main`
- `tempo_link` 与 `grafana_link` 可用于跳转
- `related_log_count / related_alert_count` 可联查

### 5. 告警 / 事故 / SLO

```bash
curl -sS "http://127.0.0.1:18080/api/v1/ops/alerts?severity=high&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud023-alerts'

curl -sS "http://127.0.0.1:18080/api/v1/ops/incidents?owner_role_key=platform_audit_security&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud023-incidents'

curl -sS "http://127.0.0.1:18080/api/v1/ops/slos?service_name=platform-core&page=1&page_size=20" \
  -H 'x-role: platform_audit_security' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud023-slos'
```

## SQL 回查

导出动作审计：

```sql
SELECT action_name,
       result_code,
       step_up_challenge_id::text,
       metadata
FROM audit.audit_event
WHERE request_id = 'req-aud023-export'
  AND action_name = 'ops.log.export';
```

访问留痕：

```sql
SELECT request_id,
       access_mode,
       target_type,
       target_id::text,
       step_up_challenge_id::text
FROM audit.access_audit
WHERE request_id IN (
  'req-aud023-overview',
  'req-aud023-logs',
  'req-aud023-export',
  'req-aud023-trace',
  'req-aud023-alerts',
  'req-aud023-incidents',
  'req-aud023-slos'
)
ORDER BY created_at, access_audit_id;
```

系统日志留痕：

```sql
SELECT request_id, message_text, trace_id, structured_payload
FROM ops.system_log
WHERE request_id IN (
  'req-aud023-overview',
  'req-aud023-logs',
  'req-aud023-export',
  'req-aud023-trace',
  'req-aud023-alerts',
  'req-aud023-incidents',
  'req-aud023-slos'
)
ORDER BY created_at, system_log_id;
```

Trace / Alert / Incident / SLO 对象：

```sql
SELECT trace_id, backend_key, root_service_name, root_span_name, request_id
FROM ops.trace_index
WHERE trace_id = '<trace_id>';

SELECT alert_event_id::text, severity, status, trace_id, request_id
FROM ops.alert_event
WHERE trace_id = '<trace_id>';

SELECT incident_ticket_id::text, incident_key, status, owner_role_key
FROM ops.incident_ticket
ORDER BY created_at DESC, incident_ticket_id DESC
LIMIT 5;

SELECT slo_definition_id::text, slo_key, service_name, status
FROM ops.slo_definition
ORDER BY created_at DESC, slo_definition_id DESC
LIMIT 5;
```

## MinIO 回查

`logs/export` 响应会返回 `bucket_name` 与 `object_key`。至少确认：

- `bucket_name=report-results`
- 对象实际存在于 MinIO `report-results`
- JSON 内 `exported_count` 与接口返回一致

如需图形化检查，可通过 MinIO Console `http://127.0.0.1:9001` 进入 `report-results` bucket 回查对象。

## 观测联查

- `Prometheus`：`curl -fsS 'http://127.0.0.1:9090/api/v1/query?query=up{job="platform-core"}'`
- `Grafana`：`curl -fsS -u admin:admin123456 'http://127.0.0.1:3000/api/search?query=Platform%20Overview'`
- `Tempo`：使用 trace lookup 返回的 `tempo_link`
- `Loki`：在 Grafana Explore 里检索 `request_id=req-aud023-export` 或 `trace_id=<trace_id>`
- `Alertmanager`：`curl -fsS 'http://127.0.0.1:9093/api/v2/alerts'`

## 排障

1. 返回 `400`

- 先检查 `x-request-id`
- `logs/export` 还要检查 `reason` 与至少一个 selector（如 `trace_id`）
- 缺少 `x-step-up-token / x-step-up-challenge-id` 时，`logs/export` 应直接失败

2. 返回 `403`

- 检查角色是否具备 `ops.observability.read / ops.log.query / ops.log.export / ops.trace.read / ops.alert.read / ops.incident.read / ops.slo.read`
- `logs/export` 还要确认 step-up challenge 属于当前 `user_id` 且状态为 `verified`

3. 返回 `404`

- `GET /api/v1/ops/traces/{traceId}` 仅在 `ops.trace_index` 命中时返回

4. 导出对象缺失

- 先回查 `audit.audit_event(action_name='ops.log.export')`
- 再回查同一 `request_id` 的 `ops.system_log`
- 确认 `report-results` bucket 已初始化：`./infra/minio/init-minio.sh`

5. overview 中 backend 变成 `down`

- 先跑 `./scripts/check-observability-stack.sh`
- 再检查对应容器端口与 `docker ps`
- `platform-core / notification-worker / outbox-publisher` 的 `key_services` 依赖 `Prometheus up{job="..."}`，若宿主机进程未启动会显示 `unknown`
