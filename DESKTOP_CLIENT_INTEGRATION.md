# Desktop Client Integration Guide

This guide explains how to connect the NozyWallet Desktop Client to the backend API server.

## 🚀 Quick Start

### 1. Start the Backend API Server

First, ensure the API server is running:

```bash
cd api-server
cargo run
```

The server will start on `http://localhost:3000` by default.

**Verify it's running:**
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

### 2. Configure Desktop Client

The desktop client needs to be configured to connect to `http://localhost:3000`.

## 📝 Integration Examples

### Electron Application

#### 1. Create API Client (`src/api/client.ts`)

```typescript
// src/api/client.ts
const API_BASE_URL = process.env.API_URL || 'http://localhost:3000';
const API_KEY = process.env.API_KEY; // Optional

export interface ApiResponse<T> {
  data?: T;
  error?: string;
}

class NozyWalletAPI {
  private baseUrl: string;
  private apiKey?: string;

  constructor(baseUrl: string = API_BASE_URL, apiKey?: string) {
    this.baseUrl = baseUrl;
    this.apiKey = apiKey;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (this.apiKey) {
      headers['X-API-Key'] = this.apiKey;
    }

    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        ...options,
        headers,
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: `HTTP ${response.status}` }));
        throw new Error(error.error || `HTTP ${response.status}`);
      }

      return response.json();
    } catch (error) {
      if (error instanceof TypeError && error.message.includes('fetch')) {
        throw new Error('Cannot connect to API server. Make sure it is running on http://localhost:3000');
      }
      throw error;
    }
  }

  // Health check
  async healthCheck() {
    return this.request<{ status: string; service: string; version: string }>('/health');
  }

  // Wallet operations
  async checkWalletExists() {
    return this.request<{ exists: boolean; has_password: boolean }>('/api/wallet/exists');
  }

  async createWallet(password?: string): Promise<string> {
    return this.request<string>('/api/wallet/create', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });
  }

  async restoreWallet(mnemonic: string, password: string) {
    return this.request('/api/wallet/restore', {
      method: 'POST',
      body: JSON.stringify({ mnemonic, password }),
    });
  }

  async unlockWallet(password: string) {
    return this.request<string>('/api/wallet/unlock', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });
  }

  async getWalletStatus() {
    return this.request<{ exists: boolean; unlocked: boolean; has_password: boolean }>('/api/wallet/status');
  }

  // Balance
  async getBalance() {
    return this.request<{ balance_zec: number; balance_zatoshis: number }>('/api/balance');
  }

  // Address operations
  async generateAddress(password?: string) {
    return this.request<{ address: string }>('/api/address/generate', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });
  }

  // Sync
  async syncWallet(startHeight?: number, endHeight?: number, password?: string) {
    return this.request('/api/sync', {
      method: 'POST',
      body: JSON.stringify({
        start_height: startHeight,
        end_height: endHeight,
        password,
      }),
    });
  }

  // Transactions
  async sendTransaction(
    recipient: string,
    amount: number,
    memo?: string,
    password?: string
  ) {
    return this.request('/api/transaction/send', {
      method: 'POST',
      body: JSON.stringify({
        recipient,
        amount,
        memo,
        password,
      }),
    });
  }

  async getTransactionHistory() {
    return this.request('/api/transaction/history');
  }

  async getTransaction(txid: string) {
    return this.request(`/api/transaction/${txid}`);
  }

  async estimateFee() {
    return this.request<{ fee_zatoshis: number; fee_zec: number; estimated_at: string }>('/api/transaction/fee-estimate');
  }

  // Address book
  async getAddressBook() {
    return this.request('/api/address-book');
  }

  async addAddressBookEntry(name: string, address: string) {
    return this.request('/api/address-book', {
      method: 'POST',
      body: JSON.stringify({ name, address }),
    });
  }

  async removeAddressBookEntry(name: string) {
    return this.request(`/api/address-book/${encodeURIComponent(name)}`, {
      method: 'DELETE',
    });
  }

  async searchAddressBook(query: string) {
    return this.request(`/api/address-book/search?q=${encodeURIComponent(query)}`);
  }

  // Config
  async getConfig() {
    return this.request('/api/config');
  }

  async setZebraUrl(url: string) {
    return this.request('/api/config/zebra-url', {
      method: 'POST',
      body: JSON.stringify({ url }),
    });
  }

  async setTheme(theme: 'dark' | 'light') {
    return this.request('/api/config/theme', {
      method: 'POST',
      body: JSON.stringify({ theme }),
    });
  }

  async testZebraConnection(zebraUrl?: string) {
    return this.request<string>('/api/config/test-zebra', {
      method: 'POST',
      body: JSON.stringify({ zebra_url: zebraUrl }),
    });
  }

  // Proving parameters
  async getProvingStatus() {
    return this.request<{
      spend_params: boolean;
      output_params: boolean;
      spend_vk: boolean;
      output_vk: boolean;
      can_prove: boolean;
    }>('/api/proving/status');
  }

  async downloadProvingParameters() {
    return this.request('/api/proving/download');
  }
}

// Export singleton instance
export const api = new NozyWalletAPI(API_BASE_URL, API_KEY);
export default api;
```

