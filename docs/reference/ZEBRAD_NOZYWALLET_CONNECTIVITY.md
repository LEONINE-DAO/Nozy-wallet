# Zebrad ↔ NozyWallet connectivity (paper & lecture)

**Status:** Operator guide — June 2026  
**Audience:** Lectures, white paper appendix, facilitators, Windows + WSL operators  
**Stack:** Zebrad JSON-RPC + NozyWallet (CLI, desktop, api-server)

**Related:**

| Doc | Role |
|-----|------|
| [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md) | Mainnet send timings assume a **live** Zebrad RPC |
| [`../issues/bugs/2026-06-desktop-pre-release-debug-session.md`](../issues/bugs/2026-06-desktop-pre-release-debug-session.md) | NET_001 RCA, Windows `zebrad.toml` path, port 8232 vs 18232 |
| [`../../ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md) | What Zebrad does **not** provide (Orchard witnesses) |
| [`../../book/src/advanced/zebra-node.md`](../../book/src/advanced/zebra-node.md) | User-facing book chapter (summary + link here) |

---

## Executive summary (lecture slide)

1. **NozyWallet is not a full node** — it needs a reachable **Zebrad JSON-RPC** endpoint for chain tip, broadcast, and treestate.
2. **Connectivity is config-driven** — `zebra_url` in `config.json`, overridable by `ZEBRA_RPC_URL`.
3. **“Connected” means `getblockcount` works** — run `nozy test-zebra` before sync or send.
4. **Windows + WSL is a common layout** — Zebrad in Linux, wallet on Windows; use the **WSL IP**, not `127.0.0.1`, unless localhost forwarding is explicitly set up.
5. **Silent misconfig is the #1 trap** — bad JSON (e.g. UTF-8 BOM from PowerShell) or wrong URL makes Nozy fall back to defaults and look “broken” while a good node runs elsewhere.

---

## What Zebrad provides (and what it does not)

| Zebrad RPC | NozyWallet use |
|------------|----------------|
| `getblockcount` / tip | Sync progress, send readiness |
| `sendrawtransaction` | Broadcast shielded txs |
| Treestate / block fetch | **Local** Orchard witness derivation (wallet-side) |
| Mempool / network info | Diagnostics |

NozyWallet **does not** embed consensus. Orchard witnesses are derived in the wallet ([`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md)). If RPC is down, sync and send stop even when the UI still shows an old balance.

---

## Configuration paths

| Platform | Wallet config | Typical contents |
|----------|---------------|------------------|
| **Windows** | `%APPDATA%\nozy\nozy\config\config.json` | `zebra_url`, `last_scan_height`, network |
| **Linux / macOS** | `~/.config/nozy/config.json` (via `directories` crate) | same |
| **Override** | Env `ZEBRA_RPC_URL` | Wins over file for this process |

Example `config.json`:

```json
{
  "zebra_url": "http://172.20.199.206:8232",
  "last_scan_height": 3395373,
  "privacy_network": "mainnet"
}
```

Set URL from CLI:

```bash
nozy config --set-zebra-url http://YOUR_HOST:8232
nozy test-zebra
```

**UTF-8 BOM:** Editing `config.json` with PowerShell `Set-Content -Encoding utf8` can prepend a BOM. Older builds failed to parse and silently used `http://127.0.0.1:8232`. Current `load_config()` strips `\u{feff}` before parse (`src/config.rs`). Prefer `utf8NoBOM` or VS Code when hand-editing.

---

## Setup patterns

### A — Zebrad on same machine (Linux / macOS)

1. Enable RPC in `~/.config/zebrad.toml`:

   ```toml
   [rpc]
   listen_addr = "127.0.0.1:8232"
   ```

2. Start: `zebrad start`
3. Wallet: `zebra_url`: `http://127.0.0.1:8232`

### B — Zebrad in WSL, NozyWallet on Windows (recorded operator stack)

1. Start Zebrad **inside WSL** with RPC bound to `0.0.0.0:8232` or the WSL interface (not only WSL-local if Windows must reach it).
2. Get WSL IP (changes after reboot):

   ```powershell
   wsl hostname -I
   ```

   Or dot-source once per session:

   ```powershell
   . .\scripts\zebra-wsl-rpc.ps1
   # sets $env:ZEBRA_RPC_URL = http://<wsl-ip>:8232
   ```

3. Persist in wallet config:

   ```powershell
   nozy config --set-zebra-url "http://<wsl-ip>:8232"
   ```

4. Verify from **Windows** (must succeed before trusting the desktop app):

   ```powershell
   .\nozy.exe test-zebra
   ```

### C — Native Windows Zebrad

On some Windows hosts **port 8232 is taken** by `svchost` (IP Helper). Use another port in **`%LOCALAPPDATA%\zebrad.toml`** (authoritative path — not only `%APPDATA%\Roaming\zebrad.toml`):

```toml
[rpc]
listen_addr = "127.0.0.1:18232"
enable_cookie_auth = false
```

