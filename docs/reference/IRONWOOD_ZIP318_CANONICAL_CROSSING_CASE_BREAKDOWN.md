# Case breakdown: ECC Swift/Android SDK RCs → Nozy ZIP 318 canonical crossings

**Status:** Implemented · unit-tested · live preflight validated on mainnet · waiting on next ZIP 318 bucket for migrate prebuild  
**Date:** 2026-07-30 (mainnet, profile `5a5c81bda1fa6343`)  
**Related code:** [`src/ironwood/migration.rs`](../../src/ironwood/migration.rs) · [`src/ironwood/mod.rs`](../../src/ironwood/mod.rs) · [`src/main.rs`](../../src/main.rs) (`ironwood preflight` / `migrate`)  
**Related PRs:** [#195](https://github.com/LEONINE-DAO/Nozy-wallet/pull/195) (funding + logo) · [#196](https://github.com/LEONINE-DAO/Nozy-wallet/pull/196) (twin-note postmortem)  
**Audience:** contributor notes / funding honesty — what ECC shipped, what Nozy already had, what we aligned

---

## One-line verdict

ECC’s Swift/Android wallet SDK RCs are **not** something Nozy should adopt as a dependency — but their ZIP 318 “canonical crossing” behavior was a useful checklist. Nozy already planned/built turnstiles; we tightened funding to **oldest zero-change Orchard note**, added a dry-run **`Zip318CrossingProposal`**, and proved it on the live wallet via `ironwood preflight`.

---

## What showed up externally

| Signal | What it is |
|--------|------------|
| [zcash-swift-wallet-sdk 2.7.0-rc.3](https://github.com/zcash/zcash-swift-wallet-sdk/releases/tag/2.7.0-rc.3) / [2.8.0-rc.2](https://github.com/zcash/zcash-swift-wallet-sdk/releases/tag/2.8.0-rc.2) | ECC iOS SDK wrapping `librustzcash` + lightwalletd |
| [2.7.0-rc.4](https://github.com/zcash/zcash-swift-wallet-sdk/releases/tag/2.7.0-rc.4) | Same stack; shorter ZIP 318 delays; anchor-age cap 4; **oldest** covering note; Ironwood accounting fixes |
| Android “Deploy Release” CI | Parallel Android packaging of the same Rust client backend |

Headline ECC behavior once NU6.3 is active: `proposeTransfer` prefers a ZIP 318 canonical crossing — `{1,2,5}×10^k` amount, single Orchard note, one unpadded Ironwood action, shared anchor bucket / rolling expiry — and falls back to an ordinary proposal when that shape cannot be funded.

---

## What Nozy already had (before this session)

| Capability | Status |
|------------|--------|
| ZIP 318 plan / split / migrate / broadcast CLI | Yes |
| `{1,2,5}×10^k` denominations (`Zooko125`) | Yes |
| 256-block anchor buckets, `k_max = 4`, transfer expiry | Yes |
| Real V6 Orchard→Ironwood turnstile build + prove | Yes (testnet/mainnet path) |
| ECC-style `proposeTransfer` object | **No** |
| Oldest-note funding for crossings | **No** (used **smallest** sufficient note) |
| Hard “no Ironwood change” on turnstile | **No** (change output allowed when note had headroom) |
| Zeaking LWD Ironwood sync | Proto fields only — wallet Ironwood path is Zebrad JSON-RPC |

Product stance stayed the same: **do not** swap Nozy for ECC’s Swift/Android SDKs. Keep `zeaking` / UniFFI / CLI / desktop as the stack; steal protocol hygiene from their release notes.

---

## What we changed

### 1. Canonical funding picker (`select_canonical_zip318_funding`)

Prefers the **oldest** eligible Orchard note (lowest `block_height`, then earliest index), matching ECC 2.7.0-rc.4+:

1. **Exact cover** — `note == transfer + fee` → Ironwood output = transfer  
2. **Fee from output** — `note == transfer` → Ironwood output = transfer − fee  
3. Otherwise **reject** — oversized notes that would create Ironwood change are not canonical

### 2. Zero-change turnstile prebuild

`build_migration_transaction_for_transfer` no longer adds an Ironwood change output. If selection somehow left residual change, prebuild errors and tells the operator to split first.

### 3. Dry-run proposal API

- `Zip318CrossingProposal` / `Zip318FundingMode`  
- `propose_zip318_crossing` (spendable notes)  
- `propose_zip318_crossing_from_orchard_notes` (cached `notes.json` rows — no unlock required)

### 4. CLI surface

`nozy ironwood preflight` prints the proposal for the next eligible **or** waiting transfer so you can inspect funding before the bucket opens.

---

## Live mainnet check (this wallet)

```text
Orchard migration balance: 270000 zat across 6 notes
ZIP 318 transfers: 1
Readiness: waiting-for-window
Next waiting transfer: #1 200000 zat at bucket 3431168

ZIP 318 proposal (preview; bucket not open yet):
   funding: fee_from_output
   note 200000 zat @ height 3428662 → Ironwood 160000 zat
   fee 40000 zat | change 0 | expires @ 3431424
```

`ironwood migrate` correctly refused with **waiting for next ZIP 318 anchor bucket** — not a funding failure. At tip ~3430961 that was ~207 blocks (~4–4.5 hours) until **3431168**.

When the window opens:

```powershell
cargo run --bin nozy -- ironwood preflight   # expect ReadyToPrebuild + proposal
cargo run --bin nozy -- ironwood migrate      # lock V6 turnstile
cargo run --bin nozy -- ironwood broadcast    # still inside the open window
```

---

## Tests

`cargo test --lib ironwood::migration::tests` — **20 passed**, including:

- oldest exact-cover preference  
- fee-from-output fallback on oldest exact denom  
- rejection of headroom-only notes  

---

## What got pushed (and what did not)

| Item | Destination |
|------|-------------|
| ZIP 318 funding + proposal + preflight | [#195](https://github.com/LEONINE-DAO/Nozy-wallet/pull/195) (`fec37148`) |
| Root `assets/logo.png` sync with desktop icon | [#195](https://github.com/LEONINE-DAO/Nozy-wallet/pull/195) (`d9bff588`) |
| Twin-note merge postmortem | [#196](https://github.com/LEONINE-DAO/Nozy-wallet/pull/196) |
| `desktop-client/src-tauri/Cargo.toml` line endings | Restored — **not** code |
| `nozy-mobile/hosted-api.env` | Local secrets — **not** pushed |
| `ironwood-migrate-proof.txt`, `.vercel/`, `lightwalletd/`, `nozy-mobile/modules/` | Local/ops — **not** product source |

---

## Gaps still open (honest)

| Gap | Notes |
|-----|--------|
| ECC multi-note ordinary fallback | Nozy still single-note only for spends/turnstiles |
| Ironwood **change classification** metadata | ECC fixed Ironwood change vs recipient accounting; Nozy turnstiles now avoid change entirely, but general Ironwood sends can still emit change without UI classification |
| Zeaking compact Ironwood sync | Still JSON-RPC / Zebra-first for Ironwood notes |
| Cover-traffic cohort query | Preflight still local-bucket scaffolding; no public-chain peer cohort health |

---

## Bottom line

We did **not** integrate ECC’s mobile SDKs. We used their RC notes as a protocol mirror, aligned Nozy’s turnstile funding with canonical ZIP 318 shape, and made that visible in `preflight` on a real mainnet schedule — then pushed only wallet/docs assets, not local secrets or clones.
