# SQLx + SeaORM 技术栈迁移实施方案

## 1. 文档目的

本文档用于指导当前仓库从 `tokio-postgres` 迁移到 `SQLx + SeaORM`。

约束如下：

- 不删减任何现有业务功能。
- 不改变现有接口路径、响应语义、错误码、状态机、审计、事件、幂等行为。
- 不切换数据库迁移主流程；`db/scripts/*` 和现有 SQL migration 继续作为唯一正式迁移机制。
- 完全移除应用层对 `tokio-postgres` 的依赖。
- 提供未来扩展到 `Postgres / MySQL` 的数据库访问抽象，但当前运行目标仍然是 `Postgres`。
- 本次仅修改数据库访问相关代码、目录、测试与必要的技术说明文档，不修改无关配置和业务规则。

当前迁移工作分支：

- `sqlx-seaorm-migration`

源分支：

- `v1core_dev`

## 2. 当前现状

当前仓库数据库访问的主要问题如下：

1. 应用层直接使用 `tokio-postgres`。
2. 多个 handler 在请求路径上自行创建数据库连接。
3. `order`、`catalog`、`iam`、`billing` 等模块的数据库访问方式不统一。
4. 事务、读查询、复杂聚合、简单 CRUD 没有明确分层。
5. 没有统一的数据库运行时状态注入。
6. 无法自然扩展到其他数据库后端。

当前典型位置：

- `apps/platform-core/src/modules/order/api/handlers.rs`
- `apps/platform-core/src/modules/order/repo/*`
- `apps/platform-core/src/modules/catalog/repository.rs`
- `apps/platform-core/src/modules/catalog/api/support.rs`
- `apps/platform-core/src/modules/iam/api.rs`
- `apps/platform-core/src/modules/iam/repository.rs`
- `apps/platform-core/src/modules/billing/db.rs`
- `apps/platform-core/crates/db/src/lib.rs`

## 3. 总体技术选型

### 3.1 角色分工

本次采用如下组合：

- `SQLx`：底层连接池、事务、复杂 SQL、核心写路径、复杂聚合查询
- `SeaORM`：标准 CRUD、稳定读模型、简单详情、列表、分页、固定关系加载

### 3.2 关键原则

1. `SQLx` 是底层数据库访问标准。
2. `SeaORM` 是 ORM 查询能力补充，不主导复杂事务。
3. 一个业务用例只允许一种主数据库抽象主导。
4. 命令侧优先 `SQLx`，查询侧优先 `SeaORM`，复杂查询允许继续 `SQLx`。
5. 不使用 `sqlx::Any` 作为统一后端入口。
6. 不切换正式 migration 机制到 `sqlx-cli` 或 `sea-orm-cli migrate`。

### 3.3 为什么不是全 SeaORM

因为当前仓库存在大量以下场景：

- 状态机推进
- 审计 + outbox 同事务写入
- `FOR UPDATE`
- webhook 幂等与乱序保护
- 多表一致性写入
- 复杂 JSON 字段与方言特性处理

这些场景继续保留显式 SQL 更稳、更可审计、更容易验证。

## 4. 目标架构

### 4.1 数据库运行时模型

建议引入统一数据库运行时状态：

```rust
pub enum DatabaseDialect {
    Postgres,
    Mysql,
}

pub enum AppDb {
    Postgres(PostgresDbRuntime),
    Mysql(MySqlDbRuntime),
}

pub struct PostgresDbRuntime {
    pub sqlx: sqlx::PgPool,
    pub orm: sea_orm::DatabaseConnection,
}

pub struct MySqlDbRuntime {
    pub sqlx: sqlx::MySqlPool,
    pub orm: sea_orm::DatabaseConnection,
}
```

说明：

- 当前只完成 `PostgresDbRuntime` 的正式实现。
- `MySqlDbRuntime` 作为扩展接口和目录预留，不承诺本次完成业务可运行。
- 应用层通过 `AppDb` 和仓储 trait 使用数据库，不直接依赖具体驱动类型。

### 4.2 应用状态注入

新增统一状态：

```rust
#[derive(Clone)]
pub struct AppState {
    pub runtime: RuntimeConfig,
    pub db: Arc<AppDb>,
}
```

Router 层改为统一 `with_state(AppState)` 注入。

不再允许：

- handler 内部 `tokio_postgres::connect(...)`
- 每个请求重复手工建连接

### 4.3 仓储分层

按业务和读写职责拆分，不按数据库库名拆分。

正确做法：

```text
repo/
  command/
  query/
  shared/
```

错误做法：

```text
repo_sqlx/
repo_seaorm/
```

