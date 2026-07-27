# dVPN sync for Ironwood season: the other half of the Nym hybrid

**Draft for Zcash Community Forum / long-form / paper appendix**  
**NozyWallet · LEONINE-DAO · July 2026**  
**Status:** Ready to paste / edit — **C2/C3 PASS** after Mark dep bump (`smoldvpn` / `develop`)  
**Case breakdown:** [NYM_DVPN_SYNC_CASE_BREAKDOWN.md](NYM_DVPN_SYNC_CASE_BREAKDOWN.md)  
**Evidence:** [`docs/reference/evidence/nym-c3-dvpn-run-smoldvpn-develop.txt`](evidence/nym-c3-dvpn-run-smoldvpn-develop.txt)  
**Companions:** [NYM_MIXNET_BROADCAST_FORUM_ARTICLE.md](NYM_MIXNET_BROADCAST_FORUM_ARTICLE.md) · [NYM_ZCASH_HYBRID_FORUM_ARTICLE.md](NYM_ZCASH_HYBRID_FORUM_ARTICLE.md) · [zcash-sdk.nym.com](https://zcash-sdk.nym.com/)

---

## Suggested title

Nym hybrid for wallets: mixnet for submit, dVPN for compact sync — NozyWallet track C (live PASS)

## Body (paste below)

Hi all —

Nym’s wallet guidance recommends a **hybrid**: small sensitive requests (especially **broadcast**) over the **mixnet**, and **bulk compact-block sync** over **2-hop dVPN**. We already wrote up hygiene (Layer-2 duties the transport cannot do) and mixnet submit (track B). This note is the sync half — including an honest first-fail / then-pass story after updating to Mark’s current SDK pins.

### Why not put sync on the mixnet?

Bulk transfers do not belong on Sphinx. The mixnet is for timing-sensitive, small messages. Compact sync is bandwidth-bound — dVPN (userspace WireGuard through entry+exit) is the practical path. That hides the client IP from the lightwalletd destination without pretending to mix packet timing.

### How this fits a desktop that already runs local Zebrad

On a machine with **local / WSL Zebrad**, the biggest IP↔tx win is already Case A1: submit never leaves for a remote indexer. Mixnet broadcast is for **remote** submit stacks.

dVPN sync is still useful when:

- the wallet pulls compact blocks from a **public** lightwalletd, or
- the operator wants sync metadata / censorship resistance without moving proving/submit off the local node.

On our operator host (2026-07-26): WSL Zebrad tip ~3.425M via `172.20.199.206:8232` (LAN). Mixnet correctly **refuses** that URL — we will not punch unauthenticated JSON-RPC onto the public internet just to force a probe PASS. Mixnet IP-relocate still **PASS**es (exit ≠ host).

### Live experiment (track C / issue #146)

Spike: `tools/nym-dvpn-lwd-spike` → public LWD `https://zec.rocks:443`, funded mainnet Nyx `$NYM` / ticketbooks.

**First night (stale `feature/nym-sdk-dvpn` / `nym-smol-dvpn`):** ticketbook issuance flaky; gateway register timed out; no tunnel throughput number.

**After Mark’s guidance (crates.io `smolmix 1.21.4`; dVPN → `smoldvpn` + `nym-sdk-session` on Nym `develop`):**

| Stage | Result |
|-------|--------|
| Clearnet compact sync (100 blocks) | **PASS** — 0.19s (~526 blocks/s), egress Atlanta / T-Mobile |
| Two-hop tunnel bring-up | **PASS** — entry FR, exit KR |
| IP relocate | **PASS** — tunnel `85.155.176.143` (Seoul) ≠ clearnet |
| Compact sync through tunnel (100 blocks) | **PASS** — 1.50s (~67 blocks/s), ~**7.9×** direct |

Evidence: `docs/reference/evidence/nym-c3-dvpn-run-smoldvpn-develop.txt`. Case scoreboard: `docs/reference/NYM_DVPN_SYNC_CASE_BREAKDOWN.md`.

### Practical notes for other wallet teams

1. **Pin current deps.** Stale feature-branch pins cost us a night of false negatives.
2. **Consumer NymVPN app ≠ SDK credentials.** Disconnect `nym-vpnd` while measuring the SDK tunnel.
3. **Ticketbook issuance can fail once and recover** — plan retry UX.
4. **Destination split still matters.** Even with dVPN sync: sync LWD and submit Zebrad should not be the same observer when both are remote.
5. Local-node users still do **not** need `$NYM` for day-to-day Ironwood submit IP privacy (Case A1).

### Status table

| Piece | Status |
|-------|--------|
| Spike `nym-dvpn-lwd-spike` (#146) | In tree; `smoldvpn` / `develop` |
| Direct + tunnel LWD sync compare | **PASS** (~7.9×) |
| Product wire into `zeaking` / companion | Next (C5) |
| Store claim “Nym sync supported” | Not until product-wired |

### Links

- Case breakdown: `docs/reference/NYM_DVPN_SYNC_CASE_BREAKDOWN.md`  
- Mixnet broadcast article: `docs/reference/NYM_MIXNET_BROADCAST_FORUM_ARTICLE.md`  
- Hygiene / hybrid article: `docs/reference/NYM_ZCASH_HYBRID_FORUM_ARTICLE.md`  
- Nym recommended architecture: https://zcash-sdk.nym.com/recommended/

— NozyWallet / LEONINE-DAO

---

## Suggested paper appendix box (short)

**Appendix — NozyWallet track C (2026-07-26).** After updating to Mark’s current Nym pins (`smolmix` 1.21.4; `smoldvpn` on `develop`), two-hop dVPN compact sync to `zec.rocks` **PASS**ed: clearnet 100 blocks in 0.19 s (Atlanta); tunnel exit Seoul; tunnel 100 blocks in 1.50 s (~7.9×). Earlier failures on a retired `feature/nym-sdk-dvpn` pin are recorded as operational risk (ticketbook issuance + register timeouts), not as architecture rejection. Local Zebrad submit (Case A1) remains the operator default for IP↔tx on LAN nodes.

---

## AI disclosure

Draft assisted by Cursor Agent; human author owns accuracy before posting or citing in a paper.
