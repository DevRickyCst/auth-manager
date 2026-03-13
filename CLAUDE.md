# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Production-grade Rust authentication service using Axum, Tokio, Diesel (PostgreSQL), and JWT (HS256). Supports both local HTTP server and AWS Lambda deployment via `lambda_http`. The service handles user authentication, password hashing with bcrypt, JWT generation/validation, refresh token rotation, brute-force protection, and authorization primitives.

**Architecture**: Multi-crate project with a separate WASM-compatible API types crate (`auth-manager-api`) that can be shared with frontend applications.

**Toolchain**: Rust 1.92 (pinned in `rust-toolchain.toml`), components: `rustfmt` + `clippy`, profile: `minimal`.

## Essential Commands

### Bases de données (PostgreSQL géré à la racine du monorepo)
```bash
make db-start       # → docker compose -f ../docker-compose.yml up -d
make db-stop        # → docker compose -f ../docker-compose.yml down (alias: make stop)
make db-logs        # → docker compose -f ../docker-compose.yml logs -f
make db-shell       # → psql $DATABASE_URL (localhost:5432)
make db-shell-prod  # → psql production Neon DB (requires .env.production)
make db-check-prod  # → scripts/neon-check.sh (health check prod)
```

### Local Development
```bash
make local          # hot-reload via cargo watch -x run (alias: make dev)
make run            # cargo run (single run)
make build          # cargo build --release

make migrate        # diesel migration run
make revert         # diesel migration revert
make db-reset       # diesel database reset (WARNING: deletes all data)
make migrate-prod   # run migrations on production Neon (requires .env.production)

make test           # cargo test -- --test-threads=1
make test t=name    # run specific test by name
make test-watch     # cargo watch -x 'test -- --test-threads=1'

make fmt            # cargo fmt --all
make fmt-check      # cargo fmt --all -- --check (no modification)
make clippy         # cargo clippy --all-targets --all-features -- -D warnings
make check          # cargo check --all-targets --all-features
make ci             # fmt-check + clippy + test (full CI pipeline locally)

make clean          # cargo clean + rm -rf bin/
make clean-all      # clean + docker volumes (reset everything)
```

### WASM API Crate
```bash
cargo build --manifest-path auth-manager-api/Cargo.toml --target wasm32-unknown-unknown
cargo test -p auth-manager-api
```

### AWS Lambda Deployment (manual, normally done via CI)
```bash
make deploy-create-stack  # create Lambda infra (first time only)
make deploy               # build + push + update Lambda
make deploy-only          # update Lambda without rebuilding
make deploy-logs          # view Lambda logs in real-time
make deploy-status        # show Lambda stack outputs and status
```

## API Routes

All routes are prefixed with the Lambda function URL or `http://localhost:3000` locally.

| Method | Path | Auth | Description | Response |
|--------|------|------|-------------|----------|
| `GET` | `/health` | — | Health check | 200 |
| `POST` | `/auth/register` | — | Register new user | 201 `UserResponse` |
| `POST` | `/auth/login` | — | Login; sets `refresh_token` HttpOnly cookie | 200 `PublicLoginResponse` |
| `POST` | `/auth/refresh` | Cookie | Rotate refresh token; reads+sets `refresh_token` cookie | 200 `RefreshTokenResponse` |
| `POST` | `/auth/logout` | Bearer JWT | Revoke all refresh tokens | 200 |
| `GET` | `/users/me` | Bearer JWT | Get current user profile | 200 `UserResponse` |
| `GET` | `/users/{id}` | Bearer JWT | Get user by ID | 200 `UserResponse` |
| `DELETE` | `/users/{id}` | Bearer JWT | Delete own account | 200 |
| `POST` | `/users/{id}/change-password` | Bearer JWT | Change password | 200 |

**Cookie details**: `refresh_token` is set as `HttpOnly; Secure; SameSite=None; Path=/auth/refresh`. The raw refresh token value is **never** returned in the response body — only the hash is stored (both in the cookie and in the database).

## Architecture

### Project Structure