## 5. 目录重组方案

### 5.1 `crates/db` 重组

目标目录：

```text
apps/platform-core/crates/db/src/
  lib.rs
  config.rs
  dialect.rs
  error.rs
  runtime/
    mod.rs
    app_db.rs
    connect.rs
    transaction.rs
  sqlx/
    mod.rs
    postgres.rs
    mysql.rs
  entity/
    mod.rs
    prelude.rs
    catalog/
    iam/
    billing/
    trade/
  query/
    mod.rs
    pagination.rs
    json.rs
  testing/
    mod.rs
    seed.rs
```

职责：

- `config.rs`：数据库 URL、池配置、连接参数
- `dialect.rs`：数据库方言识别
- `error.rs`：统一数据库错误映射
- `runtime/*`：`AppDb`、连接建立、事务模板
- `sqlx/*`：backend-specific 构造
- `entity/*`：SeaORM entity
- `query/*`：查询辅助能力
- `testing/*`：测试公用连接/seed 帮助

### 5.2 `order` 模块重组

目标目录：

```text
apps/platform-core/src/modules/order/repo/
  mod.rs
  command/
    mod.rs
    create_order.rs
    freeze_price_snapshot.rs
    confirm_contract.rs
    cancel_order.rs
    transition_file_std.rs
    transition_file_sub.rs
    transition_api_sub.rs
    transition_api_ppu.rs
    transition_share_ro.rs
    transition_qry_lite.rs
    transition_sbx_std.rs
    transition_rpt_std.rs
    authorization_transition.rs
    authorization_cutoff.rs
    deliverability_gate.rs
    pre_payment_lock.rs
  query/
    mod.rs
    order_detail.rs
    lifecycle_snapshots.rs
    relations.rs
  shared/
    mod.rs
    audit.rs
    errors.rs
    mapping.rs
```

原则：

- 所有 command 仓储改成 `SQLx`
- query 仓储优先 `SeaORM`，复杂聚合保留 `SQLx`

### 5.3 `catalog` 模块重组

目标目录：

```text
apps/platform-core/src/modules/catalog/
  repo/
    mod.rs
    command/
      mod.rs
      product_write.rs
      review_write.rs
      template_binding_write.rs
    query/
      mod.rs
      product_read.rs
      seller_profile.rs
      product_listing.rs
      scenario_read.rs
    shared/
      mod.rs
      mapping.rs
      filters.rs
  api/
    support.rs
    validators/
```

原则：

- 标准读查询优先 `SeaORM`
- 复杂校验/复杂检索可继续 `SQLx`
- 不再保留单个超大 `repository.rs`

### 5.4 `iam` 模块重组

目标目录：

```text
apps/platform-core/src/modules/iam/
  repo/
    mod.rs
    command/
    query/
    shared/
```

原则：

- 稳定读查询优先 `SeaORM`
- API 文件不再直接依赖数据库驱动

### 5.5 `billing` 模块重组

目标目录：

```text
apps/platform-core/src/modules/billing/
  repo/
    mod.rs
    command.rs
    query.rs
    mapping.rs
```

原则：

- webhook、支付意图写入、回调处理统一使用 `SQLx`
- 乱序和幂等保护逻辑保持完全不变

## 6. 文件级改造清单

### 6.1 依赖与基础设施

必须修改：

- `apps/platform-core/Cargo.toml`
- `apps/platform-core/crates/db/Cargo.toml`
- `apps/platform-core/src/lib.rs`
- `apps/platform-core/crates/http/src/lib.rs`

需要新增依赖：

- `sqlx`
- `sea-orm`
- `sea-query`
- `sea-query-binder`

需要删除：

- `tokio-postgres`

### 6.2 `crates/db`

必须重构：

- `apps/platform-core/crates/db/src/lib.rs`

必须新增：

- `apps/platform-core/crates/db/src/config.rs`
- `apps/platform-core/crates/db/src/dialect.rs`
- `apps/platform-core/crates/db/src/error.rs`
- `apps/platform-core/crates/db/src/runtime/*`
- `apps/platform-core/crates/db/src/sqlx/*`
- `apps/platform-core/crates/db/src/entity/*`

### 6.3 `order`

必须修改：

- `apps/platform-core/src/modules/order/api/handlers.rs`
- `apps/platform-core/src/modules/order/repo/mod.rs`
- `apps/platform-core/src/modules/order/repo/*.rs`

必须迁移全部测试：

- `apps/platform-core/src/modules/order/tests/*.rs`

### 6.4 `catalog`

必须修改：

