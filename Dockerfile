# Dockerfile for NozyWallet API Server
FROM rust:1.70 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY api-server/Cargo.toml ./api-server/
COPY zeaking/Cargo.toml ./zeaking/

COPY src ./src
COPY api-server/src ./api-server/src
COPY zeaking/src ./zeaking/src

WORKDIR /app/api-server
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/api-server/target/release/nozywallet-api /app/nozywallet-api

RUN mkdir -p /app/wallet_data

EXPOSE 3000

CMD ["/app/nozywallet-api"]