```
auth-manager/
├── Cargo.toml                    # Backend package (edition 2024)
├── rust-toolchain.toml           # Rust 1.92, rustfmt + clippy, profile minimal
├── diesel.toml                   # Diesel config (schema output path)
├── .env.example                  # Environment template (copy to .env)
├── auth-manager-api/             # WASM-compatible API types crate
│   ├── Cargo.toml               # Minimal deps: serde, uuid, chrono only
│   └── src/
│       ├── lib.rs               # Public re-exports
│       ├── requests.rs          # Request DTOs
│       ├── responses.rs         # Response DTOs (incl. PublicLoginResponse)
│       ├── error.rs             # ErrorResponse
│       └── result.rs            # AppResponse<T> (WASM-compatible, no Axum)
├── src/                          # Backend code
│   ├── main.rs                  # Entrypoint; selects local vs Lambda mode
│   ├── app.rs                   # Router setup, CORS, middleware
│   ├── config.rs                # Environment detection, Config struct
│   ├── error.rs                 # AppError enum, HTTP mapping
│   ├── response.rs              # Axum IntoResponse wrapper for AppResponse
│   ├── handlers/
│   │   ├── auth.rs              # register, login, refresh_token, logout
│   │   ├── health.rs            # GET /health
│   │   └── user.rs              # get_current_user, get_user_by_id, delete_user, change_password
│   ├── auth/
│   │   ├── jwt.rs               # JWT creation/validation (HS256, JwtManager)
│   │   ├── password.rs          # bcrypt hashing/verification (PasswordManager)
│   │   ├── services.rs          # All authentication business logic (AuthService)
│   │   └── extractors.rs        # Axum extractor: AuthClaims (validates Bearer JWT)
│   └── db/
│       ├── connection.rs        # r2d2 connection pool setup
│       ├── schema.rs            # Auto-generated by Diesel
│       ├── models/
│       │   ├── user.rs          # User, NewUser
│       │   ├── refresh_token.rs # RefreshToken, NewRefreshToken
│       │   └── login_attempt.rs # LoginAttempt, NewLoginAttempt
│       └── repositories/
│           ├── user_repository.rs
│           ├── refresh_token_repository.rs
│           └── login_attempt_repository.rs
├── migrations/                   # Diesel migrations (up.sql + down.sql)
├── infra/                        # Lambda deployment infrastructure
│   ├── template.yaml            # SAM template (provided.al2023, 1024MB, 30s, X-Ray)
│   ├── samconfig.toml
│   └── params/                  # Production params (not committed)
├── scripts/
│   ├── deploy-lambda.sh
│   └── neon-check.sh
├── postman/                      # Postman collection
│   └── AuthManager.postman_collection.json
└── .github/workflows/
    ├── ci.yml                   # Check, fmt, clippy, test, build (on PR)
    └── deploy.yml               # Build + SAM deploy to AWS Lambda (on push)
```

### Layered Design (Mandatory)

1. **API Types Layer** (`auth-manager-api` crate)
   - WASM-compatible pure data structures
   - Request DTOs: `RegisterRequest`, `LoginRequest`, `RefreshTokenRequest`, etc.
   - Response DTOs: `UserResponse`, `PublicLoginResponse`, `RefreshTokenResponse`, etc.
   - `PublicLoginResponse` is derived from `LoginResponse` but **excludes the raw refresh token**
   - Error format: `ErrorResponse`
   - Response wrapper: `AppResponse<T>` (no Axum dependencies)

2. **HTTP Layer** (`src/handlers/`, `src/app.rs`, `src/response.rs`)
   - Handlers are thin: validate input, call services, map errors to HTTP responses
   - NO business logic in handlers
   - `src/response.rs` wraps `auth-manager-api::AppResponse` with Axum's `IntoResponse` trait
   - Handlers use `crate::response::AppResponse` for consistent formatting

3. **Service Layer** (`src/auth/services.rs`)
   - ALL business logic and authentication flows live here
   - JWT creation/validation centralized in `src/auth/jwt.rs`
   - Password hashing/verification centralized in `src/auth/password.rs`
   - Services coordinate between repositories and domain logic

