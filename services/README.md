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
- `AUD-016` 起，`fabric-ca-admin/` 已作为 Go 执行面服务落地，负责 Fabric 身份签发 / 吊销与证书吊销；Rust `platform-core` 仅保留公网 IAM 控制面、权限、step-up 与审计主体。
- `AUD-017` 起，`fabric-adapter/` 已支持真实 `fabric-test-network` provider；Fabric Gateway、链码交互与 commit status 等执行面统一收敛在 Go 服务，不再由 Rust 直接连 Fabric。
