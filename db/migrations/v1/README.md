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
- 种子执行入口：`db/scripts/seed-runner.sh`
  - 支持命令：`up`
  - 支持 `--manifest`、`--dry-run`
  - 执行记录表：`public.seed_history`（记录 `version`、`name`、`checksum_sha256`、`executed_at`）

## 验证脚本

- `db/scripts/verify-migration-001.sh`：验证 `001_extensions_and_schemas.sql` 的扩展、schema 与公共 trigger 函数基座。
- `db/scripts/verify-migration-010-030.sh`：验证 `010/020/025/030` 的关键表、索引、触发器与外键约束基线。
- `db/scripts/verify-migration-040-056.sh`：验证 `040/050/055/056` 的关键表、索引、触发器与关键约束基线。
- `db/scripts/verify-migration-057-060.sh`：验证 `057/058/059/060` 的搜索/推荐/观测核心对象与鉴权种子数据基线。
- `db/scripts/verify-migration-061-064.sh`：验证 `061/062/063/064` 的对象家族与交易方式、元信息契约、原样加工流水线、分层存储对象与关键字段基线。
- `db/scripts/verify-migration-065-068.sh`：验证 `065/066/067/068` 的查询执行面、敏感受控交付、交易链监控对象与监控权限映射基线。
- `db/scripts/verify-migration-070.sh`：验证 `070` 角色权限最终种子与关键映射基线。
- `db/scripts/verify-migration-roundtrip.sh`：执行“全量升级 -> 全量降级 -> 全量升级”的回滚演练，校验本地重建自洽性。
- `db/scripts/verify-seed-001.sh`：验证 `db/seeds/001_base_lookup.sql` 的基础枚举/类目/标签种子落地。
