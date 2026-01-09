# User-Ready Deployment Guide

This guide explains how to set up NozyWallet so users can **download and use it immediately** without running any servers locally. The API server will be hosted remotely, and users just need to download the desktop client.

## 🎯 Goal

Users should be able to:
1. Download the desktop client
2. Install and run it
3. Use it immediately - no server setup required
4. Connect to Zcash blockchain automatically

## 🏗️ Architecture

```
┌─────────────────┐
│  Desktop Client │  (User downloads this)
│  (Electron/Tauri│
│   or Web App)   │
└────────┬────────┘
         │ HTTPS
         │ API Calls
         ▼
┌─────────────────┐
│  Public API      │  (You host this)
│  Server          │
│  api.yourdomain  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Zcash Blockchain│
│  (Zebra Node)    │
└─────────────────┘
```

## 📋 Step-by-Step Setup

### Step 1: Deploy Public API Server

Deploy the API server to a public URL that users can access.

#### Option A: Cloud Platform (Easiest)

**DigitalOcean App Platform:**
1. Connect GitHub repository
2. Select `api-server/Dockerfile`
3. Set environment variables:
   ```
   NOZY_PRODUCTION=true
   NOZY_API_KEY=your-secure-key
   NOZY_CORS_ORIGINS=https://your-frontend.com,https://app.yourdomain.com
   ```
4. Deploy - get URL like: `https://api-nozywallet-xxxxx.ondigitalocean.app`

**AWS App Runner / Google Cloud Run:**
- Similar process, deploy Docker container
- Get public HTTPS URL automatically

#### Option B: VPS with Domain

1. Deploy to VPS (DigitalOcean, AWS EC2, etc.)
2. Set up domain: `api.yourdomain.com`
3. Configure Nginx + SSL (see DEPLOYMENT_GUIDE.md)
4. Set environment variables

### Step 2: Configure API Server for Public Use

The API server needs these settings:

```bash
# Production mode
NOZY_PRODUCTION=true

# API key (optional but recommended)
NOZY_API_KEY=your-secure-api-key

# CORS - allow your desktop client origins
NOZY_CORS_ORIGINS=https://app.yourdomain.com,https://yourdomain.com

# Public Zebra node (or your own)
# Users will connect through your API server
```

**Important:** The API server will handle all blockchain connections. Users don't need their own Zebra node.

### Step 3: Update Desktop Client Configuration

Configure the desktop client to use the remote API by default.

#### For Electron/Tauri Desktop Client

Create `src/config/api.ts`:

```typescript
// API Configuration
export const API_CONFIG = {
  // Production API URL (your deployed server)
  PRODUCTION_URL: 'https://api.yourdomain.com',
  
  // Development URL (for testing)
  DEVELOPMENT_URL: 'http://localhost:3000',
  
  // Use production by default
  get baseUrl(): string {
    if (process.env.NODE_ENV === 'production') {
      return this.PRODUCTION_URL;
    }
    return this.DEVELOPMENT_URL;
  },
  
  // API key (if required)
  apiKey: process.env.API_KEY || '',
};

// API Client
class NozyWalletAPI {
  private baseUrl: string;
  private apiKey?: string;

  constructor() {
    this.baseUrl = API_CONFIG.baseUrl;
    this.apiKey = API_CONFIG.apiKey;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
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
      const error = await response.json().catch(() => ({ error: `HTTP ${response.status}` }));
      throw new Error(error.error || `HTTP ${response.status}`);
    }

    return response.json();
  }

  // All API methods...
  async getBalance() {
    return this.request<{ balance_zec: number; balance_zatoshis: number }>('/api/balance');
  }
  
  // ... other methods
}

export const api = new NozyWalletAPI();
```

#### Environment Configuration

Create `.env.production`:

```env
REACT_APP_API_URL=https://api.yourdomain.com
REACT_APP_API_KEY=
```

### Step 4: Build Desktop Client with Embedded Config

#### Electron Build

```json
// package.json
{
  "build": {
    "appId": "com.nozywallet.app",
    "productName": "NozyWallet",
    "directories": {
      "output": "dist"
    },
    "files": [
      "build/**/*",
      "node_modules/**/*"
    ],
    "extraMetadata": {
      "main": "build/electron/main.js"
    },
    "win": {
      "target": "nsis",
      "icon": "assets/icon.ico"
    },
    "mac": {
      "target": "dmg",
      "icon": "assets/icon.icns"
    },
    "linux": {
      "target": "AppImage",
      "icon": "assets/icon.png"
    }
  }
}
```

Build command:
```bash
npm run build
npm run electron:build
```

#### Tauri Build

```toml
# src-tauri/tauri.conf.json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "../dist",
    "distDir": "../dist"
  },
  "package": {
    "productName": "NozyWallet",
    "version": "1.0.0"
  },
  "tauri": {
    "allowlist": {
      "http": {
        "all": true,
        "request": true,
        "scope": ["https://api.yourdomain.com/**"]
      }
    }
  }
}
```

Build:
```bash
npm run tauri build
```

### Step 5: Handle Wallet Data Storage

Users need to store their wallet data locally. Configure the desktop client to:

1. **Store wallet data locally** (encrypted)
2. **Sync with API server** when needed
3. **Backup automatically**

#### Local Storage Configuration

