# Evidence and test results

`cargo test` output is not stored here. This index points at **recorded** runs: mainnet sends, migration txids, benches, grant proofs, and network-privacy logs.

Raw logs and JSON captures live in [`docs/reference/evidence/`](../reference/evidence/). Write-ups that interpret those runs are listed below.

## Wallet send and sync

| Document | What it records |
|----------|-----------------|
| [`MAINNET_SEND_EXPIRY_TEST.md`](../MAINNET_SEND_EXPIRY_TEST.md) | Mainnet send/expiry check (API + CLI) |
| [`PILOT_MAINNET_EVIDENCE.md`](../PILOT_MAINNET_EVIDENCE.md) | Dynamic-fee pilot: expire and speed-up |
| [`MAINNET_SEND_READINESS_EVIDENCE.md`](../reference/MAINNET_SEND_READINESS_EVIDENCE.md) | Sync and send timings, txids, witness guard |
| [`PILOT_EXPIRY_PROVING_LATENCY.md`](../reference/PILOT_EXPIRY_PROVING_LATENCY.md) | Expiry versus proving latency |
| [`MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md`](../reference/MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md) | Orchard → Ironwood turnstile txid |
| [`DYNAMIC_FEE_SINGLE_NOTE_IRONWOOD_MAINNET_2026-09.md`](../reference/DYNAMIC_FEE_SINGLE_NOTE_IRONWOOD_MAINNET_2026-09.md) | Single-note dynamic fee on Ironwood mainnet |
| [`CLI_BALANCE_NOTEINDEX.md`](../reference/CLI_BALANCE_NOTEINDEX.md) | CLI balance / NoteIndex incident notes |
| [`NOZY_LITE_BENCHES.md`](../reference/NOZY_LITE_BENCHES.md) | CLI vs desktop size and cold start (measured) |

How to reproduce the automated suite: [testing](../development/testing.md).

## Ironwood and fees

| Document | What it records |
|----------|-----------------|
| [`IRONWOOD_WALLET_READINESS.md`](../reference/IRONWOOD_WALLET_READINESS.md) | NU6.3 migration cases |
| [`DYNAMIC_FEE_A_PRIME_SOAK.md`](../rfcs/DYNAMIC_FEE_A_PRIME_SOAK.md) | Dynamic-fee soak notes |

## Network privacy (Nym)

Case write-ups: [`NYM_IP_PRIVACY_CASE_BREAKDOWN.md`](../reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md), [`NYM_SEND_EGRESS_CASE_BREAKDOWN.md`](../reference/NYM_SEND_EGRESS_CASE_BREAKDOWN.md), [`NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md`](../reference/NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md), [`NYM_LWD_MIXNET_PROXY_CASE_BREAKDOWN.md`](../reference/NYM_LWD_MIXNET_PROXY_CASE_BREAKDOWN.md).

Captured runs: [`docs/reference/evidence/`](../reference/evidence/) (`lwd-mixnet-*`, `nym-*`, `send-egress-smoke-*`).

## Crosslink

Lifecycle JSON under [`docs/reference/evidence/`](../reference/evidence/) (`crosslink-lifecycle-*`). Status notes: [`CROSSLINK_NOZY_STATUS_AND_NEXT.md`](../CROSSLINK_NOZY_STATUS_AND_NEXT.md).

## Security review

| Document | What it is |
|----------|------------|
| [`SECURITY.md`](../../SECURITY.md) | Disclosure policy and audit status |
| [`SELF_AUDIT_RESULTS.md`](../../SELF_AUDIT_RESULTS.md) | December 2025 self-audit |
| [`security-audit/`](../reference/security-audit/) | AI-assisted prep (scope, threat model, findings). Not a third-party certificate |

## Grants

Packets and proofs: [`docs/reference/grant-evidence/`](../reference/grant-evidence/).

## Sync engine smoke

[`sync-engine-smoke-20260905.txt`](../reference/evidence/sync-engine-smoke-20260905.txt), [`lwd-parity-encode-20260905.txt`](../reference/evidence/lwd-parity-encode-20260905.txt), [`lwd-parity-engine-20260905.txt`](../reference/evidence/lwd-parity-engine-20260905.txt).
