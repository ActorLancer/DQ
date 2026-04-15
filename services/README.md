# services 目录校准（BOOT-030）

`services/` 用于放置外围服务进程落位，当前先完成目录边界冻结：

- `fabric-adapter/`
- `fabric-event-listener/`
- `fabric-ca-admin/`
- `mock-payment-provider/`
- `notification-service/`

说明：

- 当前仓库同名能力在 `apps/` 中已有骨架，本目录用于后续收敛与迁移落位。
- 本批次不做业务代码迁移，只冻结命名与位置。
