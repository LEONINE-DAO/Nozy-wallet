# Quick Deployment Guide - No Local Server Required

This is a quick reference for deploying the API server so users don't need to run it locally.

## 🚀 Fastest Option: Docker

### 1. Build Docker Image

```bash
cd api-server
docker build -t nozywallet-api .
```

### 2. Run Container

```bash
docker run -d \
  --name nozywallet-api \
  -p 3000:3000 \
  -e NOZY_PRODUCTION=true \
  -e NOZY_API_KEY=$(openssl rand -hex 32) \
  -e NOZY_CORS_ORIGINS=https://your-frontend.com \
  nozywallet-api
```

### 3. Deploy to Cloud

**DigitalOcean App Platform:**
- Connect GitHub repo
- Select `api-server/Dockerfile`
- Set environment variables
- Deploy!

**AWS ECS/Fargate:**
- Push image to ECR
- Create Fargate task
- Set environment variables
- Deploy!

**Google Cloud Run:**
```bash
gcloud run deploy nozywallet-api \
  --source . \
  --platform managed \
  --region us-central1 \
  --set-env-vars NOZY_PRODUCTION=true,NOZY_API_KEY=your-key
```

## 📋 Required Configuration

### Environment Variables

```bash
NOZY_PRODUCTION=true                    # Enable production mode
NOZY_API_KEY=your-secure-key-here       # REQUIRED: Generate with: openssl rand -hex 32
NOZY_CORS_ORIGINS=https://your-app.com  # Your frontend URL(s)
NOZY_HTTP_PORT=3000                     # Port (default: 3000)
```

### Frontend Update

Change API URL in your frontend:

```typescript
// Before (local)
const API_URL = 'http://localhost:3000';

// After (remote)
const API_URL = 'https://api.yourdomain.com';
```

## 🔒 HTTPS Setup (Required for Production)

### Option 1: Cloud Platform (Automatic)
- Most cloud platforms provide HTTPS automatically
- Just configure your domain

### Option 2: Nginx + Let's Encrypt

```bash
# Install
sudo apt-get install nginx certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d api.yourdomain.com

# Use nginx.conf.example as template
```

## ✅ Deployment Checklist

- [ ] Build Docker image or release binary
- [ ] Set `NOZY_PRODUCTION=true`
- [ ] Generate and set `NOZY_API_KEY`
- [ ] Configure `NOZY_CORS_ORIGINS`
- [ ] Deploy to cloud/server
- [ ] Set up HTTPS
- [ ] Update frontend API URL
- [ ] Test: `curl https://api.yourdomain.com/health`

## 📚 Full Guide

See [api-server/DEPLOYMENT_GUIDE.md](api-server/DEPLOYMENT_GUIDE.md) for detailed instructions.

## 🆘 Quick Troubleshooting

**Can't connect?**
- Check firewall allows port 3000 (or 80/443 if using Nginx)
- Verify `NOZY_CORS_ORIGINS` includes your frontend URL
- Check logs: `docker logs nozywallet-api` or `journalctl -u nozywallet-api`

**CORS errors?**
- Ensure `NOZY_PRODUCTION=true` is set
- Verify frontend URL matches exactly in `NOZY_CORS_ORIGINS`

**API key not working?**
- Check key is set correctly
- Use header: `X-API-Key: your-key` or `Authorization: Bearer your-key`
