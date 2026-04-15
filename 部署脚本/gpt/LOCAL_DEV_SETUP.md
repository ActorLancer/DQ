# Local dev setup

This guide matches `docker-compose.local.optimized.yml`.

## 1. Prepare env file

```bash
cp .env.example .env
```

Update these values before first run:

- `POSTGRES_PASSWORD`
- `MINIO_ROOT_PASSWORD`
- `OPENSEARCH_INITIAL_ADMIN_PASSWORD`
- `KEYCLOAK_ADMIN_PASSWORD`
- `GRAFANA_ADMIN_PASSWORD`

## 2. Start services

```bash
docker compose -f docker-compose.local.optimized.yml --env-file .env up -d
```

## 3. Check status

```bash
docker compose -f docker-compose.local.optimized.yml ps
```

Optional health checks:

```bash
docker compose -f docker-compose.local.optimized.yml logs -f postgres
docker compose -f docker-compose.local.optimized.yml logs -f keycloak
docker compose -f docker-compose.local.optimized.yml logs -f kafka
```

## 4. Common local endpoints

- Postgres: `localhost:${POSTGRES_PORT:-5432}`
- Redis: `localhost:${REDIS_PORT:-6379}`
- Kafka external: `localhost:${KAFKA_EXTERNAL_PORT:-9094}`
- MinIO API: `http://localhost:${MINIO_API_PORT:-9000}`
- MinIO Console: `http://localhost:${MINIO_CONSOLE_PORT:-9001}`
- OpenSearch: `http://localhost:${OPENSEARCH_HTTP_PORT:-9200}`
- Keycloak: `http://localhost:${KEYCLOAK_PORT:-8081}`
- Prometheus: `http://localhost:${PROMETHEUS_PORT:-9090}`
- Grafana: `http://localhost:${GRAFANA_PORT:-3000}`
- Loki: `http://localhost:${LOKI_PORT:-3100}`
- Tempo: `http://localhost:${TEMPO_PORT:-3200}`
- Mock payment provider: `http://localhost:${MOCK_PAYMENT_PORT:-8089}`

## 5. Stop services

```bash
docker compose -f docker-compose.local.optimized.yml --env-file .env down
```

Keep data volumes:

```bash
docker compose -f docker-compose.local.optimized.yml --env-file .env down
```

Remove data volumes too:

```bash
docker compose -f docker-compose.local.optimized.yml --env-file .env down -v
```

## 6. Reset notes

Use `down -v` with care. It removes these persistent volumes and all local data inside them:

- `postgres_data`
- `redis_data`
- `kafka_data`
- `minio_data`
- `opensearch_data`
- `grafana_data`

## 7. Recommended next improvements

- Pin `MINIO_IMAGE` to a concrete version instead of `latest`
- Move secrets from `.env` to a local secret manager if your team already uses one
- Add service profiles if you sometimes want a lighter stack, for example `observability` or `search`
- Add explicit health checks for `minio`, `opensearch`, `grafana`, `loki`, and `tempo` if startup ordering becomes important
