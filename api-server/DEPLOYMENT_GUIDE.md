# API Server Deployment Guide

This guide explains how to deploy the NozyWallet API server so users don't need to run it locally.

## 🚀 Deployment Options

### Option 1: Docker Deployment (Recommended)

Docker makes deployment easy and consistent across platforms.

#### 1. Create Dockerfile

Create `api-server/Dockerfile`:

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY api-server/Cargo.toml ./api-server/
COPY zeaking/Cargo.toml ./zeaking/
COPY src ./src
COPY api-server/src ./api-server/src
COPY zeaking/src ./zeaking/src

WORKDIR /app/api-server
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/api-server/target/release/nozywallet-api /app/nozywallet-api

RUN mkdir -p /app/wallet_data

EXPOSE 3000

CMD ["/app/nozywallet-api"]
```

#### 2. Create .dockerignore

Create `api-server/.dockerignore`:

```
target/
.git/
*.md
.env
wallet_data/
```

#### 3. Build and Run Docker Container

```bash
cd api-server
docker build -t nozywallet-api:latest .
docker run -d \
  --name nozywallet-api \
  -p 3000:3000 \
  -e NOZY_PRODUCTION=true \
  -e NOZY_API_KEY=your-secure-api-key-here \
  -e NOZY_CORS_ORIGINS=https://your-frontend-domain.com \
  -v $(pwd)/wallet_data:/app/wallet_data \
  nozywallet-api:latest
```

#### 4. Deploy to Cloud

**Docker Hub / Container Registry:**
```bash
docker tag nozywallet-api:latest your-registry/nozywallet-api:latest
docker push your-registry/nozywallet-api:latest
```

**Deploy to cloud services:**
- **AWS ECS/Fargate**: Use the Docker image
- **Google Cloud Run**: Deploy container directly
- **DigitalOcean App Platform**: Use Dockerfile
- **Azure Container Instances**: Deploy container
- **Heroku**: Use container registry

### Option 2: Direct Server Deployment

#### 1. Build Release Binary

```bash
cd api-server
cargo build --release
```

#### 2. Copy to Server

```bash
scp target/release/nozywallet-api user@server:/opt/nozywallet/
scp -r wallet_data user@server:/opt/nozywallet/
```

#### 3. Create Systemd Service

Create `/etc/systemd/system/nozywallet-api.service`:

```ini
[Unit]
Description=NozyWallet API Server
After=network.target

[Service]
Type=simple
User=nozywallet
WorkingDirectory=/opt/nozywallet
ExecStart=/opt/nozywallet/nozywallet-api
Restart=always
RestartSec=10

Environment="NOZY_PRODUCTION=true"
Environment="NOZY_API_KEY=your-secure-api-key-here"
Environment="NOZY_CORS_ORIGINS=https://your-frontend-domain.com"
Environment="NOZY_HTTP_PORT=3000"
Environment="NOZY_RATE_LIMIT_REQUESTS=100"
Environment="NOZY_RATE_LIMIT_WINDOW=60"

NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

#### 4. Enable and Start Service

```bash
sudo systemctl daemon-reload
sudo systemctl enable nozywallet-api
sudo systemctl start nozywallet-api
sudo systemctl status nozywallet-api
```

### Option 3: Cloud Platform Deployment

#### AWS EC2 / Lightsail

1. Launch EC2 instance (Ubuntu 22.04 LTS)
2. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. Clone repository
4. Build and deploy using systemd service (see Option 2)

#### DigitalOcean Droplet

1. Create Droplet (Ubuntu 22.04)
2. Follow Option 2 steps
3. Use DigitalOcean's firewall to allow port 3000

#### Google Cloud Compute Engine

1. Create VM instance
2. Follow Option 2 steps
3. Configure firewall rules

## 🔒 Production Configuration

### Required Environment Variables

```bash
export NOZY_PRODUCTION=true

export NOZY_API_KEY=$(openssl rand -hex 32)

export NOZY_CORS_ORIGINS=https://your-frontend.com,https://www.your-frontend.com

export NOZY_HTTP_PORT=3000

export NOZY_RATE_LIMIT_REQUESTS=100
export NOZY_RATE_LIMIT_WINDOW=60
```

### HTTPS Setup with Nginx Reverse Proxy

#### 1. Install Nginx

```bash
sudo apt-get update
sudo apt-get install nginx certbot python3-certbot-nginx
```

#### 2. Configure Nginx

