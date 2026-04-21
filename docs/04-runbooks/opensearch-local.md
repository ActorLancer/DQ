# OpenSearch 本地初始化（ENV-016/017）

## 初始化内容

- 索引模板：`datab_catalog_v1_template`
- 读写别名：
  - `product_search_read`
  - `product_search_write`
  - `seller_search_read`
  - `seller_search_write`
- 物理索引：
  - `product_search_v1_bootstrap`
  - `seller_search_v1_bootstrap`
  - `search_sync_jobs_v1`
- 本地 demo 文档各 1 条

## 执行

```bash
./infra/opensearch/init-opensearch.sh
```

说明：

- 脚本会清理本地遗留的 `catalog_products_v1_000001`、`seller_profiles_v1_000001`、`search_sync_jobs_v1_000001`，并按当前冻结口径重建 bootstrap 索引与 read/write alias。
- 搜索同步正式 worker 为 `workers/search-indexer`，默认消费 `dtp.search.sync` 并写入 `product_search_write` / `seller_search_write`。

## 验证

```bash
curl -sS http://127.0.0.1:9200/_cat/aliases?v
curl -sS http://127.0.0.1:9200/product_search_write/_count
curl -sS http://127.0.0.1:9200/search_sync_jobs_v1/_count
```
