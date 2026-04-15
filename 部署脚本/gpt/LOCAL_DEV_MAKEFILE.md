# Local Dev Makefile

This Makefile wraps the most common Docker Compose commands for the profiled local stack.

## Quick start

```bash
cp .env.example .env
make up
```

## Common commands

```bash
make init        # create .env if missing
make check-env   # verify required files and Docker availability
make up          # start core stack
make up-obs      # start core stack + observability
make up-mocks    # start core stack + mocks
make up-full     # start everything
make ps          # show containers
make logs        # tail logs
make down        # stop and remove containers
make destroy     # remove containers + volumes
```

## Optional overrides

```bash
make up COMPOSE_FILE=docker-compose.local.optimized.yml
make ps PROJECT_NAME=my-local-stack
make logs ENV_FILE=.env.dev
```

## Notes

- Default compose file: `docker-compose.local.profiled.yml`
- Default env file: `.env`
- Default project name: `dt-local`
