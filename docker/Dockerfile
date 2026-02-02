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
