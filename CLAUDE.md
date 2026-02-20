# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Production-grade Rust authentication service using Axum, Tokio, Diesel (PostgreSQL), and JWT (HS256). Supports both local HTTP server and AWS Lambda deployment via `lambda_http`. The service handles user authentication, password hashing with bcrypt, JWT generation/validation, and authorization primitives.

**Architecture**: Multi-crate project with a separate WASM-compatible API types crate (`auth-manager-api`) that can be shared with frontend applications.

## Essential Commands

### Bases de données (PostgreSQL géré à la racine du monorepo)
```bash
# Démarrer postgres-dev (5432) + postgres-test (5433, tmpfs)
make db-start       # → docker compose -f ../docker-compose.yml up -d
make db-stop        # → docker compose -f ../docker-compose.yml down
make db-logs        # → docker compose -f ../docker-compose.yml logs -f
make db-shell       # → psql $DATABASE_URL (localhost:5432)
```

### Local Development
```bash
# Hot-reload (nécessite cargo-watch)
make local          # → cargo watch -x run

# Lancer une fois
cargo run

# Migrations
make migrate        # → diesel migration run
make revert         # → diesel migration revert
make db-reset       # → diesel database reset (SUPPRIME TOUTES LES DONNÉES)

# Tests (nécessite make db-start)
make test           # → cargo test -- --test-threads=1
make test t=test_name
make test-watch     # → cargo watch -x 'test -- --test-threads=1'

# Build
cargo build --release

# Format and lint
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings

# Build API crate for WASM
cargo build --manifest-path auth-manager-api/Cargo.toml --target wasm32-unknown-unknown

# Test API crate independently
cargo test -p auth-manager-api
```

### AWS Lambda Deployment
```bash
# Create Lambda infrastructure (first time only)
make deploy-create-stack

# Complete deployment pipeline (build + push + update)
make deploy

# Update Lambda without rebuilding Docker image
make deploy-only

# View Lambda logs in real-time
make deploy-logs

# Show Lambda stack outputs and status
make deploy-status
```

### Database Operations
```bash
# Les bases PostgreSQL sont gérées par docker-compose.yml à la RACINE du monorepo
# postgres-dev → localhost:5432  (volume persistant)
# postgres-test → localhost:5433 (tmpfs, pour les tests)

make db-start                    # démarrer les deux bases
diesel migration run             # ou: make migrate
diesel migration revert          # ou: make revert
diesel migration generate name   # créer une migration
diesel database reset            # ou: make db-reset
make db-shell                    # psql sur localhost:5432
```

## Architecture

### Project Structure

```
auth-manager/
├── Cargo.toml                    # Backend package
├── auth-manager-api/             # WASM-compatible API types crate
│   ├── Cargo.toml               # Minimal dependencies (serde, uuid, chrono)
│   └── src/
│       ├── lib.rs
│       ├── requests.rs           # Request DTOs
│       ├── responses.rs          # Response DTOs
│       ├── error.rs              # ErrorResponse
│       └── result.rs             # AppResponse (WASM-compatible)
├── src/                          # Backend code
│   ├── response.rs               # Axum integration wrapper
│   ├── handlers/
│   ├── auth/
│   └── db/
├── migrations/                   # Migrations Diesel
├── infra/                        # Infrastructure de déploiement Lambda
│   ├── Dockerfile                # Image Lambda (builder → runtime ECR)
│   ├── template.yaml             # SAM template
│   ├── samconfig.toml
│   └── params/                   # Paramètres prod (non commités)
└── scripts/
    ├── deploy-lambda.sh
    └── neon-check.sh
```

### Layered Design (Mandatory)

The codebase follows strict layering to separate concerns:

1. **API Types Layer** (`auth-manager-api` crate)
   - WASM-compatible pure data structures
   - Request DTOs: `RegisterRequest`, `LoginRequest`, `RefreshTokenRequest`, etc.
   - Response DTOs: `UserResponse`, `LoginResponse`, `RefreshTokenResponse`, etc.
   - Error format: `ErrorResponse`
   - Response wrapper: `AppResponse<T>` (no Axum dependencies)
   - Can be imported by frontend applications without server dependencies

2. **HTTP Layer** (`src/handlers/`, `src/app.rs`, `src/response.rs`)
   - Axum route handlers are thin - they ONLY validate input, call services, and map errors to HTTP responses
   - NO business logic in handlers
   - `src/response.rs` wraps `auth-manager-api::AppResponse` with Axum's `IntoResponse` trait
   - Handlers import types from `auth_manager_api` and use `crate::response::AppResponse`

