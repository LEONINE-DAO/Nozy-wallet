# Frontend to Backend Connection Guide

This guide explains how to connect your frontend application to the NozyWallet API server.

## 🚀 Quick Start

### 1. Start the Backend API Server

First, make sure the API server is running:

```bash
cd api-server
cargo run
```

The server will start on `http://localhost:3000` by default.

**Verify it's running:**
```bash
curl http://localhost:3000/health
```

You should see:
```json
{
  "status": "ok",
  "service": "nozywallet-api",
  "version": "0.1.0"
}
```

### 2. Configure CORS (if needed)

**For Development:**
- CORS is automatically configured to allow `localhost` origins
- No additional configuration needed for local development

**For Production:**
Set environment variables:
```bash
export NOZY_PRODUCTION=true
export NOZY_CORS_ORIGINS=https://your-frontend-domain.com,https://www.your-frontend-domain.com
```

### 3. Connect from Frontend

## 📝 Frontend Integration Examples

### JavaScript/TypeScript (Fetch API)

```javascript
// API Configuration
const API_BASE_URL = 'http://localhost:3000';
const API_KEY = process.env.REACT_APP_API_KEY; // Optional, if enabled

// Helper function to make API requests
async function apiRequest(endpoint, options = {}) {
  const headers = {
    'Content-Type': 'application/json',
    ...options.headers,
  };

  // Add API key if provided
  if (API_KEY) {
    headers['X-API-Key'] = API_KEY;
  }

  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || `HTTP ${response.status}`);
  }

  return response.json();
}

// Example: Check if wallet exists
async function checkWalletExists() {
  try {
    const data = await apiRequest('/api/wallet/exists');
    console.log('Wallet exists:', data.exists);
    return data;
  } catch (error) {
    console.error('Error:', error.message);
  }
}

// Example: Get balance
async function getBalance() {
  try {
    const data = await apiRequest('/api/balance');
    console.log(`Balance: ${data.balance_zec} ZEC`);
    return data;
  } catch (error) {
    console.error('Error:', error.message);
  }
}

// Example: Create wallet
async function createWallet(password) {
  try {
    const mnemonic = await apiRequest('/api/wallet/create', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });
    console.log('Wallet created! Mnemonic:', mnemonic);
    return mnemonic;
  } catch (error) {
    console.error('Error:', error.message);
  }
}

// Example: Send transaction
async function sendTransaction(recipient, amount, memo, password) {
  try {
    const result = await apiRequest('/api/transaction/send', {
      method: 'POST',
      body: JSON.stringify({
        recipient,
        amount,
        memo,
        password,
      }),
    });
    console.log('Transaction sent:', result);
    return result;
  } catch (error) {
    console.error('Error:', error.message);
  }
}
```

### React Hook Example

```typescript
import { useState, useEffect } from 'react';

const API_BASE_URL = 'http://localhost:3000';

interface Balance {
  balance_zec: number;
  balance_zatoshis: number;
}

function useWalletBalance() {
  const [balance, setBalance] = useState<Balance | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchBalance() {
      try {
        setLoading(true);
        const response = await fetch(`${API_BASE_URL}/api/balance`);
        
        if (!response.ok) {
          const data = await response.json();
          throw new Error(data.error);
        }
        
        const data = await response.json();
        setBalance(data);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    }

    fetchBalance();
    // Optionally refresh every 30 seconds
    const interval = setInterval(fetchBalance, 30000);
    return () => clearInterval(interval);
  }, []);

  return { balance, loading, error };
}

// Usage in component
function WalletBalance() {
  const { balance, loading, error } = useWalletBalance();

  if (loading) return <div>Loading balance...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!balance) return <div>No balance data</div>;

  return (
    <div>
      <h2>Wallet Balance</h2>
      <p>{balance.balance_zec} ZEC</p>
    </div>
  );
}
```

### React API Client Class

```typescript
class NozyWalletAPI {
  private baseUrl: string;
  private apiKey?: string;

  constructor(baseUrl: string = 'http://localhost:3000', apiKey?: string) {
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

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || `HTTP ${response.status}`);
    }

    return response.json();
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

  async getBalance() {
    return this.request<{ balance_zec: number; balance_zatoshis: number }>('/api/balance');
  }

  async generateAddress(password?: string) {
    return this.request<{ address: string }>('/api/address/generate', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });
  }

  async syncWallet(startHeight?: number, endHeight?: number, password?: string) {
    return this.request('/api/sync', {
      method: 'POST',
      body: JSON.stringify({ start_height: startHeight, end_height: endHeight, password }),
    });
  }

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

  async getConfig() {
    return this.request('/api/config');
  }

  async setZebraUrl(url: string) {
    return this.request('/api/config/zebra-url', {
      method: 'POST',
      body: JSON.stringify({ url }),
    });
  }
}

// Usage
const api = new NozyWalletAPI('http://localhost:3000');
const balance = await api.getBalance();
```

