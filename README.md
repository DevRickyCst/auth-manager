# auth-manager

Rust authentication service built on Axum, Tokio, Diesel (PostgreSQL), and JSON Web Tokens. It provides user authentication, password hashing, JWT generation/validation, and authorization primitives (roles/permissions). It targets both a local HTTP server and AWS Lambda via `lambda_http`.

## Project Overview
- What it does:
  - Exposes HTTP endpoints for authentication and authorization concerns.
  - Hashes passwords with bcrypt.
  - Issues and validates JWTs (ES256 via `jsonwebtoken` with `p256`).
  - Encodes authorization logic (roles/permissions) in the service layer.
  - Persists data in PostgreSQL through Diesel with `r2d2` pooling.
- What it does not do:
  - Provide a UI, email workflows, or OAuth/social logins.
  - Implement unrelated business logic in HTTP handlers.
  - Manage non-auth domain entities beyond what’s required for login, refresh, and identity.

## Architecture
- HTTP Layer (Axum):
  - Request/response DTOs in [src/dto](src/dto).
  - Route handlers in [src/handlers](src/handlers) orchestrate services, perform validation, and map errors to HTTP responses.
  - No business logic inside handlers; they delegate to the service layer.
- Service Layer (Auth and domain services):
  - Authentication flows, password checks, token issuing/validation in [src/auth](src/auth).
  - JWT logic is centralized in [src/auth/jwt.rs](src/auth/jwt.rs) (key parsing, claims, signing, validation).
  - Password hashing and verification in [src/auth/password.rs](src/auth/password.rs).
- Persistence Layer (Diesel + r2d2):
  - Database schema in [src/db/schema.rs](src/db/schema.rs) (generated from Diesel migrations).
  - Data models in [src/db/models](src/db/models).
  - Repositories in [src/db/repositories](src/db/repositories) encapsulate all DB access.
  - Connection management and errors in [src/db](src/db).
- Application wiring:
  - Router and app setup in [src/app.rs](src/app.rs).
  - Entrypoint(s) in [src/main.rs](src/main.rs).
  - Error types in [src/error.rs](src/error.rs).
  - Utilities, response helpers, and CORS/trace config in [src/utils](src/utils).
- Observability:
  - `tracing` + `tracing-subscriber` for structured logs.
  - `tower-http` (trace, cors) for HTTP-level tracing and CORS.

## Execution Modes
- Local HTTP Server:
  - Runs a standard Axum server under Tokio.
  - Suitable for development and local integration tests.
- AWS Lambda:
  - Uses `lambda_http` to serve Axum-compatible handlers in a serverless context.
  - Deployable as a custom runtime (Amazon Linux 2) with a `bootstrap` executable.

## Project Structure
- diesel.toml — Diesel CLI configuration.
- migrations — Diesel migrations (initial setup, users, etc.).
- src/
  - app.rs — Router and middleware setup.
  - main.rs — Entrypoint; selects execution mode and starts the app.
  - error.rs — Application error types and mapping.
  - auth/
    - jwt.rs — JWT creation/validation (ES256), claims, keys.
    - password.rs — bcrypt hashing/verification.
    - services.rs — Authentication and authorization services.
  - db/
    - connection.rs — Pooling and connection setup.
    - error.rs — DB-specific error mapping.
    - schema.rs — Diesel `table!` definitions.
    - models/ — Domain models (user, identity, tokens, login attempts).
    - repositories/ — DB queries encapsulated behind repository APIs.
  - dto/ — Request/response DTOs.
  - handlers/ — Axum handlers (auth, user, etc.).
  - utils/ — Response builders, helpers.
- docker/ — Compose files and Dockerfile for local infra.

## Configuration & Environment Variables
Define configuration via environment variables. Typical setup:

```env
# App
APP_ENV=development            # development|staging|production
SERVER_HOST=127.0.0.1          # local mode only
SERVER_PORT=8080               # local mode only
RUST_LOG=info                  # tracing level

# Database (Diesel)
DATABASE_URL=postgres://user:pass@localhost:5432/auth_manager
DB_POOL_MAX_SIZE=15            # r2d2 pool size

# CORS
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
CORS_ALLOWED_METHODS=GET,POST,PUT,DELETE

# JWT (ES256)
JWT_ISSUER=auth-manager
JWT_AUDIENCE=auth-manager-client
JWT_ACCESS_TTL_MIN=15          # access token lifetime
JWT_REFRESH_TTL_MIN=43200      # refresh token lifetime (e.g., 30 days)
JWT_PRIVATE_KEY_PATH=./secrets/es256-private.pem
JWT_PUBLIC_KEY_PATH=./secrets/es256-public.pem

# Password hashing
BCRYPT_COST=12                 # work factor; adjust per environment
```

