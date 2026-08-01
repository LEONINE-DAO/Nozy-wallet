# Dynamic-fee pilot A′ soak + metrics

**Status:** A′2 counters shipped in-repo; **testnet soak is operator-run**.  
**Blocked:** Phase **A2** (Zeaking shared `fee_policy` / observatory) until Shielded Labs policy move — do **not** start Zeaking fee work here.

## What shipped (code)

| Counter | When incremented | Storage |
|---------|------------------|---------|
| `priority_sends` | Pilot send saved to history | `pilot_metrics.json` in wallet datadir |
| `speed_ups` | Speed-up rebuild broadcast | same |
| `expired_unmined` | Pending → Expired batch | same |
| `speed_up_confirmed` | Speed-up child tx confirms | same |

- Core: `src/pilot_metrics.rs`
- Wired: `cli_helpers` send, `tx_lifecycle::speed_up_transaction`, expiry + confirm paths in `transaction_history`
- Readout: `GET /api/pilot/metrics` (counts only — no amounts/addresses/txids)

## Soak procedure (testnet preferred)

1. Point wallet at testnet Zebrad + LWD; fund with dust.
2. Send ≥5 priority pilot txs; note tip vs 5-block expiry.
3. Let ≥1 expire unmined → confirm `expired_unmined` increments and notes unlock.
4. Speed-up an expired tx → `speed_ups` +1; after mine → `speed_up_confirmed` +1.
5. Snapshot:

```bash
curl -s http://127.0.0.1:3000/api/pilot/metrics
```

6. Log results in the RFC decisions table (`docs/rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md` §10) and paste a redacted JSON into the grant evidence pack (counts only).

## Explicit non-goals this slice

- Moving `fee_policy` into Zeaking (**A2**)
- Observatory full-tx fee indexer
- Telemetry that leaves the machine
