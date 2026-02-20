# ============================================================================
# Auth Manager - Makefile
# Production-grade Rust authentication service
# ============================================================================

.PHONY: help
.DEFAULT_GOAL := help

# Root docker-compose (PostgreSQL only)
ROOT_COMPOSE = docker compose -f ../docker-compose.yml

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
	@echo "  make db-start                 # Start both PostgreSQL databases"
	@echo "  make local                    # Start app with hot reload"
	@echo "  make test                     # Run all tests"
	@echo "  make test t=test_login        # Run specific test"
	@echo "  make db-logs                  # Follow database logs"
	@echo ""
	@echo "Lambda Deployment:"
	@echo "  make deploy-create-stack      # Create Lambda stack (first time)"
	@echo "  make deploy                   # Deploy to production Lambda"
	@echo "  make deploy-logs              # View Lambda logs"
	@echo "  make deploy-status            # Show stack status and outputs"

# ============================================================================
# Database Management (via root docker-compose)
# ============================================================================

db-start: ## Start both PostgreSQL databases (dev port 5432 + test port 5433)
	$(ROOT_COMPOSE) up -d

db-stop: ## Stop both PostgreSQL databases
	$(ROOT_COMPOSE) down

db-logs: ## Follow database logs
	$(ROOT_COMPOSE) logs -f

# ============================================================================
# Development Environment
# ============================================================================

local: ## Run app locally with hot reload (requires db-start)
	cargo watch -x run

stop: db-stop ## Stop all databases

# ============================================================================
# Database Operations
# ============================================================================

migrate: ## Run database migrations
	diesel migration run

migrate-prod: ## Run database migrations (production Neon)
	@echo "🚀 Running migrations on production database..."
	@if [ ! -f .env.production ]; then \
		echo "❌ .env.production not found!"; \
		exit 1; \
	fi
	@export $$(cat .env.production | grep -v '^#' | xargs) && \
		diesel migration run && \
		echo "✅ Migrations completed successfully!"

revert: ## Revert last database migration
	diesel migration revert

db-reset: ## Reset database (WARNING: deletes all data)
	@echo "WARNING: This will delete all data in the database!"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		diesel database reset; \
		echo "Database reset complete!"; \
	else \
		echo "Cancelled."; \
	fi

db-shell: ## Open PostgreSQL shell (local dev)
	psql $(DATABASE_URL)

db-shell-prod: ## Open PostgreSQL shell (production Neon)
	@if [ ! -f .env.production ]; then \
		echo "❌ .env.production not found!"; \
		exit 1; \
	fi
	@export $$(cat .env.production | grep -v '^#' | xargs) && \
		psql $$DATABASE_URL

db-check-prod: ## Check production database health
	@./scripts/neon-check.sh

# ============================================================================
# Testing
# ============================================================================

test: ## Run all tests (requires db-start)
	@echo "🧪 Running tests..."
	@cargo test $(t) -- --test-threads=1

test-watch: ## Run tests in watch mode
	cargo watch -x 'test $(t) -- --test-threads=1'

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

run: ## Run the project locally (requires db-start and .env)
	cargo run

dev: ## Run with cargo-watch for hot reload
	cargo watch -x run

# ============================================================================
# Lambda Deployment (AWS SAM + ECR)
# ============================================================================

deploy-create-stack: ## Create Lambda infrastructure (first time only)
	AWS_PROFILE=perso ./scripts/deploy-lambda.sh --create-stack

deploy: ## Deploy to Lambda (build + push + update)
	@if [ ! -f infra/params/prod.json ]; then \
		echo "❌ infra/params/prod.json not found!"; \
		echo "Create it from template: cp infra/params/prod.json.example infra/params/prod.json"; \
		echo "Then edit it with your production credentials."; \
		exit 1; \
	fi
	@echo "✅ Using credentials from infra/params/prod.json"
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

clean-all: clean ## Remove all artifacts, volumes, and databases
	$(ROOT_COMPOSE) down -v
	docker volume prune -f
	@echo "All cleaned up!"

# ============================================================================
# Docker Build Context Verification
# ============================================================================

check-context: ## Verify Docker build context size
	@echo "Checking Docker build context size..."
	@docker build -f infra/Dockerfile --no-cache --target builder . 2>&1 | grep "Sending build context" || echo "Build context check complete"
