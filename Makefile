SHELL := /bin/bash

.PHONY: up-local down-local logs migrate-up migrate-down seed-local test lint
COMPOSE_FILE ?= infra/docker/docker-compose.local.yml
COMPOSE_ENV_FILE ?= infra/docker/.env.local

up-local:
	COMPOSE_FILE="$(COMPOSE_FILE)" COMPOSE_ENV_FILE="$(COMPOSE_ENV_FILE)" ./scripts/up-local.sh

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
