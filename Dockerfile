# Dockerfile for NozyWallet API Server
FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY api-server/Cargo.toml ./api-server/
COPY zeaking/Cargo.toml ./zeaking/

# Copy source code
COPY src ./src
COPY api-server/src ./api-server/src
COPY zeaking/src ./zeaking/src

# Build from workspace root (not api-server directory)
RUN cargo build --release --bin nozywallet-api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/api-server/target/release/nozywallet-api /app/nozywallet-api

RUN mkdir -p /app/wallet_data

EXPOSE 3000

CMD ["/app/nozywallet-api"]
