SHELL := /bin/bash

PROJECT_ROOT ?= $(abspath $(CURDIR)/..)
DEPLOY_DIR ?= $(CURDIR)
COMPOSE_FILE ?= $(DEPLOY_DIR)/docker-compose.local.yml
ENV_FILE ?= $(DEPLOY_DIR)/.env
ENV_TEMPLATE ?= $(DEPLOY_DIR)/env.example
PROJECT_NAME ?= dt-local
DOCKER ?= docker

COMPOSE := $(DOCKER) compose -p $(PROJECT_NAME) -f $(COMPOSE_FILE) --env-file $(ENV_FILE)

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
	@echo "  make up-obs     # alias of make up (compose has no profiles now)"
	@echo "  make up-mocks   # alias of make up (compose has no profiles now)"
	@echo "  make up-full    # alias of make up"
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
	@echo "  PROJECT_ROOT=$(PROJECT_ROOT)"
	@echo "  DEPLOY_DIR=$(DEPLOY_DIR)"
	@echo "  COMPOSE_FILE=$(COMPOSE_FILE)"
	@echo "  ENV_FILE=$(ENV_FILE)"
	@echo "  ENV_TEMPLATE=$(ENV_TEMPLATE)"
	@echo "  PROJECT_NAME=$(PROJECT_NAME)"
	@echo "  DOCKER=$(DOCKER)"

init:
	@if [ ! -f "$(ENV_FILE)" ]; then \
		cp "$(ENV_TEMPLATE)" "$(ENV_FILE)"; \
		echo "Created $(ENV_FILE) from $(ENV_TEMPLATE)"; \
	else \
		echo "$(ENV_FILE) already exists"; \
	fi

check-env:
	@DOCKER_CMD="$(DOCKER)" bash "$(PROJECT_ROOT)/scripts/check-local-env.sh" "$(COMPOSE_FILE)" "$(ENV_FILE)" "$(ENV_TEMPLATE)" files

check-runtime:
	@DOCKER_CMD="$(DOCKER)" bash "$(PROJECT_ROOT)/scripts/check-local-env.sh" "$(COMPOSE_FILE)" "$(ENV_FILE)" "$(ENV_TEMPLATE)" runtime

config: check-env
	@$(COMPOSE) config

pull: check-runtime
	@$(COMPOSE) pull

up: up-core

up-core: check-runtime
	@$(COMPOSE) up -d

up-obs: check-runtime
	@$(COMPOSE) up -d

up-mocks: check-runtime
	@$(COMPOSE) up -d

up-full: check-runtime
	@$(COMPOSE) up -d

ps: check-runtime
	@$(COMPOSE) ps

logs: check-runtime
	@$(COMPOSE) logs -f --tail=200

verify: verify-core

verify-core: check-runtime
	@ENV_FILE=$(ENV_FILE) bash "$(PROJECT_ROOT)/scripts/verify-local-stack.sh" core

verify-obs: check-runtime
	@ENV_FILE=$(ENV_FILE) bash "$(PROJECT_ROOT)/scripts/verify-local-stack.sh" observability

verify-mocks: check-runtime
	@ENV_FILE=$(ENV_FILE) bash "$(PROJECT_ROOT)/scripts/verify-local-stack.sh" mocks

verify-full: check-runtime
	@ENV_FILE=$(ENV_FILE) bash "$(PROJECT_ROOT)/scripts/verify-local-stack.sh" full

down: check-runtime
	@$(COMPOSE) down

stop: check-runtime
	@$(COMPOSE) stop

start: check-runtime
	@$(COMPOSE) start

restart: check-runtime
	@$(COMPOSE) restart

clean: check-runtime
	@$(COMPOSE) down --remove-orphans

destroy: check-runtime
	@$(COMPOSE) down -v --remove-orphans

reset: destroy init up-full
