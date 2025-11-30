# NozyWallet REST API Server

REST API server for NozyWallet mobile app. This server wraps the Rust wallet backend and exposes it via HTTP endpoints.

## 🚀 Quick Start

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

## 📋 API Endpoints

### Health Check
- `GET /health` - Server health check

### Wallet Endpoints
- `GET /api/wallet/exists` - Check if wallet exists
- `POST /api/wallet/create` - Create new wallet
- `POST /api/wallet/restore` - Restore wallet from mnemonic
- `POST /api/wallet/unlock` - Unlock wallet with password

### Address Endpoints
- `POST /api/address/generate` - Generate new address

### Balance Endpoints
- `GET /api/balance` - Get wallet balance

### Sync Endpoints
- `POST /api/sync` - Sync wallet with blockchain

### Transaction Endpoints
- `POST /api/transaction/send` - Send transaction

### Config Endpoints
- `GET /api/config` - Get configuration
- `POST /api/config/zebra-url` - Set Zebra URL
- `POST /api/config/theme` - Set theme
- `POST /api/config/test-zebra` - Test Zebra connection

### Proving Endpoints
- `GET /api/proving/status` - Check proving parameters status
- `POST /api/proving/download` - Download proving parameters

## 📝 Example Requests

### Create Wallet
```bash
curl -X POST http://localhost:3000/api/wallet/create \
  -H "Content-Type: application/json" \
  -d '{"password": "optional_password"}'
```

### Get Balance
```bash
curl http://localhost:3000/api/balance
```

### Send Transaction
```bash
curl -X POST http://localhost:3000/api/transaction/send \
  -H "Content-Type: application/json" \
  -d '{
    "recipient": "u1...",
    "amount": 0.1,
    "memo": "Optional memo",
    "password": "wallet_password"
  }'
```

## 🔧 Configuration

The API server uses the same wallet data directory as the CLI:
- Wallet data: `wallet_data/`
- Config: `wallet_data/config.json`

## 🔒 Security Features

The API server includes comprehensive security features:

### ✅ Implemented Security Features

1. **API Key Authentication** (Optional)
   - Set `NOZY_API_KEY` environment variable to enable
   - Supports `X-API-Key` header or `Authorization: Bearer <key>` format
   - Health check endpoint excluded from authentication

2. **Rate Limiting**
   - Configurable via `NOZY_RATE_LIMIT_REQUESTS` (default: 100)
   - Time window via `NOZY_RATE_LIMIT_WINDOW` (default: 60 seconds)
   - Returns `429 Too Many Requests` when exceeded
   - Includes rate limit headers in responses

3. **Input Validation**
   - All endpoints validate input data
   - Address validation (shielded addresses only)
   - Amount validation (0 < amount <= 21M ZEC)
   - Mnemonic validation (12/15/18/21/24 words)
   - URL and theme validation

4. **Security Headers**
   - X-Content-Type-Options: nosniff
   - X-Frame-Options: DENY
   - X-XSS-Protection: 1; mode=block
   - Content-Security-Policy
   - Permissions-Policy
   - Referrer-Policy

5. **CORS Configuration**
   - Development: Allows localhost origins dynamically
   - Production: Restricted to specific origins via `NOZY_CORS_ORIGINS`
   - Environment-based configuration

6. **Request Logging**
   - All requests logged with method, path, IP, status, and duration
   - Structured logging for monitoring

### Configuration

See [SECURITY_CONFIG.md](SECURITY_CONFIG.md) for detailed security configuration options.

### Production Deployment

⚠️ **For production use**:
1. Set `NOZY_API_KEY` with a strong, random key
2. Set `NOZY_PRODUCTION=true`
3. Configure `NOZY_CORS_ORIGINS` with your frontend URLs
4. Use HTTPS (configure reverse proxy like Nginx)
5. Review and adjust rate limits based on traffic

## 🐛 Troubleshooting

### Port Already in Use
```bash
# Find process using port 3000
lsof -i :3000  # macOS/Linux
netstat -ano | findstr :3000  # Windows

# Kill the process or change port in main.rs
```

### Wallet Not Found
Make sure you're running from the project root directory where `wallet_data/` exists.

### CORS Issues
- Development: CORS allows localhost origins automatically
- Production: Set `NOZY_CORS_ORIGINS` environment variable with comma-separated allowed origins
- See [SECURITY_CONFIG.md](SECURITY_CONFIG.md) for details

## 📚 Dependencies

- **axum**: HTTP web framework
- **tokio**: Async runtime
- **tower-http**: Middleware (CORS, tracing)
- **serde**: Serialization
- **nozy**: Local wallet library (from parent directory)

## 🚀 Production Deployment

1. Build release binary:
   ```bash
   cargo build --release
   ```

2. Run as service (systemd example):
   ```ini
   [Unit]
   Description=NozyWallet API Server
   After=network.target

   [Service]
   Type=simple
   User=nozywallet
   WorkingDirectory=/opt/nozywallet
   ExecStart=/opt/nozywallet/api-server/target/release/nozywallet-api
   Restart=always

   [Install]
   WantedBy=multi-user.target
   ```

3. Use reverse proxy (nginx) for HTTPS and load balancing