3. **Service Layer** (`src/auth/services.rs`)
   - ALL business logic and authentication flows live here
   - JWT creation/validation centralized in `src/auth/jwt.rs`
   - Password hashing/verification centralized in `src/auth/password.rs`
   - Services coordinate between repositories and domain logic

4. **Persistence Layer** (`src/db/`)
   - All database access isolated in `src/db/repositories/`
   - NO Diesel queries outside repositories
   - Models in `src/db/models/`
   - Schema in `src/db/schema.rs` (generated by Diesel)
   - Connection pooling via r2d2 in `src/db/connection.rs`

### Using AppResponse in Handlers

All handlers should use `crate::response::AppResponse<T>` for consistent response formatting:

```rust
use auth_manager_api::{LoginRequest, UserResponse};
use crate::response::AppResponse;

// 200 OK
Ok(AppResponse::ok(data))

// 201 Created
Ok(AppResponse::created(data))

// 204 No Content
Ok(AppResponse::no_content())

// With custom headers
Ok(AppResponse::ok(data).with_headers(headers))
```

### Frontend Integration

Frontend applications can import the API types crate without pulling in server dependencies:

```toml
# In frontend Cargo.toml
[dependencies]
auth-manager-api = { path = "../auth-manager/auth-manager-api" }
```

The frontend will get **ONLY**:
- ✅ Request/Response DTOs
- ✅ Lightweight dependencies (serde, uuid, chrono)
- ❌ No Axum, Diesel, Tokio, or server code

The API crate is WASM-compatible and can be built with:
```bash
cargo build --manifest-path auth-manager-api/Cargo.toml --target wasm32-unknown-unknown
```

### Key Modules

#### Backend (`auth-manager`)

- **`src/response.rs`** - Axum wrapper for `auth-manager-api::AppResponse` with `IntoResponse` trait
- **`src/auth/jwt.rs`** - JWT creation, validation, claims handling (HS256 with jsonwebtoken)
- **`src/auth/password.rs`** - bcrypt hashing and verification
- **`src/auth/services.rs`** - Authentication and authorization services
- **`src/auth/extractors.rs`** - Axum extractors for JWT validation
- **`src/error.rs`** - Application-wide error types and HTTP mapping
- **`src/app.rs`** - Router setup, middleware configuration (CORS, tracing)
- **`src/main.rs`** - Entrypoint; selects execution mode (local vs Lambda)

#### API Types (`auth-manager-api`)

- **`src/lib.rs`** - Public API with re-exports
- **`src/requests.rs`** - Request DTOs (RegisterRequest, LoginRequest, etc.)
- **`src/responses.rs`** - Response DTOs (UserResponse, LoginResponse, etc.)
- **`src/error.rs`** - ErrorResponse format
- **`src/result.rs`** - AppResponse wrapper (WASM-compatible, no Axum)

## Configuration

Environment variables required (copier `.env.example` → `.env`):

```env
# App
APP_ENV=development
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=debug

# Database (PostgreSQL géré par docker-compose.yml à la racine du monorepo)
DATABASE_URL=postgres://postgres:postgres@localhost:5432/auth_db
TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5433/auth_test_db

# CORS
FRONTEND_URL=http://localhost:8080

# JWT (HS256)
JWT_SECRET=your_secret_key_here

# Password
BCRYPT_COST=12
```

## Code Style & Rules

### Mandatory Architecture Rules

- **API types belong in `auth-manager-api` crate** - Request/Response DTOs must be WASM-compatible
- **Handlers import from `auth_manager_api`** - Use `use auth_manager_api::{...}` for DTOs
- **Handlers use `crate::response::AppResponse`** - For Axum integration and header support
- **HTTP handlers must be thin** - No business logic in handlers
- **Core logic lives in `src/auth/services.rs`** - All authentication flows centralized
- **Database access isolated in `src/db/repositories/`** - NO Diesel queries outside repositories
- **JWT logic centralized in `src/auth/jwt.rs`** - Never create/validate JWTs elsewhere
- **Password hashing centralized in `src/auth/password.rs`** - All bcrypt operations here
- **NO global mutable state** - Use dependency injection
- **Secrets and keys are NEVER hardcoded** - Always use environment variables

### Rust Style

