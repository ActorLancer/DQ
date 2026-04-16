# V1 Migration 执行基线（DB-001）

## 命名与顺序规则

- 迁移版本号使用三位数字前缀：`NNN_*`
- 执行顺序以 `manifest.csv` 的 `version` 升序为准
- 回滚顺序以 `manifest.csv` 的 `version` 降序为准
- 版本号跨域不可复用

## 文件说明

- 执行清单：`db/migrations/v1/manifest.csv`
- checksum 锁文件：`db/migrations/v1/checksums.sha256`
- SQL 基线来源：`docs/数据库设计/V1/{upgrade,downgrade}/*.sql`

## Runner 规则

- 执行入口：`db/scripts/migration-runner.sh`
- 支持命令：
  - `up`：按版本升序执行 upgrade
  - `down`：按版本降序执行 downgrade
  - `status`：查看已执行记录和待执行版本
- 支持 `--dry-run`，只打印将执行的文件，不改数据库
- 执行记录表：`public.schema_migration_history`
  - 记录字段：`version`、`name`、`direction`、`checksum_sha256`、`executed_at`
  - 用于检测同版本 checksum 漂移
