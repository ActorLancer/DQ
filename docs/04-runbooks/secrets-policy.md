# Local Secrets Policy（ENV-035）

## 目标

- 冻结本地环境的 secrets 管理边界。
- 明确哪些变量允许进入 `.env.local`，哪些必须走 secret 文件或 CI Secret。
- 避免把生产级凭据误放入仓库或普通环境文件。

## 适用范围

- 本地 Docker Compose：`infra/docker/.env.local`
- 项目根环境模板：`.env.local.example`、`.env.example`
- CI/CD 注入：Pipeline Secret Store（GitHub Actions Secret / 私有 CI Secret）

## 正式映射规则

- 数据库：
  - `POSTGRES_*` 只用于本地 PostgreSQL / Keycloak bootstrap
  - 应用与大部分脚本正式读取 `DATABASE_URL`
  - `PGHOST / PGPORT / PGDATABASE / PGUSER / PGPASSWORD` 仅用于 `psql / libpq` 临时调试
- MinIO：
  - `MINIO_ROOT_*` 只用于 MinIO 服务端 bootstrap
  - 应用与脚本正式读取 `MINIO_ENDPOINT / MINIO_ACCESS_KEY / MINIO_SECRET_KEY`
- Keycloak：
  - `KEYCLOAK_ADMIN / KEYCLOAK_ADMIN_PASSWORD` 只用于本地 Keycloak bootstrap
  - 应用与脚本正式读取 `KEYCLOAK_BASE_URL / KEYCLOAK_REALM / KEYCLOAK_CLIENT_*`
  - `KEYCLOAK_ADMIN_USERNAME` 不再作为正式主配置名

## 分类规则

| 分类 | 是否允许写入 `.env.local` | 说明 |
| --- | --- | --- |
| 非敏感运行配置 | 允许 | 端口、topic 名、bucket 名、索引别名、日志级别、开关。 |
| 本地演示凭据（仅 fake/local） | 允许（仅本地） | 如 `POSTGRES_PASSWORD`、`MINIO_ROOT_PASSWORD` 的本地演示值。禁止复用到测试/生产。 |
| 敏感凭据（可接管环境） | 不允许 | 如真实 DB/Redis/OpenSearch/Keycloak 密码、第三方 API Key。必须走 secret 文件或 CI Secret。 |
| 私钥/证书材料 | 严禁 | 如 Fabric 私钥、签名私钥、TLS 私钥。必须走文件挂载或 Secret Volume。 |

## 允许进入 `.env.local` 的变量（V1 本地）

- 端口类：`POSTGRES_PORT`、`REDIS_PORT`、`KAFKA_PORT`、`KAFKA_EXTERNAL_PORT`、`MINIO_API_PORT` 等
- 本地网络绑定类：`KAFKA_BIND_HOST`、`KAFKA_EXTERNAL_BIND_HOST`、`KAFKA_EXTERNAL_ADVERTISED_HOST`
- 命名类：`TOPIC_*`、`BUCKET_*`、`INDEX_ALIAS_*`
- 本地演示账号：`POSTGRES_USER`、`MINIO_ROOT_USER`、`KEYCLOAK_ADMIN`
- 本地演示密码（仅 local fake）：`POSTGRES_PASSWORD`、`MINIO_ROOT_PASSWORD`、`KEYCLOAK_ADMIN_PASSWORD`、`REDIS_PASSWORD`
- 本地运行时入口样例：`DATABASE_URL`、`MINIO_ACCESS_KEY`、`MINIO_SECRET_KEY`、`KEYCLOAK_REALM`

## 必须走 secret 文件或 CI Secret 的变量

- 外部支付/通知/身份集成密钥：`*_API_KEY`、`*_CLIENT_SECRET`、`*_SIGNING_SECRET`
- 真实环境密码：带密码的 `DATABASE_URL`、`REDIS_PASSWORD`、`OPENSEARCH_PASSWORD`、`MINIO_SECRET_KEY`、`KEYCLOAK_CLIENT_SECRET`
- 链路与证书密钥：`FABRIC_PRIVATE_KEY_PATH` 指向的私钥文件内容、TLS 私钥、服务签名私钥

推荐落位：

- 本地：`.env.local.secret`（不入库）+ `secrets/` 文件目录（不入库）
- CI：平台 Secret Store 注入到运行时环境变量

## 禁止事项

- 禁止提交真实密钥到 Git（包括 `.env.local`、脚本、fixtures、截图）
- 禁止在日志打印完整凭据
- 禁止将私钥材料以内联字符串形式放入 Compose 或代码常量

## 最小执行流程

1. 复制模板：`cp .env.local.example .env.local`（或使用 `infra/docker/.env.local`）
2. 仅填充本地演示值，不填真实生产值
3. 真实凭据放入 `.env.local.secret` 或 CI Secret
4. 启动并校验：
   - `make up-local`
   - `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh core`

补充说明：

- 若修改 `POSTGRES_*`、`MINIO_ROOT_*`、`KEYCLOAK_ADMIN*` 本地演示值，必须同步更新 `infra/docker/.env.local` 中派生出的 `DATABASE_URL`、`MINIO_ACCESS_KEY / MINIO_SECRET_KEY`、`KEYCLOAK_REALM` 等运行时入口样例。

## 审计与轮换建议

- 本地环境每个迭代至少轮换一次演示密码。
- CI Secret 至少按月轮换或按事件触发轮换。
- 一旦发现泄漏，立即吊销并更新对应变量来源。
