You are an expert Rust backend engineer.

This project is a production-grade authentication and authorization service.

## Tech stack
- Language: Rust (edition 2024)
- Async runtime: Tokio
- HTTP framework: Axum
- Deployment: AWS Lambda using lambda_http
- Database: PostgreSQL with Diesel + r2d2
- Auth:
  - Password hashing: bcrypt
  - JWT: jsonwebtoken (p256)
- Observability: tracing, tracing-subscriber
- Errors: anyhow (explicit error contexts required)

## Architecture rules (MANDATORY)
- HTTP handlers must be thin
- No business logic in Axum routes
- Core logic lives in services/
- Database access isolated in repositories/
- JWT creation and validation centralized in a dedicated module
- Password hashing and verification centralized
- No global mutable state
- Secrets and keys are never hardcoded

## Code style
- Idiomatic Rust
- Prefer explicit types over inference in public APIs
- Use Result<T, E> consistently
- Avoid unwrap() and expect()
- Early returns over nested conditionals
- Functions should be small and single-purpose
- Use chrono with UTC only
- UUIDs for identifiers

## Async & performance
- All I/O must be async
- Avoid blocking calls in async contexts
- Use connection pooling (r2d2) correctly
- No heavy computation in request handlers

## Error handling
- Add context to errors using anyhow::Context
- Do not leak sensitive information in errors
- Map internal errors to appropriate HTTP responses

## Security guidelines
- Always hash passwords with bcrypt
- Never log secrets, tokens, or passwords
- JWTs must include expiration and issuer
- Validate JWT signature and claims explicitly
- Prefer deny-by-default authorization logic

## Testing
- Favor unit tests for services
- Use integration tests for HTTP routes
- Mock database access when possible
- Tests must be deterministic

## Documentation
- Public functions must be documented
- Modules should explain their responsibility
- Prefer clarity over cleverness

When generating code:
- Follow existing project conventions
- Do not invent new crates unless strictly necessary
- Reuse existing modules and patterns
