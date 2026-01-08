# DigitalOcean Step-by-Step Guide - Complete Setup

This is a **beginner-friendly, detailed guide** to deploy NozyWallet API so users can download and use the desktop app without running anything locally.

## 🎯 Goal

After completing this guide:
- ✅ API server running on DigitalOcean (public URL)
- ✅ Users can download desktop client
- ✅ Desktop client connects automatically to your API
- ✅ No local setup needed for users

---

## 📋 BEFORE YOU START - Checklist

Make sure these files are pushed to GitHub:
- [ ] `Dockerfile` (in root directory)
- [ ] All `api-server/` code
- [ ] All `zeaking/` code

**If not pushed yet:**
```bash
git add Dockerfile api-server/ zeaking/
git commit -m "Add files for DigitalOcean deployment"
git push origin master
```

---

## 🚀 STEP 1: Create DigitalOcean Account

1. Go to: **https://www.digitalocean.com**
2. Click **"Sign Up"** (top right)
3. Create account (can use GitHub to sign up)
4. Verify your email

---

## 🚀 STEP 2: Navigate to App Platform

1. After logging in, you'll see the DigitalOcean dashboard
2. Look for **"Apps"** in the left sidebar menu
3. Click **"Apps"**
4. You should see: **"Create App"** button (big blue button)
5. Click **"Create App"**

---

## 🚀 STEP 3: Connect GitHub Repository

1. You'll see options: **"GitHub"**, **"Docker Hub"**, **"Source Code"**
2. Click **"GitHub"** (first option)
3. If not connected:
   - Click **"Connect GitHub"** or **"Authorize GitHub"**
   - Authorize DigitalOcean to access your repositories
   - You may need to enter your GitHub password
4. After authorization, you'll see your repositories
5. **Find and select**: `LEONINE-DAO/Nozy-wallet`
6. Click **"Next"** or **"Continue"**

---

## 🚀 STEP 4: Configure Repository Settings

You'll see a page asking about your repository:

