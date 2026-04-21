# Repo Layout（BOOT-002）

本文件是当前阶段目录结构的唯一参考入口。后续任务若涉及新建目录或迁移目录，需先更新本文件并通过审批。

## 1. 当前目录树（V1-Core）

```text
repo-root/
  README.md
  .gitignore
  .editorconfig
  .gitattributes
  .env.example
  .env.local.example
  .env.staging.example
  .env.demo.example
  Cargo.toml
  Cargo.lock
  apps/
    platform-core/
    portal-web/
    console-web/
    fabric-adapter/
    fabric-event-listener/
    search-indexer/
    data-processing-worker/
    notification-worker/
    mock-payment-provider/
  services/
    fabric-adapter/
    fabric-event-listener/
    fabric-ca-admin/
    mock-payment-provider/
  workers/
    search-indexer/
    outbox-publisher/
    data-processing-worker/
    quality-profiler/
    report-job/
  packages/
    api-contracts/
    event-contracts/
    domain-types/
    test-fixtures/
    openapi/
    sdk-ts/
    ui/
    shared-config/
    observability-contracts/
  db/
    migrations/
    seeds/
    scripts/
  infra/
    docker/
    fabric/
    keycloak/
    kafka/
    postgres/
    minio/
    opensearch/
    redis/
    prometheus/
    grafana/
    loki/
    tempo/
    otel/
    monitoring/
    k8s/
  scripts/
  docs/
    00-context/
    01-architecture/
    02-openapi/
    03-db/
    04-runbooks/
    05-test-cases/
    开发任务/
    开发准备/
    开发前设计文档/
    全集成文档/
    数据库设计/
    领域模型/
    业务流程/
  fixtures/
    local/
  tests/
  tools/
  .github/
    workflows/
  部署脚本/
```

## 2. 约束说明

1. `apps/` 保留当前可运行骨架，`services/` 与 `workers/` 保留为后续收敛落位。
2. `docs/开发任务/v1-core-开发任务清单.csv` 是执行源；本文件仅定义目录边界，不改动任务依赖。
3. 所有新增目录必须落在本文件定义边界内，禁止新增平行顶层目录。
