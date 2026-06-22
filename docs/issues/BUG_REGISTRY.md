# Bug registry

**Last updated:** 2026-06-21

Add new rows at the top. Link GitHub issue when filed. Detailed writeups live in [`bugs/`](bugs/).

| ID | Status | Sev | Surface | Summary | Reporter | GitHub | Fixed in | Detail |
|----|--------|-----|---------|---------|----------|--------|----------|--------|
| BUG-2026-011 | Fixed (master) | P1 | core + api-server | Send broadcast fails when proving outruns pilot expiry (rebuild/retry; 5-block delta kept) | Gilmore | — | unreleased | [`bugs/2026-06-send-expiry-before-broadcast.md`](bugs/2026-06-send-expiry-before-broadcast.md) |
| BUG-2026-002 | Fixed (master) | P1 | api-server | History empty despite balance | Gilmore | — | unreleased | [`bugs/2026-06-history-empty-despite-balance.md`](bugs/2026-06-history-empty-despite-balance.md) |
| BUG-2026-001 | Fixed (master) | P1 | api-server | Send rescans ~50k blocks when synced | Gilmore | — | unreleased | [`bugs/2026-06-send-50k-rescan.md`](bugs/2026-06-send-50k-rescan.md) |
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

**BUG-2026-012**
