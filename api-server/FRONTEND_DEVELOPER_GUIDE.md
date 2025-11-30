# Frontend Developer Quick Reference Guide

This guide provides everything frontend developers need to integrate with the NozyWallet API server.

## 🚀 Quick Start

### Base URL
- **Development**: `http://localhost:3000`
- **Production**: Configure based on deployment

### Authentication (Optional)

If API key authentication is enabled, include the API key in requests:

```javascript
// Option 1: X-API-Key header
headers: {
  'X-API-Key': 'your-api-key-here'
}

// Option 2: Authorization header
headers: {
  'Authorization': 'Bearer your-api-key-here'
}
```

### Content Type
All POST requests require:
```javascript
headers: {
  'Content-Type': 'application/json'
}
```

## 📋 API Endpoints Reference

### Health Check
```javascript
GET /health
// No authentication required
// Response: { status: "ok", service: "nozywallet-api", version: "0.1.0" }
```

### Wallet Operations

#### Check if Wallet Exists
```javascript
GET /api/wallet/exists
// Response: { exists: boolean, has_password: boolean }
```

#### Create Wallet
```javascript
POST /api/wallet/create
Body: { password?: string }
// Response: string (mnemonic phrase)
// Error: 400 if wallet already exists
```

#### Restore Wallet
```javascript
POST /api/wallet/restore
Body: { mnemonic: string, password: string }
// Response: { success: boolean }
// Error: 400 if invalid mnemonic
```

#### Unlock Wallet
```javascript
POST /api/wallet/unlock
Body: { password: string }
// Response: string (success message)
// Error: 401 if wrong password
```

#### Get Wallet Status
```javascript
GET /api/wallet/status
// Response: { exists: boolean, unlocked: boolean, has_password: boolean }
```

### Address Operations

#### Generate Address
```javascript
POST /api/address/generate
Body: { password?: string }
// Response: { address: string }
```

### Balance

#### Get Balance
```javascript
GET /api/balance
// Response: { balance_zec: number, balance_zatoshis: number }
```

### Sync

#### Sync Wallet
```javascript
POST /api/sync
Body: {
  start_height?: number,
  end_height?: number,
  zebra_url?: string,
  password?: string
}
// Response: {
//   success: boolean,
//   balance_zec: number,
//   notes_found: number,
//   message: string
// }
```

### Transactions

#### Send Transaction
```javascript
POST /api/transaction/send
Body: {
  recipient: string,      // u1... shielded address
  amount: number,         // Amount in ZEC
  memo?: string,          // Optional memo (max 512 chars)
  zebra_url?: string,     // Optional Zebra node URL
  password?: string       // Wallet password if required
}
// Response: {
//   success: boolean,
//   txid?: string,
//   message: string
// }
// Errors:
//   - Invalid address: success=false, message describes issue
//   - Invalid amount: success=false, message describes issue
//   - Insufficient funds: error response
```

#### Estimate Fee
```javascript
GET /api/transaction/fee-estimate
// Response: {
//   fee_zatoshis: number,
//   fee_zec: number,
//   estimated_at: string (ISO 8601)
// }
```

#### Get Transaction History
```javascript
GET /api/transaction/history
// Response: Array of transaction objects
```

#### Get Transaction by ID
```javascript
GET /api/transaction/:txid
// Response: Transaction object
```

#### Check Transaction Confirmations
```javascript
POST /api/transaction/check-confirmations
Body: { txid: string }
// Response: { confirmations: number, ... }
```

### Address Book

#### List Address Book
```javascript
GET /api/address-book
// Response: Array of address book entries
```

#### Add Address Book Entry
```javascript
POST /api/address-book
Body: { name: string, address: string }
// Response: { success: boolean }
```

#### Remove Address Book Entry
```javascript
DELETE /api/address-book/:name
// Response: { success: boolean }
```

#### Search Address Book
```javascript
GET /api/address-book/search?q=search_term
// Response: Array of matching entries
```

### Notes

#### Get Notes
```javascript
GET /api/notes
// Response: Array of note objects
```

### Configuration

#### Get Config
```javascript
GET /api/config
// Response: {
//   zebra_url: string,
//   network: string,
//   last_scan_height?: number,
//   theme: string
// }
```

#### Set Zebra URL
```javascript
POST /api/config/zebra-url
Body: { url: string }
// Response: { success: boolean }
// Error: 400 if invalid URL format
```

#### Set Theme
```javascript
POST /api/config/theme
Body: { theme: "dark" | "light" }
// Response: { success: boolean }
// Error: 400 if theme is not "dark" or "light"
```

#### Test Zebra Connection
```javascript
POST /api/config/test-zebra
Body: { zebra_url?: string }
// Response: string (connection status message)
```

### Proving Parameters

