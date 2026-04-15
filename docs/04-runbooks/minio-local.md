# MinIO 本地初始化（ENV-014/015）

## 目标 bucket

- `raw-data`
- `preview-artifacts`
- `delivery-objects`
- `report-results`
- `evidence-packages`
- `model-artifacts`

## 执行

```bash
./infra/minio/init-minio.sh
```

## 默认行为

- 自动创建以上 bucket（幂等）
- 为 `preview-artifacts` 设置匿名下载策略
- 为 `preview-artifacts` 增加 30 天过期生命周期规则
- 上传测试对象到 `evidence-packages/_health/init.txt`
