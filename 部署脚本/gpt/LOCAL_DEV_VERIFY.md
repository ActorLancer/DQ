# Local Stack Verification

This helper verifies that the local stack is reachable after startup.

## Quick usage

```bash
make verify
```

## Modes

```bash
make verify        # verify core services
make verify-obs    # verify core + observability services
make verify-mocks  # verify core + mock service
make verify-full   # verify everything
```

## What it checks

### Core
- PostgreSQL TCP port
- Redis TCP port
- Kafka external TCP port
- MinIO API and console
- OpenSearch HTTP endpoint
- Keycloak HTTP endpoint

### Observability
- Prometheus health endpoint
- Grafana health endpoint
- Loki ready endpoint
- Tempo ready endpoint

### Mocks
- Mock payment provider admin endpoint

## Notes

- The script reads `.env` by default, or `ENV_FILE=/path/to/file make verify`.
- A reachable endpoint means the service is up enough to accept traffic. It does not validate application-level auth or business data.