4. **Persistence Layer** (`src/db/`)
   - All database access isolated in `src/db/repositories/`
   - NO Diesel queries outside repositories
   - Models in `src/db/models/`
   - Schema in `src/db/schema.rs` (generated by Diesel — do not edit manually)
   - Connection pooling via r2d2 in `src/db/connection.rs`

### Using AppResponse in Handlers

```rust
use auth_manager_api::{LoginRequest, UserResponse};
use crate::response::AppResponse;

Ok(AppResponse::ok(data))           // 200 OK
Ok(AppResponse::created(data))      // 201 Created
Ok(AppResponse::no_content())       // 204 No Content
Ok(AppResponse::ok(data).with_headers(headers))  // with custom headers
```

### Frontend Integration

```toml
# In frontend Cargo.toml
[dependencies]
auth-manager-api = { path = "../auth-manager/auth-manager-api" }
```

The frontend gets **only**: Request/Response DTOs + lightweight deps (serde, uuid, chrono).
No Axum, Diesel, Tokio, or server code.

## Key Modules

### Backend (`auth-manager`)

- **`src/config.rs`** — `Environment` enum (Local/Dev/Production) with auto-detection; `Config` struct loaded from env vars. CORS origins are driven by `Environment`, not by `FRONTEND_URL`.
- **`src/response.rs`** — Axum `IntoResponse` wrapper for `auth-manager-api::AppResponse`
- **`src/auth/jwt.rs`** — JWT creation, validation, claims (HS256 via `jsonwebtoken`); `JwtManager` holds secret + expiration config
- **`src/auth/password.rs`** — bcrypt hashing and verification (`PasswordManager`)
- **`src/auth/services.rs`** — `AuthService`: all auth flows (register, login, logout, refresh, change_password, brute-force protection)
- **`src/auth/extractors.rs`** — Axum extractor `AuthClaims` that validates Bearer JWT from `Authorization` header
- **`src/error.rs`** — `AppError` enum with HTTP status mapping; no sensitive data leaks
- **`src/app.rs`** — Router composition, CORS layer (environment-based origins), TraceLayer
- **`src/main.rs`** — Entrypoint: detects Lambda vs local mode via `AWS_LAMBDA_FUNCTION_NAME`
- **`src/db/models/login_attempt.rs`** — Tracks login attempts for brute-force protection
- **`src/db/repositories/login_attempt_repository.rs`** — `count_failed_attempts(user_id, window_minutes)`

### API Types (`auth-manager-api`)

- **`src/requests.rs`** — `RegisterRequest`, `LoginRequest`, `RefreshTokenRequest`, `ChangePasswordRequest`
- **`src/responses.rs`** — `UserResponse`, `LoginResponse`, `PublicLoginResponse`, `RefreshTokenResponse`
- **`src/error.rs`** — `ErrorResponse`
- **`src/result.rs`** — `AppResponse<T>` (WASM-compatible)

## Configuration

Environment variables (copy `.env.example` → `.env`):

```env
# App
APP_ENV=development          # "dev" or "development" on Lambda Dev; absent/other → Production
RUST_LOG=debug

# Server (local only; ignored on Lambda)
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/auth_db
TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5433/auth_test_db

# JWT (HS256)
JWT_SECRET=dev_secret_key_change_in_production_12345678
JWT_EXPIRATION_HOURS=1       # access token TTL in hours (default: 1)

# Frontend CORS (informational; actual CORS is controlled by Environment enum in config.rs)
FRONTEND_URL=http://localhost:8080
```

**Validation rules** (enforced by `Config::from_env`):
- `JWT_SECRET` is **required** on Lambda (will panic at startup if absent)
- `JWT_SECRET` must be **≥ 32 characters** on Lambda
- `DATABASE_URL` is **required** on Lambda
- In `Local` mode, `DATABASE_URL` can be assembled from `POSTGRES_USER`, `POSTGRES_PASSWORD`, `DB_HOST`, `DB_PORT`, `POSTGRES_DB`

**Note**: `BCRYPT_COST` is not a configurable variable — bcrypt cost is hardcoded internally.

## Environment & CORS

