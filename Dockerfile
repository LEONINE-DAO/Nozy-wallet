
FROM rust:latest AS builder

WORKDIR /app


COPY Cargo.toml Cargo.lock ./
COPY zeaking ./zeaking
COPY src ./src
COPY api-server ./api-server

WORKDIR /app/api-server
RUN cargo build --release
WORKDIR /app


FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/nozywallet-api /app/nozywallet-api

RUN mkdir -p /app/wallet_data

EXPOSE 3000

CMD ["/app/nozywallet-api"]
