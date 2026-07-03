# Bug registry

**Last updated:** 2026-06-28

Add new rows at the top. Link GitHub issue when filed. Detailed writeups live in [`bugs/`](bugs/).

| ID | Status | Sev | Surface | Summary | Reporter | GitHub | Fixed in | Detail |
|----|--------|-----|---------|---------|----------|--------|----------|--------|
| BUG-2026-014 | Fixed (ops) | P1 | desktop + operator | NET_001 — Zebra RPC unreachable (wrong config path, port 8232, RPC off) | Internal | — | ops doc | [`bugs/2026-06-desktop-pre-release-debug-session.md`](bugs/2026-06-desktop-pre-release-debug-session.md#issue-1--net_001-cannot-connect-to-node) |
| BUG-2026-015 | Fixed (master) | P2 | desktop + core | Send stalls on sync-status when lightwalletd :9067 down (no timeout) | Internal | — | master | [session § Issue 2](bugs/2026-06-desktop-pre-release-debug-session.md#issue-2--send-appears-to-stall-on-checking-sync-status) |
| BUG-2026-016 | Fixed (master) | P1 | desktop | Send allowed while `zebra_tip` < `last_scan_height` (node catching up) | Internal | — | master | [session § Issue 3](bugs/2026-06-desktop-pre-release-debug-session.md#issue-3--send-blocked-while-zebra-node-is-still-syncing) |
| BUG-2026-017 | Fixed (master) | P1 | desktop + core | Post-send balance wrong — all notes marked spent | Internal | — | master | [session § Issue 4](bugs/2026-06-desktop-pre-release-debug-session.md#issue-4--balance-wrong-after-send) |
| BUG-2026-018 | Fixed (master) | P2 | desktop + core | Recent Activity sort — receive above latest send | Internal | — | master | [session § Issue 5](bugs/2026-06-desktop-pre-release-debug-session.md#issue-5--recent-activity--history-sort-wrong) |
| BUG-2026-019 | Fixed (master) | P3 | desktop + core | History shows 1969 for received txs (epoch timestamp) | Internal | — | master | [session § Issue 6](bugs/2026-06-desktop-pre-release-debug-session.md#issue-6--received-transactions-show-date-1969) |
| BUG-2026-013 | Fixed | P1 | core + api-server | Status synced but send fails — Orchard witness lag not tracked | Gilmore | — | v2.3.6.7 | [`bugs/2026-06-witness-sync-status-mismatch.md`](bugs/2026-06-witness-sync-status-mismatch.md) |
| BUG-2026-012 | Fixed | P1 | cli | `nozy balance` / status show 0 on v2 NoteIndex (legacy array parser) | Internal | — | v2.3.6.6 | [`bugs/2026-06-cli-balance-v2-noteindex.md`](bugs/2026-06-cli-balance-v2-noteindex.md) |
| BUG-2026-011 | Fixed | P1 | core + api-server | Send broadcast fails when proving outruns pilot expiry (rebuild/retry; 5-block delta kept) | Gilmore | — | v2.3.6.6 | [`bugs/2026-06-send-expiry-before-broadcast.md`](bugs/2026-06-send-expiry-before-broadcast.md) |
| BUG-2026-002 | Fixed | P1 | api-server | History empty despite balance | Gilmore | — | v2.3.6.6 | [`bugs/2026-06-history-empty-despite-balance.md`](bugs/2026-06-history-empty-despite-balance.md) |
| BUG-2026-001 | Fixed | P1 | api-server | Send rescans ~50k blocks when synced | Gilmore | — | v2.3.6.6 | [`bugs/2026-06-send-50k-rescan.md`](bugs/2026-06-send-50k-rescan.md) |
| BUG-2026-003 | Fixed | P1 | api-server | Balance 0 when cache empty but “already synced” | — | — | v2.3.6.5 | CHANGELOG |
| BUG-2026-004 | Fixed | P1 | api-server | Remote VPS Zebra connect blocked by privacy policy | — | — | v2.3.6.4 | CHANGELOG |
| BUG-2026-005 | Fixed | P1 | api-server | History confirmations stuck Pending | — | — | v2.3.6.3 | CHANGELOG |
| BUG-2026-006 | Fixed | P2 | api-server | Long u1 address rejected on send (>100 chars) | — | — | v2.3.5 | CHANGELOG |
| BUG-2026-007 | Fixed | P0 | cli | Wrong NU6.2 branch ID — all sends rejected (-25) | aphelionz | #58 | v2.3.3 | CHANGELOG |
| BUG-2026-008 | Fixed | P1 | cli | Multi-note send IncorrectFee (-25) | — | — | v2.3.4 | CHANGELOG |
| BUG-2026-009 | Fixed | P0 | cli | Spent notes re-selected on second send (#61) | — | #61 | v2.3.3 | CHANGELOG |
| BUG-2026-010 | Open | P3 | docs | api-server README missing sync start_height/end_height | Gilmore | — | — | [`api-sync-scan-height-response.md`](api-sync-scan-height-response.md) |

---

## Status legend

| Status | Meaning |
|--------|---------|
| **Open** | Not fixed on released tag |
| **Fixed (master)** | On `master`, not yet tagged |
| **Fixed** | Shipped in named version |
| **Wontfix** | By design or out of scope |
| **Duplicate** | See other ID |

---

## Next ID

**BUG-2026-020**
