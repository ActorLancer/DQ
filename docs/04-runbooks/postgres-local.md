# PostgreSQL 本地初始化与自检

## 初始化来源

- `infra/postgres/initdb/001_extensions_and_schemas.sql`
- `infra/postgres/initdb/002_roles_and_grants.sql`
- `infra/postgres/postgresql.conf`
- `infra/postgres/pg_hba.conf`

## 启动后检查

1. 执行 `db/scripts/check-db-ready.sh`。
2. 确认 schema：`iam/catalog/trade/delivery/billing/audit/ops`。
3. 确认扩展：`pgcrypto`、`uuid-ossp`。

## 迁移验证

- 执行 `./scripts/validate_database_migrations.sh`，运行 `TEST-004` 正式 migration smoke：
  - 启动 current local core stack
  - 初始化 MinIO buckets
  - 执行 migration/seed roundtrip
  - 最终启动 `platform-core-bin` 并回查健康端点
