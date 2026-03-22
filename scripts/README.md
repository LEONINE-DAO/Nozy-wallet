# Nozy-wallet helper scripts

PowerShell / bash helpers for **Nozy** (desktop, `api-server` / zeaking companion, WSL). **Zebrad node** setup lives in the [Zebrad](https://github.com/ZcashFoundation/zebra) / Zebrad repo (`ZEBRAD_NODE_SETUP.md`, `QUICK_REFERENCE.md`).

| Script | Purpose |
|--------|---------|
| `run-nozy-api.ps1` | Windows: start **`nozywallet-api`** (`api-server`) with `LIGHTWALLETD_GRPC` → WSL lightwalletd. **`NOZY_HTTP_PORT`**: default **`-HttpPort 0`** picks the first free port **3000–3100** (avoids error 10048). Use **`-HttpPort 3000`** to pin. If a non-default port is chosen, set the extension companion `baseUrl` to match. Default `-NozyRoot` = repo root. |
| `run-nozy.ps1` | Windows: ensure Zebrad in WSL, then `cargo tauri dev` for desktop. |
| `stop-nozy.ps1` | Stop Nozy desktop / optionally Zebrad in WSL. |
| `run-nozy-wsl.sh` | Linux/WSL: same idea as `run-nozy.ps1` (bash). |
| `stop-nozy-wsl.sh` | Stop desktop / optionally Zebrad. |
| `run-nozy-api.sh` | **WSL/Linux:** start **`nozywallet-api`** (not PowerShell). Auto-picks a free port **3000–3100** or set **`NOZY_HTTP_PORT`**. |

**Do not** run `run-nozy-api.ps1` from a **bash** shell — use **`bash scripts/run-nozy-api.sh`** instead. The `.ps1` file lives in this repo’s **`scripts/`** (not under Zebrad).

From a **Zebrad** checkout (WSL), you can run **`bash scripts/run-nozy-api.sh`** or **`bash run-nozy-api.sh`** if that repo includes the thin **forwarder** (see Zebrad **`scripts/README.md`**). It delegates to this clone. Otherwise **`cd` here** and run **`bash scripts/run-nozy-api.sh`** as below.

Example (from a clone of this repo on Windows):

```powershell
cd C:\path\to\Nozy-wallet\scripts
powershell -ExecutionPolicy Bypass -File .\run-nozy-api.ps1
```

Optional: `-NozyRoot "D:\repos\Nozy-wallet"` if you did not start from inside the clone.

Example (**WSL**, path under `/mnt/c/...` or native Linux clone):

```bash
cd ~/projects/Nozy-wallet   # or /mnt/c/Users/User/NozyWallet
bash scripts/run-nozy-api.sh
```
