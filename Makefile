.PHONY: help build up down logs restart clean test backup restore

# Load environment variables
ifneq (,$(wildcard ./.env))
    include .env
    export
endif

# Colors for output
BLUE=\033[0;34m
GREEN=\033[0;32m
YELLOW=\033[1;33m
RED=\033[0;31m
NC=\033[0m # No Color

help: ## Show this help message
	@echo "$(BLUE)Collateral Vault Management System - Docker Commands$(NC)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""

# ==============================================================================
# DEVELOPMENT COMMANDS
# ==============================================================================

build: ## Build all Docker images
	@echo "$(BLUE)Building Docker images...$(NC)"
	docker-compose build --parallel

build-backend: ## Build only backend image
	@echo "$(BLUE)Building backend image...$(NC)"
	docker-compose build backend

build-frontend: ## Build only frontend image
	@echo "$(BLUE)Building frontend image...$(NC)"
	docker-compose build frontend

up: ## Start all services
	@echo "$(GREEN)Starting all services...$(NC)"
	docker-compose up -d
	@echo "$(GREEN)Services started!$(NC)"
	@echo "Frontend: http://localhost:$(FRONTEND_PORT:-3000)"
	@echo "Backend:  http://localhost:$(BACKEND_PORT:-8080)"

down: ## Stop all services
	@echo "$(YELLOW)Stopping all services...$(NC)"
	docker-compose down

restart: ## Restart all services
	@echo "$(YELLOW)Restarting all services...$(NC)"
	docker-compose restart

logs: ## View logs from all services
	docker-compose logs -f

logs-backend: ## View backend logs
	docker-compose logs -f backend

logs-frontend: ## View frontend logs
	docker-compose logs -f frontend

logs-postgres: ## View postgres logs
	docker-compose logs -f postgres

ps: ## List running containers
	docker-compose ps

# ==============================================================================
# DATABASE COMMANDS
# ==============================================================================

db-shell: ## Open PostgreSQL shell
	@echo "$(BLUE)Opening PostgreSQL shell...$(NC)"
	docker-compose exec postgres psql -U $(POSTGRES_USER:-vault_user) -d $(POSTGRES_DB:-collateral_vault)

db-backup: ## Backup database
	@echo "$(BLUE)Creating database backup...$(NC)"
	@mkdir -p backups
	docker-compose exec -T postgres pg_dump -U $(POSTGRES_USER:-vault_user) $(POSTGRES_DB:-collateral_vault) > backups/backup_$(shell date +%Y%m%d_%H%M%S).sql
	@echo "$(GREEN)Backup created in backups/ directory$(NC)"

db-restore: ## Restore database from latest backup (use FILE=path/to/backup.sql to specify)
	@echo "$(YELLOW)Restoring database from backup...$(NC)"
	@if [ -z "$(FILE)" ]; then \
		LATEST=$$(ls -t backups/*.sql | head -1); \
		docker-compose exec -T postgres psql -U $(POSTGRES_USER:-vault_user) $(POSTGRES_DB:-collateral_vault) < $$LATEST; \
	else \
		docker-compose exec -T postgres psql -U $(POSTGRES_USER:-vault_user) $(POSTGRES_DB:-collateral_vault) < $(FILE); \
	fi
	@echo "$(GREEN)Database restored!$(NC)"

db-reset: ## Reset database (WARNING: destroys all data)
	@echo "$(RED)WARNING: This will delete all data!$(NC)"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		docker-compose down -v; \
		docker-compose up -d postgres; \
		sleep 5; \
		docker-compose up -d; \
		echo "$(GREEN)Database reset complete!$(NC)"; \
	fi

migrate: ## Run database migrations
	@echo "$(BLUE)Running database migrations...$(NC)"
	docker-compose exec postgres psql -U $(POSTGRES_USER:-vault_user) -d $(POSTGRES_DB:-collateral_vault) -f /docker-entrypoint-initdb.d/001_initial_schema.sql
	docker-compose exec postgres psql -U $(POSTGRES_USER:-vault_user) -d $(POSTGRES_DB:-collateral_vault) -f /docker-entrypoint-initdb.d/002_tvl_snapshots.sql
	docker-compose exec postgres psql -U $(POSTGRES_USER:-vault_user) -d $(POSTGRES_DB:-collateral_vault) -f /docker-entrypoint-initdb.d/003_mfa_support.sql
	docker-compose exec postgres psql -U $(POSTGRES_USER:-vault_user) -d $(POSTGRES_DB:-collateral_vault) -f /docker-entrypoint-initdb.d/004_allow_multiple_snapshots_per_day.sql
	@echo "$(GREEN)Migrations complete!$(NC)"

# ==============================================================================
# TESTING & QUALITY
# ==============================================================================

test: ## Run tests in Docker
	@echo "$(BLUE)Running tests...$(NC)"
	docker-compose run --rm backend cargo test
	@echo "$(GREEN)Tests complete!$(NC)"

test-backend: ## Run backend tests
	@echo "$(BLUE)Running backend tests...$(NC)"
	docker-compose exec backend cargo test

lint-backend: ## Run backend linter
	@echo "$(BLUE)Running Rust linter...$(NC)"
	docker-compose exec backend cargo clippy -- -D warnings

fmt-backend: ## Format backend code
	@echo "$(BLUE)Formatting Rust code...$(NC)"
	docker-compose exec backend cargo fmt

check-backend: ## Check backend without building
	@echo "$(BLUE)Checking backend...$(NC)"
	docker-compose exec backend cargo check

# ==============================================================================
# CLEANUP COMMANDS
# ==============================================================================

clean: ## Remove all containers, volumes, and images
	@echo "$(RED)Cleaning up all Docker resources...$(NC)"
	docker-compose down -v --rmi all --remove-orphans
	@echo "$(GREEN)Cleanup complete!$(NC)"

clean-volumes: ## Remove all volumes (WARNING: destroys data)
	@echo "$(RED)Removing all volumes...$(NC)"
	docker-compose down -v
	@echo "$(GREEN)Volumes removed!$(NC)"

clean-images: ## Remove all project images
	@echo "$(YELLOW)Removing project images...$(NC)"
	docker-compose down --rmi all
	@echo "$(GREEN)Images removed!$(NC)"

prune: ## Prune unused Docker resources
	@echo "$(YELLOW)Pruning unused Docker resources...$(NC)"
	docker system prune -af --volumes
	@echo "$(GREEN)Prune complete!$(NC)"

# ==============================================================================
# PRODUCTION COMMANDS
# ==============================================================================

prod-build: ## Build production images
	@echo "$(BLUE)Building production images...$(NC)"
	docker-compose -f docker-compose.prod.yml build --parallel

prod-up: ## Start production services
	@echo "$(GREEN)Starting production services...$(NC)"
	docker-compose -f docker-compose.prod.yml up -d

prod-down: ## Stop production services
	@echo "$(YELLOW)Stopping production services...$(NC)"
	docker-compose -f docker-compose.prod.yml down

prod-logs: ## View production logs
	docker-compose -f docker-compose.prod.yml logs -f

prod-ps: ## List production containers
	docker-compose -f docker-compose.prod.yml ps

# ==============================================================================
# MONITORING & DEBUGGING
# ==============================================================================

health: ## Check health of all services
	@echo "$(BLUE)Checking service health...$(NC)"
	@docker-compose ps
	@echo ""
	@echo "$(BLUE)Backend health:$(NC)"
	@curl -s http://localhost:$(BACKEND_PORT:-8080)/health | jq . || echo "$(RED)Backend not responding$(NC)"
	@echo ""
	@echo "$(BLUE)Frontend health:$(NC)"
	@curl -s -o /dev/null -w "Status: %{http_code}\n" http://localhost:$(FRONTEND_PORT:-3000)

stats: ## Show container resource usage
	docker stats --no-stream

exec-backend: ## Execute shell in backend container
	docker-compose exec backend /bin/bash

exec-frontend: ## Execute shell in frontend container
	docker-compose exec frontend /bin/sh

exec-postgres: ## Execute shell in postgres container
	docker-compose exec postgres /bin/bash

# ==============================================================================
# UTILITY COMMANDS
# ==============================================================================

env: ## Copy .env.example to .env if it doesn't exist
	@if [ ! -f .env ]; then \
		cp .env.example .env; \
		echo "$(GREEN).env file created! Please edit it with your configuration.$(NC)"; \
	else \
		echo "$(YELLOW).env file already exists!$(NC)"; \
	fi

update: ## Pull latest changes and rebuild
	@echo "$(BLUE)Updating application...$(NC)"
	git pull
	docker-compose down
	docker-compose build --no-cache
	docker-compose up -d
	@echo "$(GREEN)Update complete!$(NC)"

version: ## Show version information
	@echo "$(BLUE)Docker Version:$(NC)"
	@docker --version
	@echo "$(BLUE)Docker Compose Version:$(NC)"
	@docker-compose --version

config: ## Validate and view docker-compose configuration
	docker-compose config

# ==============================================================================
# QUICK START
# ==============================================================================

setup: env build up ## First-time setup (create .env, build, and start)
	@echo "$(GREEN)Setup complete!$(NC)"
	@echo "Frontend: http://localhost:$(FRONTEND_PORT:-3000)"
	@echo "Backend:  http://localhost:$(BACKEND_PORT:-8080)"

dev: build up logs ## Development mode (build, start, and follow logs)

stop: down ## Alias for down

start: up ## Alias for up
