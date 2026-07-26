# Nym × Zcash for Ironwood: hygiene first, then the hybrid

**Draft for Zcash Community Forum / long-form article**  
**NozyWallet · LEONINE-DAO · July 2026**  
**Status:** Ready to paste / edit before posting  
**Engineering case breakdown:** [NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md](NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md) · [NYM_IP_PRIVACY_CASE_BREAKDOWN.md](NYM_IP_PRIVACY_CASE_BREAKDOWN.md)  
**Nym docs (Harry / Mark):** [zcash-sdk.nym.com](https://zcash-sdk.nym.com/) · [guidance](https://zcash-sdk.nym.com/guidance/) · [recommended architecture](https://zcash-sdk.nym.com/recommended/)

---

## Suggested title

Nym × Zcash for Ironwood: what wallets must do that the mixnet cannot — NozyWallet notes

## Suggested thread

Reply under the Ironwood / safer-migration discussions, or a short standalone post that links:

- Shielded Labs — *Security issues in migrating user funds from Orchard to Ironwood*
- ZIP 318 migration draft
- Nym’s [Zcash × Nym wallet developer site](https://zcash-sdk.nym.com/)

---

## Body (paste below)

Hi all —

Thank you to **Harry and Mark at Nym** for publishing the wallet integrator site at [zcash-sdk.nym.com](https://zcash-sdk.nym.com/). The Ironwood turnstile makes migration amounts public by design; the hard problem left for wallets is **attribution** — who can join those amounts to a person, IP, or lightwalletd session.

Nym’s writeup separates two layers cleanly, and that split should be normative for wallet work:

1. **Transport** — hide the client from network observers and from the destination’s view of the real IP (mixnet for small sensitive submits; 2-hop dVPN for bulk compact sync on native apps).
2. **Baseline hygiene** — disciplines the **destination still sees**: request content and wall-clock arrival. The network cannot do this part for you.

This post is NozyWallet’s product/engineering note on how we are implementing that stack for Orchard → Ironwood migration.

### The threat in one paragraph

Inside a shielded pool, light servers do not learn amounts. At the Ironwood turnstile, **amounts are public**. If a wallet syncs and broadcasts in the clear (or even over a private path but with naive timing and exact resume heights), an L2 operator or correlated observer can still:

- chain sync sessions via exact start heights,
- join “this IP just caught the tip” to “this turnstile appeared,”
- treat broadcast arrival time as a fingerprint even when packets were mixed in transit.

Canonical denominations and shared ZIP 318 buckets fix **on-chain** collision. They do **not** replace network hygiene.

### What Nym recommends (and what we agree with)

From their [recommended architecture](https://zcash-sdk.nym.com/recommended/) for **native** wallets:

| Path | Mode | Why |
|------|------|-----|
| Compact-block / bulk sync | **2-hop dVPN** | Fast; hides client IP from destination; not for mixing timing |
| Broadcast / small sensitive RPC | **Mixnet (smolmix)** | Timing protection in transit + per-request unlinkability posture |
| Browser wallets | **mixFetch only** | dVPN unavailable in the browser sandbox; do not put bulk sync on the mixnet |

Plus the hygiene checklist:

- **Start-height obfuscation** — random overlap and/or checkpoint snap (V3 vs L2 content).
- **Broadcast scheduling** — never submit on the sync session; never broadcast immediately on reaching tip; randomized delay; prefer destination splitting.
- **Credentials** — acquire NYM for bandwidth before migration day.
- **Crowding** — shared exits / cohorts matter; anonymity is not a solo sport.

### What NozyWallet just shipped (baseline hygiene)

We treated hygiene as **Priority 0 for the Nym workstream** — it helps even when the user is on a local Zebrad and never opens a remote tunnel.

Landed in Nozy (defaults **on**):

1. **Sync start obfuscation**  
   Auto-resume rewinds a random overlap (default up to **128** blocks) and snaps down to a **256-block** checkpoint (aligned with ZIP 318 anchor spacing). Explicit `--start-height` stays deterministic for debugging.

2. **Randomized migrate-broadcast delay**  
   Real `ironwood broadcast` sleeps a uniform **30–300s** delay before submit (configurable). Dry-run still shows the policy without sleeping the full window in a misleading way for operators reading blockers.

3. **Tip-sync decorrelation**  
   When sync catches tip we record `last_tip_sync_unix`. Migrate-broadcast refuses if tip was caught fewer than **120s** ago (configurable). This is the “do not broadcast on reaching the tip” rule as a hard gate, not a suggestion.

4. **Operator surface**  
   `config.baseline_hygiene`, `nozy ironwood preflight` notes, desktop/API `baseline_hygiene_notes`, and `--skip-broadcast-hygiene` for tests only.

Numbers are **engineering defaults**, not a claim of a measured anonymity set. Nym correctly leaves the quantitative sizing of overlap/checkpoint as an open question. We want community feedback if wallets should converge on shared defaults.

### What we already had (and still prioritize)

Safer migration Priorities 1–3 remain:

1. **Protect the broadcasting IP** — prefer local Zebrad; Tor/I2P SOCKS detection; Nym smolmix subprocess for remote `sendrawtransaction` (IP-relocate **PASS** in our spike; live remote submit proof still needs an exit-reachable RPC).
2. **Shared cover** — ZIP 318 buckets; thin-cohort warnings.
3. **Amount/timing algorithm** — Shielded Labs / Zooko `{1,2,5}×10^k` with shared anchor buckets; residual below 0.001 ZEC abandoned rather than fingerprinted.

The smolmix helper is still a **subprocess** (sqlite `links` clash with in-process Nym). That is an engineering scar, not the product end-state.

### What is next (honest roadmap)

| Step | Status |
|------|--------|
| Baseline hygiene (this post) | **Landed** |
| Live mixnet remote broadcast proof | **Harness ready** — D2b/D2c-live still need exit-reachable Zebrad ([NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md](NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md)) |
| Product-wire **smol-dvpn** compact sync | Spike **C2/C3 PASS**; **C5** zeaking connector; **C6** desktop Network privacy opt-in + probe (subprocess) — [NYM_DVPN_SYNC_CASE_BREAKDOWN.md](NYM_DVPN_SYNC_CASE_BREAKDOWN.md) |
| Browser **mixFetch** for extension submits | Later; sync stays companion/local |
| Dual-endpoint destination split UX | Guidance now; auto-config later |
| NYM credential / swap UX for users | Prerequisite before marketing “Nym supported” |

We will **not** claim “Nym integrated” in store copy until broadcast-over-mixnet is proven on a real remote path and the hybrid sync story is more than a spike.

### Ask for the ecosystem

1. **Wallet implementers:** please read Nym’s guidance and ship hygiene even if your Nym transport is still in progress. Exact resume heights and tip-coupled broadcasts are free wins for observers.
2. **Nym / researchers:** if there is a preferred shared default for overlap distribution or checkpoint spacing for Ironwood season, publish it — wallets should collide on policy, not invent private fingerprints.
3. **Operators:** share exit-reachable test endpoints carefully; our D2b JSON-RPC-over-mixnet work is blocked on reachability, not on Sphinx.

### Links

- Nym × Zcash site: https://zcash-sdk.nym.com/  
- Nozy hygiene case breakdown: `docs/reference/NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md`  
- Nozy Nym IP case breakdown: `docs/reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md`  
- Nozy dVPN sync (track C) case + forum draft: `docs/reference/NYM_DVPN_SYNC_CASE_BREAKDOWN.md` · `docs/reference/NYM_DVPN_SYNC_FORUM_ARTICLE.md`  
- Earlier safer-migration forum draft: `docs/reference/SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md`

Happy to take technical pushback on the defaults (128 / 256 / 30–300s / 120s) and on sequencing (hygiene → mixnet broadcast proof → dVPN sync).

— NozyWallet / LEONINE-DAO

---

## AI disclosure (keep in PR / forum footer as appropriate)

Draft assisted by Cursor Agent; human author responsible for accuracy before posting.