Then `"zebra_url": "http://127.0.0.1:18232"`. See [desktop pre-release debug session](../issues/bugs/2026-06-desktop-pre-release-debug-session.md#issue-1--net_001-cannot-connect-to-node).

### D — Remote VPS

```bash
nozy config --use-remote http://your-vps:8232
nozy test-zebra
```

Ensure firewall allows your IP to the RPC port. Treat RPC as **trusted infrastructure** — it sees broadcast metadata.

---

## Verification checklist (5 minutes)

Run in order. Stop at the first failure and fix before continuing.

| Step | Command / action | Pass criteria |
|------|------------------|---------------|
| 1 | `zebrad` running (WSL, Windows, or VPS) | Process up, logs show sync |
| 2 | Raw RPC | `getblockcount` returns increasing integer |
| 3 | `nozy test-zebra` | “Connection successful”, block height printed |
| 4 | Config matches | `nozy config` or read `config.json` — URL is the node you tested |
| 5 | Wallet sync | `nozy sync --to-tip` or desktop **Sync** — `last_scan_height` → tip |
| 6 | Desktop (optional) | Settings → Network → **Test Connection**; Home shows block sync strip advancing |

### One-shot diagnostic script (Windows)

From repo root:

```powershell
.\scripts\test-zebrad-nozywallet.ps1
```

Prints: parsed `config.json`, RPC probes (config URL, localhost, WSL IP), `nozy test-zebra`, whether a Windows `zebrad` process exists.

### Shell / Linux

```bash
./test_zebra_node.sh http://127.0.0.1:8232
# or
curl -s -H 'content-type: application/json' \
  --data-binary '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}' \
  http://127.0.0.1:8232
```

---

## What “healthy” looks like

| Signal | Healthy | Unhealthy |
|--------|---------|-----------|
| `nozy test-zebra` | Block height, network info | NET_001 / connection refused / closed unexpectedly |
| Tip over time | `getblockcount` increases every few minutes | Stuck height → node stalled or wrong host |
| `last_scan_height` vs tip | Equal or within sync gap | Large gap → run sync; don’t send |
| Desktop sync strip | “N blocks behind” → 0 after sync | “Cannot connect to node” |
| Witness lag (desktop status) | ≤ 50 blocks for send | “Witness N blocks behind…” on send |

**Lecture point:** Scan height caught up ≠ send-ready if **witness lag** is high. Nozy exposes `witness_lag_blocks` and blocks send when lag > `MAX_SEND_WITNESS_LAG_BLOCKS` (50).

---

## Common failure modes

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| NET_001, `127.0.0.1:8232` fails | No Zebrad on Windows; wrong default after config parse failure | Start node or fix `zebra_url`; check BOM/JSON |
| TCP connects but RPC closes | Port owned by non-Zebrad service (e.g. Windows IP Helper on 8232) | Change Zebrad `listen_addr` (e.g. 18232) |
| CLI works, desktop fails | Opened Vite URL in browser (`localhost:5173`) not Tauri app | Use taskbar **NozyWallet** window (`RUNTIME_001`) |
| `test-zebra` OK, sync stuck | lightwalletd / compact path separate from Zebrad | Check `lightwalletd_url`, LWD tip in sync status |
| Sync “success” but send blocked | Witness lag | Sync to tip; wait for witness refresh |
| Wrong chain / height 0 | Testnet vs mainnet mismatch | Align `privacy_network` and Zebrad network |

---

## Lecture diagram (wallet ↔ node)

```text
┌─────────────────────┐         JSON-RPC          ┌──────────────────┐
│   NozyWallet        │  getblockcount, broadcast │   Zebrad         │
│   CLI / Desktop     │ ─────────────────────────►│   (full node)    │
│   api-server        │  treestate / blocks       │                  │
└─────────┬───────────┘                           └────────┬─────────┘
          │                                                │
          │  compact blocks (optional)                     │ P2P
          ▼                                                ▼
┌─────────────────────┐                           ┌──────────────────┐
│ lightwalletd        │                           │ Zcash network    │
│ (local or remote)   │                           │                  │
└─────────────────────┘                           └──────────────────┘

Verify layer 1 first: NozyWallet ──RPC──► Zebrad  (nozy test-zebra)
Then layer 2: sync / witness / send readiness
```

---

## Evidence commands (for papers)

Record these in appendices when claiming “mainnet-ready stack”:

```powershell
# Config in use
Get-Content $env:APPDATA\nozy\nozy\config\config.json

# Nozy view
.\nozy.exe test-zebra
.\nozy.exe sync --to-tip

# Raw tip (replace URL)
$body = '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}'
Invoke-RestMethod -Uri "http://YOUR_ZEBRA:8232" -Method POST -ContentType "application/json" -Body $body
```

Example recorded environment: Windows host + WSL Zebrad at `http://172.20.199.206:8232` — see [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md). **WSL IP is not stable**; always document the IP or URL used at test time.

---

## Desktop operator notes

- **Settings → Network:** edit `zebra_url`, **Test Connection** (calls `test_zebra_connection`).
- **Home → block sync panel:** shows blocks behind tip; stall detection if tip stops advancing.
- **Sync button:** runs catch-up until gap closes; “synced” should mean scan at tip, not merely “RPC responded once”.
- Run the desktop via `npm run tauri dev` or the installed app — not the browser dev URL.

---

## Maintainer references

| Artifact | Path |
|----------|------|
| Config load + BOM strip | `src/config.rs` |
| `nozy test-zebra` | `src/main.rs` |
| Desktop test RPC | `desktop-client/src-tauri/src/commands/config.rs` |
| Sync status API | `desktop-client/src-tauri/src/commands/status.rs`, `src/sync_status.rs` |
| WSL URL helper | `scripts/zebra-wsl-rpc.ps1` |
| Connectivity smoke test | `scripts/test-zebrad-nozywallet.ps1` |

---

## Suggested white-paper paragraph

> NozyWallet treats Zebrad JSON-RPC as the trust anchor for chain tip and transaction broadcast. Operator readiness is verified with `nozy test-zebra` and raw `getblockcount` before any mainnet send evidence is recorded. On Windows development hosts, Zebrad commonly runs in WSL while the wallet runs natively; the wallet must target the WSL interface IP, not localhost, unless explicit port forwarding is configured. Configuration is stored in platform-specific `config.json`; parse failures or UTF-8 BOM prefixes historically caused silent fallback to localhost defaults—a class of bug addressed by BOM-tolerant config loading and documented operator checklists.
