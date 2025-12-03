# Crosslink Quick Start - Switch Backend in 30 Seconds

## 🚀 Fastest Way to Switch to Crosslink

### Step 1: Switch Backend

```bash
# Switch to Crosslink
nozy config --use-crosslink --set-crosslink-url http://127.0.0.1:8232
```

### Step 2: Verify

```bash
# Check which backend is active
nozy config --show-backend

# Test connection
nozy test-zebra
```

### Step 3: Use It!

All commands now use Crosslink:
```bash
nozy sync      # Syncs with Crosslink
nozy balance   # Checks balance via Crosslink
nozy send ...  # Sends via Crosslink
```

## 🔄 Switch Back to Zebra

```bash
nozy config --use-zebra
```

## 📋 All Backend Commands

```bash
# View current backend
nozy config --show-backend

# Switch to Crosslink
nozy config --use-crosslink

# Set Crosslink URL
nozy config --set-crosslink-url http://127.0.0.1:8232

# Switch to Zebra (standard)
nozy config --use-zebra

# Test current backend
nozy test-zebra
```

## ⚠️ Important

- **Crosslink is experimental** - Use testnet only
- **Don't use mainnet funds** with Crosslink
- **Standard Zebra** is safe for production

## ✅ That's It!

Your wallet now uses Crosslink! All operations (`sync`, `send`, `balance`) automatically use the configured backend.

See `CROSSLINK_SETUP_GUIDE.md` for detailed setup instructions.

