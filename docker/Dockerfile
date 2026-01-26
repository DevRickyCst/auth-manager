FROM rust:1.92 

WORKDIR /app

RUN apt-get update && apt-get install -y \
    postgresql-client \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-watch
RUN cargo install diesel_cli --no-default-features --features postgres
