# Fabric CA Admin（AUD-016）

`AUD-016` 起，`services/fabric-ca-admin/` 作为正式 Go 进程落地，负责执行：

- Fabric 身份签发
- Fabric 身份吊销
- 证书吊销

冻结边界：

- Rust `platform-core` 保留公网 IAM API、权限点、step-up、审计主体与正式错误码
- Go `fabric-ca-admin` 承接执行面、数据库状态变更与 `ops.external_fact_receipt` 回执落盘
- 当前 provider mode 固定 `mock`
- 真实 `Fabric CA / test-network / Gateway` 切换留待 `AUD-017`
- 当前不把 Go 服务反向当成业务主状态机

## 命令入口

```bash
make fabric-ca-admin-bootstrap
make fabric-ca-admin-test
make fabric-ca-admin-run
```

等价脚本：

```bash
./scripts/fabric-ca-admin-bootstrap.sh
./scripts/fabric-ca-admin-test.sh
./scripts/fabric-ca-admin-run.sh
```

Go 依赖缓存统一落在：

```text
third_party/external-deps/go
```

## 本地配置

默认从 `infra/docker/.env.local` 加载：

- `DATABASE_URL`
- `FABRIC_CA_ADMIN_PORT`
- `FABRIC_CA_ADMIN_BASE_URL`
- `FABRIC_CA_ADMIN_MODE`

当前本地默认值：

- PostgreSQL：`postgres://datab:datab_local_pass@127.0.0.1:5432/datab`
- listen：`127.0.0.1:18112`
- base URL：`http://127.0.0.1:18112`
- provider mode：`mock`

## 正式执行链

当前 V1 本地最小闭环为：

```text
step-up -> platform-core IAM API -> fabric-ca-admin -> PostgreSQL -> audit.audit_event / ops.system_log
```

写回口径：

- `iam.fabric_identity_binding.status`
- `iam.certificate_record.status`
- `iam.certificate_revocation_record`
- `ops.external_fact_receipt`
- `ops.system_log`

Rust `platform-core` 同时写入：

- `audit.audit_event(action_name='iam.fabric.identity.issue')`
- `audit.audit_event(action_name='iam.fabric.identity.revoke')`
- `audit.audit_event(action_name='iam.certificate.revoke')`

## 手工 Smoke

1. 启动 `fabric-ca-admin`：

```bash
set -a
source infra/docker/.env.local
set +a

./scripts/fabric-ca-admin-run.sh
```

2. 启动 `platform-core`：

```bash
set -a
source infra/docker/.env.local
set +a

APP_PORT=18080 \
KAFKA_BROKERS=127.0.0.1:9094 \
KAFKA_BOOTSTRAP_SERVERS=127.0.0.1:9094 \
FABRIC_CA_ADMIN_BASE_URL=http://127.0.0.1:18112 \
cargo run -p platform-core-bin
```

3. 执行 live smoke：

```bash
IAM_DB_SMOKE=1 DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab \
  cargo test -p platform-core iam_fabric_ca_admin_db_smoke -- --nocapture
```

4. 若要手工验证，先获取 step-up challenge，再发起签发 / 吊销：

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/iam/step-up/check" \
  -H 'content-type: application/json' \
  -H 'x-role: platform_admin' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud016-step-up' \
  -H 'x-trace-id: trace-aud016-step-up' \
  -d '{
    "target_action": "iam.fabric.identity.issue",
    "ref_type": "fabric_identity_binding",
    "ref_id": "<binding_id>"
  }'
```

校验 challenge 后，再调用：

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/iam/fabric-identities/<binding_id>/issue" \
  -H 'x-role: platform_admin' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud016-issue' \
  -H 'x-trace-id: trace-aud016-issue' \
  -H 'x-step-up-challenge-id: <verified_step_up_id>'
```

证书吊销：

```bash
curl -sS -X POST "http://127.0.0.1:18080/api/v1/iam/certificates/<certificate_id>/revoke" \
  -H 'x-role: platform_admin' \
  -H 'x-user-id: <operator_user_id>' \
  -H 'x-request-id: req-aud016-cert-revoke' \
  -H 'x-trace-id: trace-aud016-cert-revoke' \
  -H 'x-step-up-challenge-id: <verified_step_up_id>'
```

5. 回查数据库：

```sql
SELECT fabric_identity_binding_id::text, status, certificate_id::text
FROM iam.fabric_identity_binding
WHERE fabric_identity_binding_id = '<binding_id>'::uuid;

SELECT certificate_id::text, status, serial_number
FROM iam.certificate_record
WHERE certificate_id = '<certificate_id>'::uuid;

SELECT certificate_id::text, revocation_reason, provider_reference
FROM iam.certificate_revocation_record
WHERE certificate_id = '<certificate_id>'::uuid;

SELECT request_id,
       fact_type,
       receipt_status,
       provider_reference,
       metadata ->> 'event_type' AS event_type
FROM ops.external_fact_receipt
WHERE request_id IN ('req-aud016-issue', 'req-aud016-cert-revoke')
ORDER BY received_at ASC, external_fact_receipt_id ASC;

SELECT request_id, action_name, actor_id::text, metadata ->> 'step_up_challenge_id' AS step_up_challenge_id
FROM audit.audit_event
WHERE request_id IN ('req-aud016-issue', 'req-aud016-cert-revoke')
ORDER BY created_at ASC, audit_event_id ASC;

SELECT request_id, message_text, structured_payload
FROM ops.system_log
WHERE request_id IN ('req-aud016-issue', 'req-aud016-cert-revoke')
ORDER BY created_at ASC, system_log_id ASC;
```

预期：

- 签发后 `iam.fabric_identity_binding.status='issued'`
- 吊销后 `iam.certificate_record.status='revoked'`，并存在 `iam.certificate_revocation_record`
- `ops.external_fact_receipt` 中出现：
  - `certificate_issue_receipt / ca.certificate_issued`
  - `certificate_revocation_receipt / ca.certificate_revoked`
- `audit.audit_event` 中出现真实操作者主体
- `ops.system_log` 中出现：
  - `fabric ca admin issued identity`
  - `fabric ca admin revoked certificate`

## 排障

- 若 `platform-core` 返回 `FABRIC_CA_ADMIN_UNAVAILABLE`，先检查 `curl http://127.0.0.1:18112/healthz` 是否可达，再确认 `FABRIC_CA_ADMIN_BASE_URL` 是否和 `platform-core` 运行时一致。
- 若返回 `STEP_UP_REQUIRED` 或 `STEP_UP_INVALID`，先回查 `iam.step_up_challenge.target_action / ref_type / ref_id` 是否与当前目标对象匹配。
- 若签发返回 `409`，优先检查 `iam.fabric_identity_binding.status` 是否已是 `issued / revoked`，以及本批是否按冻结口径先完成审批，使其处于 `approved`。
- `audit.audit_event` 与 `ops.system_log` 为 append-only；只清理业务测试数据，不清理这两类记录。
