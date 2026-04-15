SHELL := /bin/bash

COMPOSE_FILE ?= docker-compose.local.profiled.yml
ENV_FILE ?= .env
PROJECT_NAME ?= dt-local

COMPOSE := docker compose -p $(PROJECT_NAME) -f $(COMPOSE_FILE) --env-file $(ENV_FILE)

.DEFAULT_GOAL := help

.PHONY: help init check-env config pull up up-core up-obs up-mocks up-full ps logs verify verify-core verify-obs verify-mocks verify-full down restart stop start clean destroy reset

help:
	@echo "Available targets:"
	@echo "  make init       # create .env from .env.example if missing"
	@echo "  make check-env  # validate required files exist"
	@echo "  make config     # render merged compose config"
	@echo "  make pull       # pull images"
	@echo "  make up         # start core stack"
	@echo "  make up-core    # same as make up"
	@echo "  make up-obs     # start core stack + observability profile"
	@echo "  make up-mocks   # start core stack + mocks profile"
	@echo "  make up-full    # start all profiles"
	@echo "  make ps         # show running services"
	@echo "  make logs       # tail logs"
	@echo "  make verify     # verify core services are reachable"
	@echo "  make verify-obs # verify core + observability services"
	@echo "  make verify-mocks # verify core + mock service"
	@echo "  make verify-full  # verify all enabled services"
	@echo "  make down       # stop and remove containers"
	@echo "  make stop       # stop containers"
	@echo "  make start      # start existing containers"
	@echo "  make restart    # restart containers"
	@echo "  make clean      # remove containers and anonymous networks"
	@echo "  make destroy    # remove containers, networks, and named volumes"
	@echo "  make reset      # destroy, then init, then start full stack"
	@echo ""
	@echo "Variables:"
	@echo "  COMPOSE_FILE=$(COMPOSE_FILE)"
	@echo "  ENV_FILE=$(ENV_FILE)"
	@echo "  PROJECT_NAME=$(PROJECT_NAME)"

init:
	@if [ ! -f "$(ENV_FILE)" ]; then \
		cp .env.example $(ENV_FILE); \
		echo "Created $(ENV_FILE) from .env.example"; \
	else \
		echo "$(ENV_FILE) already exists"; \
	fi

check-env:
	@bash ./scripts/check-local-env.sh "$(COMPOSE_FILE)" "$(ENV_FILE)"

config: check-env
	@$(COMPOSE) config

pull: check-env
	@$(COMPOSE) pull

up: up-core

up-core: check-env
	@$(COMPOSE) up -d

up-obs: check-env
	@$(COMPOSE) --profile observability up -d

up-mocks: check-env
	@$(COMPOSE) --profile mocks up -d

up-full: check-env
	@$(COMPOSE) --profile observability --profile mocks up -d

ps: check-env
	@$(COMPOSE) ps

logs: check-env
	@$(COMPOSE) logs -f --tail=200

verify: verify-core

verify-core: check-env
	@ENV_FILE=$(ENV_FILE) bash ./scripts/verify-local-stack.sh core

verify-obs: check-env
	@ENV_FILE=$(ENV_FILE) bash ./scripts/verify-local-stack.sh observability

verify-mocks: check-env
	@ENV_FILE=$(ENV_FILE) bash ./scripts/verify-local-stack.sh mocks

verify-full: check-env
	@ENV_FILE=$(ENV_FILE) bash ./scripts/verify-local-stack.sh full

down: check-env
	@$(COMPOSE) down

stop: check-env
	@$(COMPOSE) stop

start: check-env
	@$(COMPOSE) start

restart: check-env
	@$(COMPOSE) restart

clean: check-env
	@$(COMPOSE) down --remove-orphans

destroy: check-env
	@$(COMPOSE) down -v --remove-orphans

reset: destroy init up-full
