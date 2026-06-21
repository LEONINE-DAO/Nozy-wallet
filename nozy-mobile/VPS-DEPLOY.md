# NozyWallet Mobile API — VPS deploy guide

Deploy a **dedicated VPS** for `nozywallet-api` so mobile users can connect **away from home** (mobile data, any Wi‑Fi).

| Host | Purpose |
|------|---------|
| **`nozywallet.leoninedao.org`** (new VPS) | Mobile app **API URL** — runs `nozywallet-api` |
| **`zec.leoninedao.org`** (existing) | **Zebra node** + uptime monitoring only — not the mobile API |

```text
Phone  →  https://nozywallet.leoninedao.org  →  nozywallet-api (VPS)
                                                      ↓
                                            https://zec.leoninedao.org:443 (Zebra)
```

---

## 1. DNS & VPS

1. Create VPS (Ubuntu 22.04+ recommended, 2 GB+ RAM, 20 GB+ disk).
2. Point DNS A record:
   - `nozywallet.leoninedao.org` → VPS public IP
3. Open firewall:
   - **80, 443** (HTTPS via nginx)
   - **Do not** expose port 3000 publicly — nginx proxies to localhost only.

---

## 2. Server setup (Ubuntu)

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y build-essential pkg-config libssl-dev protobuf-compiler git curl nginx certbot python3-certbot-nginx

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

Create service user:

```bash
sudo useradd -r -m -d /opt/nozywallet -s /bin/bash nozywallet
sudo mkdir -p /opt/nozywallet
sudo chown nozywallet:nozywallet /opt/nozywallet
```

---

## 3. Build & install API

As `nozywallet` user:

```bash
sudo -u nozywallet -i
cd /opt/nozywallet
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git repo
cd repo
cargo build --release -p nozywallet-api
cp target/release/nozywallet-api /opt/nozywallet/
exit
```

---

## 4. Wallet config (Zebra = zec.leoninedao.org)

On the VPS, create config so sync uses your **existing node**, not localhost:

```bash
sudo -u nozywallet -i
mkdir -p ~/.local/share/nozy ~/.config/nozy
cat > ~/.config/nozy/config.json << 'EOF'
{
  "zebra_url": "https://zec.leoninedao.org:443",
  "network": "mainnet",
  "theme": "dark",
  "backend": "zebra",
  "protocol": "jsonrpc",
  "privacy_network": {
    "tor_enabled": false,
    "require_privacy_network": false
  }
}
EOF
exit
```

Test Zebra from VPS:

```bash
curl -s -X POST https://zec.leoninedao.org:443 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}'
```

---

## 5. systemd service

```bash
sudo tee /etc/systemd/system/nozywallet-api.service << 'EOF'
[Unit]
Description=NozyWallet API (mobile companion)
After=network.target

[Service]
Type=simple
User=nozywallet
Group=nozywallet
WorkingDirectory=/opt/nozywallet
ExecStart=/opt/nozywallet/nozywallet-api
Restart=always
RestartSec=10

Environment="NOZY_PRODUCTION=true"
Environment="NOZY_HTTP_PORT=3000"
Environment="NOZY_RATE_LIMIT_REQUESTS=50"
Environment="NOZY_RATE_LIMIT_WINDOW=60"
Environment="NOZY_API_KEY=your-long-random-key"

StandardOutput=journal
StandardError=journal
SyslogIdentifier=nozywallet-api

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable nozywallet-api
sudo systemctl start nozywallet-api
sudo systemctl status nozywallet-api
```

Check locally:

```bash
curl -s http://127.0.0.1:3000/health
```

---

## 6. nginx + HTTPS (Let's Encrypt)

**Important:** `/api/sync` can run **many minutes** on first scan. Use a **long read timeout**.

```bash
sudo tee /etc/nginx/sites-available/nozywallet-api << 'EOF'
server {
    listen 80;
    listen [::]:80;
    server_name nozywallet.leoninedao.org;

    location /.well-known/acme-challenge/ {
        root /var/www/html;
    }
    location / {
        return 301 https://$host$request_uri;
    }
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name nozywallet.leoninedao.org;

    ssl_certificate     /etc/letsencrypt/live/nozywallet.leoninedao.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/nozywallet.leoninedao.org/privkey.pem;

    proxy_connect_timeout 60s;
    proxy_send_timeout    3600s;
    proxy_read_timeout    3600s;

    location /health {
        proxy_pass http://127.0.0.1:3000/health;
    }

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
EOF

sudo ln -sf /etc/nginx/sites-available/nozywallet-api /etc/nginx/sites-enabled/
sudo nginx -t
sudo certbot --nginx -d nozywallet.leoninedao.org
sudo systemctl reload nginx
```

Public health check:

```bash
curl -s https://nozywallet.leoninedao.org/health
```

---

## 7. Mobile app — what users enter

On **Welcome** (or **Settings**):

| Field | Value |
|-------|--------|
| **API server URL** | `https://nozywallet.leoninedao.org` |
| **API key** | Same value as `NOZY_API_KEY` on the VPS |

- **Emulator / home PC:** URL `http://10.0.2.2:3000`, leave API key blank (unless local API uses a key).
- **Phone on mobile data:** HTTPS URL + API key above.

They do **not** put `zec.leoninedao.org` in the API URL field.

---

## 8. Verify end-to-end

1. VPS: `curl https://nozywallet.leoninedao.org/health` → `"status":"ok"`
2. Phone: Welcome → Save & connect → **API connected**
3. Create or restore wallet
4. Dashboard → **Sync wallet** (first sync may take 5–30+ minutes)
5. Balance updates after sync

From your PC:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\verify-operator-stack.ps1 -OperatorHost zec.leoninedao.org
```

(confirms Zebra side; separate from mobile API host)

---

## 9. Security notes (read before going public)

| Topic | Guidance |
|-------|----------|
| **Wallet data** | Lives on the **API VPS** (`~/.local/share/nozy` for the `nozywallet` user). This is a **hosted companion** model — users trust your server. |
| **API key** | Set `NOZY_API_KEY` on the VPS; users enter the same key in the app (sent as `X-API-Key` on every request). |
| **HTTPS** | Required for real-world mobile use. |
| **Backups** | Back up wallet data dir if you host user wallets. |
| **Single tenant** | Simplest: one VPS = one wallet (you or one beta user). Multi-user needs isolation + auth (future). |

---

## 10. Architecture summary

```text
zec.leoninedao.org     →  Zebra + uptime (operator stack)
nozywallet.leoninedao.org  →  nozywallet-api only (mobile API URL)
User phone             →  HTTPS to nozywallet VPS → sync via zec.leoninedao.org
```

---

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Mobile **API offline** | Check nginx, cert, `systemctl status nozywallet-api` |
| Sync **502 / timeout** | Increase nginx `proxy_read_timeout`; check Zebra reachable from VPS |
| Sync **ZEBRA_RPC_UNREACHABLE** | Fix `zebra_url` in `~/.config/nozy/config.json`; test curl to zec.leoninedao.org |
| **503 on sync** | Zebra down or privacy/Tor misconfig — use direct VPS Zebra URL, no Tor on server |

Logs:

```bash
sudo journalctl -u nozywallet-api -f
```

---

See also: `api-server/DEPLOYMENT_GUIDE.md`, `api-server/PUBLIC_API_SETUP.md`
