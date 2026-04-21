# 运行模式冻结（CTX-008）

## 1. 模式清单

`V1-Core` 固定三套运行模式：

1. `local`
2. `staging`
3. `demo`

## 2. 切换原则

- 环境切换必须通过配置完成（环境变量/配置文件/部署参数）。
- 禁止通过手工改代码切换运行模式。
- 模式切换后，接口契约、事件契约和权限策略不应出现语义漂移。

## 3. 模式边界

- `local`：本地开发与最小闭环验证，允许使用 mock provider。
- `staging`：联调与回归验证，要求契约和数据流与正式口径一致。
- `demo`：演示场景，允许开启受控展示配置，但不得改写主业务规则。

## 3.1 `local` 下的联调子场景

`local` 允许通过 compose profile、feature flag、外围依赖开关形成不同联调姿态，例如：

- 最小 `local`
- `mocks` 联调 `local`
- 全量 `local`
- staging-like `local`

这些都只是 `local` 的子场景说明，不是新的正式运行模式。

冻结约束：

- `staging-local` 不是第四套正式 mode
- `mocks` 不是第四套正式 mode；它仅表示 `local` 下叠加 `mock-payment-provider` 等受控 mock 依赖的联调姿态
- 不允许把 `staging-local`、`qa-local`、`preprod-local` 等术语直接写成新的 `APP_ENV` 或测试矩阵主维度
- 若未来确需新增正式运行模式，必须先同步修改本文件与 CSV 执行源中的 `CTX-008`

## 4. 最小一致性要求

- 三种模式都必须遵循：
  - `PostgreSQL` 主状态权威
  - 异步副作用链路（outbox + Kafka + worker/adapter）
  - 搜索/推荐结果回主库校验
