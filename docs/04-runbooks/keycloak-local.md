# Keycloak Realm Import（ENV-018/019）

## Realm 文件

- `infra/keycloak/realm-export/platform-local-realm.json`

## 导入机制

- `docker-compose.local.yml` 中 `keycloak` 使用 `start-dev --import-realm`
- 挂载路径：`/opt/keycloak/data/import`

## 导入后验证

```bash
curl -sS http://127.0.0.1:8081/realms/platform-local/.well-known/openid-configuration
```

应返回 `issuer` 为 `.../realms/platform-local` 的 JSON。
