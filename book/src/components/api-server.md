# API Server

REST API server for NozyWallet mobile app. This server wraps the Rust wallet backend and exposes it via HTTP endpoints.

## Quick Start

### Build and Run

```bash
cd api-server
cargo build --release
cargo run
```

The server will start on `http://0.0.0.0:3000`

### Development

```bash
# Run in development mode with hot reload
cargo run

# Or use cargo watch for auto-reload
cargo install cargo-watch
cargo watch -x run
```

## API Endpoints

### Health Check

```bash
GET /health
```

### Wallet Operations

- `POST /wallet/new` - Create a new wallet
- `POST /wallet/restore` - Restore wallet from mnemonic
- `GET /wallet/info` - Get wallet information

### Address Operations

- `GET /addresses` - Generate addresses
- `GET /addresses/:count` - Generate multiple addresses

### Transaction Operations

- `POST /scan` - Scan blockchain for notes
- `GET /notes` - List stored notes
- `POST /send` - Send ZEC transaction

## Security

The API server includes security middleware for:
- CORS configuration
- Request validation
- Rate limiting
- Authentication (if configured)

See the [API Server README](../../../api-server/README.md) for complete documentation.
