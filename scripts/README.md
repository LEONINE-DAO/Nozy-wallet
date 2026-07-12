# Nozy-wallet helper scripts

**Windows:** **Run Zebrad only in WSL** — do not run a separate native Windows Zebrad for Nozy development. If you **already have a synced Zebrad** in WSL (or elsewhere), skip installing another; just set Nozy’s RPC URL to that node.

PowerShell / bash helpers for **Nozy** (CLI, desktop, `api-server` / zeaking companion, WSL). **Zebrad node** launchers live in your [Zebrad](https://github.com/ZcashFoundation/zebra) checkout (`C:\Zebrad\scripts\start-zebrad-wsl.ps1`) — see Zebrad `scripts/README.md`.

**Full terminal layout:** [`TERMINAL-PLAYBOOK.md`](TERMINAL-PLAYBOOK.md) · **Stack health check:** `.\scripts\show-dev-stack.ps1`

## Script reference

| Script | Purpose |
|--------|---------|
| `zebra-wsl-rpc.ps1` | Windows: **dot-source** to set **`ZEBRA_RPC_URL`** to `http://<WSL-IP>:8232` once per shell. `-Localhost` if RPC is reachable at `127.0.0.1:8232` from Windows. |
| `start-lightwalletd-wsl.ps1` | Windows/WSL: start or check **lightwalletd** on `0.0.0.0:9067` (backend: zebrad `127.0.0.1:8232` inside WSL). `-Status`, `-Stop`. |
| `run-nozy-api.ps1` | Windows: start **`nozywallet-api`** with `LIGHTWALLETD_GRPC` → WSL lightwalletd. **`-HttpPort 0`** (default) picks first free port **3000–3100**; **`-HttpPort 3000`** to pin. Set extension companion `baseUrl` if not 3000. |
| `run-nozy.ps1` | Windows: ensure Zebrad in WSL, then `cargo tauri dev` for desktop. |
| `set-wallet-rpc.ps1` | Windows: one RPC URL across CLI/desktop/api (`config --set-zebra-url` + `ZEBRA_RPC_URL`) and print URL for extension Settings. |
| `show-dev-stack.ps1` | Windows: which services should run, port UP/DOWN for zebrad `:8232`, lightwalletd `:9067`, API `:3000`. |
| `zeaking-lwd-smoke.ps1` | Windows: compact-sync smoke test; **`-LiveSync`** hits live lightwalletd (needs 9067 UP). |
| `extension-smoke.ps1` | Windows: automated extension smoke suite (from repo root). |
| `stop-nozy.ps1` | Stop Nozy desktop / optionally Zebrad in WSL. |
| `run-nozy-wsl.sh` | Linux/WSL: same idea as `run-nozy.ps1` (bash). |
| `stop-nozy-wsl.sh` | Stop desktop / optionally Zebrad. |
| `run-nozy-api.sh` | WSL/Linux: start **`nozywallet-api`**. Auto-picks port **3000–3100** or set **`NOZY_HTTP_PORT`**. |
| `build-release.ps1` / `build-release.sh` | Release build helpers. |
| `nozy-lite-bench.ps1` | Measure Nozy Lite CLI size / `--version` / `health` timings for [`docs/reference/NOZY_LITE_BENCHES.md`](../docs/reference/NOZY_LITE_BENCHES.md). |

**Do not** run `run-nozy-api.ps1` from **bash** — use **`bash scripts/run-nozy-api.sh`** instead.

From a **Zebrad** checkout, `bash scripts/run-nozy-api.sh` may forward here if `NOZY_WALLET_ROOT` or auto-search finds this clone.

## Windows quick start (CLI)

```powershell
cd C:\path\to\Nozy-wallet

# 1) Zebrad in WSL (separate Zebrad repo)
powershell -ExecutionPolicy Bypass -File C:\Zebrad\scripts\start-zebrad-wsl.ps1
powershell -ExecutionPolicy Bypass -File C:\Zebrad\scripts\start-zebrad-wsl.ps1 -Status

# 2) lightwalletd in WSL
powershell -ExecutionPolicy Bypass -File .\scripts\start-lightwalletd-wsl.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\start-lightwalletd-wsl.ps1 -Status

# 3) Wallet sync (v2.3.3+: use --to-tip after receives or upgrades)
. .\scripts\zebra-wsl-rpc.ps1 -Force
$env:LIGHTWALLETD_GRPC = "http://$((wsl -d Ubuntu -- hostname -I).Trim().Split()[0]):9067"
$env:NOZY_PLAIN_OUTPUT = "1"
.\target\release\nozy.exe sync --to-tip
# or: cargo run --bin nozy -- sync --to-tip
```

**Extension dev:** leave `run-nozy-api.ps1` running in a second terminal; use `show-dev-stack.ps1` to verify ports.

## Examples

**API server (Windows):**

```powershell
cd C:\path\to\Nozy-wallet\scripts
powershell -ExecutionPolicy Bypass -File .\run-nozy-api.ps1
```

Optional: `-NozyRoot "D:\repos\Nozy-wallet"` if you did not start from inside the clone.

**API server (WSL):**

```bash
cd ~/projects/Nozy-wallet   # or /mnt/c/Users/User/NozyWallet
bash scripts/run-nozy-api.sh
```

## Extension smoke check

From repo root:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\extension-smoke.ps1
```

**Windows note:** WASM build (`wasm32-unknown-unknown`) may require `clang` in `PATH` (e.g. for `secp256k1-sys`). The smoke script fails fast if `clang` is missing.
