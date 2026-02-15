# ============================================================================
# Stage: base
# Base image with common dependencies for all stages
# ============================================================================
FROM rust:1.92 AS base

WORKDIR /app

# Install system dependencies (PostgreSQL client and build tools)
RUN apt-get update && apt-get install -y \
    postgresql-client \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# ============================================================================
# Stage: dev-tools
# Development tools (cargo-watch, diesel_cli)
# Separate stage for caching - these tools rarely change
# ============================================================================
FROM base AS dev-tools

# Install development tools
RUN cargo install cargo-watch
RUN cargo install diesel_cli --no-default-features --features postgres

# ============================================================================
# Stage: development
# Development environment with hot-reload
# Used by docker-compose.yml for local development (make local)
# ============================================================================
FROM dev-tools AS development

# Copy dev tools from previous stage
COPY --from=dev-tools /usr/local/cargo/bin/cargo-watch /usr/local/cargo/bin/
COPY --from=dev-tools /usr/local/cargo/bin/diesel /usr/local/cargo/bin/

# Development container stays running with cargo-watch
CMD ["cargo", "watch", "-x", "run"]

# ============================================================================
# Stage: builder
# Build optimized release binary for production/Lambda
# ============================================================================
FROM rust:1.92 AS builder

WORKDIR /build

# Install dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for caching
COPY Cargo.toml Cargo.lock ./

# Copy auth-manager-api workspace (required dependency)
COPY auth-manager-api ./auth-manager-api

# Create dummy src to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual binary
RUN cargo build --release

# ============================================================================
# Stage: runtime (for Lambda)
# Minimal runtime image with AWS Lambda Runtime Interface Client
# ============================================================================
FROM public.ecr.aws/lambda/provided:al2023 AS runtime

# Install PostgreSQL client library (required by Diesel)
RUN dnf install -y libpq && \
    dnf clean all && \
    rm -rf /var/cache/dnf

# Copy the compiled binary and rename it to bootstrap (required by Lambda)
COPY --from=builder /build/target/release/auth-manager ${LAMBDA_RUNTIME_DIR}/bootstrap

# Set the CMD to the handler (could also be done using the Docker CMD parameter)
CMD ["bootstrap"]
