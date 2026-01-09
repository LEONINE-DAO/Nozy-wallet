# Connecting NozyWallet Desktop Client to Backend

This guide will help you connect the [NozyWallet-DesktopClient](https://github.com/LEONINE-DAO/NozyWallet-DesktopClient) frontend to the Nozy-wallet backend.

## 🎯 Architecture Options

### Option 1: Tauri (Recommended) ⭐
**Direct Rust integration** - No HTTP server needed!
- See [TAURI_IMPLEMENTATION.md](TAURI_IMPLEMENTATION.md) for complete guide
- **Landing Page Integration**: See [LANDING_PAGE_TAURI_INTEGRATION.md](LANDING_PAGE_TAURI_INTEGRATION.md) for integrating Tauri into the NozyWallet-landing repository
- Direct function calls from JavaScript to Rust
- Smaller binary (~5-10MB vs 100MB+)
- Better security and performance

### Option 2: HTTP API Server
**Traditional REST API** - Useful for web clients or remote access
- Continue reading this guide
- Requires running API server separately
- Good for development and testing

---

## 📋 Prerequisites (HTTP API Option)

## 📋 Prerequisites

1. **Backend Repository**: Nozy-wallet (this repository)
2. **Frontend Repository**: [NozyWallet-DesktopClient](https://github.com/LEONINE-DAO/NozyWallet-DesktopClient)
3. **Rust** (for backend) - Install from [rustup.rs](https://rustup.rs/)
4. **Node.js** v18+ (for frontend) - Install from [nodejs.org](https://nodejs.org/)
5. **Zebra Node** (optional, but required for full functionality) - See [Zebra documentation](https://zebra.zcash.dev/)

---

## 🚀 Step 1: Build and Start the Backend API Server

### Option A: Quick Start (Development)

```bash
# Navigate to the backend repository
cd Nozy-wallet

# Build the API server
cd api-server
cargo build --release

# Run the API server (defaults to http://localhost:3000)
cargo run --bin nozywallet-api
```

The server will start on `http://localhost:3000` by default.

### Option B: Using Release Binary

```bash
# Build release binary
cd api-server
cargo build --release

# Run the binary
./target/release/nozywallet-api
# Or on Windows:
# target\release\nozywallet-api.exe
```

### Verify Backend is Running

Open a browser or use curl:

```bash
curl http://localhost:3000/health
```

Expected response:
```json
{
  "status": "ok",
  "service": "nozywallet-api",
  "version": "0.1.0"
}
```

---

## 🔧 Step 2: Configure Backend (Optional)

### Environment Variables

The backend supports these environment variables (all optional for development):

```bash
# Port configuration (default: 3000)
export NOZY_HTTP_PORT=3000

# API Key authentication (optional - disabled by default in dev)
export NOZY_API_KEY=your-secret-api-key

# Rate limiting (default: 100 requests per 60 seconds)
export NOZY_RATE_LIMIT_REQUESTS=100
export NOZY_RATE_LIMIT_WINDOW=60

# Production mode (affects CORS settings)
export NOZY_PRODUCTION=false

# CORS origins (only needed in production)
export NOZY_CORS_ORIGINS=http://localhost:5173,http://localhost:3001
```

**For development, you don't need to set any of these** - the defaults work fine.

---

## 🎨 Step 3: Configure Frontend

### Clone and Setup Frontend

```bash
# Clone the frontend repository
git clone https://github.com/LEONINE-DAO/NozyWallet-DesktopClient.git
cd NozyWallet-DesktopClient

# Install dependencies
npm install
```

### Create Environment File

Create a `.env` file in the frontend root directory:

```bash
# .env file
VITE_API_URL=http://localhost:3000
```

**Important**: 
- The frontend uses Vite, so environment variables must start with `VITE_`
- The default port is `3000` (matches backend default)
- Use `http://` (not `https://`) for local development

### Verify Frontend Configuration

Check that your frontend code uses the environment variable:

```typescript
// Should be something like this in your API client
const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';
```

---

## 🧪 Step 4: Test the Connection

### Start Both Services

**Terminal 1 - Backend:**
```bash
cd Nozy-wallet/api-server
cargo run --bin nozywallet-api
```

**Terminal 2 - Frontend:**
```bash
cd NozyWallet-DesktopClient
npm run dev
```

### Test from Frontend

1. Open the frontend in your browser (usually `http://localhost:5173` for Vite)
2. Check the browser console for any connection errors
3. Try making a request to the health endpoint

### Manual API Test

You can test the API directly:

```bash
# Health check
curl http://localhost:3000/health

# Check if wallet exists
curl http://localhost:3000/api/wallet/exists

# Get balance (requires wallet)
curl http://localhost:3000/api/balance
```

---

## 🔍 Step 5: Troubleshooting

### Issue: "Connection Refused" or "Network Error"

**Symptoms**: Frontend can't connect to backend

**Solutions**:
1. ✅ Verify backend is running: `curl http://localhost:3000/health`
2. ✅ Check backend logs for errors
3. ✅ Verify `VITE_API_URL` in frontend `.env` matches backend port
4. ✅ Check firewall settings
5. ✅ Try `http://127.0.0.1:3000` instead of `localhost:3000`

### Issue: CORS Errors

**Symptoms**: Browser console shows CORS policy errors

**Solutions**:
1. ✅ Backend allows `localhost` by default in development mode
2. ✅ Ensure `NOZY_PRODUCTION=false` (or not set) in backend
3. ✅ Check that frontend URL matches allowed origins
4. ✅ For production, set `NOZY_CORS_ORIGINS` with your frontend URL

### Issue: "Port Already in Use"

**Symptoms**: Backend fails to start with port binding error

**Solutions**:
```bash
# Change backend port
export NOZY_HTTP_PORT=3001
cargo run --bin nozywallet-api

# Then update frontend .env
VITE_API_URL=http://localhost:3001
```

### Issue: API Key Authentication Errors

**Symptoms**: 401 Unauthorized errors

**Solutions**:
1. ✅ If you set `NOZY_API_KEY`, include it in frontend requests:
   ```typescript
   headers: {
     'X-API-Key': 'your-api-key-here'
   }
   ```
2. ✅ Or remove `NOZY_API_KEY` for development (authentication is optional)

### Issue: Frontend Can't Find Backend

**Symptoms**: Frontend shows "API not found" or connection timeout

**Solutions**:
1. ✅ Verify backend is running on correct port
2. ✅ Check `VITE_API_URL` in frontend `.env` file
3. ✅ Restart frontend dev server after changing `.env`
4. ✅ Check browser network tab to see actual request URL

---

## 📝 Step 6: API Endpoints Reference

The backend provides these endpoints (all under `/api/`):

### Wallet Management
- `GET /api/wallet/exists` - Check if wallet exists
- `POST /api/wallet/create` - Create new wallet
- `POST /api/wallet/restore` - Restore from mnemonic
- `POST /api/wallet/unlock` - Unlock wallet
- `GET /api/wallet/status` - Get wallet status

### Addresses
- `POST /api/address/generate` - Generate new address

### Balance & Sync
- `GET /api/balance` - Get wallet balance
- `POST /api/sync` - Sync wallet with blockchain

### Transactions
- `POST /api/transaction/send` - Send transaction
- `GET /api/transaction/fee-estimate` - Estimate fee
- `GET /api/transaction/history` - Get transaction history
- `GET /api/transaction/:txid` - Get transaction details

### Configuration
- `GET /api/config` - Get configuration
- `POST /api/config/zebra-url` - Set Zebra node URL
- `POST /api/config/test-zebra` - Test Zebra connection

### Proving Parameters
- `GET /api/proving/status` - Check proving parameters status
- `POST /api/proving/download` - Download proving parameters

### Health Check
- `GET /health` - Server health check

**Full API documentation**: See `api-server/FRONTEND_DEVELOPER_GUIDE.md`

---

## 🎯 Quick Start Checklist

- [ ] Backend built and running on `http://localhost:3000`
- [ ] Backend health check returns `{"status": "ok"}`
- [ ] Frontend `.env` file created with `VITE_API_URL=http://localhost:3000`
- [ ] Frontend dependencies installed (`npm install`)
- [ ] Frontend dev server running
- [ ] Browser console shows no connection errors
- [ ] Can make API calls from frontend

---

## 🔐 Security Notes

### Development
- ✅ API key authentication is **disabled by default**
- ✅ CORS allows all localhost origins
- ✅ HTTP is fine for local development

### Production
- ⚠️ **Enable API key authentication**: Set `NOZY_API_KEY`
- ⚠️ **Use HTTPS**: Set `NOZY_HTTPS_ENABLED=true`
- ⚠️ **Configure CORS**: Set `NOZY_CORS_ORIGINS` with your frontend URL
- ⚠️ **Set production mode**: Set `NOZY_PRODUCTION=true`

---

## 📚 Additional Resources

- **Backend API Guide**: `api-server/FRONTEND_DEVELOPER_GUIDE.md`
- **Security Configuration**: `api-server/SECURITY_CONFIG.md`
- **Backend README**: `api-server/README.md`
- **Main Project README**: `README.md`

---

## 🆘 Still Having Issues?

1. **Check Backend Logs**: Look for errors in the terminal running the backend
2. **Check Browser Console**: Look for JavaScript errors or network errors
3. **Test API Directly**: Use `curl` or Postman to test backend endpoints
4. **Verify Ports**: Make sure ports aren't conflicting with other services
5. **Check Environment Variables**: Verify all `.env` files are correct

---

## 💡 Example Frontend API Client

Here's a simple example of how to connect from the frontend:

```typescript
// api.ts
const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export async function checkHealth() {
  const response = await fetch(`${API_URL}/health`);
  return response.json();
}

export async function checkWalletExists() {
  const response = await fetch(`${API_URL}/api/wallet/exists`);
  return response.json();
}

export async function getBalance() {
  const response = await fetch(`${API_URL}/api/balance`);
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to get balance');
  }
  return response.json();
}

export async function createWallet(password?: string) {
  const response = await fetch(`${API_URL}/api/wallet/create`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ password }),
  });
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to create wallet');
  }
  return response.json();
}
```

---

**Need more help?** Check the troubleshooting section above or review the API documentation in `api-server/FRONTEND_DEVELOPER_GUIDE.md`.