`src/config.rs` auto-detects the environment:

| Condition | Environment | CORS origins |
|-----------|-------------|--------------|
| No `AWS_LAMBDA_FUNCTION_NAME` | `Local` | `http://localhost:8080`, `http://127.0.0.1:8080`, `http://0.0.0.0:8080` |
| Lambda + `APP_ENV=dev` | `Dev` | `https://dev.dofus-graal.eu`, `http://127.0.0.1:8080` |
| Lambda + other `APP_ENV` | `Production` | `https://dofus-graal.eu` |

## Security

### Password Handling
- All passwords hashed with bcrypt via `PasswordManager`
- NEVER log passwords or plaintext secrets

### JWT (Access Tokens)
- Algorithm: HS256 (`jsonwebtoken` crate)
- Claims: `sub` (user UUID), `exp`, `iat`
- TTL: configurable via `JWT_EXPIRATION_HOURS` (default 1 hour)
- All JWT operations go through `src/auth/jwt.rs` — never create/validate JWTs elsewhere
- Bearer token passed in `Authorization: Bearer <token>` header

### Refresh Tokens
- Generated as UUID v4, **hashed with bcrypt** before storage
- The **bcrypt hash** is stored in the database (`refresh_tokens` table) and set as HttpOnly cookie
- The **raw token** is **never** returned in the response body (only in cookie via `Set-Cookie`)
- Cookie attributes: `HttpOnly; Secure; SameSite=None; Path=/auth/refresh`
- TTL: **7 days**
- Rotation: each `/auth/refresh` call invalidates the old token and issues a new one
- Logout: all refresh tokens for the user are deleted

### Brute-Force Protection
- Tracked via `login_attempts` table
- **5 failed attempts** within a **15-minute window** → account temporarily locked (`TooManyAttempts` error)
- All login attempts (success and failure) are logged with user_id and user_agent

### General
- Validate JWT signature and all claims explicitly (deny-by-default)
- CORS: restrict origins by environment (never `*` in production)
- Secrets never hardcoded — always from environment variables

## CI/CD

### `.github/workflows/ci.yml` — runs on Pull Request to `master` or `prod`

Five parallel jobs:
1. **check** — `cargo check --all-targets --all-features`
2. **fmt** — `cargo fmt --all -- --check`
3. **clippy** — `cargo clippy --all-targets --all-features`
4. **test** — spins up `postgres:17-alpine` on port 5432, runs migrations, then `cargo test --all-features -- --test-threads=1`
5. **build** — `cargo build --release --all-features`

All jobs use `dtolnay/rust-toolchain@1.92` and `Swatinem/rust-cache@v2`.

**Note**: In CI, `TEST_DATABASE_URL` points to the same port 5432 (only one PostgreSQL service in GitHub Actions).

### `.github/workflows/deploy.yml` — runs on Push to `master` or `prod`

| Branch | AWS Environment | SAM Stack |
|--------|-----------------|-----------|
| `master` | `development` | `auth-manager-dev` |
| `prod` | `production` | `auth-manager-prod` |

Steps:
1. Checkout + `dtolnay/rust-toolchain@1.92`
2. Install `cargo-lambda` (pip) + `aws-sam-cli` (pip)
3. Configure AWS credentials via **OIDC** (`aws-actions/configure-aws-credentials@v4`)
4. **Build**: `cargo lambda build --release --x86-64`
   - Uses Zig cross-compiler → produces statically linked binary
   - OpenSSL vendored + pq-sys bundled → no dynamic system libs on Lambda (avoids GLIBC incompatibility)
5. Verify no dynamic `libpq` dependency
6. **Deploy**: `sam deploy` with `DatabaseUrl` and `JwtSecret` from GitHub Secrets

### Lambda Infrastructure (eu-central-1)

- Runtime: `provided.al2023` (custom Rust binary `bootstrap`)
- Memory: **1024 MB**
- Timeout: **30 seconds**
- Tracing: **AWS X-Ray** (Active)
- Production database: **Neon PostgreSQL** (managed, not Docker)

## Testing

