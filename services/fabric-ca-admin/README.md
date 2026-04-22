# fabric-ca-admin

`fabric-ca-admin` 是 `AUD-016` 起正式落地的 Go 执行面服务，职责是：

- 执行 Fabric 身份签发
- 执行 Fabric 身份吊销
- 执行证书吊销

边界：

- Rust `platform-core` 负责公网 IAM API、权限、step-up、审计主体与错误码
- Go `fabric-ca-admin` 只负责执行面与回执写回
- 当前 provider mode 为 `mock`
- 真实 `Fabric CA / test-network / Gateway` 切换留待 `AUD-017`

## 目录

- `cmd/fabric-ca-admin/`：服务入口
- `internal/api/`：内部 HTTP handler
- `internal/service/`：执行面编排
- `internal/store/`：PostgreSQL 状态读写
- `internal/provider/`：当前 mock CA provider
- `internal/config/`：环境变量装配

## 命令

```bash
./scripts/fabric-ca-admin-bootstrap.sh
./scripts/fabric-ca-admin-test.sh
./scripts/fabric-ca-admin-run.sh
```

或：

```bash
make fabric-ca-admin-bootstrap
make fabric-ca-admin-test
make fabric-ca-admin-run
```

## 运行配置

- `DATABASE_URL`
- `FABRIC_CA_ADMIN_PORT`
- `FABRIC_CA_ADMIN_BASE_URL`
- `FABRIC_CA_ADMIN_MODE`

默认本地值：

- `DATABASE_URL=postgres://datab:datab_local_pass@127.0.0.1:5432/datab`
- `FABRIC_CA_ADMIN_PORT=18112`
- `FABRIC_CA_ADMIN_BASE_URL=http://127.0.0.1:18112`
- `FABRIC_CA_ADMIN_MODE=mock`

## 内部接口

- `GET /healthz`
- `POST /internal/fabric-identities/{id}/issue`
- `POST /internal/fabric-identities/{id}/revoke`
- `POST /internal/certificates/{id}/revoke`

所有内部写接口都要求从 `platform-core` 透传：

- `x-role`
- `x-user-id`
- `x-permission-code`
- `x-step-up-challenge-id`
- `x-request-id`
- `x-trace-id`

## 当前回写对象

- `iam.fabric_identity_binding`
- `iam.certificate_record`
- `iam.certificate_revocation_record`
- `ops.external_fact_receipt`
- `ops.system_log`

公网审计事件由 `platform-core` 写入：

- `audit.audit_event`

不要在这里直接扩展第二套公网 IAM API，也不要把 Go 服务当作业务主状态机。