```typescript
// Wallet storage in desktop client
const WALLET_DATA_PATH = 
  process.platform === 'win32' 
    ? path.join(process.env.APPDATA, 'NozyWallet', 'wallet_data')
    : process.platform === 'darwin'
    ? path.join(process.env.HOME, 'Library', 'Application Support', 'NozyWallet', 'wallet_data')
    : path.join(process.env.HOME, '.nozywallet', 'wallet_data');

// API server will handle blockchain operations
// Desktop client stores encrypted wallet locally
```

### Step 6: Distribution

#### Create Installer Packages

**Windows:**
- NSIS installer (Electron Builder)
- MSI installer
- Portable .exe

**macOS:**
- DMG file
- App Store (optional)

**Linux:**
- AppImage
- .deb package
- .rpm package

#### Distribution Channels

1. **GitHub Releases**
   - Upload installers to GitHub Releases
   - Users download from releases page

2. **Website Download**
   - Host installers on your website
   - Provide download links

3. **Auto-Updates**
   - Implement auto-update mechanism
   - Check for updates on startup

## 🔧 Complete Setup Example

### 1. Deploy API Server

```bash
# On your server/VPS
cd api-server
docker build -t nozywallet-api .
docker run -d \
  --name nozywallet-api \
  -p 3000:3000 \
  -e NOZY_PRODUCTION=true \
  -e NOZY_API_KEY=$(openssl rand -hex 32) \
  -e NOZY_CORS_ORIGINS=https://app.yourdomain.com \
  nozywallet-api
```

### 2. Configure Nginx

```nginx
server {
    listen 443 ssl;
    server_name api.yourdomain.com;
    
    location / {
        proxy_pass http://127.0.0.1:3000;
    }
}
```

### 3. Update Desktop Client

```typescript
// Hardcode production API URL
const API_URL = 'https://api.yourdomain.com';
```

### 4. Build and Distribute

```bash
# Build desktop client
npm run build
npm run electron:build

# Creates installers in dist/ folder
# Upload to GitHub Releases or website
```

## 📦 User Experience Flow

1. **User downloads** NozyWallet installer
2. **User installs** the application
3. **User opens** NozyWallet
4. **App connects** to `https://api.yourdomain.com` automatically
5. **User creates** or restores wallet
6. **Wallet data** stored locally (encrypted)
7. **All operations** go through your API server
8. **User uses** wallet normally - no setup needed!

## 🔒 Security Considerations

### For Public API Server

1. **Rate Limiting**: Prevent abuse
   ```bash
   NOZY_RATE_LIMIT_REQUESTS=50  # Lower for public
   NOZY_RATE_LIMIT_WINDOW=60
   ```

2. **API Key**: Optional but recommended
   - Can be embedded in desktop client
   - Or require user registration

3. **CORS**: Restrict to your domains only
   ```bash
   NOZY_CORS_ORIGINS=https://app.yourdomain.com
   ```

4. **HTTPS**: Always use HTTPS in production

5. **Monitoring**: Monitor for abuse/attacks

### For Desktop Client

1. **Encrypt wallet data** locally
2. **Never store** API keys in plain text
3. **Validate** all user input
4. **Code signing** for installers

## 🚀 Quick Start for Users

Once deployed, users just need to:

1. Go to your website
2. Download NozyWallet for their OS
3. Install it
4. Open and use it!

No server setup, no configuration needed.

## 📋 Deployment Checklist

### API Server
- [ ] Deploy to public URL (cloud or VPS)
- [ ] Set `NOZY_PRODUCTION=true`
- [ ] Configure CORS for your domains
- [ ] Set up HTTPS/SSL
- [ ] Configure rate limiting
- [ ] Test all endpoints
- [ ] Set up monitoring

### Desktop Client
- [ ] Update API URL to production
- [ ] Build installers for all platforms
- [ ] Test installation process
- [ ] Test wallet creation/restore
- [ ] Test all wallet operations
- [ ] Set up auto-updates (optional)
- [ ] Create user documentation

### Distribution
- [ ] Upload installers to GitHub Releases
- [ ] Create download page on website
- [ ] Write user guide
- [ ] Create installation instructions
- [ ] Set up support channels

## 🎯 Example: Complete User Setup

### What Users See

1. **Download Page**: "Download NozyWallet"
   - Windows installer
   - macOS installer  
   - Linux installer

2. **Installation**: Standard installer
   - No configuration needed
   - Just click "Install"

3. **First Launch**: 
   - App opens
   - Shows "Create Wallet" or "Restore Wallet"
   - Connects to API automatically
   - Ready to use!

### Behind the Scenes

- Desktop client connects to `https://api.yourdomain.com`
- API server handles all blockchain operations
- Wallet data encrypted and stored locally
- All transactions go through your API server

## 📚 Additional Resources

- [Deployment Guide](api-server/DEPLOYMENT_GUIDE.md) - Detailed server deployment
- [Frontend Integration](FRONTEND_CONNECTION_GUIDE.md) - API integration
- [Desktop Client Integration](DESKTOP_CLIENT_INTEGRATION.md) - Desktop setup

## 🆘 Troubleshooting

**Users can't connect?**
- Check API server is running
- Verify CORS configuration
- Check firewall rules

**Wallet not working?**
- Verify API endpoints are accessible
- Check API server logs
- Ensure Zebra node is accessible from API server

**Slow performance?**
- Optimize API server
- Consider CDN for static assets
- Add caching where appropriate