Tests are **inline** (`#[cfg(test)]` blocks within source modules) — no separate `tests/` directory.

Prerequisites: `make db-start` (starts postgres-dev on 5432 + postgres-test on 5433).

```bash
make test                      # all tests, sequential (--test-threads=1)
make test t=test_login         # run tests matching "test_login"
make test t=test_name -- --nocapture  # verbose output
make test-watch                # re-run on file changes
```

**Testing Guidelines**:
- Favor unit tests for services and repositories
- Use integration tests for HTTP routes (see `src/app.rs`)
- Tests must be deterministic
- Clean up test data after tests (delete created users/tokens)
- Avoid coupling tests to handlers

## Common Workflows

### Adding New Endpoint

1. **Create DTOs in `auth-manager-api`**: add to `requests.rs` / `responses.rs` (WASM-compatible, no Axum/Diesel)
2. **Business logic** in `src/auth/services.rs` (or new service module)
3. **Repository methods** in `src/db/repositories/` if needed
4. **Handler** in `src/handlers/`:
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
6. **Add tests**

### Database Schema Changes

1. `diesel migration generate migration_name`
2. Write `up.sql` and `down.sql` in `migrations/`
3. `make migrate` (applies locally)
4. Update models in `src/db/models/` if needed
5. Update repositories in `src/db/repositories/` if needed
6. `src/db/schema.rs` is auto-generated — do not edit manually

### Working with JWT

All JWT operations go through `src/auth/jwt.rs`. Never create or validate JWTs outside this module.

### Error Mapping

Application errors flow through `src/error.rs`. When adding new error types, add proper HTTP status mapping and ensure no sensitive data leaks.

## Execution Modes

Detected automatically in `src/main.rs` based on `AWS_LAMBDA_FUNCTION_NAME`:

- **Local**: standard Axum HTTP server under Tokio (port 3000 by default)
- **Lambda**: `lambda_http` adapter for serverless execution

Environment sub-type (`Local` / `Dev` / `Production`) further determined by `APP_ENV` — see `src/config.rs`.

## Code Style & Rules

### Mandatory Architecture Rules

- **API types belong in `auth-manager-api`** — Request/Response DTOs must be WASM-compatible
- **Handlers import from `auth_manager_api`** — use `use auth_manager_api::{...}` for DTOs
- **Handlers use `crate::response::AppResponse`** — for Axum integration and header support
- **HTTP handlers must be thin** — no business logic
- **All auth flows in `src/auth/services.rs`**
- **Database access isolated in `src/db/repositories/`** — no Diesel queries outside
- **JWT in `src/auth/jwt.rs`** — never elsewhere
- **Password hashing in `src/auth/password.rs`** — all bcrypt operations here
- **NO global mutable state** — use dependency injection
- **Secrets never hardcoded** — always from environment variables

### Rust Style

- Idiomatic Rust, edition 2024
- Prefer explicit types over inference in public APIs
- `Result<T, E>` consistently; NEVER `unwrap()` or `expect()` in production code
- Early returns over nested conditionals; small, single-purpose functions
- `chrono` with UTC only; UUIDs for identifiers

### Error Handling

- Add context with `anyhow::Context`
- NEVER leak sensitive info in errors (passwords, tokens, keys)
- Use explicit domain error types; `anyhow` only at boundaries

### Async & Performance

- All I/O must be async; NEVER blocking calls in async contexts
- Use r2d2 connection pooling correctly
- No heavy computation in request handlers

### Logging

- Use `tracing` macros: `info!`, `warn!`, `error!`, `instrument`
- NEVER log secrets, tokens, passwords, or raw refresh tokens

## Resources

- Health endpoint: `GET /health`
- Postman collection: `postman/AuthManager.postman_collection.json`
- Production DB: Neon PostgreSQL (managed); use `make db-shell-prod` / `make db-check-prod`

## Notes

- This project does NOT provide UI, email workflows, or OAuth/social logins
- `src/db/models/user_identity.rs` exists but is currently unused (model commented out) — do not build on it
- Keep queries inside repositories; avoid raw SQL in handlers/services
- Format: GitHub-flavored markdown when documenting
