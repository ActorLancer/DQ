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

- 执行 `./scripts/validate_database_migrations.sh`，确保迁移脚本在当前数据库上可执行。
