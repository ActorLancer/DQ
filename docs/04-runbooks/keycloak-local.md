# Keycloak Realm Import（ENV-018/019）

## Realm 文件

- `infra/keycloak/realm-export/platform-local-realm.json`

## 导入机制

- `docker-compose.local.yml` 中 `keycloak` 使用 `start-dev --import-realm`
- 挂载路径：`/opt/keycloak/data/import`
- Keycloak 使用独立数据库 `KEYCLOAK_DB_NAME=keycloak`，避免执行 `db/scripts/migrate-reset.sh` 重建业务库 `datab` 时误删 IAM / realm 表。

## 正式本地角色与用户

本地 realm 现在直接对齐平台正式核心角色键，不再以连字符临时角色作为 authority。当前本地演示用户至少包括：

| 用户名 | 密码 | 正式角色 | 绑定业务 `user_id` | 绑定 `org_id` |
| --- | --- | --- | --- | --- |
| `local-platform-admin` | `LocalPlatformAdmin123!` | `platform_admin` | `10000000-0000-0000-0000-000000000353` | `10000000-0000-0000-0000-000000000103` |
| `local-audit-security` | `LocalAuditSecurity123!` | `platform_audit_security` | `10000000-0000-0000-0000-000000000354` | `10000000-0000-0000-0000-000000000103` |
| `local-platform-reviewer` | `LocalPlatformReviewer123!` | `platform_reviewer` | `10000000-0000-0000-0000-000000000358` | `10000000-0000-0000-0000-000000000103` |
| `local-risk-settlement` | `LocalRiskSettlement123!` | `platform_risk_settlement` | `10000000-0000-0000-0000-000000000359` | `10000000-0000-0000-0000-000000000103` |
| `local-tenant-admin` | `LocalTenantAdmin123!` | `tenant_admin` | `10000000-0000-0000-0000-000000000352` | `10000000-0000-0000-0000-000000000102` |
| `local-tenant-developer` | `LocalTenantDeveloper123!` | `tenant_developer` | `10000000-0000-0000-0000-000000000355` | `10000000-0000-0000-0000-000000000102` |
| `local-buyer-operator` | `LocalBuyerOperator123!` | `buyer_operator` | `10000000-0000-0000-0000-000000000356` | `10000000-0000-0000-0000-000000000102` |
| `local-seller-operator` | `LocalSellerOperator123!` | `seller_operator` | `10000000-0000-0000-0000-000000000357` | `10000000-0000-0000-0000-000000000101` |

`portal-web` token 会额外携带正式 `user_id` / `org_id` claims，供 `platform-core` 在 Bearer 鉴权、step-up 和审计链里直接使用。

## 运行态重置 / 重导入

如果 realm 导入被旧 DB 残留污染，或 password grant 返回 `Unable to find factory for Required Action 'null'` 之类错误，必须使用正式重置脚本：

```bash
make keycloak-reset-local
```

等价命令：

```bash
COMPOSE_FILE=infra/docker/docker-compose.local.yml \
COMPOSE_ENV_FILE=infra/docker/.env.local \
./scripts/reset-keycloak-local.sh
```

该脚本会：

1. 停止 `datab-keycloak`
2. 重建独立 `KEYCLOAK_DB_NAME`
3. 重新启动 `keycloak`
4. 用真实 password grant 校验 realm、角色 claim 与 `user_id/org_id` claims

## 导入后验证

```bash
./scripts/check-keycloak-realm.sh
```

该校验不再只检查 `issuer`，还必须真实验证：

- `portal-web` password grant 可返回 `access_token`
- token 的 `realm_access.roles` 包含正式核心角色
- token 携带正式 `user_id` / `org_id` claims

手工获取本地 `platform_admin` Bearer 示例：

```bash
curl -sS -X POST \
  'http://127.0.0.1:8081/realms/platform-local/protocol/openid-connect/token' \
  -H 'content-type: application/x-www-form-urlencoded' \
  --data-urlencode 'grant_type=password' \
  --data-urlencode 'client_id=portal-web' \
  --data-urlencode 'username=local-platform-admin' \
  --data-urlencode 'password=LocalPlatformAdmin123!'
```