### Vue.js Example

```javascript
// api.js
const API_BASE_URL = 'http://localhost:3000';

export const walletAPI = {
  async request(endpoint, options = {}) {
    const headers = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error);
    }

    return response.json();
  },

  async getBalance() {
    return this.request('/api/balance');
  },

  async createWallet(password) {
    return this.request('/api/wallet/create', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });
  },

  async sendTransaction(recipient, amount, memo, password) {
    return this.request('/api/transaction/send', {
      method: 'POST',
      body: JSON.stringify({ recipient, amount, memo, password }),
    });
  },
};
```

```vue
<!-- WalletComponent.vue -->
<template>
  <div>
    <h2>Wallet Balance</h2>
    <p v-if="loading">Loading...</p>
    <p v-else-if="error">Error: {{ error }}</p>
    <p v-else>{{ balance?.balance_zec }} ZEC</p>
  </div>
</template>

<script>
import { walletAPI } from './api';

export default {
  data() {
    return {
      balance: null,
      loading: false,
      error: null,
    };
  },
  async mounted() {
    await this.fetchBalance();
  },
  methods: {
    async fetchBalance() {
      this.loading = true;
      this.error = null;
      try {
        this.balance = await walletAPI.getBalance();
      } catch (error) {
        this.error = error.message;
      } finally {
        this.loading = false;
      }
    },
  },
};
</script>
```

## 🔧 Configuration

### Environment Variables

Create a `.env` file in your frontend project:

```env
REACT_APP_API_URL=http://localhost:3000
REACT_APP_API_KEY=your-api-key-here  # Optional
```

### Backend Environment Variables

For the API server, you can configure:

```bash
# Optional: Enable API key authentication
export NOZY_API_KEY=your-secure-api-key

# Optional: Rate limiting
export NOZY_RATE_LIMIT_REQUESTS=100
export NOZY_RATE_LIMIT_WINDOW=60

# Production settings
export NOZY_PRODUCTION=true
export NOZY_CORS_ORIGINS=https://yourdomain.com

# Port configuration
export NOZY_HTTP_PORT=3000
```

## 🛡️ Security Best Practices

1. **Never store API keys in client-side code** - Use environment variables
2. **Always validate input** - Validate on frontend before sending
3. **Handle errors gracefully** - Show user-friendly error messages
4. **Use HTTPS in production** - Never send sensitive data over HTTP
5. **Handle rate limiting** - Show appropriate messages when rate limited

## 🐛 Troubleshooting

### CORS Errors

**Problem:** Browser blocks requests due to CORS policy

**Solution:**
- Development: Should work automatically with localhost
- Production: Set `NOZY_CORS_ORIGINS` with your frontend URL

### Connection Refused

**Problem:** Can't connect to API server

**Solution:**
1. Verify server is running: `curl http://localhost:3000/health`
2. Check firewall settings
3. Verify port number matches (default: 3000)

### Authentication Errors

**Problem:** 401 Unauthorized errors

**Solution:**
1. Check if API key is required (check server logs)
2. Verify API key format in headers
3. Ensure API key matches server configuration

## 📚 Available Endpoints

See [FRONTEND_DEVELOPER_GUIDE.md](api-server/FRONTEND_DEVELOPER_GUIDE.md) for complete API documentation.

### Key Endpoints:

- `GET /health` - Health check
- `GET /api/wallet/exists` - Check wallet exists
- `POST /api/wallet/create` - Create wallet
- `POST /api/wallet/restore` - Restore wallet
- `POST /api/wallet/unlock` - Unlock wallet
- `GET /api/balance` - Get balance
- `POST /api/sync` - Sync wallet
- `POST /api/transaction/send` - Send transaction
- `GET /api/transaction/history` - Get transaction history
- `POST /api/address/generate` - Generate address

## 🚀 Next Steps

1. **Start the API server** (if not already running)
2. **Create your frontend** (React, Vue, Angular, or vanilla JS)
3. **Use the examples above** to connect to the API
4. **Test with the health endpoint** first
5. **Build your UI** using the API endpoints

## 🖥️ Desktop Client Integration

For desktop applications (Electron, Tauri, etc.), see:
- **[Desktop Client Integration Guide](DESKTOP_CLIENT_INTEGRATION.md)** - Complete guide for desktop apps
- Includes Electron, Tauri, and web-based desktop examples
- Auto-start API server configuration
- Desktop-specific troubleshooting

For detailed API documentation, see:
- [Frontend Developer Guide](api-server/FRONTEND_DEVELOPER_GUIDE.md)
- [API Server README](api-server/README.md)
- [Desktop Client Integration](DESKTOP_CLIENT_INTEGRATION.md)