Create `/etc/nginx/sites-available/nozywallet-api`:

```nginx
server {
    listen 80;
    server_name api.yourdomain.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    # SSL certificates (Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/api.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.yourdomain.com/privkey.pem;

    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Proxy to API server
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check endpoint
    location /health {
        proxy_pass http://127.0.0.1:3000/health;
        access_log off;
    }
}
```

#### 3. Enable Site and Get SSL Certificate

```bash
sudo ln -s /etc/nginx/sites-available/nozywallet-api /etc/nginx/sites-enabled/
sudo nginx -t
sudo certbot --nginx -d api.yourdomain.com
sudo systemctl reload nginx
```

## 🌐 Frontend Configuration

Update your frontend to use the remote API:

```typescript
const API_BASE_URL = 'https://api.yourdomain.com';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'https://api.yourdomain.com';
```

## 📋 Deployment Checklist

- [ ] Build release binary or Docker image
- [ ] Set `NOZY_PRODUCTION=true`
- [ ] Generate and set `NOZY_API_KEY`
- [ ] Configure `NOZY_CORS_ORIGINS` with frontend URLs
- [ ] Set up HTTPS (Nginx + Let's Encrypt)
- [ ] Configure firewall (allow ports 80, 443, 3000)
- [ ] Set up systemd service or Docker container
- [ ] Test API endpoints
- [ ] Update frontend API URL
- [ ] Monitor logs: `journalctl -u nozywallet-api -f`
- [ ] Set up monitoring/alerting
- [ ] Configure backups for `wallet_data/`

## 🔧 Quick Deploy Script

Create `api-server/deploy.sh`:

```bash
set -e

echo "🚀 Deploying NozyWallet API Server..."

echo "📦 Building..."
cargo build --release

sudo systemctl stop nozywallet-api || true

sudo cp target/release/nozywallet-api /opt/nozywallet/
sudo chmod +x /opt/nozywallet/nozywallet-api

sudo systemctl start nozywallet-api
sudo systemctl status nozywallet-api

echo "✅ Deployment complete!"
```

## 🐳 Docker Compose (Full Stack)

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  api:
    build:
      context: .
      dockerfile: api-server/Dockerfile
    ports:
      - "3000:3000"
    environment:
      - NOZY_PRODUCTION=true
      - NOZY_API_KEY=${NOZY_API_KEY}
      - NOZY_CORS_ORIGINS=${NOZY_CORS_ORIGINS}
    volumes:
      - ./wallet_data:/app/wallet_data
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - api
    restart: unless-stopped
```

## 🔐 Security Best Practices

1. **Always use HTTPS in production**
2. **Set a strong API key**: `openssl rand -hex 32`
3. **Restrict CORS origins** to your frontend domains only
4. **Use firewall** to restrict access
5. **Keep dependencies updated**: `cargo update`
6. **Monitor logs** for suspicious activity
7. **Backup wallet data** regularly
8. **Use environment variables** for secrets (never commit them)

## 📊 Monitoring

### View Logs

```bash
sudo journalctl -u nozywallet-api -f

docker logs -f nozywallet-api
```

### Health Check

```bash
curl https://api.yourdomain.com/health
```

## 🆘 Troubleshooting

### Service Won't Start

```bash
sudo journalctl -u nozywallet-api -n 50
```

### Port Already in Use

```bash
sudo lsof -i :3000
sudo kill -9 <PID>
```

### CORS Errors

- Verify `NOZY_CORS_ORIGINS` includes your frontend URL
- Check that `NOZY_PRODUCTION=true` is set
- Ensure frontend uses exact URL (no trailing slash differences)

### API Key Not Working

- Verify API key is set correctly
- Check header format: `X-API-Key: your-key` or `Authorization: Bearer your-key`
- Check server logs for authentication errors

## 🚀 Quick Start Commands

```bash
cd api-server
cargo build --release
NOZY_PRODUCTION=true NOZY_API_KEY=your-key ./target/release/nozywallet-api

docker build -t nozywallet-api .
docker run -p 3000:3000 -e NOZY_PRODUCTION=true -e NOZY_API_KEY=your-key nozywallet-api

sudo systemctl start nozywallet-api
sudo systemctl status nozywallet-api
```

## 📚 Additional Resources

- [API Server README](README.md)
- [Security Configuration](SECURITY_CONFIG.md)
- [Frontend Developer Guide](FRONTEND_DEVELOPER_GUIDE.md)
