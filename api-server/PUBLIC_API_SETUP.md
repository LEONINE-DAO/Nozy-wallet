# Public API Server Setup

This guide explains how to configure the API server for public use so desktop clients can connect without local setup.

## 🎯 Configuration for Public Access

### Required Environment Variables

```bash
# Enable production mode
NOZY_PRODUCTION=true

# API key (optional but recommended for public APIs)
NOZY_API_KEY=your-secure-api-key-here

# CORS - allow your desktop client origins
# Add all domains where your desktop client will be hosted/downloaded from
NOZY_CORS_ORIGINS=https://app.yourdomain.com,https://yourdomain.com,https://github.com

# Port (default: 3000)
NOZY_HTTP_PORT=3000

# Rate limiting (important for public APIs)
NOZY_RATE_LIMIT_REQUESTS=50  # Lower for public access
NOZY_RATE_LIMIT_WINDOW=60
```

### CORS Configuration

The API server automatically handles CORS based on `NOZY_PRODUCTION`:

- **Development**: Allows localhost origins
- **Production**: Only allows origins in `NOZY_CORS_ORIGINS`

For desktop clients, you may want to allow all origins or specific ones:

```bash
# Allow all origins (less secure, but works for desktop apps)
NOZY_CORS_ORIGINS=*

# Or allow specific domains
NOZY_CORS_ORIGINS=https://app.yourdomain.com,https://yourdomain.com
```

**Note:** Desktop applications (Electron/Tauri) typically don't have CORS restrictions, but web-based desktop apps do.

## 🔧 API Server Modifications for Public Use

### Option 1: Allow All Origins (Desktop Apps)

If you want to allow any desktop client to connect, modify `api-server/src/main.rs`:

```rust
// In the CORS configuration section
let cors_layer = if is_production && !cors_origins.is_empty() {
    // Check if CORS_ORIGINS is "*" to allow all
    if cors_origins.contains(&"*".to_string()) {
        CorsLayer::new()
            .allow_origin(AllowOrigin::any())
    } else {
        let origins: Vec<HeaderValue> = cors_origins
            .iter()
            .filter_map(|s| HeaderValue::from_str(s).ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
    }
} else {
    // Development mode - allow localhost
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _request_head: &_| {
            let origin_str = origin.to_str().unwrap_or("");
            origin_str.starts_with("http://localhost:") ||
            origin_str.starts_with("http://127.0.0.1:")
        }))
};
```

### Option 2: Keep Restricted CORS (More Secure)

Keep CORS restricted and configure desktop clients with the correct origin.

## 🌐 Deployment Options

### Option 1: Cloud Platform (Recommended)

**DigitalOcean App Platform:**
```yaml
# .do/app.yaml
name: nozywallet-api
services:
- name: api
  github:
    repo: LEONINE-DAO/Nozy-wallet
    branch: master
  dockerfile_path: api-server/Dockerfile
  http_port: 3000
  instance_count: 1
  instance_size_slug: basic-xxs
  envs:
  - key: NOZY_PRODUCTION
    value: "true"
  - key: NOZY_API_KEY
    value: ${NOZY_API_KEY}  # Set in dashboard
  - key: NOZY_CORS_ORIGINS
    value: "https://app.yourdomain.com"
```

**Google Cloud Run:**
```bash
gcloud run deploy nozywallet-api \
  --source . \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars NOZY_PRODUCTION=true,NOZY_CORS_ORIGINS=https://app.yourdomain.com
```

**AWS App Runner:**
- Connect GitHub repo
- Use Dockerfile
- Set environment variables
- Deploy

### Option 2: VPS with Domain

1. Deploy to VPS
2. Set up domain: `api.yourdomain.com`
3. Configure Nginx + SSL
4. Set environment variables

## 🔒 Security for Public API

### 1. Rate Limiting

```bash
# Stricter limits for public API
NOZY_RATE_LIMIT_REQUESTS=50   # 50 requests
NOZY_RATE_LIMIT_WINDOW=60     # per 60 seconds
```

### 2. API Key (Optional)

You can require API keys or make them optional:

```bash
# Require API key
NOZY_API_KEY=your-key

# Or make optional (no key required)
# Don't set NOZY_API_KEY
```

### 3. Monitoring

Set up monitoring to track:
- Request volume
- Error rates
- Response times
- Suspicious activity

### 4. Logging

Enable structured logging:
```bash
RUST_LOG=info cargo run
```

## 📊 Health Check Endpoint

The API provides a health check endpoint that doesn't require authentication:

```bash
curl https://api.yourdomain.com/health
```

Response:
```json
{
  "status": "ok",
  "service": "nozywallet-api",
  "version": "0.1.0"
}
```

## 🧪 Testing Public API

### Test from Desktop Client

```typescript
// Test connection
const response = await fetch('https://api.yourdomain.com/health');
const data = await response.json();
console.log(data); // { status: "ok", ... }
```

### Test CORS

```bash
curl -H "Origin: https://app.yourdomain.com" \
     -H "Access-Control-Request-Method: GET" \
     -H "Access-Control-Request-Headers: X-API-Key" \
     -X OPTIONS \
     https://api.yourdomain.com/api/balance
```

## 🚀 Quick Deploy Script

```bash
#!/bin/bash
# deploy-public-api.sh

echo "🚀 Deploying NozyWallet Public API..."

# Set environment variables
export NOZY_PRODUCTION=true
export NOZY_API_KEY=$(openssl rand -hex 32)
export NOZY_CORS_ORIGINS=https://app.yourdomain.com
export NOZY_RATE_LIMIT_REQUESTS=50
export NOZY_RATE_LIMIT_WINDOW=60

# Build and run
cd api-server
docker build -t nozywallet-api .
docker run -d \
  --name nozywallet-api \
  -p 3000:3000 \
  --env-file .env.production \
  --restart unless-stopped \
  nozywallet-api

echo "✅ API deployed at http://your-server:3000"
echo "📋 Update your desktop client to use this URL"
```

## 📋 Checklist

- [ ] Deploy API server to public URL
- [ ] Set `NOZY_PRODUCTION=true`
- [ ] Configure `NOZY_CORS_ORIGINS`
- [ ] Set up HTTPS/SSL
- [ ] Configure rate limiting
- [ ] Test health endpoint
- [ ] Test from desktop client
- [ ] Set up monitoring
- [ ] Document API URL for desktop client

## 🎯 Next Steps

1. Deploy API server using one of the options above
2. Get the public URL (e.g., `https://api.yourdomain.com`)
3. Update desktop client to use this URL
4. Build and distribute desktop client
5. Users can now download and use without setup!
