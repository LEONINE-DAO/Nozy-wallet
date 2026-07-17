# Zakura ↔ NozyWallet connectivity

**Status:** Operator guide — July 2026  
**Audience:** NozyWallet users who run their own [Zakura](https://zakura.com/) node  
**Stack:** Zakurad JSON-RPC + lightwalletd + NozyWallet (CLI, desktop, api-server, mobile)

NozyWallet does **not** bundle Zakura. This guide is for users who install and operate `zakurad` themselves.

**Related:**

| Doc | Role |
|-----|------|
| [`ZEBRAD_NOZYWALLET_CONNECTIVITY.md`](ZEBRAD_NOZYWALLET_CONNECTIVITY.md) | Same wallet wiring for Zebrad (config keys are shared) |
| [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md) | Witness derivation stays in the wallet |
| [Zakura lightwalletd book](https://github.com/zakura-core/zakura/blob/v1.0.0/book/src/user/lightwalletd.md) | Upstream node + LWD setup |

---

## Executive summary

1. **Zakura is a Zebra fork** — NozyWallet uses the same JSON-RPC surface (`z_gettreestate`, `sendrawtransaction`, `getblock`, …). Point `zebra_url` at Zakura’s RPC port.
2. **Do not use zcashd compat mode** for NozyWallet — that path is for legacy `zcashd` wallet RPC (exchanges). Nozy keeps keys locally and speaks Zebra-family RPC.
3. **Run lightwalletd against Zakura** for compact sync — same as Zebrad. Disable Zakura RPC cookie auth when lightwalletd connects (or use Zakura’s `docker-compose.lwd.yml`).
4. **Cookie auth:** Zakura enables RPC cookies by default. Nozy reads `~/.cache/zakura/.cookie` automatically, or set `ZAKURA_RPC_COOKIE` / disable auth for local dev.

---

## What NozyWallet needs from the node

| RPC / service | NozyWallet use |
|---------------|----------------|
| `getblockcount` | Tip, sync progress, send readiness |
| `z_gettreestate` | Orchard/Ironwood witness checkpoints |
| `z_getsubtreesbyindex` | Subtree sync (when used) |
| `getblock` (verbose) | RPC scan path |
| `sendrawtransaction` | Broadcast |
| **lightwalletd** gRPC | Compact-block sync (`zeaking::lwd`) |

Orchard witnesses are **not** served by the node — derived locally in the wallet.

---

## Quick setup (mainnet, Linux / WSL)

### 1. Install and configure Zakura

```bash
zakurad generate -o ~/.config/zakurad.toml
```

Edit `~/.config/zakurad.toml`:

```toml
[rpc]
listen_addr = "127.0.0.1:8232"
enable_cookie_auth = false   # required for lightwalletd; OK for local-only dev
```

Start sync:

```bash
zakurad start
```

**Faster bootstrap:** use [Zakura snapshots](https://zakura.com/snapshots/) instead of full P2P sync.

### 2. lightwalletd

Follow [Zakura’s lightwalletd guide](https://github.com/zakura-core/zakura/blob/v1.0.0/book/src/user/lightwalletd.md). Minimal flow:

```bash
git clone https://github.com/zcash/lightwalletd
cd lightwalletd && make && make install
touch ~/.config/zcash.conf   # empty OK when RPC is 127.0.0.1:8232
lightwalletd --zcash-conf-path ~/.config/zcash.conf \
  --data-dir ~/.cache/lightwalletd --log-file /dev/stdout
```

Default gRPC: `127.0.0.1:9067`

**Docker:** `docker compose -f docker/docker-compose.lwd.yml up` from the [zakura-core/zakura](https://github.com/zakura-core/zakura) repo (RPC + cookie settings preconfigured).

### 3. Point NozyWallet at Zakura

```bash
nozy config --set-zebra-url http://127.0.0.1:8232
export LIGHTWALLETD_GRPC=http://127.0.0.1:9067   # api-server / sync
nozy test-zebra
nozy sync --to-tip
```

`nozy test-zebra` detects **Zakura** from `getnetworkinfo` subversion and probes `z_gettreestate` at tip.

---

## RPC authentication (cookie)

When `enable_cookie_auth = true` (Zakura default):

| Method | Usage |
|--------|--------|
| Auto cookie file | `%USERPROFILE%\.cache\zakura\.cookie` (Windows) or `~/.cache/zakura/.cookie` (Linux) |
| Env inline | `ZAKURA_RPC_COOKIE=user:password` (contents of `.cookie` file) |
| Env path | `ZAKURA_RPC_COOKIE_PATH=/path/to/.cookie` |

Zebrad cookie env vars (`ZEBRA_RPC_COOKIE`, etc.) still work for Zebrad-only setups.

For **lightwalletd**, Zakura docs require `enable_cookie_auth = false` because lightwalletd does not send cookies.

---

## Windows + WSL

Same pattern as Zebrad:

1. Run `zakurad` + `lightwalletd` inside WSL.
2. From Windows PowerShell:

```powershell
wsl hostname -I   # note WSL IP
nozy config --set-zebra-url http://<wsl-ip>:8232
$env:LIGHTWALLETD_GRPC = "http://<wsl-ip>:9067"
nozy test-zebra
```

Smoke script: `.\scripts\test-zakura-nozywallet.ps1`

---

## Verify connectivity

| Check | Command |
|-------|---------|
| RPC tip | `nozy test-zebra` — should show **Zakura** subversion |
| Treestate probe | Included in `test-zebra` (`z_gettreestate` at tip) |
| Compact sync | `nozy sync --to-tip` or desktop **Sync** |
| Full smoke (Windows) | `.\scripts\test-zakura-nozywallet.ps1` |

---

## Zakura vs zcashd compat (do not mix up)

| Mode | For | NozyWallet? |
|------|-----|-------------|
| **Zakurad JSON-RPC** (`listen_addr = :8232`) | Wallets, lightwalletd, explorers | **Yes** |
| **zcashd compat** (`zakurad start --zcashd-compat`) | Legacy `zcashd` wallet RPC | **No** |

---

## Troubleshooting

| Problem | What to try |
|---------|-------------|
| Connection refused on `:8232` | Enable `[rpc] listen_addr` in `zakurad.toml`; RPC is **off** by default in Docker unless configured |
| 401 / auth errors | Set `ZAKURA_RPC_COOKIE` or disable `enable_cookie_auth` locally |
| `test-zebra` OK but sync fails | Is lightwalletd running on `:9067`? Is `LIGHTWALLETD_GRPC` set for api-server? |
| Treestate probe fails | Node not synced to tip; or RPC missing Orchard/Ironwood treestate (upgrade Zakura) |
| Send fails after “synced” | Witness lag — rescan; see [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md) |

---

## Status and testing

Zakura 1.0.0 is a **new** node release ([announcement](https://zakura.com/announcements/introducing-zakura/)). NozyWallet targets the shared Zebra-family RPC contract; use `nozy test-zebra` (detects Zakura + probes treestate) before sync or send.
