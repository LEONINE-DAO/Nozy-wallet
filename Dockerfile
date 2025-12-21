# Dockerfile for NozyWallet API Server (Root level for DigitalOcean)
# This Dockerfile is at the root for DigitalOcean App Platform detection
FROM rust:1.70 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY api-server/Cargo.toml ./api-server/
COPY zeaking/Cargo.toml ./zeaking/

# Copy source code
COPY src ./src
COPY api-server/src ./api-server/src
COPY zeaking/src ./zeaking/src

# Build the API server
WORKDIR /app/api-server
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/api-server/target/release/nozywallet-api /app/nozywallet-api

# Create wallet data directory
RUN mkdir -p /app/wallet_data

# Expose port
EXPOSE 3000

# Run the server
CMD ["/app/nozywallet-api"]
