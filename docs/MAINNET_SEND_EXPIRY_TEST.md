# Mainnet test guide — send expiry / proving latency (BUG-2026-011)

**Purpose:** Verify the BUG-2026-011 fix on mainnet: shielded sends succeed on slow VPS/WSL stacks without raising `PILOT_EXPIRY_DELTA_BLOCKS` above 5.

**Related docs:**

| Doc | Contents |
|-----|----------|
| [`issues/bugs/2026-06-send-expiry-before-broadcast.md`](issues/bugs/2026-06-send-expiry-before-broadcast.md) | Bug RCA, fix, paper draft |
| [`reference/PILOT_EXPIRY_PROVING_LATENCY.md`](reference/PILOT_EXPIRY_PROVING_LATENCY.md) | Two clocks, 5 vs 15, Zodl FAQ, design principles |
| [`PILOT_MAINNET_EVIDENCE.md`](PILOT_MAINNET_EVIDENCE.md) | Post-broadcast expiry + speed-up (different failure mode) |

**Commits (master):**

| Commit | Summary |
|--------|---------|
| `42505ff1` | Late tip refresh, rebuild loop, broadcast retry |
| `a72bc6e8` | Revert 15-block bump; keep 5-block pilot expiry |

---

## What you are testing

| Pass | Fail (pre-fix) |
|------|----------------|
| `success: true` + TXID from send | `Failed to broadcast… greater than its expiry Height… (code: -25)` |
| Optional rebuild logs on slow hosts | Balance unchanged; tx never in mempool |
| History `expiry_height` matches on-chain formula | — |

---

## Prerequisites

- **Mainnet Zebrad** synced with JSON-RPC reachable (local or VPS).
- **lightwalletd** (or Zaino) for compact sync if using api-server sync path.
- Wallet with **small mainnet balance** (≥ amount + ~0.0001 ZEC ZIP-317 fee).
- **Second wallet** `u1` recipient (Zodl, another Nozy wallet, etc.) — recipient does not affect prove time.
- Fixed build from `master` at or after `a72bc6e8`.

```powershell
cd C:\Users\User\NozyWallet
git pull origin master
cargo build --release
cargo build --release -p nozywallet-api
```

---

## Path A — api-server (Gilmore-style)

### 1. Start stack

**Terminal 1 — Zebrad** (mainnet, port 8232)

**Terminal 2 — API**

```powershell
cd C:\Users\User\NozyWallet
.\RUN-API.bat
# or: target\release\nozywallet-api.exe
```

### 2. Configure Zebrad URL

```powershell
curl -X POST http://localhost:3000/api/config/zebra-url `
  -H "Content-Type: application/json" `
  -d '{"zebra_url": "http://127.0.0.1:8232"}'

curl http://localhost:3000/api/config/test-zebra
```

For remote VPS Zebrad, use your public hostname (see [`api-server/README.md`](../api-server/README.md) trusted URL policy).

### 3. Wallet + sync

```powershell
# Create or restore once, then:
curl -X POST http://localhost:3000/api/sync `
  -H "Content-Type: application/json" `
  -d '{"password": "YOUR_PASSWORD"}'

curl http://localhost:3000/api/balance
```

Repeat sync until balance reflects spendable Orchard funds.

### 4. Send (dust amount)

```powershell
curl -X POST http://localhost:3000/api/transaction/send `
  -H "Content-Type: application/json" `
  -d '{
    "recipient": "u1YOUR_RECIPIENT_UA",
    "amount": 0.0001,
    "memo": "BUG-2026-011 mainnet test",
    "password": "YOUR_PASSWORD"
  }'
```

### 5. Check history

```powershell
curl http://localhost:3000/api/transaction/history
```

---

## Path B — CLI

```powershell
$env:ZEBRA_RPC_URL = "http://127.0.0.1:8232"
cargo run --release --bin nozy -- sync --to-tip
cargo run --release --bin nozy -- balance
cargo run --release --bin nozy -- send `
  --recipient "u1YOUR_RECIPIENT_UA" `
  --amount 0.0001 `
  --memo "BUG-2026-011 mainnet test"
```

---

## Log patterns to record

**Success:**

```text
✅ Orchard bundle built successfully!
   Expiry height: 33xxxxx