#### 2. React Hook for API (`src/hooks/useWallet.ts`)

```typescript
// src/hooks/useWallet.ts
import { useState, useEffect, useCallback } from 'react';
import api from '../api/client';

export function useWallet() {
  const [walletStatus, setWalletStatus] = useState<{
    exists: boolean;
    unlocked: boolean;
    has_password: boolean;
  } | null>(null);
  const [balance, setBalance] = useState<{
    balance_zec: number;
    balance_zatoshis: number;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkWalletStatus = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const status = await api.getWalletStatus();
      setWalletStatus(status);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchBalance = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const bal = await api.getBalance();
      setBalance(bal);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, []);

  const unlockWallet = useCallback(async (password: string) => {
    try {
      setLoading(true);
      setError(null);
      await api.unlockWallet(password);
      await checkWalletStatus();
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      return false;
    } finally {
      setLoading(false);
    }
  }, [checkWalletStatus]);

  const createWallet = useCallback(async (password?: string) => {
    try {
      setLoading(true);
      setError(null);
      const mnemonic = await api.createWallet(password);
      await checkWalletStatus();
      return mnemonic;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [checkWalletStatus]);

  useEffect(() => {
    checkWalletStatus();
  }, [checkWalletStatus]);

  return {
    walletStatus,
    balance,
    loading,
    error,
    checkWalletStatus,
    fetchBalance,
    unlockWallet,
    createWallet,
  };
}
```

#### 3. Environment Configuration (`.env`)

```env
# API Configuration
API_URL=http://localhost:3000
API_KEY=

# Development
NODE_ENV=development
```

#### 4. Main Process Configuration (Electron)

```typescript
// main.ts or main.js
import { app, BrowserWindow } from 'electron';
import path from 'path';

// Ensure API server is accessible
// You might want to check if it's running or start it automatically

function createWindow() {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'),
    },
  });

  win.loadFile('index.html');
}

app.whenReady().then(createWindow);
```

### Tauri Application

#### 1. API Client (`src/api/client.ts`)

```typescript
// src/api/client.ts
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

class NozyWalletAPI {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: `HTTP ${response.status}` }));
      throw new Error(error.error || `HTTP ${response.status}`);
    }

    return response.json();
  }

  // Same methods as Electron example above
  async healthCheck() {
    return this.request<{ status: string; service: string; version: string }>('/health');
  }

  async getBalance() {
    return this.request<{ balance_zec: number; balance_zatoshis: number }>('/api/balance');
  }

  // ... (add all other methods from Electron example)
}

export const api = new NozyWalletAPI();
export default api;
```

#### 2. Tauri Config (`src-tauri/tauri.conf.json`)

```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "package": {
    "productName": "NozyWallet",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "http": {
        "all": false,
        "request": true,
        "scope": ["http://localhost:3000/**"]
      }
    }
  }
}
```

### React/Vue/Angular (Web-based Desktop)

#### React Example Component

```typescript
// src/components/WalletDashboard.tsx
import React, { useEffect, useState } from 'react';
import api from '../api/client';

export function WalletDashboard() {
  const [balance, setBalance] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadBalance() {
      try {
        const data = await api.getBalance();
        setBalance(data.balance_zec);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load balance');
      } finally {
        setLoading(false);
      }
    }

    loadBalance();
    const interval = setInterval(loadBalance, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  }, []);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div>
      <h2>Wallet Balance</h2>
      <p>{balance} ZEC</p>
    </div>
  );
}
```

## 🔧 Configuration

### CORS Configuration for Desktop Apps

Desktop applications typically don't have CORS restrictions, but if you encounter issues:

1. **Development**: The API server automatically allows localhost origins
2. **Production**: Ensure your desktop app's origin is allowed

The API server CORS configuration in `api-server/src/main.rs` allows:
- `http://localhost:*` (any port)
- `http://127.0.0.1:*` (any port)
- Custom origins via `NOZY_CORS_ORIGINS` environment variable

### Environment Variables

#### Backend (API Server)

```bash
# Optional: API key authentication
export NOZY_API_KEY=your-secure-api-key

# Optional: Rate limiting
export NOZY_RATE_LIMIT_REQUESTS=100
export NOZY_RATE_LIMIT_WINDOW=60

# Port configuration
export NOZY_HTTP_PORT=3000

# Production mode (stricter CORS)
export NOZY_PRODUCTION=false  # Set to true for production
```

#### Frontend (Desktop Client)

Create a `.env` file in your desktop client project:

```env
# API Configuration
API_URL=http://localhost:3000
API_KEY=  # Optional, if backend requires it

# Development
NODE_ENV=development
```

