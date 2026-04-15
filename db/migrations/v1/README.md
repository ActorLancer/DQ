# V1 Migration 命名规则（BOOT-008）

目录约定：

- 升级脚本：`db/migrations/v1/NNN_<domain>_<purpose>_up.sql`
- 回滚脚本：`db/migrations/v1/NNN_<domain>_<purpose>_down.sql`
- NNN 三位递增编号，跨域不可复用。

Seed 约定：

- 幂等 seed：`db/seeds/NNN_<domain>_seed.sql`
- 测试数据 seed：`db/seeds/test/NNN_<domain>_test_seed.sql`

脚本约定：

- 执行脚本：`db/scripts/apply-*.sh`
- 校验脚本：`db/scripts/check-*.sh`
