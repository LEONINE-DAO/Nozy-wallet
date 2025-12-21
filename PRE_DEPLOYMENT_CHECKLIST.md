# Pre-Deployment Checklist - Verify Everything Before Deploying

**Check this list BEFORE clicking "Create Resources" in DigitalOcean!**

## ✅ File Verification

### 1. Dockerfile in Root Directory
- [ ] `Dockerfile` exists in repository root (not in `api-server/`)
- [ ] Dockerfile is committed to git
- [ ] Dockerfile is pushed to GitHub

**Check:**
```bash
git ls-files Dockerfile
```
Should show: `Dockerfile`

### 2. Required Source Files
- [ ] `api-server/src/main.rs` is in repository
- [ ] `api-server/src/handlers.rs` is in repository
- [ ] `api-server/src/middleware.rs` is in repository
- [ ] `zeaking/Cargo.toml` is in repository
- [ ] `zeaking/src/` files are in repository

**Check:**
```bash
git ls-files api-server/src/main.rs
git ls-files zeaking/Cargo.toml
```

### 3. Dockerfile Content Check
- [ ] Dockerfile copies from root directory
- [ ] Dockerfile builds `api-server`
- [ ] Dockerfile exposes port 3000
- [ ] Dockerfile has correct CMD

**Current Dockerfile should:**
- Start with `FROM rust:1.70 as builder`
- Copy `Cargo.toml`, `api-server/`, `zeaking/`
- Build in `/app/api-server`
- Expose port 3000
- Run `/app/nozywallet-api`

---

## ✅ DigitalOcean Configuration Checklist

### On the Page: https://cloud.digitalocean.com/apps/new?source_provider=github

#### Step 1: Repository Selection
- [ ] GitHub is connected/authorized
- [ ] Repository `LEONINE-DAO/Nozy-wallet` is visible
- [ ] Repository `LEONINE-DAO/Nozy-wallet` is selected
- [ ] Branch is set to `master` (or `main`)

#### Step 2: Build Settings
- [ ] **Type**: Should show "Docker" (or manually select it)
- [ ] **Dockerfile Path**: `Dockerfile` (root level)
- [ ] **Source Directory**: Empty or `/` (root)
- [ ] **Port**: `3000`

**If Dockerfile NOT detected:**
- [ ] Click "Edit" or "Configure"
- [ ] Manually select "Docker" from dropdown
- [ ] Set Dockerfile Path: `Dockerfile`
- [ ] Set Port: `3000`

#### Step 3: Environment Variables (CRITICAL!)

Add these **EXACTLY** as shown:

**Variable 1:**
- [ ] Key: `NOZY_PRODUCTION`
- [ ] Value: `true`
- [ ] Type: Plain text

**Variable 2:**
- [ ] Key: `NOZY_API_KEY`
- [ ] Value: (generate with `openssl rand -hex 32` or use a secure random string)
- [ ] Type: **SECRET** (important!)

**Variable 3:**
- [ ] Key: `NOZY_CORS_ORIGINS`
- [ ] Value: `*` (just an asterisk)
- [ ] Type: Plain text

**Variable 4:**
- [ ] Key: `NOZY_HTTP_PORT`
- [ ] Value: `3000`
- [ ] Type: Plain text

**Variable 5:**
- [ ] Key: `NOZY_RATE_LIMIT_REQUESTS`
- [ ] Value: `100`
- [ ] Type: Plain text

**Variable 6:**
- [ ] Key: `NOZY_RATE_LIMIT_WINDOW`
- [ ] Value: `60`
- [ ] Type: Plain text

#### Step 4: Plan Selection
- [ ] Plan selected (Basic $5/month minimum)
- [ ] Instance size chosen (smallest is fine)

---

## 🔍 Quick Verification Commands

Run these to verify files are ready:

```bash
# Check Dockerfile exists and is tracked
git ls-files Dockerfile

# Check all required files exist
git ls-files api-server/src/main.rs
git ls-files api-server/src/handlers.rs
git ls-files api-server/src/middleware.rs
git ls-files zeaking/Cargo.toml

# Verify Dockerfile content
cat Dockerfile | head -5
```

**Expected output:**
- All files should be listed (not empty)
- Dockerfile should start with `FROM rust:1.70`

---

## ⚠️ Common Mistakes to Avoid

### ❌ DON'T:
- Don't set Dockerfile path to `api-server/Dockerfile` (use root `Dockerfile`)
- Don't forget to add environment variables
- Don't set port to something other than 3000
- Don't forget to set `NOZY_PRODUCTION=true`
- Don't use `localhost` in CORS origins (use `*` for desktop apps)

### ✅ DO:
- Use `Dockerfile` in root directory
- Add ALL 6 environment variables
- Set port to 3000
- Set `NOZY_PRODUCTION=true`
- Use `*` for `NOZY_CORS_ORIGINS` (allows desktop apps)

---

## 🎯 Final Check Before Deploying

Before clicking **"Create Resources"**:

1. [ ] All files pushed to GitHub
2. [ ] Dockerfile in root directory
3. [ ] Repository selected correctly
4. [ ] Build type is "Docker"
5. [ ] Port is 3000
6. [ ] All 6 environment variables added
7. [ ] `NOZY_PRODUCTION=true` is set
8. [ ] `NOZY_API_KEY` is set as SECRET
9. [ ] Plan selected

---

## 🚨 If Something is Missing

### Dockerfile not in git:
```bash
git add Dockerfile
git commit -m "Add Dockerfile for deployment"
git push origin master
```

### Files not pushed:
```bash
git add .
git commit -m "Prepare for deployment"
git push origin master
```

### Then refresh DigitalOcean page and reconnect repository

---

## ✅ Ready to Deploy?

If ALL items above are checked ✅, you're ready to click **"Create Resources"**!

After deployment:
1. Wait 5-10 minutes for build
2. Get your API URL
3. Test: `https://your-url/health`
4. Should see: `{"status":"ok"}`

---

**Take your time!** It's better to check everything twice than to have to fix it later.
