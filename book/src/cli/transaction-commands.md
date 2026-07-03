# Transaction Commands

## `nozy sync`

Scan the chain for incoming notes and update witnesses.

```bash
# Incremental (default chunk)
nozy sync

# Full catch-up to chain tip
nozy sync --to-tip

# Height range
nozy sync --start-height 3380000 --end-height 3381000

# Override node for this run
nozy sync --to-tip --zebra-url http://host:8232
```

**Before sending:** run `--to-tip` and confirm `nozy status` shows acceptable witness lag.

## `nozy send`

Shielded Orchard send to a unified address (`u1…`).

```bash
nozy send -r u1qr0zfsta9… -a 0.0001
nozy send -r u1… -a 0.0001 --memo "note text"
nozy send -r u1… -a 0.0001 --zebra-url http://host:8232
```

Expect **minutes** for first prove in a session. ZIP-317 fee policy applies automatically in current builds.

## `nozy history`

Local transaction history (sent and detected receives where indexed).

```bash
nozy history
```

## `nozy check-confirmations`

Look up confirmation depth for a TXID.

```bash
nozy check-confirmations -t <txid>
```

## Send readiness guards

The wallet may **reject** send when:

- Witness lag > 50 blocks (`MAX_SEND_WITNESS_LAG_BLOCKS`)
- Insufficient spendable balance for amount + fee
- Node unreachable or tip stale vs expiry window

See [Mainnet send evidence](../../../docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md) for timing and expiry behavior.

## Fees

Orchard ZIP-317 conventional fee is computed in the transaction builder — no separate `estimatefee` RPC (Zebrad does not mirror zcashd fee estimation).

Desktop: `estimate_fee` Tauri command for UI preview.

## Related

- [Sending ZEC](../user-guide/sending-zec.md)
- [Transaction History](../user-guide/transaction-history.md)
