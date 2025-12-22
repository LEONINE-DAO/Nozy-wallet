
FROM rust:latest AS builder

WORKDIR /app


# Copy workspace root files
COPY Cargo.toml Cargo.lock ./
COPY zeaking ./zeaking
COPY src ./src
COPY api-server ./api-server

# Build using manifest path to api-server (workspace will be auto-detected)
RUN cargo build --release --manifest-path ./api-server/Cargo.toml


FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/nozywallet-api /app/nozywallet-api

RUN mkdir -p /app/wallet_data

EXPOSE 3000

CMD ["/app/nozywallet-api"]
