# Desktop pre-release debug session (June 2026)

**Surface:** `desktop` (+ shared `core` where noted)  
**Context:** Pre-release smoke testing of the Tauri desktop wallet on Windows with local Zebrad.  
**Error codes:** Mapped in [`desktop-client/src/utils/errors.ts`](../../../desktop-client/src/utils/errors.ts).

This document records symptoms, runtime evidence, root causes, fixes, and operator setup discovered during the session. Use it for onboarding, support, and follow-up PRs.

---

## Quick reference — desktop error codes

| Code | User message (typical) | Common cause |
|------|------------------------|--------------|
| **NET_001** | Cannot connect to node | Zebra not running, RPC disabled, wrong port/URL, or wrong process on port |
| **NET_002** | Request timed out | Slow or overloaded node |
| **NET_004** | Node unavailable | Zebra reachable but RPC errors |
| **SYNC_001** | Sync failed | Node down or scan/witness path failed |
| **SEND_001** | Insufficient balance | Wrong balance after send, or genuinely low funds |
| **RUNTIME_001** | Use desktop window | Opened Vite URL (`localhost:5173`) in browser instead of Tauri app |

---

## Issue 1 — NET_001: “Cannot connect to node”

### Symptoms

- Toast or banner: `Cannot connect to node. Check the node URL and that it's running. (NET_001)`
- Often seen at `http://localhost:5173` or in Settings → Network → Test Connection
- `.\nozy.exe test-zebra` fails with “Failed to connect to local Zebra node at http://127.0.0.1:8232”

### Runtime evidence

| Check | Result |
|-------|--------|
| TCP to `127.0.0.1:8232` | Connects (`TcpTestSucceeded: True`) |
| JSON-RPC `getblockcount` | **Fails** — “connection was closed unexpectedly” |
| Process on port 8232 | **`svchost.exe` (IP Helper / `iphlpsvc`)**, not `zebrad` |
| `Get-Process zebrad` | **Empty** (node not running) |
| `zebrad start` (no config) | Log: `No config file provided, using default configuration` → **`rpc.listen_addr: None`** (RPC **disabled**) |

### Root causes (three separate problems)

1. **Zebra was not running** — wallet correctly reported unreachable RPC.
2. **Wrong config path on Windows** — RPC settings were in `%APPDATA%\Roaming\zebrad.toml`, but Zebrad reads **`%LOCALAPPDATA%\zebrad.toml`** (`C:\Users\<you>\AppData\Local\zebrad.toml`). Starting `zebrad` without `-c` ignored the Roaming file.
3. **Port 8232 occupied by Windows** — IP Helper listens on `0.0.0.0:8232`. Even with RPC enabled, Zebrad cannot bind there on this machine.

### Fix (operator setup — verified)

1. Copy/create config at **`%LOCALAPPDATA%\zebrad.toml`** with RPC enabled, e.g.:

   ```toml
   [rpc]
   listen_addr = "127.0.0.1:18232"
   enable_cookie_auth = false
   cookie_dir = 'C:\Users\<you>\AppData\Local\zebra'
   ```

2. Start node: `zebrad start` (picks up Local config).

3. Point wallet at RPC URL — `%APPDATA%\nozy\nozy\config\config.json`:

   ```json
   "zebra_url": "http://127.0.0.1:18232"
   ```

4. Verify: `.\nozy.exe test-zebra` → block height returned.

### Code / UX notes

- **`RUNTIME_001`**: Opening `localhost:5173` in a browser has no Tauri backend; use the **NozyWallet taskbar window** (`npm run tauri dev` from `desktop-client/`).
- **`errors.ts`**: NET_001 maps to “failed to connect” / “connection refused” strings from the Rust backend.

---

## Issue 2 — Send appears to “stall” on “Checking sync status…”

### Symptoms

- Send button shows loading toast **“Checking sync status…”** for many seconds with no progress.
- User described this as “sending stall” before proving even starts.

### Runtime evidence

| Check | Result |
|-------|--------|
| `Test-NetConnection 127.0.0.1 -Port 9067` | **`open=False`**, probe ~5.5s |
| lightwalletd | **Not running** on default gRPC port 9067 |
| Code path | Every send calls `getSyncStatus()` → `gather_sync_status()` → **`connect_lightwalletd()` with no timeout** |

### Root cause

`src/sync_status.rs` probes lightwalletd on **every** sync-status call. When nothing listens on `:9067`, the gRPC connect blocks until the OS times out — the UI waits on that before send readiness runs.

**Note:** lightwalletd is **optional** for RPC-based desktop sync; the probe should not block send.

### Fix status

| Item | Status |
|------|--------|
| **Identified** | Yes — port 9067 closed, ~5s+ hang |
| **Fix** | `tokio::time::timeout(3s)` around `connect_lightwalletd` in `gather_sync_status` |
| **In repo** | **Fixed** — `src/sync_status.rs` |

### Workaround

- Start lightwalletd on `127.0.0.1:9067`, **or**
- Wait for timeout fix to land, **or**
- Ensure send path does not depend on LWD for RPC-only wallets.

---

## Issue 3 — Send blocked while Zebra node is still syncing

### Symptoms

- Zebra connects (`test-zebra` OK) but send still fails or behaves oddly.
- Wallet `last_scan_height` **greater than** current `zebra_tip` (node catching up after restart).

### Runtime evidence (example from session)

| Field | Value |
|-------|-------|
| Wallet `last_scan_height` | ~3,391,926 |
| Zebra tip (after restart) | ~3,364,603 |
| Gap | Node **~27k blocks behind** wallet scan checkpoint |

### Root cause

