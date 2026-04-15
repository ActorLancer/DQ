SHELL := /bin/bash

.PHONY: up-local down-local up-core up-observability up-fabric up-demo logs migrate-up migrate-down seed-local test lint fabric-up fabric-down fabric-reset fabric-channel
COMPOSE_FILE ?= infra/docker/docker-compose.local.yml
COMPOSE_ENV_FILE ?= infra/docker/.env.local

up-local:
	COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

up-core:
	COMPOSE_PROFILES="core" COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

up-observability:
	COMPOSE_PROFILES="core,observability" COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

up-fabric:
	COMPOSE_PROFILES="core,fabric" COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

up-demo:
	COMPOSE_PROFILES="demo" COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

down-local:
	COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/down-local.sh

logs:
	docker compose --env-file "$(COMPOSE_ENV_FILE)" -f "$(COMPOSE_FILE)" logs -f

migrate-up:
	./scripts/validate_database_migrations.sh

migrate-down:
	@echo "migrate-down placeholder: define downgrade flow in db/scripts/"

seed-local:
	./scripts/seed-demo.sh

test:
	cargo test

lint:
	cargo check

fabric-up:
	./infra/fabric/fabric-up.sh

fabric-down:
	./infra/fabric/fabric-down.sh

fabric-reset:
	./infra/fabric/fabric-reset.sh

fabric-channel:
	./infra/fabric/fabric-channel.sh
