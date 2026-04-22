SHELL := /bin/bash

.PHONY: up-local down-local up-core up-observability up-mocks up-fabric up-demo logs migrate-up migrate-down seed-local test lint query-compile-check openapi-check xtask core-verify fabric-up fabric-down fabric-reset fabric-channel fabric-adapter-bootstrap fabric-adapter-test fabric-adapter-run fabric-event-listener-bootstrap fabric-event-listener-test fabric-event-listener-run fabric-ca-admin-bootstrap fabric-ca-admin-test fabric-ca-admin-run
COMPOSE_FILE ?= infra/docker/docker-compose.local.yml
COMPOSE_ENV_FILE ?= infra/docker/.env.local

up-local:
	COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

up-core:
	COMPOSE_PROFILES="core" COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

up-observability:
	COMPOSE_PROFILES="core,observability" COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

up-mocks:
	COMPOSE_PROFILES="core,mocks" COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

up-fabric:
	COMPOSE_PROFILES="core" COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh
	./infra/fabric/fabric-up.sh

up-demo:
	COMPOSE_PROFILES="core,observability,mocks" COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh
	./infra/fabric/fabric-up.sh

down-local:
	COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/down-local.sh
	./infra/fabric/fabric-down.sh

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

query-compile-check:
	./scripts/check-query-compile.sh

openapi-check:
	./scripts/check-openapi-schema.sh

xtask:
	cargo xtask all

core-verify:
	cargo xtask all

fabric-up:
	./infra/fabric/fabric-up.sh

fabric-down:
	./infra/fabric/fabric-down.sh

fabric-reset:
	./infra/fabric/fabric-reset.sh

fabric-channel:
	./infra/fabric/fabric-channel.sh

fabric-adapter-bootstrap:
	./scripts/fabric-adapter-bootstrap.sh

fabric-adapter-test:
	./scripts/fabric-adapter-test.sh

fabric-adapter-run:
	./scripts/fabric-adapter-run.sh

fabric-event-listener-bootstrap:
	./scripts/fabric-event-listener-bootstrap.sh

fabric-event-listener-test:
	./scripts/fabric-event-listener-test.sh

fabric-event-listener-run:
	./scripts/fabric-event-listener-run.sh

fabric-ca-admin-bootstrap:
	./scripts/fabric-ca-admin-bootstrap.sh

fabric-ca-admin-test:
	./scripts/fabric-ca-admin-test.sh

fabric-ca-admin-run:
	./scripts/fabric-ca-admin-run.sh