✅ Transaction broadcast successfully!
```

**Fix recovering (normal on slow VPS — still a pass):**

```text
⚠️  Orchard proof outran pilot expiry; rebuilding (2/3)
⚠️  Broadcast hit pilot expiry; rebuilding send (2/3)
```

**Pre-fix failure (must NOT appear on fixed build):**

```text
Failed to broadcast transaction: ... greater than its expiry Height(...) (code: -25)
```

Save full terminal or api-server stdout for [`PILOT_MAINNET_EVIDENCE.md`](PILOT_MAINNET_EVIDENCE.md)-style evidence.

---

## On-chain verification

```powershell
curl -s -X POST http://127.0.0.1:8232 `
  -H "Content-Type: application/json" `
  -d '{"jsonrpc":"2.0","method":"getrawtransaction","params":["TXID_HERE",1],"id":1}'
```

Confirm:

- Transaction exists (confirmed or in mempool).
- History `expiry_height` ≈ `(tip_at_encode + 1) + 5`.

---

## Stress scenarios (optional)

| Scenario | Why |
|----------|-----|
| **VPS + api-server** | Replays Gilmore’s environment |
| **Witness behind tip** | Send when `last_scan_height` is a few blocks behind — witness catch-up adds RPC time |
| **Cold first prove** | First send after process restart (ProvingKey build) |
| **Fast local sanity** | Same send on laptop Zebrad — should succeed, often no rebuild logs |

Operator stack smoke test:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\verify-operator-stack.ps1 `
  -OperatorHost YOUR_HOST -RunNozyStatus
```

---

## Post-broadcast pilot check (separate test)

After a **successful** broadcast with standard fee:

1. Wait ~6 minutes (~5 blocks) without confirmation.
2. Confirm history shows **Expired** (or run speed-up flow).
3. `POST /api/transaction/speed-up` or CLI/desktop **Speed up** at ×4 fee.

This validates the **mempool expiry clock** still works at 5 blocks. See [`PILOT_MAINNET_EVIDENCE.md`](PILOT_MAINNET_EVIDENCE.md).

---

## Evidence template (paste into bug doc or forum)

```markdown
### BUG-2026-011 mainnet verification — YYYY-MM-DD

| Field | Value |
|-------|--------|
| Environment | (local / VPS / WSL) |
| Surface | (api-server / CLI) |
| Zebrad | (version / host) |
| Amount | 0.0001 ZEC |
| TXID | |
| Expiry height (history) | |
| Rebuild logs? | yes / no |
| Broadcast | success / -25 |
| getrawtransaction | found / not found |
```

---

## Safety

- Use **dust amounts** for regression tests.
- Double-check recipient `u1` — mainnet sends are irreversible.
- Insufficient funds / sync errors are unrelated to expiry — sync first.

---

## Troubleshooting

| Symptom | Likely cause |
|---------|----------------|
| `ZEBRA_UNAVAILABLE` | RPC URL, firewall, or Tor policy — see api-server connect fixes |
| `Insufficient funds` | Sync not complete or notes cache empty |
| 50k-block scan on send | Pre BUG-2026-001 build — pull latest master |
| Empty history for receives | Pre BUG-2026-002 build |
| `-25` expiry on **fixed** build | File issue with full logs + tip/expiry heights |
| `Sync to tip before sending` | Witness lag >50 blocks — run `nozy sync --to-tip` |
| Send hangs 2+ min then times out | Pre guard fix: stale wallet triggered rescan — pull latest |
| `could not mark spent notes locally` | Pre v2 `NoteIndex` mark-spent fix — pull latest |

---

## Recorded mainnet results (June 2026)

Full tables, TXIDs, timings, and lecture outline: [`reference/MAINNET_SEND_READINESS_EVIDENCE.md`](reference/MAINNET_SEND_READINESS_EVIDENCE.md).

| Run | Sync | Send time | TXID (prefix) | Result |
|-----|------|-----------|---------------|--------|
| Post-fix stale | — | ~419 s | `e4f0f504…` | PASS (slow witness) |
| Sync 5132 blk + send | 32 s | 198.7 s | `5a03fbd1…` | PASS |
| Stale guard | — | 0.09 s | — | Rejected (expected) |
| Sync 54 blk + send | 0.3 s | 205.8 s | `902cf006…` | PASS + mark-spent OK |

Environment: WSL Zebrad `172.20.199.206:8232`, CLI release, `PILOT_EXPIRY_DELTA_BLOCKS = 5`.
