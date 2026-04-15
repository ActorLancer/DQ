# OpenSearch 本地初始化（ENV-016/017）

## 初始化内容

- 索引模板：`datab_catalog_v1_template`
- 别名：
  - `catalog_products_v1`
  - `seller_profiles_v1`
  - `search_sync_jobs_v1`
- 对应物理索引：`*_000001`
- 本地 demo 文档各 1 条

## 执行

```bash
./infra/opensearch/init-opensearch.sh
```

## 验证

```bash
curl -sS http://127.0.0.1:9200/_cat/aliases?v
curl -sS http://127.0.0.1:9200/catalog_products_v1/_count
```
