# OTel Collector 本地配置（ENV-025）

配置文件：`infra/otel/otel-collector-config.yaml`

## 接收端口

- OTLP gRPC: `4317`
- OTLP HTTP: `4318`

## 转发目标

- traces -> `tempo:4317`（OTLP）
- logs -> `loki:3100`（Loki push）
- metrics -> `prometheus exporter :8889`

## 健康检查

- `http://127.0.0.1:13133/`