- `apps/platform-core/src/modules/catalog/repository.rs`
- `apps/platform-core/src/modules/catalog/api/support.rs`
- `apps/platform-core/src/modules/catalog/api/validators/mod.rs`
- `apps/platform-core/src/modules/catalog/tests/*.rs`

### 6.5 `iam`

必须修改：

- `apps/platform-core/src/modules/iam/api.rs`
- `apps/platform-core/src/modules/iam/repository.rs`
- `apps/platform-core/tests/iam_party_access_integration.rs`

### 6.6 `billing`

必须修改：

- `apps/platform-core/src/modules/billing/db.rs`
- `apps/platform-core/src/modules/billing/handlers.rs`

## 7. 功能分配到 SQLx / SeaORM 的规则

### 7.1 必须使用 SQLx 的场景

- 所有事务命令
- 状态机推进
- webhook 处理
- 幂等控制
- 审计写入
- outbox 写入
- 行锁和并发保护
- 复杂聚合查询
- 强依赖 Postgres 语义的 SQL

### 7.2 优先使用 SeaORM 的场景

- 标准 CRUD
- 单表详情读取
- 固定关联加载
- 列表 + 分页
- 后台管理型查询
- 稳定读模型

### 7.3 允许继续 SQLx 的查询

即便是查询，只要满足以下条件之一，也允许继续用 `SQLx`：

- SQL 明显比 ORM 更清晰
- 查询高度聚合
- 包含复杂 `jsonb` 运算
- 查询结果直接映射 API 返回结构

## 8. 多数据库扩展设计

### 8.1 扩展目标

目标不是现在直接运行 MySQL，而是现在把架构改到可以承载未来多数据库。

### 8.2 抽象方式

不使用一个“大一统 Any 连接”。

采用：

- dialect enum
- repository trait
- backend-specific 实现

示例：

```rust
pub trait OrderCommandRepo: Send + Sync {
    async fn create_order(...);
    async fn confirm_contract(...);
}

pub struct PostgresOrderCommandRepo { ... }
pub struct MySqlOrderCommandRepo { ... }
```

当前只注册：

- `Postgres*Repo`

未来扩展：

- `MySql*Repo`

### 8.3 当前必须明确的限制

由于现有数据库设计和 SQL 大量使用 Postgres 语法，本次迁移完成后：

- `Postgres`：正式支持
- `MySQL`：架构扩展点准备完成，但业务实现暂不承诺可运行

## 9. 迁移与 CLI 策略

### 9.1 正式迁移机制

保持不变：

- `db/scripts/migrate-up.sh`
- `db/scripts/migrate-down.sh`
- `db/scripts/migration-runner.sh`
- `db/scripts/migrate-status.sh`
- `db/scripts/verify-migration-*.sh`
- `docs/数据库设计/V1/upgrade/*.sql`
- `docs/数据库设计/V1/downgrade/*.sql`

### 9.2 `sqlx-cli`

允许使用：

- `cargo sqlx prepare --workspace`

用途：

- 生成 `.sqlx/` 元数据
- 支持 query 宏编译期校验

不作为正式 migration runner。

### 9.3 `sea-orm-cli`

允许使用：

- 从现有数据库生成 entity

用途：

- 初始化实体代码
- 后续按需增量刷新

不作为正式 migration runner。

## 10. 实施阶段划分

### 阶段 1：基础设施切换

目标：

- 建立 `AppDb`
- 建立统一 pool
- 建立统一错误模型
- Router 支持 `AppState`
- 移除 handler 直连数据库模式

主要改动：

- `crates/db`
- `src/lib.rs`
- `crates/http/src/lib.rs`
- 各模块 router / handler 签名

完成标准：

- 所有 handler 可通过 `State<AppState>` 获取数据库访问入口
- 仓库内不再新增任何 `tokio_postgres::connect`

### 阶段 2：`order` 模块迁移

目标：

- 所有交易主链路仓储改成 `SQLx`
- 保证状态机、审计、outbox、幂等行为完全不变

主要改动：

- `apps/platform-core/src/modules/order/repo/*`
- `apps/platform-core/src/modules/order/api/handlers.rs`
- `apps/platform-core/src/modules/order/tests/*.rs`

完成标准：

- `order` 模块无 `tokio-postgres`
- 全量 trade smoke 与 API 联调通过

### 阶段 3：`billing` 模块迁移

目标：

- 支付意图和 webhook 路径改成 `SQLx`
- 保留乱序保护与忽略规则

主要改动：

- `apps/platform-core/src/modules/billing/db.rs`
- `apps/platform-core/src/modules/billing/handlers.rs`

完成标准：

- billing 路径无 `tokio-postgres`
- webhook 集成验证通过

