FROM rust:1.75-slim as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY backend/Cargo.toml backend/Cargo.lock ./backend/
COPY program/Cargo.toml ./program/
COPY program/programs/collateral_vault/Cargo.toml ./program/programs/collateral_vault/

RUN mkdir -p backend/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    mkdir -p program/programs/collateral_vault/src && \
    echo "pub fn add(a: i32, b: i32) -> i32 { a + b }" > program/programs/collateral_vault/src/lib.rs && \
    cd backend && cargo build --release && \
    rm -rf src target/release/deps/vault_backend* target/release/vault-backend*

COPY backend/src ./backend/src
COPY program/programs/collateral_vault/src ./program/programs/collateral_vault/src
COPY backend/migrations ./backend/migrations

RUN cd backend && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/backend/target/release/vault-backend /usr/local/bin/vault-backend

COPY --from=builder /app/backend/migrations ./migrations

RUN useradd -m -u 1001 appuser && \
    chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["vault-backend"]
