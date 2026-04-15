# Local dev profiles guide

This guide matches `docker-compose.local.profiled.yml`.

The profiled compose keeps the core stack on by default and makes heavier or more situational services opt-in:

- `observability`: `prometheus`, `grafana`, `loki`, `tempo`
- `mocks`: `mock-payment-provider`

## 1. Prepare env file

```bash
cp .env.example .env
```

## 2. Start only the default core stack

This starts the day-to-day development dependencies without observability and mock services:

```bash
docker compose -f docker-compose.local.profiled.yml --env-file .env up -d
```

## 3. Start core + observability

```bash
docker compose \
  -f docker-compose.local.profiled.yml \
  --env-file .env \
  --profile observability \
  up -d
```

## 4. Start core + mocks

```bash
docker compose \
  -f docker-compose.local.profiled.yml \
  --env-file .env \
  --profile mocks \
  up -d
```

## 5. Start the full stack

```bash
docker compose \
  -f docker-compose.local.profiled.yml \
  --env-file .env \
  --profile observability \
  --profile mocks \
  up -d
```

## 6. Show which services will run

Core only:

```bash
docker compose -f docker-compose.local.profiled.yml --env-file .env config --services
```

With observability enabled:

```bash
docker compose \
  -f docker-compose.local.profiled.yml \
  --env-file .env \
  --profile observability \
  config --services
```

## 7. Switch between profiles cleanly

When changing which profiles are active, use `--remove-orphans` so containers from a previously enabled profile do not hang around:

```bash
docker compose \
  -f docker-compose.local.profiled.yml \
  --env-file .env \
  up -d --remove-orphans
```

Example: turn off observability after previously starting it:

```bash
docker compose \
  -f docker-compose.local.profiled.yml \
  --env-file .env \
  up -d --remove-orphans
```

## 8. Stop everything

```bash
docker compose -f docker-compose.local.profiled.yml --env-file .env down
```

Remove persistent data too:

```bash
docker compose -f docker-compose.local.profiled.yml --env-file .env down -v
```

## 9. Suggested team convention

A practical default is:

- normal development: core only
- debugging metrics, logs, traces: `observability`
- testing payment flows: add `mocks`
- CI or end-to-end local rehearsal: both profiles