1. **Repository**: Should show `LEONINE-DAO/Nozy-wallet` (auto-selected)
2. **Branch**: Select **"master"** (or "main" if that's your default branch)
3. **Autodeploy**: You can check this if you want auto-deploy on push (optional)
4. Click **"Next"** or **"Continue"**

---

## 🚀 STEP 5: Configure Build Settings

This is where DigitalOcean detects your app type:

### If Dockerfile is Detected (Best Case):

1. DigitalOcean should show: **"Docker"** or **"Dockerfile detected"**
2. You'll see:
   - **Type**: Docker
   - **Dockerfile Path**: `Dockerfile` (should be auto-filled)
   - **Source Directory**: Leave empty (or `/`)
3. **Port**: Set to **`3000`**
4. Click **"Next"**

### If NOT Detected (Manual Setup):

1. Look for **"Edit"** or **"Configure"** button
2. Click it
3. Select **"Docker"** from the dropdown
4. Set **Dockerfile Path**: `Dockerfile`
5. Set **Port**: `3000`
6. Click **"Save"** or **"Next"**

---

## 🚀 STEP 6: Set Environment Variables

This is **VERY IMPORTANT** - these settings make your API work:

1. Look for **"Environment Variables"** section
2. Click **"Edit"** or **"Add Variable"**
3. Add these **ONE BY ONE**:

### Variable 1:
- **Key**: `NOZY_PRODUCTION`
- **Value**: `true`
- **Type**: Plain text (default)
- Click **"Add"** or **"Save"**

### Variable 2:
- **Key**: `NOZY_API_KEY`
- **Value**: Generate one using: `openssl rand -hex 32` (run in terminal)
  - Or use: `your-secure-random-key-here-32-characters`
- **Type**: **SECRET** (important - click the dropdown and select "Secret")
- Click **"Add"**

### Variable 3:
- **Key**: `NOZY_CORS_ORIGINS`
- **Value**: `*` (asterisk - allows all origins, safe for desktop apps)
- **Type**: Plain text
- Click **"Add"**

### Variable 4:
- **Key**: `NOZY_HTTP_PORT`
- **Value**: `3000`
- **Type**: Plain text
- Click **"Add"**

### Variable 5:
- **Key**: `NOZY_RATE_LIMIT_REQUESTS`
- **Value**: `100`
- **Type**: Plain text
- Click **"Add"**

### Variable 6:
- **Key**: `NOZY_RATE_LIMIT_WINDOW`
- **Value**: `60`
- **Type**: Plain text
- Click **"Add"**

4. After adding all variables, click **"Next"** or **"Continue"**

---

## 🚀 STEP 7: Choose Plan & Deploy

1. You'll see pricing options
2. **Minimum**: Select **"Basic"** plan ($5/month)
   - Or use free trial if available
3. **Instance Size**: Smallest is fine to start (512MB RAM)
4. Click **"Next"** or **"Continue"**

---

## 🚀 STEP 8: Review & Create

1. Review your settings:
   - Repository: `LEONINE-DAO/Nozy-wallet`
   - Type: Docker
   - Port: 3000
   - Environment variables: All 6 added
2. **App Name**: DigitalOcean will suggest one, or you can change it
3. Click **"Create Resources"** or **"Deploy"** (big button at bottom)

---

## 🚀 STEP 9: Wait for Build & Deploy

1. You'll see a progress screen
2. DigitalOcean will:
   - Build your Docker image (takes 5-10 minutes)
   - Deploy it
   - Start the app
3. **DON'T CLOSE THE TAB** - wait for it to finish
4. You'll see logs showing the build progress

---

## 🚀 STEP 10: Get Your API URL

After deployment completes:

1. You'll see a **"Success"** or **"Deployed"** message
2. Look for **"App URL"** or **"Live App"** section
3. You'll see a URL like:
   - `https://api-nozywallet-xxxxx.ondigitalocean.app`
4. **COPY THIS URL** - this is your public API endpoint!

---

## ✅ STEP 11: Test Your API

1. Open a new browser tab
2. Go to: `https://your-app-url.ondigitalocean.app/health`
3. You should see:
   ```json
   {
     "status": "ok",
     "service": "nozywallet-api",
     "version": "0.1.0"
   }
   ```
4. **If you see this, it's working!** ✅

---

## 🔧 STEP 12: Update Desktop Client

Now update your desktop client to use this URL:

### In your desktop client code:

```typescript
// Change this:
const API_URL = 'http://localhost:3000';

// To this (use your DigitalOcean URL):
const API_URL = 'https://api-nozywallet-xxxxx.ondigitalocean.app';
```

Then rebuild your desktop client and users can download it!

---

## 🆘 TROUBLESHOOTING

### Problem: "No components detected"

**Solution:**
1. Make sure `Dockerfile` is in the **root** of your repository
2. Make sure it's **committed and pushed** to GitHub
3. In DigitalOcean, try:
   - Click **"Edit"** on the build settings
   - Manually select **"Docker"**
   - Set Dockerfile path to: `Dockerfile`

### Problem: Build fails

**Solution:**
1. Click on your app in DigitalOcean
2. Go to **"Runtime Logs"** tab
3. Check the error message
4. Common issues:
   - Missing files (make sure `zeaking/` is committed)
   - Wrong Dockerfile path
   - Port conflict

### Problem: App won't start

**Solution:**
1. Check **"Runtime Logs"** in DigitalOcean
2. Verify all environment variables are set correctly
3. Make sure port is `3000`
4. Check that `NOZY_PRODUCTION=true` is set

### Problem: Can't find "Apps" in sidebar

**Solution:**
1. Make sure you're logged into DigitalOcean
2. Look for **"App Platform"** in the left menu
3. Or go directly to: `https://cloud.digitalocean.com/apps`

### Problem: Health check doesn't work

**Solution:**
1. Wait a few minutes after deployment (app might still be starting)
2. Check the app status in DigitalOcean dashboard
3. Look at **"Runtime Logs"** for errors
4. Verify the URL is correct (no typos)

---

## 📋 Quick Reference: All Environment Variables

Copy-paste this list when setting up:

```
NOZY_PRODUCTION=true
NOZY_API_KEY=your-secure-key-here
NOZY_CORS_ORIGINS=*
NOZY_HTTP_PORT=3000
NOZY_RATE_LIMIT_REQUESTS=100
NOZY_RATE_LIMIT_WINDOW=60
```

**Generate API Key:**
```bash
# Run this in terminal:
openssl rand -hex 32
```

---

## 🎯 What Happens Next?

After successful deployment:

1. ✅ Your API is live at: `https://your-url.ondigitalocean.app`
2. ✅ Users can connect to it from anywhere
3. ✅ Update desktop client to use this URL
4. ✅ Users download desktop client → it works immediately!

---

## 📞 Need More Help?

If you get stuck at any step:

1. **Check the error message** in DigitalOcean logs
2. **Verify all files are pushed** to GitHub
3. **Make sure environment variables** are set correctly
4. **Check the troubleshooting section** above

---

## ✅ Final Checklist

Before considering it done:

- [ ] App deployed successfully in DigitalOcean
- [ ] Health check works: `https://your-url/health`
- [ ] All 6 environment variables are set
- [ ] Desktop client updated with new API URL
- [ ] Desktop client tested and working
- [ ] Users can download and use without setup!

---

**You've got this!** Take it one step at a time. If you get stuck, check which step you're on and refer to the troubleshooting section.
