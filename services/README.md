# services 目录校准（BOOT-030）

`services/` 用于放置外围服务进程落位，当前先完成目录边界冻结：

- `fabric-adapter/`
- `fabric-event-listener/`
- `fabric-ca-admin/`
- `mock-payment-provider/`

说明：

- 通知进程当前正式落位为 `apps/notification-worker`，`services/` 目录不再承载通知进程。
- 本目录用于其余外围服务的收敛与迁移落位。
- 本批次不做业务代码迁移，只冻结命名与位置。
