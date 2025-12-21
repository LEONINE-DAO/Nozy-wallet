# DigitalOcean App Platform Deployment Guide

This guide helps you deploy the NozyWallet API server to DigitalOcean App Platform.

## 🚀 Quick Setup

### Step 1: Push Dockerfile to GitHub

The Dockerfile needs to be in the repository root for DigitalOcean to detect it:

```bash
# Add and commit the Dockerfile
git add Dockerfile
git commit -m "Add Dockerfile for DigitalOcean deployment"
git push origin master
```

### Step 2: Configure DigitalOcean App Platform

1. **Go to**: https://cloud.digitalocean.com/apps
2. **Click**: "Create App"
3. **Connect GitHub**: Authorize and select `LEONINE-DAO/Nozy-wallet` repository
4. **Configure App**:
   - **Source**: Select your repository and `master` branch
   - **Type**: DigitalOcean should auto-detect "Docker" (because of Dockerfile in root)
   - **Dockerfile Path**: Leave as `Dockerfile` (it's in root)
   - **Port**: `3000`

### Step 3: Set Environment Variables

In the App Settings → Environment Variables, add:

```
NOZY_PRODUCTION=true
NOZY_API_KEY=your-secure-api-key-here
NOZY_CORS_ORIGINS=https://app.yourdomain.com
NOZY_HTTP_PORT=3000
NOZY_RATE_LIMIT_REQUESTS=50
NOZY_RATE_LIMIT_WINDOW=60
```

**Generate API Key:**
```bash
openssl rand -hex 32
```

### Step 4: Deploy

1. Click "Next" through the configuration
2. Choose a plan (Basic $5/month minimum, or use free trial)
3. Click "Create Resources"
4. DigitalOcean will build and deploy automatically

### Step 5: Get Your API URL

After deployment, you'll get a URL like:
- `https://api-nozywallet-xxxxx.ondigitalocean.app`

This is your public API endpoint!

## 🔧 Alternative: If Dockerfile Detection Fails

If DigitalOcean still doesn't detect the Dockerfile:

### Option 1: Manual Configuration

1. In DigitalOcean, select "Docker" manually
2. Set **Dockerfile Path**: `Dockerfile` (root level)
3. Set **Source Directory**: Leave empty (root)

### Option 2: Use .do/app.yaml

Create `.do/app.yaml` in your repository root:

```yaml
name: nozywallet-api
services:
- name: api
  github:
    repo: LEONINE-DAO/Nozy-wallet
    branch: master
  dockerfile_path: Dockerfile
  http_port: 3000
  instance_count: 1
  instance_size_slug: basic-xxs
  envs:
  - key: NOZY_PRODUCTION
    value: "true"
  - key: NOZY_API_KEY
    scope: RUN_TIME
    type: SECRET
  - key: NOZY_CORS_ORIGINS
    value: "https://app.yourdomain.com"
  - key: NOZY_HTTP_PORT
    value: "3000"
  - key: NOZY_RATE_LIMIT_REQUESTS
    value: "50"
  - key: NOZY_RATE_LIMIT_WINDOW
    value: "60"
```

Then DigitalOcean will use this configuration automatically.

## ✅ Verification

After deployment, test your API:

```bash
curl https://your-app-url.ondigitalocean.app/health
```

Expected response:
```json
{
  "status": "ok",
  "service": "nozywallet-api",
  "version": "0.1.0"
}
```

## 🔒 Security Notes

1. **API Key**: Set `NOZY_API_KEY` as a **SECRET** in DigitalOcean (not plain text)
2. **CORS**: Update `NOZY_CORS_ORIGINS` with your actual frontend domains
3. **HTTPS**: DigitalOcean provides HTTPS automatically
4. **Rate Limiting**: Adjust based on your needs

## 📋 Checklist

- [ ] Dockerfile is in repository root
- [ ] Dockerfile is committed and pushed to GitHub
- [ ] DigitalOcean app is created
- [ ] Environment variables are set
- [ ] App is deployed successfully
- [ ] Health check endpoint works
- [ ] API URL is documented for desktop client

## 🆘 Troubleshooting

**"No components detected" error:**
- Make sure `Dockerfile` is in the repository root
- Verify it's committed and pushed to GitHub
- Check that DigitalOcean has access to your repository

**Build fails:**
- Check build logs in DigitalOcean dashboard
- Verify all source files are in the repository
- Ensure `zeaking` directory is committed

**App won't start:**
- Check environment variables are set correctly
- Verify port 3000 is exposed
- Check app logs in DigitalOcean dashboard

## 🎯 Next Steps

After successful deployment:

1. **Get your API URL** from DigitalOcean dashboard
2. **Update desktop client** to use this URL
3. **Test all endpoints** to ensure everything works
4. **Set up custom domain** (optional) in DigitalOcean settings

## 📚 Additional Resources

- [DigitalOcean App Platform Docs](https://docs.digitalocean.com/products/app-platform/)
- [Deployment Guide](api-server/DEPLOYMENT_GUIDE.md)
- [Public API Setup](api-server/PUBLIC_API_SETUP.md)
