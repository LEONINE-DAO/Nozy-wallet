# Dynamic-fee pilot — mainnet evidence (v2.3.6.x)

Evidence for Shielded Labs / Mark: send → expiry → speed-up rebuild at ×4 fee, plus v2.3.6.3 confirmation-detection fix.

**Environment:** local zebrad (WSL) + `nozy` CLI **v2.3.6.3** · mainnet · pilot expiry **5 blocks**

---

## Completed end-to-end (2026-06-17)

### 1. Original send (expired)

| Field | Value |
|-------|--------|
| TXID | `ecbd068880e9512983f9ec36a1014c84ed51065886b0bcc657415feaccd4016b` |
| Amount | 0.001 ZEC |
| Fee | 0.0001 ZEC (standard) |
| Status | Expired (past `expiry_height` 3373648) |
| Broadcast | 2026-06-11 |

### 2. Speed-up (×4 fee, confirmed on chain)

| Field | Value |
|-------|--------|
| Original TXID | `ecbd068880e9512983f9ec36a1014c84ed51065886b0bcc657415feaccd4016b` |
| Speed-up TXID | `73d1c24c10d282e850947102449c2a6dd5aef440943cd4393440702679f14e17` |
| Amount | 0.001 ZEC |
| Fee | **0.0004 ZEC** (×4 priority) |
| Block | **3381141** |
| Wallet status | Confirmed (after v2.3.6.3 `check-confirmations` fix) |

Verify on any mainnet node:

```bash
curl -s POST http://ZEbra:8232 -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"getrawtransaction","params":["73d1c24c10d282e850947102449c2a6dd5aef440943cd4393440702679f14e17",1],"id":1}'
```

---

## v2.3.6.3 pilot send (in progress — 2026-06-17)

Standard-fee pilot send to a **new** recipient wallet (`u148n8x…`), for repeat expire → speed-up test.

| TXID | Broadcast (UTC) | Recipient | Status |
|------|-----------------|-----------|--------|
| `83986045b7015b29a02c8058b901cb0a2438ec7e6c2c2cb9b6e0d257a3d41f9c` | 18:09:48 | `u148n8x…` | Pending (mempool) |
| `3c6ba9b5775e03bf1b3fdd96aee72d9277fb221049e44b83edec9748232f743b` | 18:22:47 | `u148n8x…` | Pending (mempool) |

- Amount: **0.001 ZEC** each · Fee: **0.0001 ZEC** · Memo: `nozy 2.3.6.3 pilot test`
- After ~5 blocks without confirmation → expect **Expired** → `POST /api/transaction/speed-up` or desktop History **Speed up**

### Terminal proof (broadcast success)

Full `nozy send` output (v2.3.6.3, mainnet, 2026-06-17):

```
🛡️  Shielded Address - Privacy Protected

🔨 Building transaction...

🛡️  PRIVACY PROTECTED
   ✅ Shielded Transaction
   ✅ Sender Hidden
   ✅ Receiver Hidden
   ✅ Amount Hidden
   ✅ Untraceable

Processing...
Building Orchard transaction...
Adding spend action for 290000 zatoshis
✅ Orchard bundle built successfully!
🔧 Proving Status: ✅ Orchard proving ready (Halo 2 - no external parameters required)
🔐 Bundle authorized (Orchard proof + binding + spend signatures)
✅ Transaction signed and serialized (ZIP-225 v5)
   Bundle contains 2 actions
   TXID: 3c6ba9b5775e03bf1b3fdd96aee72d9277fb221049e44b83edec9748232f743b
   Transaction size: 9165 bytes

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🛡️  TRANSACTION PRIVACY SUMMARY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   Privacy Level:     MAXIMUM (Monero-Equivalent)
   Transaction Type: Shielded (Orchard)
   Sender:           Hidden ✅
   Receiver:          Hidden ✅
   Amount:            Hidden ✅
   Traceability:     Untraceable ✅
   Fungibility:      True ✅

   🔒 This transaction is completely private.
   🛡️  Privacy is enforced by NozyWallet.
   ✅ No transparent transactions possible.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

╔════════════════════════════════════════════════════════════╗
║          🛡️  PRIVACY PROTECTED TRANSACTION 🛡️              ║
║  ✅ Shielded Transaction                                    ║
║  ✅ Untraceable                                              ║
║  ✅ Fungible                                                 ║
║  ✅ Monero-Level Privacy                                     ║
╚════════════════════════════════════════════════════════════╝

✅ Transaction broadcast successfully!
🆔 Network TXID: 3c6ba9b5775e03bf1b3fdd96aee72d9277fb221049e44b83edec9748232f743b
📝 Transaction saved to history - will track confirmations

✅ Transaction sent successfully!
🆔 Transaction ID: 3c6ba9b5775e03bf1b3fdd96aee72d9277fb221049e44b83edec9748232f743b
============================================================
  Amount sent: 0.00100000 ZEC
  Fee paid:    0.00010000 ZEC
  Total spent: 0.00110000 ZEC
  Remaining:   0.01050484 ZEC

🛡️  This transaction is private and untraceable.
🔒 Privacy is enforced by NozyWallet.
============================================================

💡 Run 'nozy history' to view transaction details
```

Optional screenshot: save terminal as `docs/assets/pilot-send-terminal-2026-06-17.png` (Win+Shift+S).

---

## Releases & PRs

| Item | Link |
|------|------|
| v2.3.6.2 | Pilot speed-up + GUI |
| v2.3.6.3 | Confirmation detection + unified Zebra RPC policy |
| PR #80 | Fix wallet history confirmations |
| PR #81 | `trusted_zebra_urls` + test-zebra parity with sync |

---

## Short message for Mark

> Mainnet pilot proven on v2.3.6.3: expired tx speed-up rebuilt at ×4 fee and confirmed in block 3381141. New pilot send broadcast to alternate wallet for second expire/speed-up cycle. CLI + API + desktop History wired.
