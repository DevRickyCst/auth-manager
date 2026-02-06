# ============================================================================
# Auth Manager - Makefile
# Production-grade Rust authentication service
# ============================================================================

.PHONY: help
.DEFAULT_GOAL := help

# Docker Compose configurations
COMPOSE_DEV = docker compose -f docker/docker-compose.yml
COMPOSE_TEST = $(COMPOSE_DEV) -f docker/docker-compose.test.yml

# Test parameter (usage: make test t=test_name)
t ?=

# ============================================================================
# Help
# ============================================================================

help: ## Show this help message
	@echo "Auth Manager - Available Commands"
	@echo "=================================="
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' | \
		sort
	@echo ""
	@echo "Usage Examples:"
	@echo "  make local                    # Start development environment"
	@echo "  make test                     # Run all tests"
	@echo "  make test t=test_login        # Run specific test"
	@echo "  make logs                     # Follow all logs"
	@echo "  make shell                    # Open shell in app container"
	@echo ""
	@echo "Lambda Deployment:"
	@echo "  make deploy-create-stack      # Create Lambda stack (first time)"
	@echo "  make deploy                   # Deploy to production Lambda"
	@echo "  make deploy-logs              # View Lambda logs"
	@echo "  make deploy-status            # Show stack status and outputs"

# ============================================================================
# Development Environment
# ============================================================================

local: ## Start local development environment (app + PostgreSQL)
	$(COMPOSE_DEV) up --build

local-detached: ## Start local environment in background
	$(COMPOSE_DEV) up --build -d

stop: ## Stop all running containers
	$(COMPOSE_DEV) down
	$(COMPOSE_TEST) down

restart: ## Restart development environment
	$(COMPOSE_DEV) restart

# ============================================================================
# Database Management
# ============================================================================

migrate: ## Run database migrations
	$(COMPOSE_DEV) run --rm auth-manager diesel migration run

revert: ## Revert last database migration
	$(COMPOSE_DEV) run --rm auth-manager diesel migration revert

db-reset: ## Reset database (WARNING: deletes all data)
	@echo "WARNING: This will delete all data in the database!"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		$(COMPOSE_DEV) down -v; \
		$(COMPOSE_DEV) up -d auth_db; \
		sleep 5; \
		$(COMPOSE_DEV) run --rm auth-manager diesel database setup; \
		echo "Database reset complete!"; \
	else \
		echo "Cancelled."; \
	fi

db-shell: ## Open PostgreSQL shell
	$(COMPOSE_DEV) exec auth_db psql -U postgres -d auth_db

# ============================================================================
# Testing
# ============================================================================

test: ## Run all tests
	$(COMPOSE_TEST) run --rm test-runner bash -c "diesel database setup && cargo test $(t) -- --test-threads=5"
	@$(MAKE) test-cleanup

test-watch: ## Run tests in watch mode
	$(COMPOSE_TEST) run --rm test-runner bash -c "diesel database setup && cargo watch -x 'test $(t) -- --test-threads=5'"
	@$(MAKE) test-cleanup

test-cleanup: ## Cleanup test containers and volumes
	$(COMPOSE_TEST) down -v

# ============================================================================
# Logs & Debugging
# ============================================================================

logs: ## Follow logs from all containers
	$(COMPOSE_DEV) logs -f

logs-app: ## Follow logs from application only
	$(COMPOSE_DEV) logs -f auth-manager

logs-db: ## Follow logs from database only
	$(COMPOSE_DEV) logs -f auth_db

shell: ## Open shell in application container
	$(COMPOSE_DEV) exec auth-manager bash

shell-test: ## Open shell in test runner container
	$(COMPOSE_TEST) run --rm test-runner bash

# ============================================================================
# Code Quality & CI
# ============================================================================

check: ## Run cargo check
	cargo check --all-targets --all-features

fmt: ## Format code with rustfmt
	cargo fmt --all

fmt-check: ## Check code formatting without modifying
	cargo fmt --all -- --check

clippy: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

ci: fmt-check clippy test ## Run all CI checks (format, lint, test)

# ============================================================================
# Local Build & Run (without Docker)
# ============================================================================

build: ## Build the project locally
	cargo build --release

run: ## Run the project locally (requires PostgreSQL and .env)
	cargo run

dev: ## Run with cargo-watch for hot reload
	cargo watch -x run

# ============================================================================
# Lambda Deployment (AWS SAM + ECR)
# ============================================================================

deploy-create-stack: ## Create Lambda infrastructure (first time only)
	AWS_PROFILE=perso ./scripts/deploy-lambda.sh --create-stack

deploy: ## Deploy to Lambda (build + push + update)
	AWS_PROFILE=perso ./scripts/deploy-lambda.sh

deploy-only: ## Update Lambda without rebuilding Docker image
	AWS_PROFILE=perso ./scripts/deploy-lambda.sh --skip-build

deploy-logs: ## View Lambda logs in real-time
	AWS_PROFILE=perso sam logs -n auth-manager-prod --tail --region eu-central-1

deploy-status: ## Show Lambda stack outputs and status
	@echo "Stack Status:"
	@AWS_PROFILE=perso aws cloudformation describe-stacks \
		--stack-name auth-manager-prod \
		--region eu-central-1 \
		--query 'Stacks[0].StackStatus' \
		--output text
	@echo ""
	@echo "Stack Outputs:"
	@AWS_PROFILE=perso aws cloudformation describe-stacks \
		--stack-name auth-manager-prod \
		--region eu-central-1 \
		--query 'Stacks[0].Outputs[].[OutputKey,OutputValue]' \
		--output table

deploy-delete: ## Delete Lambda stack (WARNING: destroys all resources)
	@echo "WARNING: This will delete the entire Lambda stack!"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		AWS_PROFILE=perso sam delete --stack-name auth-manager-prod --region eu-central-1 --no-prompts; \
		echo "Stack deleted!"; \
	else \
		echo "Cancelled."; \
	fi

# ============================================================================
# Cleanup
# ============================================================================

clean: ## Remove build artifacts
	cargo clean
	rm -rf bin/

clean-all: clean ## Remove all artifacts, volumes, and containers
	$(COMPOSE_DEV) down -v --remove-orphans
	$(COMPOSE_TEST) down -v --remove-orphans
	docker volume prune -f
	@echo "All cleaned up!"

# ============================================================================
# Docker Build Context Verification
# ============================================================================

check-context: ## Verify Docker build context size
	@echo "Checking Docker build context size..."
	@docker build -f docker/Dockerfile --no-cache --target base .. 2>&1 | grep "Sending build context" || echo "Build context check complete"
