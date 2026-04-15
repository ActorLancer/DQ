# Local development guide

This guide matches `docker-compose.local.yml`.
Use the Makefile in this directory as the primary entrypoint.

## 1. Prepare env file

```bash
cp 部署脚本/env.example 部署脚本/.env
```

## 2. Validate local prerequisites

```bash
make -C 部署脚本 check-env
```

If you see a Docker permission error (cannot access `/var/run/docker.sock`), you can run the Makefile targets with sudo:

```bash
make -C 部署脚本 DOCKER="sudo docker" check-env
make -C 部署脚本 DOCKER="sudo docker" up
```

## 3. Start local stack

```bash
make -C 部署脚本 up
```

## 4. Check service status

```bash
make -C 部署脚本 ps
make -C 部署脚本 verify
```

## 5. Common endpoints

Default ports (override in `部署脚本/.env` if needed):

- PostgreSQL: `127.0.0.1:5432`
- Redis: `127.0.0.1:6379`
- Kafka external: `127.0.0.1:9094`
- MinIO API: `http://127.0.0.1:9000`
- MinIO Console: `http://127.0.0.1:9001`
- OpenSearch: `http://127.0.0.1:9200`
- Keycloak: `http://127.0.0.1:8081`
- Prometheus: `http://127.0.0.1:9090`
- Grafana: `http://127.0.0.1:3000`
- Loki: `http://127.0.0.1:3100`
- Tempo: `http://127.0.0.1:3200`
- Mock payment provider: `http://127.0.0.1:8089`

## 6. Stop everything

```bash
make -C 部署脚本 down
```

Remove persistent volumes too:

```bash
make -C 部署脚本 destroy
```

If PostgreSQL was started before this volume layout change, remove the old database volume once before restarting:

```bash
make -C 部署脚本 destroy
make -C 部署脚本 up
```
