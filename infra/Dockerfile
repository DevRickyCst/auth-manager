# ============================================================================
# Stage: builder
# Build optimized release binary for Lambda
# ============================================================================
FROM rust:1.92 AS builder

WORKDIR /build

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
# Stage: runtime
# Minimal Lambda image (AWS provided runtime)
# ============================================================================
FROM public.ecr.aws/lambda/provided:al2023 AS runtime

# Install PostgreSQL client library (required by Diesel)
RUN dnf install -y libpq && \
    dnf clean all && \
    rm -rf /var/cache/dnf

# Copy the compiled binary (Lambda requires the entrypoint to be named bootstrap)
COPY --from=builder /build/target/release/auth-manager ${LAMBDA_RUNTIME_DIR}/bootstrap

CMD ["bootstrap"]
