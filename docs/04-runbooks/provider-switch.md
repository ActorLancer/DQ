# Provider Switch（BOOT-009）

- Provider 切换必须通过配置，不改代码。
- `local` 默认使用 mock provider。
- `fabric-adapter` 当前正式开关：`FABRIC_ADAPTER_PROVIDER_MODE=mock|fabric-test-network`
- `fabric-test-network` 模式要求同时具备：
  - `FABRIC_GATEWAY_ENDPOINT`
  - `FABRIC_GATEWAY_PEER`
  - `FABRIC_MSP_ID`
  - `FABRIC_TLS_CERT_PATH`
  - `FABRIC_SIGN_CERT_PATH`
  - `FABRIC_PRIVATE_KEY_DIR` 或 `FABRIC_PRIVATE_KEY_PATH`
- 本地验证入口：
  - `ENV_FILE=infra/docker/.env.local ./scripts/check-provider-switch.sh`
  - `FABRIC_ADAPTER_PROVIDER_MODE=fabric-test-network ./scripts/fabric-adapter-run.sh`
  - `./scripts/fabric-adapter-live-smoke.sh`
- 进入 `staging/demo` 前确认签名与回调配置。