### 阶段 4：`catalog` 迁移

目标：

- 读查询逐步迁移到 `SeaORM`
- 写和复杂校验保留 `SQLx`

主要改动：

- `apps/platform-core/src/modules/catalog/repository.rs`
- `apps/platform-core/src/modules/catalog/api/support.rs`
- `apps/platform-core/src/modules/catalog/tests/*.rs`

完成标准：

- `repository.rs` 拆分完成
- 简单读查询改为 SeaORM
- 复杂规则不退化

### 阶段 5：`iam` 迁移

目标：

- 读查询改造到 `SeaORM + SQLx` 组合
- API 层去除数据库驱动耦合

### 阶段 6：测试与清理

目标：

- 所有测试切换到 SQLx
- 删除 `tokio-postgres`
- 更新锁文件和技术说明

完成标准：

- `rg -n "tokio_postgres|tokio-postgres" apps/platform-core` 返回空

## 11. 设计细节约束

### 11.1 不允许直接让 Entity 泄漏到 API 层

必须保持：

- Entity：数据库映射
- Domain / Read Model：业务模型
- DTO：接口模型

### 11.2 事务策略

涉及以下行为必须全部使用 SQLx transaction：

- create order
- confirm contract
- payment result orchestration
- authorization transition
- authorization cutoff
- deliverability gate
- pre-payment lock
- cancellation

### 11.3 错误映射策略

统一在 `crates/db/src/error.rs` 做：

- SQLx 错误 -> 业务错误
- 唯一约束冲突
- FK 冲突
- not found
- serialization / deadlock / timeout

模块层不得散落大量 driver-specific 错误转换。

### 11.4 审计与 outbox

必须继续和业务写路径共事务。

任何迁移都不允许把以下行为拆出事务：

- `audit.audit_event`
- `ops.outbox_event`

## 12. 测试与验证策略

### 12.1 基础验证

```bash
cargo fmt --all
cargo test -p platform-core
```

### 12.2 SQLx 校验

```bash
cargo sqlx prepare --workspace
```

产物：

- `.sqlx/`

### 12.3 迁移体系验证

继续使用现有流程：

```bash
./db/scripts/migrate-reset.sh
./db/scripts/migrate-up.sh
./db/scripts/migrate-status.sh
./db/scripts/verify-migration-*.sh
```

### 12.4 模块验证

至少覆盖：

- `order` 全部 trade smoke
- `catalog` 现有 DB smoke
- `iam` 集成测试
- `billing` webhook 联调
- 真实 API + curl + psql 回查

### 12.5 最终硬校验

```bash
rg -n "tokio_postgres|tokio-postgres" apps/platform-core
```

必须为空。

## 13. 文档同步范围

虽然本实施方案单独放在仓库根目录 `plans/`，但实际迁移完成后仍需同步以下技术说明：

- `docs/开发准备/技术选型正式版.md`
- `docs/开发准备/仓库拆分与目录结构建议.md`
- `docs/04-runbooks/local-startup.md`
- `docs/README.md`

同步内容仅限：

- 数据访问技术选型
- 目录分层
- 编译期 SQL 校验流程
- 运行与联调方式

不修改任何业务基线说明。

## 14. 风险清单

1. SQL 方言迁移时误改业务语义
2. ORM 引入后读模型与现有 API DTO 混用
3. Router 状态注入不完整导致局部仍然直连数据库
4. 测试全部切换前，旧测试帮助函数与新 runtime 并存产生分叉
5. 误把 migration 主流程切换到 ORM/CLI 工具

## 15. 回滚策略

如果迁移阶段中发现系统性回归，按以下方式回退：

1. 停止继续扩散迁移范围
2. 保留分支内阶段提交
3. 只回退迁移分支最近阶段提交，不回退已审批主线分支
4. 不改动 `db/scripts/*` 和 migration SQL 基线

## 16. 合并策略

迁移完成并通过验证后：

```bash
git checkout v1core_dev
git merge --no-ff sqlx-seaorm-migration
```

建议：

- 迁移分支内部按阶段提交
- 合并时保留完整历史，不使用 squash
- 合并后再做一次全量验证

## 17. 完成标准

本次迁移完成的判定标准：

1. `apps/platform-core` 范围内完全移除 `tokio-postgres`
2. 核心命令侧统一为 `SQLx`
3. 标准读模型已引入 `SeaORM`
4. 所有现有功能、接口、错误码、审计、事件、状态机语义保持不变
5. 所有相关测试、DB smoke、API 联调通过
6. 保留现有 SQL migration 体系不变
7. 形成未来 MySQL 扩展点，但当前仍以 Postgres 为正式后端
