# ZIP 318 twin balance recovery

**Symptom:** Available balance looks **~half** of what you expect after Ironwood prep (`ironwood split`) or incremental desktop sync — e.g. **0.0445 ZEC** on screen but CLI shows **~0.02 ZEC**, or the reverse after a partial rescan.

**Cause:** ZIP 318 splits can create **equal-value twin notes** in one transaction. An older note cache or a bad incremental merge could keep only one twin. Funds are still on-chain; the wallet is under-counting unspent notes.

**Related fix (merged):** twin nullifier merge in `notes.rs` / `wallet_sync.rs` (PR #198). Existing caches may still need a one-time rebuild.

---

## 1. Audit the cache

```bash
nozy notes-doctor
```

| Output | Meaning |
|--------|---------|
| `Equal-value unspent groups: 0` after splits | Possible collapsed twin — rebuild (step 2) |
| `Equal-value unspent groups: 1 (+1 twin/extra notes)` | Twins present — balance should be full after sync to tip |
| `No collapsed-twin hazard detected` | Nullifier integrity OK |

---

## 2. Rebuild from before the splits

Use a start height **at or before** your first ZIP 318 split on this profile. For mainnet Ironwood prep on profile `5a5c81bda1fa6343`, splits landed near height **3,428,651** — a safe operator window:

```bash
# Close desktop/extension first so they do not rewrite notes mid-scan
nozy sync --start-height 3428000 --to-tip
nozy balance
nozy notes-doctor
```

Full `--to-tip` rescan of ~35k blocks can take **~60–90 minutes** and prints little progress; that is normal.

If you know your split block from Activity or `ironwood` logs, you can use that height instead of `3428000`.

---

## 3. Refresh UI surfaces

After CLI balance looks correct:

1. Open **Nozy Desktop** → **Sync to tip**
2. Extension / companion: unlock and sync so Activity matches CLI

---

## 4. When balance is still low

- Check **pending sends** (`nozy balance` — pending lock reduces available).
- Confirm you are on the **same profile** (mainnet Wallet 1 vs testnet / `_inactive`).
- Phantom **testnet-format** rows in `sent_transactions.json` (`utest…` on mainnet) do not explain large gaps; ignore or mark expired per ops runbook.

---

## Prevention (incremental sync)

Desktop and the companion API sync in **bounded height chunks**. This PR hardens that path so chunks:

- do not rewind into a full-history rescan when `end_height` is set,
- stay **forward-only** from `last_scan_height`,
- checkpoint witness catch-up per chunk instead of always targeting tip in one pass.

Operators should still run **`nozy sync --to-tip`** after Ironwood splits or if `notes-doctor` warns about equal-value groups.

**Full case write-up:** [`ZIP318_TWIN_BALANCE_CASE_BREAKDOWN.md`](ZIP318_TWIN_BALANCE_CASE_BREAKDOWN.md)

---

**See also:** [`MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md`](MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md) · [`IRONWOOD_WALLET_READINESS.md`](IRONWOOD_WALLET_READINESS.md)