## 🚀 Auto-Start API Server (Optional)

You can configure the desktop client to automatically start the API server:

### Electron Example

```typescript
// main.ts
import { spawn } from 'child_process';
import path from 'path';

function startAPIServer() {
  const apiServerPath = path.join(__dirname, '../../api-server');
  const apiServer = spawn('cargo', ['run'], {
    cwd: apiServerPath,
    shell: true,
  });

  apiServer.stdout.on('data', (data) => {
    console.log(`API Server: ${data}`);
  });

  apiServer.stderr.on('data', (data) => {
    console.error(`API Server Error: ${data}`);
  });

  return apiServer;
}

// Start API server when app launches
const apiServerProcess = startAPIServer();

// Cleanup on app quit
app.on('before-quit', () => {
  apiServerProcess.kill();
});
```

### Tauri Example

```rust
// src-tauri/src/main.rs
use std::process::Command;
use tauri::Manager;

fn start_api_server() -> std::process::Child {
    Command::new("cargo")
        .args(&["run"])
        .current_dir("../api-server")
        .spawn()
        .expect("Failed to start API server")
}

#[tauri::command]
fn check_api_health() -> Result<bool, String> {
    // Check if API server is responding
    // Implementation here
    Ok(true)
}

fn main() {
    // Start API server
    let _api_server = start_api_server();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![check_api_health])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 🛡️ Security Considerations

1. **API Key Storage**: Never hardcode API keys in client code
2. **Localhost Only**: For desktop apps, always use `localhost` or `127.0.0.1`
3. **HTTPS in Production**: Consider using HTTPS even for localhost in production
4. **Input Validation**: Always validate user input before sending to API
5. **Error Handling**: Don't expose sensitive error messages to users

## 🐛 Troubleshooting

### Connection Refused

**Problem**: Desktop client can't connect to API server

**Solutions**:
1. Verify API server is running: `curl http://localhost:3000/health`
2. Check firewall settings
3. Verify port 3000 is not in use by another application
4. Check API server logs for errors

### CORS Errors

**Problem**: Browser-based desktop app shows CORS errors

**Solutions**:
1. Development: Should work automatically with localhost
2. Check API server CORS configuration
3. Verify request origin matches allowed origins
4. For Electron, ensure `webSecurity: false` is NOT set (security risk)

### API Key Authentication Errors

**Problem**: 401 Unauthorized errors

**Solutions**:
1. Check if API key is required (check server logs)
2. Verify API key is correctly set in environment variables
3. Ensure API key is included in request headers
4. Check API key format (no spaces, correct header name)

### Network Timeout

**Problem**: Requests timeout

**Solutions**:
1. Check API server is responding: `curl http://localhost:3000/health`
2. Increase timeout in API client configuration
3. Check network connectivity
4. Verify no proxy is interfering

## 📚 Complete API Reference

See [api-server/FRONTEND_DEVELOPER_GUIDE.md](api-server/FRONTEND_DEVELOPER_GUIDE.md) for complete API documentation.

### Key Endpoints:

- `GET /health` - Health check
- `GET /api/wallet/exists` - Check if wallet exists
- `GET /api/wallet/status` - Get wallet status
- `POST /api/wallet/create` - Create new wallet
- `POST /api/wallet/restore` - Restore wallet from mnemonic
- `POST /api/wallet/unlock` - Unlock wallet
- `GET /api/balance` - Get wallet balance
- `POST /api/sync` - Sync wallet with blockchain
- `POST /api/transaction/send` - Send transaction
- `GET /api/transaction/history` - Get transaction history
- `POST /api/address/generate` - Generate new address
- `GET /api/address-book` - List address book
- `GET /api/config` - Get configuration
- `POST /api/config/zebra-url` - Set Zebra URL

## 🚀 Next Steps

1. **Clone/Setup Desktop Client**: Get the desktop client codebase
2. **Install Dependencies**: Install required packages
3. **Configure API URL**: Set `API_URL=http://localhost:3000` in environment
4. **Start API Server**: Run `cd api-server && cargo run`
5. **Test Connection**: Use health check endpoint first
6. **Integrate API Client**: Use the examples above to integrate
7. **Build UI**: Create UI components using the API client

## 📝 Example Integration Checklist

- [ ] API server running on `http://localhost:3000`
- [ ] Health check endpoint responding
- [ ] API client configured with correct base URL
- [ ] Error handling implemented
- [ ] Loading states handled
- [ ] Wallet status check working
- [ ] Balance fetching working
- [ ] Transaction sending tested
- [ ] Address generation working
- [ ] Sync functionality tested

For more details, see:
- [Frontend Connection Guide](FRONTEND_CONNECTION_GUIDE.md)
- [Frontend Developer Guide](api-server/FRONTEND_DEVELOPER_GUIDE.md)
- [API Server README](api-server/README.md)
