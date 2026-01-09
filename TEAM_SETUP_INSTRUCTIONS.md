# Team Setup Instructions - Frontend/Desktop Client Integration

This document provides instructions for your team to connect the frontend/desktop client to the NozyWallet backend API server.

## 📋 Quick Overview

Your team has access to all the necessary documentation and code to connect the frontend to the backend:

1. **Backend API Server**: Already implemented in `api-server/` directory
2. **Integration Guides**: Complete documentation for connecting frontend/desktop clients
3. **Example Code**: Ready-to-use API client implementations

## 🚀 For Your Team

### Step 1: Clone the Repository

```bash
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd Nozy-wallet
```

### Step 2: Review Integration Guides

Your team should read these guides in order:

1. **[FRONTEND_CONNECTION_GUIDE.md](FRONTEND_CONNECTION_GUIDE.md)**
   - General frontend connection guide
   - Web application examples (React, Vue, Angular)
   - Basic API integration

2. **[DESKTOP_CLIENT_INTEGRATION.md](DESKTOP_CLIENT_INTEGRATION.md)**
   - Desktop application integration (Electron, Tauri)
   - Complete API client implementations
   - Auto-start server configuration

3. **[api-server/FRONTEND_DEVELOPER_GUIDE.md](api-server/FRONTEND_DEVELOPER_GUIDE.md)**
   - Complete API endpoint reference
   - Request/response formats
   - Error handling

### Step 3: Start the Backend API Server

**Option A: Using the provided script**
```bash
# Windows
.\start-api-for-desktop.ps1

# Linux/Mac
./start-api-for-desktop.sh
```

**Option B: Manual start**
```bash
cd api-server
cargo run
```

The server will start on `http://localhost:3000`

### Step 4: Test the Connection

```bash
# Test health endpoint
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

### Step 5: Integrate with Frontend/Desktop Client

Use the code examples from the integration guides:

- **Web Frontend**: See `FRONTEND_CONNECTION_GUIDE.md` for React/Vue/Angular examples
- **Desktop Client**: See `DESKTOP_CLIENT_INTEGRATION.md` for Electron/Tauri examples

## 📁 Key Files for Your Team

### Documentation Files
- `FRONTEND_CONNECTION_GUIDE.md` - Web frontend integration
- `DESKTOP_CLIENT_INTEGRATION.md` - Desktop client integration
- `api-server/FRONTEND_DEVELOPER_GUIDE.md` - Complete API reference
- `api-server/README.md` - API server documentation

### Configuration Files
- `desktop-client-config.example.json` - Example configuration for desktop clients

### Scripts
- `start-api-for-desktop.ps1` - Windows script to start API server
- `start-api-for-desktop.sh` - Linux/Mac script to start API server

### Backend Code
- `api-server/src/main.rs` - API server implementation
- `api-server/src/handlers.rs` - API endpoint handlers

## 🔧 Configuration

### Backend Configuration

The API server runs on `http://localhost:3000` by default. No additional configuration needed for development.

**Optional environment variables:**
```bash
# Optional: API key authentication
export NOZY_API_KEY=your-api-key

# Optional: Custom port
export NOZY_HTTP_PORT=3000
```

### Frontend Configuration

Set the API URL in your frontend/desktop client:

```env
# .env file
API_URL=http://localhost:3000
```

Or in code:
```typescript
const API_BASE_URL = 'http://localhost:3000';
```

## 📝 Example: Quick Integration

### 1. Copy API Client Code

From `DESKTOP_CLIENT_INTEGRATION.md` or `FRONTEND_CONNECTION_GUIDE.md`, copy the API client class:

```typescript
// src/api/client.ts
class NozyWalletAPI {
  private baseUrl: string = 'http://localhost:3000';
  
  async getBalance() {
    const response = await fetch(`${this.baseUrl}/api/balance`);
    return response.json();
  }
  
  // ... other methods
}
```

### 2. Use in Your Application

```typescript
import api from './api/client';

// Get balance
const balance = await api.getBalance();
console.log(`Balance: ${balance.balance_zec} ZEC`);
```

## ✅ Checklist for Your Team

- [ ] Repository cloned
- [ ] Integration guides reviewed
- [ ] Backend API server running (`http://localhost:3000`)
- [ ] Health check endpoint working
- [ ] API client code integrated into frontend/desktop client
- [ ] Test connection with a simple API call
- [ ] Error handling implemented
- [ ] Ready to build UI components

## 🐛 Troubleshooting

### API Server Won't Start

1. Check if Rust/Cargo is installed: `cargo --version`
2. Check if port 3000 is available
3. Review `api-server/README.md` for setup instructions

### Connection Errors

1. Verify API server is running: `curl http://localhost:3000/health`
2. Check API URL in frontend matches backend (`http://localhost:3000`)
3. Check CORS configuration (should work automatically for localhost)

### Need Help?

- Review the integration guides for detailed examples
- Check `api-server/FRONTEND_DEVELOPER_GUIDE.md` for API reference
- See troubleshooting sections in the integration guides

## 📚 Additional Resources

- **API Endpoints**: See `api-server/FRONTEND_DEVELOPER_GUIDE.md`
- **Security**: See `api-server/SECURITY_CONFIG.md`
- **Main README**: See `README.md` for project overview

## 🎯 Next Steps

1. **Start Backend**: Run the API server
2. **Review Guides**: Read the integration documentation
3. **Copy Examples**: Use the provided code examples
4. **Test Connection**: Verify API connectivity
5. **Build UI**: Create your frontend/desktop interface

Your team has everything they need to connect the frontend to the backend! 🚀
