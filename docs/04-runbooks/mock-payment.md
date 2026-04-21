# Mock Payment Provider 使用说明（ENV-021）

## 启动前提

- `mock-payment-provider` 不属于默认 `core` 主线。
- 本地手工联调前，请先执行以下任一命令：
  - `make up-mocks`
  - `make up-demo`
- `make up-local` 仅启动 `core`，不会自动带起 Mock Payment Provider。
- 启动后建议先执行：
  - `ENV_FILE=infra/docker/.env.local ./scripts/check-local-stack.sh mocks`
  - `./scripts/check-mock-payment.sh`

## 健康检查

- 管理接口：`GET http://127.0.0.1:8089/__admin`
- 就绪接口：`GET http://127.0.0.1:8089/health/ready`

## 场景触发接口

1. 支付成功  
`POST /mock/payment/charge/success`

2. 支付失败  
`POST /mock/payment/charge/fail`

3. 支付超时（约 15 秒返回）  
`POST /mock/payment/charge/timeout`

4. 退款成功  
`POST /mock/payment/refund/success`

5. 人工打款成功  
`POST /mock/payment/manual-transfer/success`

## 快速测试命令

```bash
curl -sS -X POST http://127.0.0.1:8089/mock/payment/charge/success
curl -sS -X POST http://127.0.0.1:8089/mock/payment/charge/fail
curl -m 3 -sS -X POST http://127.0.0.1:8089/mock/payment/charge/timeout || echo timeout-expected
curl -sS -X POST http://127.0.0.1:8089/mock/payment/refund/success
curl -sS -X POST http://127.0.0.1:8089/mock/payment/manual-transfer/success
```
