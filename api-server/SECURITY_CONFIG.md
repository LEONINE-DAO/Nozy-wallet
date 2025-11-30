# API Server Security Configuration

This document describes the security features implemented in the NozyWallet API server and how to configure them.

## Security Features

### 1. API Key Authentication

The API server supports optional API key authentication to protect endpoints.

**Configuration:**
- Set the `NOZY_API_KEY` environment variable to enable authentication
- If not set, authentication is disabled (development mode)

**Usage:**
Clients must include the API key in one of two ways:
1. `X-API-Key` header: `X-API-Key: your-api-key-here`
2. `Authorization` header: `Authorization: Bearer your-api-key-here`

**Example:**
```bash
# With X-API-Key header
curl -H "X-API-Key: your-secret-key" http://localhost:3000/api/balance

# With Authorization header
curl -H "Authorization: Bearer your-secret-key" http://localhost:3000/api/balance
```

### 2. Rate Limiting

Rate limiting prevents abuse and DoS attacks by limiting the number of requests per IP address.

**Configuration:**
- `NOZY_RATE_LIMIT_REQUESTS`: Maximum requests per window (default: 100)
- `NOZY_RATE_LIMIT_WINDOW`: Time window in seconds (default: 60)

**Example:**
```bash
export NOZY_RATE_LIMIT_REQUESTS=200
export NOZY_RATE_LIMIT_WINDOW=60
```

**Response Headers:**
- `X-RateLimit-Limit`: Maximum requests allowed
- `X-RateLimit-Remaining`: Remaining requests in current window

When rate limit is exceeded, the server returns `429 Too Many Requests`.

### 3. CORS Configuration

CORS (Cross-Origin Resource Sharing) is configured based on environment.

**Development Mode (default):**
- Allows localhost origins on any port
- Allows 127.0.0.1 origins
- Allows Android emulator (10.0.2.2)
- Allows network access (0.0.0.0)

**Production Mode:**
- Set `NOZY_PRODUCTION=true` to enable production mode
- Set `NOZY_CORS_ORIGINS` to comma-separated list of allowed origins
- Example: `NOZY_CORS_ORIGINS=https://app.example.com,https://www.example.com`

**Example:**
```bash
export NOZY_PRODUCTION=true
export NOZY_CORS_ORIGINS=https://app.nozywallet.com,https://www.nozywallet.com
```

### 4. Security Headers

The API server includes the following security headers:

- `X-Content-Type-Options: nosniff` - Prevents MIME type sniffing
- `X-Frame-Options: DENY` - Prevents clickjacking
- `X-XSS-Protection: 1; mode=block` - XSS protection
- `Referrer-Policy: strict-origin-when-cross-origin` - Controls referrer information
- `Content-Security-Policy` - Restricts resource loading
- `Permissions-Policy` - Controls browser features

**HSTS (Strict-Transport-Security):**
- Currently disabled (commented out) for development
- Enable in production when using HTTPS

### 5. Input Validation

All endpoints validate input to prevent injection attacks and invalid data:

- **Addresses**: Must be valid shielded addresses (u1...)
- **Amounts**: Must be > 0 and <= 21,000,000 ZEC
- **Mnemonics**: Must be 12, 15, 18, 21, or 24 words
- **URLs**: Must start with http:// or https://
- **Themes**: Must be "dark" or "light"
- **Memo length**: Max 512 characters
- **Password length**: Max 256 characters
- **URL length**: Max 2048 characters

### 6. Request Logging

All requests are logged with:
- HTTP method
- Request path
- Client IP address
- Response status
- Request duration

## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `NOZY_API_KEY` | API key for authentication | None (disabled) | No |
| `NOZY_RATE_LIMIT_REQUESTS` | Max requests per window | 100 | No |
| `NOZY_RATE_LIMIT_WINDOW` | Time window in seconds | 60 | No |
| `NOZY_PRODUCTION` | Enable production mode | false | No |
| `NOZY_CORS_ORIGINS` | Comma-separated allowed origins | localhost only | No |

## Production Deployment

### Recommended Configuration

```bash
# Generate a secure API key (use a secure random generator)
export NOZY_API_KEY=$(openssl rand -hex 32)

# Set rate limits (adjust based on expected traffic)
export NOZY_RATE_LIMIT_REQUESTS=1000
export NOZY_RATE_LIMIT_WINDOW=60

# Enable production mode
export NOZY_PRODUCTION=true

# Set allowed CORS origins
export NOZY_CORS_ORIGINS=https://app.nozywallet.com,https://www.nozywallet.com
```

### Security Checklist

- [ ] Set `NOZY_API_KEY` with a strong, random key
- [ ] Set `NOZY_PRODUCTION=true`
- [ ] Configure `NOZY_CORS_ORIGINS` with your actual frontend URLs
- [ ] Use HTTPS (configure reverse proxy like Nginx)
- [ ] Enable HSTS header (uncomment in middleware.rs)
- [ ] Review and adjust rate limits based on traffic
- [ ] Monitor logs for suspicious activity
- [ ] Keep dependencies updated

### HTTPS Setup

The API server runs on HTTP by default. For production:

1. Use a reverse proxy (Nginx, Caddy, etc.) for HTTPS termination
2. Configure SSL/TLS certificates
3. Uncomment HSTS header in `middleware.rs`
4. Set `NOZY_PRODUCTION=true`

### Example Nginx Configuration

```nginx
server {
    listen 443 ssl http2;
    server_name api.nozywallet.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Testing Security Features

### Test API Key Authentication

```bash
# Should fail without API key (if enabled)
curl http://localhost:3000/api/balance

# Should succeed with API key
curl -H "X-API-Key: your-key" http://localhost:3000/api/balance
```

### Test Rate Limiting

```bash
# Make many requests quickly
for i in {1..150}; do
  curl http://localhost:3000/health
done
# Should get 429 after rate limit
```

### Test Input Validation

```bash
# Invalid address
curl -X POST http://localhost:3000/api/transaction/send \
  -H "Content-Type: application/json" \
  -d '{"recipient": "invalid", "amount": 1.0}'

# Invalid amount
curl -X POST http://localhost:3000/api/transaction/send \
  -H "Content-Type: application/json" \
  -d '{"recipient": "u1test...", "amount": -1.0}'
```

## Notes

- Health check endpoint (`/health`) does not require authentication
- Rate limiting is per IP address
- API key authentication is optional - set `NOZY_API_KEY` to enable
- All security features work independently - you can enable/disable them as needed

