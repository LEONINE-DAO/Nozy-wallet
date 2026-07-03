# Quick Send Tutorial

Send a small shielded ZEC transaction end-to-end.

## Prerequisites

- Zebrad running — `nozy test-zebra` OK
- Wallet funded (shielded balance > 0)
- Recipient `u1…` address

## CLI path

```bash
# 1. Verify node
nozy test-zebra

# 2. Sync
nozy sync --to-tip

# 3. Check balance
nozy balance

# 4. Send dust amount
nozy send -r u1RECIPIENT… -a 0.0001

# 5. Confirm
nozy check-confirmations -t <txid>
```

First send may take **2–4+ minutes** for Orchard proving.

## Desktop path

1. **Sync** until caught up (Home strip shows 0 blocks behind).
2. **Send** tab → recipient → amount → confirm.
3. Enter password if prompted.
4. Wait for success toast with TXID.
5. **History** → explorer link.

## Self-send test

Send to your own **Receive** address — valid regression test on mainnet with dust.

## If send fails

| Error | Action |
|-------|--------|
| NET_001 | Fix Zebrad — [Zebra Node Setup](../advanced/zebra-node.md) |
| Witness lag | Sync again |
| SEND_001 | Lower amount or sync |
| Timeout / -25 | Sync to tip, retry (expiry window) |

Evidence and timings: [mainnet send reference](../../../docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md).

Next: [Set Up Your Own Node](own-node.md)