#### Check Proving Status
```javascript
GET /api/proving/status
// Response: {
//   spend_params: boolean,
//   output_params: boolean,
//   spend_vk: boolean,
//   output_vk: boolean,
//   can_prove: boolean
// }
```

#### Download Proving Parameters
```javascript
POST /api/proving/download
// Response: { success: boolean, message: string }
```

## 🔄 Error Handling

### Standard Error Response Format

All errors follow this format:
```typescript
{
  error: string,           // Human-readable error message
  code?: string            // Optional error code (e.g., "INVALID_MNEMONIC", "WALLET_EXISTS")
}
```

Example error responses:
```json
// Simple error
{
  "error": "Invalid mnemonic format. Must be 12, 15, 18, 21, or 24 words."
}

// Error with code
{
  "error": "A wallet already exists!",
  "code": "WALLET_EXISTS"
}
```

### HTTP Status Codes

- `200 OK` - Success
- `400 Bad Request` - Invalid input/request
- `401 Unauthorized` - Authentication failed or missing
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error

### Rate Limiting

When rate limited, the server returns:
- Status: `429 Too Many Requests`
- Headers:
  - `X-RateLimit-Limit`: Maximum requests allowed
  - `X-RateLimit-Remaining`: Remaining requests in window
- Response: `{ error: "Rate limit exceeded. Please try again later." }`

## 💡 JavaScript/TypeScript Examples

### Fetch API Example

```typescript
// Type definitions
interface WalletInfo {
  exists: boolean;
  has_password: boolean;
}

interface BalanceResponse {
  balance_zec: number;
  balance_zatoshis: number;
}

interface ErrorResponse {
  error: string;
}

// API Client Class
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
      const error: ErrorResponse = await response.json();
      throw new Error(error.error || `HTTP ${response.status}`);
    }

    return response.json();
  }

  // Wallet operations
  async checkWalletExists(): Promise<WalletInfo> {
    return this.request<WalletInfo>('/api/wallet/exists');
  }

  async createWallet(password?: string): Promise<string> {
    return this.request<string>('/api/wallet/create', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });
  }

  async getBalance(): Promise<BalanceResponse> {
    return this.request<BalanceResponse>('/api/balance');
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
}

// Usage
const api = new NozyWalletAPI('http://localhost:3000', 'your-api-key');
const balance = await api.getBalance();
console.log(`Balance: ${balance.balance_zec} ZEC`);
```

### React Hook Example

```typescript
import { useState, useEffect } from 'react';

function useWalletBalance(apiUrl: string) {
  const [balance, setBalance] = useState<BalanceResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchBalance() {
      try {
        setLoading(true);
        const response = await fetch(`${apiUrl}/api/balance`);
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
  }, [apiUrl]);

  return { balance, loading, error };
}
```

## 🛡️ Security Best Practices

1. **Never store API keys in client-side code** - Use environment variables or secure storage
2. **Always validate user input** - Even though the API validates, validate on frontend too
3. **Handle rate limiting** - Show user-friendly messages when rate limited
4. **Use HTTPS in production** - Never send sensitive data over HTTP
5. **Sanitize addresses** - Validate address format before sending

## 📝 Input Validation Rules

### Addresses
- Must start with `u1` (shielded addresses)
- Length: 78-100 characters
- Transparent addresses (`t1`) are not supported

### Amounts
- Must be greater than 0
- Maximum: 21,000,000 ZEC (total supply)
- Precision: Up to 8 decimal places (ZEC has 8 decimal places)

### Mnemonics
- Must be 12, 15, 18, 21, or 24 words
- Space-separated
- BIP39 standard

### Memos
- Maximum length: 512 characters
- Optional field

### URLs
- Must start with `http://` or `https://`
- Maximum length: 2048 characters

### Themes
- Must be exactly `"dark"` or `"light"`

## 🔍 Testing

### Test API Connection
```bash
curl http://localhost:3000/health
```

### Test with Authentication
```bash
curl -H "X-API-Key: your-key" http://localhost:3000/api/balance
```

## 📚 Additional Resources

- [API Security Configuration](SECURITY_CONFIG.md) - Detailed security setup
- [API Server README](README.md) - Server documentation
- [Main Project README](../README.md) - Project overview

## 🐛 Common Issues

### CORS Errors
- **Development**: Should work automatically with localhost
- **Production**: Ensure your frontend URL is in `NOZY_CORS_ORIGINS`

### Authentication Errors
- Check if API key is required (check server logs)
- Verify API key format (no spaces, correct header name)

### Rate Limiting
- Default: 100 requests per 60 seconds per IP
- Check `X-RateLimit-Remaining` header to monitor usage
- Implement exponential backoff for retries

### Network Errors
- Verify server is running on correct port
- Check firewall settings
- Verify Zebra node is accessible (for sync/transaction operations)