Notes:
- Keys: ES256 requires a P-256 ECDSA key pair in PEM format. Store securely and mount via file paths or secrets manager.
- Values may differ per environment; prefer production overrides via your deployment system.
- Any additional variables should be documented alongside their module (e.g., see JWT module for claim details).

## Database & Migrations (Diesel)
- Ensure PostgreSQL is available and `DATABASE_URL` is set.
- Install Diesel CLI if not already:

```bash
cargo install diesel_cli --no-default-features --features postgres
```

- Initialize and run migrations:

```bash
diesel setup
diesel migration run
```

- Updating schema:
  - After modifying migrations, regenerate schema into [src/db/schema.rs](src/db/schema.rs) as needed using `diesel print-schema` or the project’s workflow.
- Keep queries inside repositories and avoid raw SQL in handlers/services.

## Security Considerations
- Passwords:
  - Always store bcrypt hashes, never plaintext.
  - Choose `BCRYPT_COST` appropriate to environment; benchmark and tune for production.
- JWT:
  - Use ES256 with strong, rotated keys; restrict access to private key.
  - Validate `iss`, `aud`, `exp`, `iat`, and signature; reject tokens failing any checks.
  - Keep JWT logic centralized in [src/auth/jwt.rs](src/auth/jwt.rs) for consistency.
- Secrets Management:
  - Do not commit keys or credentials; use environment-specific secret stores.
  - Limit exposure via file mounts or env vars; prefer IAM/parameter stores in cloud.
- CORS & HTTP:
  - Restrict allowed origins/methods; avoid `*` in production.
  - Use structured logging (`tracing`) and avoid leaking sensitive info in logs.

## Running Locally
Prerequisites: Rust (edition 2024 toolchain), PostgreSQL, Diesel CLI.

1) Set environment variables (create `.env` or export in shell).
2) Start PostgreSQL and run migrations.
3) Run the server:

```bash
cargo run
```

- The service starts an Axum server using Tokio.
- Use `RUST_LOG=debug` for more verbose traces during development.

## AWS Lambda: Build & Deploy
Two options: manual packaging or using `cargo-lambda`.

### Option A: Manual packaging (provided.al2)
1) Build a static binary for Amazon Linux 2:

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

2) Prepare deployment artifact:

```bash
mkdir -p lambda
cp target/x86_64-unknown-linux-musl/release/auth-manager lambda/bootstrap
cd lambda
zip -9r auth-manager.zip bootstrap
```

3) Create a Lambda function:
- Runtime: `provided.al2` (Custom runtime).
- Upload `auth-manager.zip` as the function code.
- Configure environment variables (see above) and IAM permissions.

### Option B: cargo-lambda (optional CLI)
Install and use [`cargo-lambda`](https://github.com/sigmaSd/cargo-lambda) for streamlined builds:

```bash
cargo install cargo-lambda
cargo lambda build --release
cargo lambda deploy --enable-function-url
```

Adjust deploy flags according to your AWS setup. This project uses `lambda_http`, which `cargo-lambda` supports.

## Development Guidelines
- Handlers:
  - Keep HTTP handlers thin; no business logic. Validate input, call services, map errors to responses.
- Services:
  - Centralize auth flows, permission checks, and JWT in `auth` modules.
- Repositories:
  - Encapsulate all DB access. Avoid leaking Diesel queries outside repositories.
- Errors:
  - Prefer explicit domain error types; use `anyhow` only at boundaries or for non-domain failures.
- Concurrency:
  - Avoid blocking operations in async contexts; use Tokio-aware libraries only.
- Logging & Tracing:
  - Use `tracing` macros (`info!`, `warn!`, `error!`, `instrument`) and configure `tracing-subscriber` per environment.
- HTTP Middleware:
  - Configure CORS and tracing via `tower-http`. Keep cross-cutting concerns at the router/middleware level.
- Style & Quality:
  - Format and lint:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

- Testing:
  - Unit-test services and repositories; avoid coupling tests to handlers.

## Notes
- Verify runtime selection (local vs Lambda) in [src/main.rs](src/main.rs) and `docker/` files.
- Keep secrets out of version control. Consider `.gitignore` for `target` and lock files as configured.
