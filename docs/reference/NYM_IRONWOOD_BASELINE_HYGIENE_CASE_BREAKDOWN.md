# Nym × Ironwood baseline hygiene — case breakdown (NozyWallet)

**Status:** Engineering landed (2026-07-25) — baseline hygiene **on by default**; hybrid Nym transport still phased  
**Date:** 2026-07-25  
**Nym guidance:** [zcash-sdk.nym.com](https://zcash-sdk.nym.com/) · [Implementation guidance](https://zcash-sdk.nym.com/guidance/) · [Recommended architecture](https://zcash-sdk.nym.com/recommended/)  
**Related:** [NYM_IP_PRIVACY_CASE_BREAKDOWN.md](NYM_IP_PRIVACY_CASE_BREAKDOWN.md) · [SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md](SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md) · [NYM_ZCASH_HYBRID_FORUM_ARTICLE.md](NYM_ZCASH_HYBRID_FORUM_ARTICLE.md)  
**Code:** [`src/ironwood/baseline_hygiene.rs`](../../src/ironwood/baseline_hygiene.rs) · [`src/wallet_sync.rs`](../../src/wallet_sync.rs) · [`src/ironwood/migration.rs`](../../src/ironwood/migration.rs)

Case IDs in **this document** track the Nym “what the transport can’t do for you” workstream plus how it fits the hybrid (dVPN sync + mixnet broadcast) roadmap.

---

## Living scoreboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| H1 | Start-height obfuscation (random overlap) | **Landed** | Default max overlap **128** blocks |
| H2 | Checkpoint snap of sync start | **Landed** | Spacing **256** (ZIP 318 bucket align) |
| H3 | Randomized migrate-broadcast delay | **Landed** | Default **30–300s** |
| H4 | Tip-sync decorrelation guard | **Landed** | Default wait **≥120s** after tip catch-up |
| H5 | Sync ≠ broadcast transport (policy) | **Documented + partial** | Local sync preferred; remote submit → smolmix helper |
| H6 | Destination splitting (two LWD/RPC ends) | **Guidance** | Operational; not dual-endpoint auto-config yet |
| H7 | Config + preflight / status surface | **Landed** | `baseline_hygiene` in config; CLI preflight prints notes |
| N1 | Mixnet broadcast (smolmix) | **Wired + D2a re-PASS** | Local WSL Zebrad = Case A1 (correct). Remote D2b/D2c only if public RPC — [NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md](NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md) |
| N2 | dVPN compact sync (smol-dvpn) | **Case breakdown open** | [NYM_DVPN_SYNC_CASE_BREAKDOWN.md](NYM_DVPN_SYNC_CASE_BREAKDOWN.md); live run needs MNEMONIC |
| N3 | Extension mixFetch | **Not started** | Browser is mixnet-only per Nym |

**Defaults are privacy-preserving.** Operators may set `baseline_hygiene.* = false` or pass `--skip-broadcast-hygiene` for tests only.

---

## Why this workstream exists

Nym’s Ironwood migration guidance is blunt:

1. **Amounts at the turnstile are public by design.** Network privacy is about who can attribute them.
2. **Recommended native hybrid:** bulk sync over **2-hop dVPN**; broadcasts / small sensitive requests over the **mixnet**.
3. **Baseline hygiene is always the wallet’s job.** The destination (L2) still sees request content and wall-clock arrival. Mixing does not erase that.

Harry/Mark’s note (2026): see [zcash-sdk.nym.com/guidance](https://zcash-sdk.nym.com/guidance/) — “don’t skip the baseline hygiene, because the network can’t do that part for you.”

Nozy already had Priority 1–3 safer-migration scaffolding (local node / Tor / Nym attestation; ZIP 318 buckets; Zooko `{1,2,5}×10^k`). This pass closes the **Layer-2 content + timing** gaps Nym named explicitly.

---

## Case family H — Baseline hygiene (this pass)

### Case H1 — Randomized overlap on sync resume

**Problem (V3 / content vs L2):** Exact resume height `last_scan + 1` is a linking key. An L2 that sees successive sync starts can chain sessions.

**Decision:** On auto-resume (no explicit `--start-height`), rewind start by `U{0..=max_overlap_blocks}` then continue.

**Default:** `max_overlap_blocks = 128`.

**Cost:** Re-download only. Correctness unchanged (idempotent note merge).

**Code:** `obfuscate_scan_start` → `apply_start_height_obfuscation` in `wallet_sync`.

### Case H2 — Checkpoint snap

**Problem:** Even with overlap, starts can remain unique fingerprints.

**Decision:** After overlap, snap start down to a multiple of `checkpoint_spacing_blocks`.

**Default:** `256` — same as `ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS`, so Nozy clients whose prior sync ended in the same interval emit identical snapped starts more often.

**Code:** same path as H1.

### Case H3 — Randomized broadcast delay

**Problem (V2 / timing vs L2):** Mixnet shapes *in-transit* timing; the destination still sees wall-clock arrival. Broadcasting immediately when a bucket opens (or when the user clicks) is a timing leak — especially after sync milestones.

**Decision:** Before real `ironwood broadcast`, sleep a uniform random delay in `[min, max]` seconds.

**Default:** `30–300` seconds. Skip with `--skip-broadcast-hygiene` / API `skip_broadcast_hygiene` (tests / emergency).

**Code:** `plan_broadcast_delay` + `apply_broadcast_delay` in `execute_orchard_migration_broadcast`.

### Case H4 — Tip-sync decorrelation

**Problem:** “Sync to tip → immediately broadcast turnstile” is the classic L2 join Nym calls out.

**Decision:** Persist `last_tip_sync_unix` when sync catches tip. Refuse migrate-broadcast if elapsed `< min_secs_after_tip_sync` unless hygiene skipped.

**Default:** `120` seconds.

**Code:** `update_last_tip_sync_unix` · `require_tip_sync_guard`.

### Case H5 — Never broadcast on the sync transport

**Problem:** Same session / same tunnel for bulk sync and submit lets L2 join sync identity to the turnstile.

**Nozy posture:**

| Flow | Path |
|------|------|
| Day-to-day sync | Local / LAN Zebrad preferred |
| Remote submit | Optional Nym smolmix subprocess (`broadcast_via_nym_mixnet`) |
| Ironwood migrate-broadcast | Same Priority 1 gate + hygiene |

**Not yet:** Fully separate configured sync URL vs broadcast URL (H6).

### Case H6 — Destination splitting

**Nym ask:** Sync from one lightwalletd; broadcast through another.

**Status:** Documented operational guidance. Automatic dual-endpoint config is **follow-up** (needs product UX + trusted endpoint list). Local Zebrad sync + mixnet remote submit is the interim split.

### Case H7 — Operator surface

| Surface | What landed |
|---------|-------------|
| `config.json` → `baseline_hygiene` | All knobs + defaults |
| `config.json` → `last_tip_sync_unix` | Tip guard input |
| `nozy ironwood preflight` | Prints hygiene notes |
| `nozy ironwood broadcast --skip-broadcast-hygiene` | Escape hatch |
| Desktop / API safer_migration | `baseline_hygiene_notes[]` |

---

## Case family N — Transport (roadmap alignment with Nym hybrid)

### Case N1 — Mixnet broadcast (smolmix) — Priority 1 biggest win

See [NYM_IP_PRIVACY_CASE_BREAKDOWN.md](NYM_IP_PRIVACY_CASE_BREAKDOWN.md) D2a–D2d.

| Step | Status |
|------|--------|
| IP relocate | **PASS** 2026-07-11 |
| Subprocess wallet hook | **Wired** |
| Live remote submit proof | **Open** (needs exit-reachable RPC) |

### Case N2 — dVPN sync (smol-dvpn) — recommended hybrid bulk path

Spike: [`tools/nym-dvpn-lwd-spike`](../../tools/nym-dvpn-lwd-spike/). Product wire is **next** after H* + live N1.

### Case N3 — Browser extension

Nym: **mixFetch only**; dVPN impossible in browser sandbox; compact sync over mixnet **not recommended**. Extension work is later and broadcast-scoped.

---

## Defaults & trade-offs (honest)

Nym leaves overlap distribution and checkpoint spacing as an **open quantitative question**. Our defaults are conservative engineering choices:

| Knob | Default | Trade-off |
|------|---------|-----------|
| Max overlap | 128 | Larger ⇒ bigger anonymity set, more re-download |
| Checkpoint | 256 | Matches ZIP 318 buckets; wider ⇒ more waste, more collisions |
| Broadcast delay | 30–300s | Longer ⇒ harder tip/bucket joins; worse UX |
| Tip guard | 120s | Stops sync→submit reflex |

We will revisit numbers with community / Nym feedback; do **not** treat them as anonymity proofs.

---

## Operator checklist

1. Leave `baseline_hygiene` defaults on for migration season.
2. Prefer local Zebrad for sync.
3. For remote submit: build smolmix helper, set `NOZY_NYM_SMOLMIX_BIN` + `broadcast_via_nym_mixnet`.
4. Acquire NYM for bandwidth credentials before relying on mixnet/dVPN ([swap.nym.com](https://swap.nym.com/)).
5. Do **not** market “Nym integrated” until D2b/D2c live PASS + dVPN product path exists.
6. Use `--skip-broadcast-hygiene` only in automated tests.

---

## Evidence log

| Date (UTC) | Step | Result |
|------------|------|--------|
| 2026-07-25 | H1–H4 unit tests | In-crate (`baseline_hygiene` module tests) |
| 2026-07-25 | Wire sync + migrate-broadcast + preflight + API/desktop notes | Landed in tree |
| 2026-07-25 | Forum article draft | [NYM_ZCASH_HYBRID_FORUM_ARTICLE.md](NYM_ZCASH_HYBRID_FORUM_ARTICLE.md) |

---

## AI disclosure

Implementation and this case breakdown assisted by Cursor Agent. Human review required before forum posting or release notes that claim mainnet Nym product readiness.