Send readiness treated “scan gap = 0” when `tip.saturating_sub(last)` underflows (last > tip). User could appear “synced” while the **node** cannot serve blocks/witnesses at the wallet’s scanned height.

### Fix (in repo)

- **`desktop-client/src/lib/syncHelpers.ts`** — reject send when `last_scan_height > zebra_tip` with a clear message.
- **`desktop-client/src-tauri/src/commands/status.rs`** — banner text and `scan_gap_blocks` handling when node tip is behind wallet scan.

---

## Issue 4 — Balance wrong after send

### Symptoms

- Successful send; balance drops by **full wallet value** instead of send amount + fee.
- Example: ~0.05 ZEC note incorrectly marked spent locally.

### Root cause (session diagnosis)

Desktop `send_transaction` marked **all** spendable notes as spent after broadcast, but Orchard sends consume **one** note (change stays in wallet).

### Fix

- **`select_single_spend_note`** picks the one note covering amount + fee; only that note is marked spent.
- **`mark_wallet_notes_spent_from_spendables(std::slice::from_ref(spent_note))`** after broadcast.
- Surfaces updated: `desktop-client/src-tauri/src/commands/transaction.rs`, `src/cli_helpers.rs`, `src/tx_lifecycle.rs`, `api-server/src/handlers.rs`.

### Fix status

| Item | Status |
|------|--------|
| **In repo** | **Fixed** |

---

## Issue 5 — Recent Activity / History sort wrong

### Symptoms

- Latest **send** missing from top of Recent Activity.
- Old **receive** sorted above recent send.

### Root cause

- Received entries lacked stable timestamps; reload used `Utc::now()`.
- Sort used string dates instead of **block height**.

### Fix (in repo)

- **`src/transaction_history.rs`** — sort views by `block_height`, then `created_at`, then `txid`.
- **`desktop-client/src/lib/history.ts`** — `sortHistoryNewestFirst()` by `blockHeight`.

---

## Issue 6 — Received transactions show date “1969”

### Symptoms

- History detail shows epoch / 1969 for received deposits.

### Root cause

`DateTime::UNIX_EPOCH` placeholder exported as `broadcast_at` in JSON; UI parsed it as a real date.

### Fix (in repo)

- Omit invalid epoch timestamps in history JSON.
- **`formatHistoryDate()`** / **`formatHistoryDetailDate()`** — fallback to **“Block N”** when date is before Zcash mainnet era.

---

## Issue 7 — Witness lag / sync-before-send (shared with api-server)

### Symptoms

- Status looks caught up on scan height but send fails: “Orchard witness is N blocks behind…”

### Root cause

See **[BUG-2026-013](2026-06-witness-sync-status-mismatch.md)** — scan checkpoint advanced without refreshing witnesses on cached notes.

### Fix (in repo)

- **`src/wallet_sync.rs`** — `refresh_cached_witnesses_to_tip` when scan at tip.
- Desktop **`get_sync_status`** exposes `witness_lag_blocks`, `witness_fresh_for_send`.
- **`SyncStatusBanner`** + **`useWalletAutoSync`** — background sync while unlocked.

---

## Operator checklist — Windows + local Zebrad

```powershell
# 1. Config must live here (not Roaming only)
notepad $env:LOCALAPPDATA\zebrad.toml

# 2. Start node (keep terminal open)
zebrad start

# 3. Verify RPC (adjust port if not 18232)
cd C:\Users\User\NozyWallet
.\nozy.exe test-zebra

# 4. Run desktop app (not browser tab)
cd desktop-client
npm run tauri dev
# Use the NozyWallet window from the taskbar — not http://localhost:5173
```

### Wallet config paths

| File | Purpose |
|------|---------|
| `%APPDATA%\nozy\nozy\config\config.json` | `zebra_url`, `last_scan_height`, privacy network |
| `%LOCALAPPDATA%\zebrad.toml` | Zebrad RPC listen address (**authoritative on Windows**) |
| `%LOCALAPPDATA%\zebra\` | Chain state / cookie dir |

---

## Files touched during session (reference)

| Area | Files |
|------|-------|
| Network / status | `src/sync_status.rs`, `desktop-client/src-tauri/src/commands/status.rs`, `desktop-client/src/lib/syncHelpers.ts` |
| Send | `desktop-client/src/components/SendForm.tsx`, `desktop-client/src-tauri/src/commands/transaction.rs` |
| History | `src/transaction_history.rs`, `desktop-client/src/lib/history.ts`, `History.tsx`, `RecentActivityPanel.tsx` |
| Auto-sync UX | `desktop-client/src/hooks/useWalletAutoSync.ts`, `SyncStatusBanner.tsx`, `AuthenticatedLayout.tsx` |
| Errors | `desktop-client/src/utils/errors.ts`, `desktop-client/src/lib/debugRuntime.ts` |
| Witness / sync (core) | `src/wallet_sync.rs`, `src/send_readiness.rs` |

---

## Follow-up before release

- [x] Land **lightwalletd connect timeout** in `gather_sync_status` (Issue 2).
- [x] Confirm **single-note mark-spent** in desktop `send_transaction` (Issue 4).
- [ ] Expand **`errors.ts`** with RUNTIME_001 / clearer NET_001 when Zebra RPC disabled.
- [ ] Update operator docs if default RPC port differs from `8232` on Windows.

---

## Related registry entries

- [BUG-2026-013](2026-06-witness-sync-status-mismatch.md) — witness lag vs scan height
- Operator connectivity: [`../../reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md`](../../reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md)
- [`desktop-client/README.md`](../../../desktop-client/README.md) — Troubleshooting (Windows Zebrad section)