- Idiomatic Rust
- Prefer explicit types over inference in public APIs
- Use `Result<T, E>` consistently
- NEVER use `unwrap()` or `expect()` in production code
- Early returns over nested conditionals
- Functions should be small and single-purpose
- Use `chrono` with UTC only
- UUIDs for identifiers

### Error Handling

- Add context to errors using `anyhow::Context`
- NEVER leak sensitive information in errors (passwords, tokens, keys)
- Map internal errors to appropriate HTTP responses in handlers
- Use explicit domain error types; `anyhow` only at boundaries

### Async & Performance

- All I/O must be async
- NEVER use blocking calls in async contexts
- Use connection pooling (r2d2) correctly
- NO heavy computation in request handlers

### Security

- Always hash passwords with bcrypt
- NEVER log secrets, tokens, or passwords
- JWTs must include expiration (`exp`), issued at (`iat`), and subject (`sub`)
- Validate JWT signature and all claims explicitly
- Prefer deny-by-default authorization logic
- Restrict CORS origins; avoid `*` in production
- Refresh tokens stored as hashes in database
- HttpOnly cookies for refresh token storage

## Testing

Les tests utilisent `postgres-test` sur `localhost:5433` (tmpfs, rapide, pas de persistance).
Prérequis : `make db-start` depuis auth-manager (ou `docker compose up -d` depuis la racine).

```bash
# Run all tests
make test           # → cargo test -- --test-threads=1

# Run specific test
make test t=test_login

# Run with more verbose output
make test t=test_name -- --nocapture

# Run tests in watch mode
make test-watch
```

### Testing Guidelines

- Favor unit tests for services and repositories
- Use integration tests for HTTP routes
- Mock database access when possible
- Tests must be deterministic
- Avoid coupling tests to handlers
- Clean up test data after tests (delete created users/tokens)

## Common Workflows

### Adding New Endpoint

1. **Create DTOs in `auth-manager-api` crate**:
   - Add request DTO in `auth-manager-api/src/requests.rs`
   - Add response DTO in `auth-manager-api/src/responses.rs`
   - Ensure types are WASM-compatible (no Axum/Diesel dependencies)

2. **Implement business logic**:
   - Add logic in `src/auth/services.rs` (or create new service module)
   - Services use DTOs from `auth_manager_api`

3. **Create repository methods** (if needed):
   - Add methods in `src/db/repositories/`
   - Keep database access isolated

4. **Create handler in `src/handlers/`**:
   ```rust
   use auth_manager_api::{MyRequest, MyResponse};
   use crate::response::AppResponse;
   use crate::error::AppError;

   pub async fn my_handler(
       Json(payload): Json<MyRequest>,
   ) -> Result<AppResponse<MyResponse>, AppError> {
       let result = MyService::handle(payload)?;
       Ok(AppResponse::ok(result))
   }
   ```

5. **Register route** in `src/app.rs`

6. **Add tests** - Test both API crate types and backend logic

### Database Schema Changes

1. Create migration: `diesel migration generate migration_name`
2. Write `up.sql` and `down.sql` in `migrations/`
3. Apply migration: `make migrate` (or `diesel migration run`)
4. Update models in `src/db/models/` if needed
5. Update repositories in `src/db/repositories/` if needed
6. Schema in `src/db/schema.rs` is auto-generated by Diesel

### Working with JWT

All JWT operations go through `src/auth/jwt.rs`. Never create or validate JWTs outside this module. The module uses HS256 (HMAC with SHA-256) with a secret key from environment variables.

### Error Mapping

Application errors flow through `src/error.rs` which maps domain errors to HTTP status codes. When adding new error types, ensure proper mapping to HTTP responses and that no sensitive data leaks.

## Execution Modes

The service has two runtime modes selected in `src/main.rs`:

- **Local HTTP Server**: Standard Axum server under Tokio for development (port 3000)
- **AWS Lambda**: Uses `lambda_http` adapter for serverless deployment

Detection is based on `AWS_LAMBDA_FUNCTION_NAME` environment variable.

## Resources

- Health endpoint: `GET /health`
- Postman collection: `postman/AuthManager.postman_collection.json`

## Notes

- This is Rust edition 2024
- The project does NOT provide UI, email workflows, or OAuth/social logins
- Keep queries inside repositories; avoid raw SQL in handlers/services
- Use `tracing` macros (`info!`, `warn!`, `error!`, `instrument`) for logging
- Format: GitHub-flavored markdown when documenting
