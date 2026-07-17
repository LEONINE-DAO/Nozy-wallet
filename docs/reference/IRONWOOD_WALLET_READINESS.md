# Ironwood (NU6.3) wallet readiness — NozyWallet

**Status:** CLI **v2.4.0** shipped (July 2026) — testnet-validated Ironwood scan, migration, and send; desktop/API surfaces in progress  
**Release:** [v2.4.0 Ironwood (CLI)](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/v2.4.0)  
**Target:** Mainnet activation **2026-07-28** at height **3,428,143** (ecosystem PSA; wallet falls back to this height if `zcash_protocol` has not pinned NU6.3 yet)  
**Forum:** [Ironwood: Verifying the Soundness of Zcash's Circulating Supply](https://forum.zcashcommunity.com/t/ironwood-verifying-the-soundness-of-zcash-s-circulating-supply/56044)  
**Wallet migration draft:** [ZIP 318 PR #1317](https://github.com/zcash/zips/pull/1317)  
**Privacy guidance:** Shielded Labs — *Security issues in migrating user funds from Orchard to Ironwood* (Zooko Wilcox & Taylor Hornby): network-level privacy (Defense A) + `{1,2,5}×10^k` concurrent mixing (Defense B / Appendix A)

---

## What Ironwood requires from wallets

| Requirement | Nozy status |
|-------------|-------------|
| NU6.2 mainnet (emergency Orchard fix) | **Done** — v2.3.x stack |
| NU6.3 / Ironwood deps (`librustzcash`, `orchard` 0.15-pre) | **Done (compiles)** — mainline pins in `Cargo.toml` |
| `ShieldedPool` note tagging (Orchard vs Ironwood) | **Started** — `pool` field plus pool-specific witness slots |
| Ironwood commitment tree + witnesses | **Started (compiles)** — treestate parser, tree codec aliases, witness tracker, scan wiring |
| Turnstile migration txs (Orchard spend → Ironwood output) | **In progress (testnet)** — 3+ turnstiles confirmed; Orchard → Ironwood migration loop complete on profile |
| Post-activation sends route to Ironwood pool | **Done (CLI validated)** — live smoke `d6794092…` block 4148837 |
| Ironwood scan/sync (LWD + JSON-RPC) | **JSON-RPC validated** — V3 `IronwoodDomain` decrypt + pool-aware spendable load; LWD needs `ironwood-valar` validation (see **Ironwood lightwalletd** below) |
| Desktop / API migration UX | **Started (desktop read-only)** — Home card/status panel added; migration actions disabled |
| Zebrad + lightwalletd NU6.3 | **Testnet WSL node synced** — use WSL IP `:18232` for migration validation (not Windows `zebrad.exe`); run `scripts/ironwood-lwd-smoke.ps1` after `ironwood-valar` LWD |
| Safer migration Priority 1 (IP) | **Started** — CLI broadcast gate + desktop/API status (`safer_migration`); attest checkbox on Home card |
| Safer migration Priority 2 (cover) | **Scaffolded** — local thin-bucket warn; **SHOULD** not MUST; next: public-chain cover estimator (no coordinator) |
| Safer migration Priority 3 (amounts/timing) | **Shielded Labs `{1,2,5}×10^k` active** — residual below 0.001 ZEC abandoned; ZIP 318 power-of-ten kept as compatibility ladder; memoryless timing / consolidation rounds next |

**Safer migration writeup:** [`SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md`](SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md) · code: `src/ironwood/network_privacy.rs`

**Formal verification** (Lean 4 `zcash/ironwood` repo) is separate from wallet work — see [`IRONWOOD_PR4_NOTES.md`](IRONWOOD_PR4_NOTES.md).

**Dynamic-fee pilot (separate case breakdown):** Shielded Labs ZIP-317 / expiry / speed-up policy lives in [`DYNAMIC_FEE_CASE_BREAKDOWN.md`](DYNAMIC_FEE_CASE_BREAKDOWN.md). Ironwood v2.4.0 **reuses** `src/fee_policy.rs` for sends and migration, but **Ironwood “Case 3-F fee dust”** and **dynamic-fee “Case 3 (15 vs 5 blocks)”** are different topics — see the cross-reference table in that doc if you read both papers.

---

## NozyWallet migration case breakdown

This is a separate decision from "Orchard vs Sapling vs transparent addresses." NozyWallet is already Orchard-first; Ironwood changes the next step for that Orchard-first posture. The migration question is how Nozy should move existing Orchard funds into the new Ironwood pool without turning a privacy upgrade into a wallet fingerprinting event.

### Case 1 — Do nothing after activation

Nozy could keep scanning and spending only Orchard notes after NU6.3 activation.

Evidence and risks:

- Existing Orchard notes would remain visible to Nozy's local wallet state, so this is the smallest immediate engineering change.
- It would make Nozy a lagging wallet after the network introduces the Ironwood pool and post-activation V6 transaction path.
- Users could believe they are on the current shielded path while their funds remain in the legacy Orchard pool.
- Normal Orchard-only sends after activation risk creating a distinct wallet behavior, so Nozy blocks that path and routes normal sends through Ironwood once V6/Ironwood builder support is validated (Phase 3.4).

Conclusion: this is acceptable only as a short safety stop before Phase 3.4 lands. Post-activation, Ironwood routing is the target product behavior.

### Case 2 — Migrate everything immediately

Nozy could spend all Orchard notes into Ironwood outputs as soon as activation happens.

Evidence and risks:

- This is simple to explain, but weak for privacy because a wallet-wide migration at one time can cluster a user's notes.
- Large or unusual note sets may produce distinctive transaction shapes.
- Users who are offline at activation would migrate at different times, creating uneven timing signals.
- It gives the user less control over network conditions, Tor/VPN use, fees, and wallet readiness.

Conclusion: one-shot migration is too blunt for a privacy-first wallet except as an explicit emergency/manual recovery path.

### Case 3 — ZIP 318-style scheduled turnstile migration

Nozy's preferred path is an explicit Orchard-spend to Ironwood-output migration, planned and executed in staged windows.

Evidence and rationale:

- ZIP 318 is specifically aimed at wallet migration behavior rather than ordinary user payments.
- Canonical denominations reduce unique amount fingerprints.
- Shared anchor-height buckets reduce timing fingerprints compared with wallet-specific ad hoc sends.
- A persisted schedule lets Nozy pre-plan the migration, recover from missed windows, and avoid surprising the user.
- A broadcast-only background task keeps network submission separate from wallet sync, which is safer for desktop/mobile/extension surfaces.
- User consent remains explicit: Nozy can show the Orchard balance at risk, explain the privacy tradeoff, and require the user to start migration.

Conclusion: this is the right NozyWallet default because it treats Ironwood migration as a privacy operation, not just a balance transfer.

### Case 4 — Post-migration normal sends

After migration support lands, Nozy should treat Ironwood as the post-activation receive/change pool for normal shielded sends.

Evidence and rationale:

- Continuing to create new Orchard outputs after activation would keep recreating the same migration problem.
- Routing recipient and change outputs through Ironwood aligns user activity with the current shielded pool.
- Pool-aware note tagging lets Nozy keep legacy Orchard notes and new Ironwood notes separate during the transition.
- Spendability should remain conservative: Ironwood notes can be indexed before they are exposed as spendable, then enabled only after transaction-builder support is validated.

Conclusion: migration is not complete until normal send routing stops producing new Orchard-only wallet state. **Phase 3.4** shipped CLI Ironwood send routing on testnet (2026-07-06); **v2.4.0** published the validated CLI stack. Desktop/API Send surfaces still need wiring.

### Product decision

NozyWallet should present Ironwood migration as an explicit, evidence-backed readiness workflow:

1. Detect NU6.3 activation and Ironwood-capable RPC support.
2. Show Orchard and Ironwood pool status separately.
3. Plan the ZIP 318-style schedule before broadcasting anything.
4. Require user consent before migration execution.
5. Prefer scheduled, denomination-aware turnstile transactions over one large migration.
6. Route post-activation normal sends to Ironwood once V6 builder support is validated.

This keeps the wallet's privacy story consistent: Nozy chose Orchard over transparent addresses and legacy-first Sapling behavior because it wanted shielded-by-default usage; Nozy migrates from Orchard to Ironwood because the modern shielded default has moved forward.

---

## Live testnet wallet restore case breakdown

### Case 1 — A synced testnet node is necessary but not sufficient

The migration prebuild spends wallet-owned Orchard notes, so a synced Zebra testnet node alone cannot validate migration. Nozy also needs a restored or newly created **testnet wallet profile** with spendable Orchard tZEC and synced witnesses.

Evidence:

- The dedicated testnet wallet restore flow succeeded.
- The restored wallet produced a testnet Orchard receiver:
  `utest1jha9q2fwxjug08svdhza6jgmvvfdqyujqes80kj2gpmjsytcltjsslzpjj5gy8z2958yhdlqn4568jeu5d9fec6yme4rh3g4lqvvkjh8`
- Live testnet sync completed through height `4_130_763`, finding `1` spendable Orchard note worth `20_000_000` zatoshis (`0.20000000` ZEC).
- The new `testnet-wallet` CLI path configures the active profile for `network = testnet` and `zebra_url = http://127.0.0.1:18232`.

Conclusion: use a dedicated testnet profile for Ironwood validation so `wallet.dat`, `notes.json`, witnesses, compact cache, and `ironwood_migration_schedule.json` do not mix with mainnet state.

### Case 2 — The first restore sync failed in witness initialization

Observed error:

```text
z_gettreestate: orchard pool has neither finalState nor finalRoot
```

Evidence and interpretation:

- The error occurred during `nozy --testnet sync --to-tip`, before migration execution.
- It came from Nozy's `z_gettreestate` parser / witness initialization path, not from the mnemonic restore.
- Fresh testnet sync starts at height `1`, and Nozy was effectively trying to initialize witness state from the pre-scan checkpoint.

Conclusion: this was a treestate parsing / first-sync initialization bug, not evidence that the wallet restore failed.

### Case 3 — Fresh testnet sync should start from empty trees

For scans beginning at height `0` or `1`, Nozy can safely initialize Orchard and Ironwood witness trackers from empty trees instead of requiring Zebra to return a non-empty checkpoint treestate.

Evidence:

- `src/notes.rs` now uses `OrchardCommitmentTree::empty()` when `start_height <= 1`.
- The Ironwood witness tracker uses the same `start_height <= 1` empty-tree rule.

Conclusion: first testnet sync should no longer fail just because `z_gettreestate(0)` is absent, empty, or shaped differently.

### Case 4 — Zebra treestate JSON needs tolerant parsing

Different Zebra builds and upgrade-era RPC paths can expose equivalent treestate fields with different naming or nesting. Nozy previously accepted only `commitments.finalState` or `commitments.finalRoot`.

Evidence:

- `src/zebra_tree_rpc.rs` now accepts:
  - `finalState`
  - `final_state`
  - `finalRoot`
  - `final_root`
  - `root`
  - both under `commitments` and at the pool level.
- Regression tests cover pool-level `final_root` and `commitments.root` shapes.

Conclusion: Nozy should parse Zebra's current treestate shape without weakening the later witness/root checks used before spends.

### Case 5 — Next validation step

Live sync now reaches the chain tip and finds the restored wallet's note:

```text
Scan complete: Orchard 1 notes (1 spendable, 20000000 ZAT)
Sync complete! Balance: 0.20000000 ZEC
Found 1 new notes, 1 total notes
Last scanned height: 4130763
```

The next attempt may still stop with:

```text
Ironwood (NU6.3) is not active on this network yet
```

That is expected while the current testnet chain tip is below `zcash_protocol`'s NU6.3 activation height. The V6 Orchard-to-Ironwood builder is only valid after activation.

Before activation, use:

```powershell
cargo run -- --testnet ironwood plan
```

Expected pre-activation result:

- `ironwood plan` should show a ZIP 318 schedule for `20_000_000` zatoshis across canonical denominations.
- The first anchor bucket should be at or after NU6.3 activation.

After activation, run:

```powershell
cargo run -- --testnet sync --to-tip
cargo run -- --testnet ironwood migrate
```

Expected post-activation result:

- `ironwood migrate` should prebuild a locked V6 Orchard-to-Ironwood transaction and save it into `ironwood_migration_schedule.json`, without broadcasting.

### Case 6 — Activation gate: plan is allowed, migration build is not

Observed error:

```text
Ironwood (NU6.3) is not active on this network yet
```

Evidence and interpretation:

- The testnet wallet has a spendable Orchard note and the ZIP 318 migration plan can be produced.
- The connected testnet tip is still below `zcash_protocol`'s NU6.3 activation height.
- V6 transactions with an Ironwood bundle are only consensus-valid after NU6.3 activation.
- Building the Orchard-to-Ironwood turnstile transaction before activation would produce an invalid transaction, so Nozy must block `ironwood migrate`.

Implementation evidence:

- CLI `ironwood plan` now uses an activation-aware schedule tip: if Ironwood is not active, scheduling starts from `activation_height - 1`, so the next ZIP 318 bucket lands at or after activation.
- CLI `ironwood migrate` checks activation before unlocking the wallet or loading spendable notes.
- Desktop readiness uses the same activation-aware schedule tip for its read-only plan preview.

Conclusion: pre-activation behavior should be read-only planning only. The next valid migration action is to wait for testnet to reach NU6.3 activation, sync again, then run `nozy --testnet ironwood migrate`.

### Case 7 — Pre-activation readiness commands

Before NU6.3 activates, Nozy can now keep the wallet state ready without attempting an invalid V6 transaction.

Operator commands:

```powershell
cargo run -- --testnet ironwood status
cargo run -- --testnet ironwood preflight
cargo run -- --testnet ironwood plan --save
```

Implementation evidence:

- `ironwood status` shows the current tip, NU6.3 activation height, blocks remaining, and explicitly reports "planning only until activation."
- `ironwood preflight` checks the active network, activation gate, Orchard note count/value, max cached witness lag, ZIP 318 transfer count, next anchor bucket, and saved schedule health without unlocking the wallet or building a transaction.
- `ironwood plan --save` persists the draft ZIP 318 schedule before activation so the operator can inspect `ironwood_migration_schedule.json`.
- Saved schedules are refreshed if pending or presigned transfer windows are missed; post-activation `ironwood migrate` rebuilds the schedule before prebuilding a replacement raw V6 transaction.

Conclusion: the wallet can now stay activation-ready with explicit diagnostics and a persisted draft schedule while still refusing migration transaction construction until NU6.3 is active.

### Case 8 — Incremental sync showed `0 ZEC` after a valid migration plan

Observed output:

```text
ironwood status: Wallet notes — Orchard: 110000000 zat | Ironwood: 0 zat
ironwood preflight: Orchard migration balance: 110000000 zat across 1 notes
ironwood plan --save: Total: 110000000 zatoshis
sync --to-tip: Sync complete! Balance: 0.00000000 ZEC
```

User catch:

- The operator noticed the contradiction immediately: the Ironwood readiness commands correctly saw the restored testnet note, but the follow-up sync reported `0 ZEC`.
- The saved `ironwood_migration_schedule.json` still recorded the source note at block `4_131_004`, worth `110_000_000` zatoshis (`1.10000000` ZEC), so the funds were not missing from the chain or the migration plan.

Evidence and interpretation:

- The problematic run was an incremental `sync --to-tip` scanning only the latest 72 blocks.
- That scan found `0` new notes because the wallet note was older than the incremental range.
- The CLI sync branch still used an older manual note-cache merge/write path. In the empty-result incremental case, it could rewrite `notes.json` as `[]` and then print `0 ZEC`.
- The shared wallet-sync path already had stronger preservation logic; the CLI path needed to use the same `merge_scanned_notes`, `save_wallet_notes`, and `wallet_unspent_balance_zatoshis` helpers.

Fix and recovery:

```powershell
cargo run -- --testnet sync --start-height 4131004 --to-tip
cargo run -- --testnet ironwood status
cargo run -- --testnet ironwood preflight
```

Implementation evidence:

- CLI sync now loads cached notes through `load_wallet_notes`.
- It merges new scan results with `merge_scanned_notes` instead of replacing the cache from the incremental result.
- It saves through `save_wallet_notes`.
- It reports balance using `wallet_unspent_balance_zatoshis`, so spent notes and cached unspent notes are handled consistently.

Conclusion: a `0 ZEC` result after an incremental no-new-note scan should be treated as a cache/reporting bug if `ironwood status`, `preflight`, or a saved schedule still proves an older unspent note exists. The targeted recovery is to rescan from the note's block height and then rerun Ironwood readiness checks.

---

## Phase 3 core CLI migration case breakdown

Phase 3 starts with the core CLI path because every later surface depends on the same migration state machine. The goal is not to make the desktop button work first; it is to make `nozy ironwood preflight` and `nozy ironwood migrate` tell the truth about whether a wallet is planning-only, ready to prebuild, waiting for a ZIP 318 window, blocked on note splitting, or already holding a locked presigned transaction.

### Case 1 — Rebuild every run without validating saved state

Nozy could ignore the saved schedule and rebuild from the current wallet notes every time `ironwood migrate` runs.

Evidence and risks:

- Rebuilding is simple, but it discards the user's previous migration plan and makes missed-window recovery hard to reason about.
- A rebuilt schedule can silently move anchor buckets, changing the timing profile that ZIP 318 is trying to normalize.
- If a presigned transaction already exists, rebuilding without validation risks orphaning raw transaction state or preparing conflicting transfers.

Conclusion: `migrate` must load and validate `ironwood_migration_schedule.json` first, then rebuild only when source notes, totals, denominations, bucket rules, or missed windows require it.

### Case 2 — Prebuild any pending transfer that has enough funds

Nozy could scan the pending schedule and prebuild the first transfer that any current Orchard note can cover.

Evidence and risks:

- This was close to the initial scaffold: a pending transfer plus a sufficiently large note could produce a locked V6 transaction.
- It does not enforce `not_before_height`, so it can prepare a transfer outside its ZIP 318 anchor bucket.
- It can skip earlier scheduled transfers, weakening the user's persisted schedule as the source of truth.

Conclusion: Phase 3 should prepare only the next eligible scheduled transfer, and only when `chain_tip >= not_before_height` and the transfer window has not expired.

### Case 3 — Treat note splitting as optional

Nozy could allow a large note to migrate directly into one canonical Ironwood output plus Ironwood change.

Evidence and risks:

- The V6 builder can technically create an Orchard spend with Ironwood outputs, including change.
- ZIP 318 separates note splitting from turnstile migration because canonical denominations are part of the privacy story.
- If the wallet needs split preparation but Nozy migrates a large source note anyway, the resulting transaction shape could fingerprint this wallet.

Conclusion: when canonical denominations cannot be covered by current notes, the CLI must return `split-required` and stop before prebuild. Nozy should not fake the split phase by creating arbitrary migration shapes.

#### Case 3 — what we did (implementation breakdown)

Case 3 separates **note splitting** (Orchard → Orchard send-to-self) from **turnstile migration** (Orchard → Ironwood). Nozy must not skip splitting by folding change into Ironwood outputs on the migration tx.

**Problem we were solving**

- Testnet wallet holds **1.3 ZEC** in **2 Orchard notes** (1.1 + 0.2) but the ZIP 318 plan needs **4 canonical slots** (1×1.0 + 3×0.1 ZEC).
- `ironwood preflight` correctly returned **`split-required`**, blocking turnstile prebuild (Case 4).
- Post-activation **normal Orchard sends are disabled** (`orchard_only_send_blocker`), but ZIP 318 Phase 1 still requires Orchard send-to-self — a migration-only exception was required.
- Faking split via Ironwood change on turnstile txs would produce non-standard shapes and fingerprint the wallet.

**What we shipped (Phase 3.1)**

| Layer | What landed | Where |
|-------|-------------|-------|
| Split detection | `note_requires_canonical_split`, `flatten_canonical_denomination_zatoshis` | `src/ironwood/migration.rs` |
| Split planning | `plan_orchard_note_split_outputs` — fee deducted from source note; smallest output adjusted | same |
| Note picker | Largest Orchard note whose value decomposes into >1 canonical denomination | `pick_orchard_note_for_split` |
| V6 split tx | Orchard spend → multiple Orchard self-outputs via PCZT prove/sign/extract | `build_orchard_split_transaction` |
| Send-blocker bypass | Split path does **not** call `orchard_only_send_blocker` (migration-only, not normal Send) | `execute_orchard_note_split` |
| Broadcast + local spend mark | `sendrawtransaction` + `mark_wallet_notes_spent_from_spendables` | same |
| CLI | `nozy ironwood split` and `nozy ironwood split --dry-run` | `IronwoodCommand::Split` in `src/main.rs` |
| Readiness gate (unchanged) | `note_split_required` when `transfer_count != orchard_note_count`; `preflight`/`migrate` stop at `split-required` | `plan_orchard_migration_at`, `assess_orchard_migration_readiness` |
| Tests | Split planning fee adjustment; composite-value detection | `migration.rs` unit tests |

**Split pipeline (one note per invocation)**

```text
pick_orchard_note_for_split (largest composite note)
  → plan_orchard_note_split_outputs (canonical flatten + fee fit)
  → ensure_witness_fresh_for_send
  → V6 Builder: add_orchard_spend + add_orchard_change_output × N (same receiver scope as source note)
  → PCZT: Orchard proof + sign + extract raw tx
  → broadcast (unless --dry-run)
  → mark source note spent locally; operator syncs to discover new notes
```

Post-NU6.3 Orchard bundles disable cross-address outputs (`CrossAddressDisabled`); split uses **change outputs**, not plain `add_orchard_output`, and does not attach an Ironwood bundle (`ironwood_anchor: None`).

**Fee handling (why outputs may not be “perfect” powers of ten)**

- Split fee uses ZIP-317 shape for `max(1 spend, N outputs)` Orchard actions (standard multiplier, not priority ×4).
- When `sum(canonical_outputs) + fee` exceeds the source note, the **smallest output is reduced** by the deficit.
- Example dry-run on testnet profile (1.1 ZEC note):

  | Field | Value |
  |-------|-------|
  | Source | 110,000,000 zat |
  | Fee | 40,000 zat |
  | Output #1 | 100,000,000 zat (1.0 ZEC — exact) |
  | Output #2 | 9,960,000 zat (0.1 ZEC minus fee — not an exact 0.1 note) |

- This is intentional for Phase 3.1: fee is paid from the note being split rather than inventing a separate fee input. Residual non-canonical notes may need another split round or will be handled in a later refinement.

**Chicken-and-egg resolution**

| Before | After Phase 3.1 |
|--------|-----------------|
| Normal Send blocked post-Ironwood | Unchanged — desktop Send still shows Ironwood banner |
| No way to Orchard send-to-self | `ironwood split` is an explicit migration entry point |
| `migrate` stopped at `split-required` | Operator runs split loop, then returns to `migrate` |

**What we deliberately did not ship**

- No automatic “split until ready” loop in one command — operator runs **split → sync → preflight** manually (safer, easier to debug proving).
- No desktop split button yet (Phase 4 surfaces).
- No multi-note spend in one split tx — **one source note per split invocation**.
- No guarantee that post-fee outputs exactly match scheduled transfer denominations on the first pass; **`plan --save` after sync** remains required.

**CLI validation (testnet, 2026-07-04)**

Profile **Ironwood Testnet** (`5e7821be3f1d89d6`):

| Command | Case 3–relevant outcome |
|---------|-------------------------|
| `ironwood preflight` | **`split-required`** — gate working before split |
| `ironwood split --dry-run` | **Pass** — plans 1.1 ZEC note → 1.0 + 0.0996 ZEC outputs, fee 40k |
| `ironwood split` (live, first attempt) | **Blocked on witness** — `Orchard witness does not match z_gettreestate` until witness rescan (see live run breakdown below) |
| `ironwood split` (live, after fixes) | **Pass** — first attempt `d363b80d…` dropped from mempool (see 2026-07-05 run); rebroadcast `386a5d16…` confirmed |
| After split #1 + sync | Balance may still show 0.2 ZEC until mempool tx confirms and scan discovers new notes |
| Second split (0.2 ZEC note) | **Pass** — `d553bedd…` after rescan from block 4008652 (witness fix) |
| `ironwood plan --save` (after both splits) | **Pass** — 23-transfer schedule saved; next bucket **4143104** |
| `ironwood migrate` before split complete | Still stops at `split-required` (correct — Case 3 gate protects Case 4) |
| `ironwood migrate` after splits + plan | **Next** — prebuild transfer #1 when bucket window open (broadcast still disabled) |

#### Case 3 — live operator run (2026-07-04 evening)

This session exercised the **full split loop** on profile **Ironwood Testnet** (`5e7821be3f1d89d6`) against Windows Zebrad `:18232`. It exposed three blockers that dry-run alone did not surface.

**What we were trying to do**

Run the operator path documented above: `sync --to-tip` → `preflight` → `split` → `sync` → repeat until `split-required` clears, then return to Case 4 prebuild.

**Blockers we hit (in order)**

| # | Symptom | Root cause | Fix |
|---|---------|------------|-----|
| 1 | `preflight`: witness lag **608 blocks** after `sync --to-tip` | CLI `Sync` scanned blocks but did **not** call `sync_wallet_notes` witness refresh | Route `nozy sync` through `sync_wallet_notes` (`src/main.rs`) |
| 2 | `ironwood split`: witness mismatch despite lag **0** | Incremental sync refreshed witness bytes via block RPC advance, but **witness tracker** path is authoritative; stale hex + tip height could disagree with `z_gettreestate` | Load `notes.json` into note index on sync (`with_index_file`); merge witness fields in `merge_scanned_notes`; **rescan from note block** (e.g. `--start-height 4130900 --to-tip`) to rebuild tracker witnesses |
| 3 | `ironwood split`: `add_orchard_output: CrossAddressDisabled` | Post-NU6.3 Orchard bundles disable cross-address plain outputs; split builder used `add_orchard_output` + empty Ironwood anchor | Use `add_orchard_change_output`, `ironwood_anchor: None`, output to **source note's Orchard address** (preserve External vs Internal scope) |

**What we shipped in this session**

| Layer | Change | Where |
|-------|--------|-------|
| Sync / witnesses | CLI sync uses shared `sync_wallet_notes` (scan + witness refresh + caught-up witness catch-up) | `src/main.rs`, `src/wallet_sync.rs` |
| Note index on sync | Reload cached notes into `NoteScanner` index so incremental rescans update witnesses for existing notes | `wallet_sync.rs` → `NoteScanner::with_index_file` |
| Merge witnesses | `merge_scanned_notes` copies `orchard_incremental_witness_hex` / tip height when rescan updates them | `src/notes.rs` |
| V6 split builder | `add_orchard_change_output`; no Ironwood bundle on Orchard-only split tx | `build_orchard_split_transaction` in `migration.rs` |
| Testnet RPC | Always pass `--zebra-url http://127.0.0.1:18232` with `--testnet` (profile default may still point at `:8232`) | operator habit |

**Live validation (same profile, same day)**

| Step | Outcome |
|------|---------|
| `sync --to-tip` (after sync fix) | Witness lag **0 blocks** |
| Rescan `--start-height 4130900 --to-tip` | Rebuilt Orchard witnesses for 1.1 ZEC note via witness tracker (~5.6k blocks, fast on testnet) |
| `ironwood split` (1.1 ZEC note) | **Broadcast OK** — fee 40k zat; outputs 100M + 9_960_000 zat |
| TXID | `d363b80d4168d7c04d7a115e956edf67583c6de3b108d493edc261e332e39460` |
| Mempool / confirm | Submitted to local Zebrad mempool; source note marked spent locally; new outputs appear after inclusion + `sync --to-tip` |
| `preflight` (after split #1, before split #2) | Still **`split-required`** + schedule rebuild hints (expected — 0.2 ZEC note not split yet; plan still reflects pre-split 1.3 ZEC note set) |

**Operator loop status after 2026-07-04 session**

```text
[done]  sync fix + witness rescan path
[done]  V6 split builder fix (change outputs)
[done]  First live split broadcast (1.1 ZEC note) — tx d363b80d… later dropped (see below)
[todo]  Automated confirm → split #2 → plan --save  → completed 2026-07-05
```

#### Case 3 — automated split loop completion (2026-07-05)

This session ran the **operator automation** requested after the evening split broadcast: poll until split #1 confirms, run split #2, then `ironwood plan --save`. It surfaced one recovery case (dropped mempool tx), one witness case (old 0.2 ZEC note), and one fee-dust case (residual `split-required`).

**What we were trying to do**

Complete the Case 3 operator loop on profile **Ironwood Testnet** (`5e7821be3f1d89d6`):

```text
poll split #1 confirm → sync → split #2 → sync → ironwood plan --save → preflight
```

Then proceed to Case 4 prebuild (`ironwood migrate`) when the bucket window allows.

**Case breakdown — what we hit (in order)**

| # | Case | Symptom | Root cause | Fix / outcome |
|---|------|---------|------------|---------------|
| A | **Dropped mempool tx** | `getrawtransaction` on `d363b80d…` → “No such mempool or main chain transaction”; balance **0.2 ZEC** after sync | First split was broadcast but **expired from mempool unmined**; source note still marked **spent locally** | Release note via `release_wallet_notes_by_nullifier_hex` (manual JSON edit on nullifier `66e3b613…`); rebroadcast split #1 as `386a5d16…` |
| B | **Split #1 confirm + discover** | Poll RPC until `in_active_chain=true` (~90s) | Normal testnet block time | `sync --to-tip` → balance **1.2996 ZEC**, 3 unspent notes (1.0 + 0.0996 + 0.2) |
| C | **Witness mismatch on split #2** | `Orchard witness does not match z_gettreestate` on 0.2 ZEC note (height **4008652**) | Rescan from 4130900 refreshed tip height but **witness root** for the old note still disagreed with `z_gettreestate` | Full rescan `--start-height 4008652 --to-tip` (~134k blocks, ~10 min) to rebuild witness tracker path for that note |
| D | **Split #2 broadcast + confirm** | 0.2 ZEC → 0.1 + 0.0996 ZEC | Same V6 split builder as Case 3 evening session | TXID `d553bedd441e0f888732fb5b4550b8a53e7f4f5f8f0cf350a2c1211a6eb93557`; confirmed + sync → **1.2992 ZEC**, 4 unspent notes |
| E | **`plan --save`** | Rebuild schedule from post-split note set | Canonical decomposition of 1.2992 ZEC spans multiple denominations | Saved **23 transfers** to `ironwood_migration_schedule.json`; next bucket **4143104** |
| F | **Fee dust still `split-required`** | `preflight` still **`split-required`** after both splits | Each split leaves **9_960_000 zat** (0.0996 ZEC) change; fee (~480k zat) **cannot fit** a further split of that note (`Cannot fit ZIP 318 split outputs and 480000 zat fee into 9960000 zat`) | Expected fee economics — not a loop failure. Canonical **1.0** and **0.1** notes are ready for Case 4; dust notes remain until consolidation or a lower-fee path |

**Live validation (2026-07-05)**

| Step | TXID / outcome |
|------|----------------|
| Split #1 (re-broadcast) | `386a5d162cb3fe0b702824cd2a8792910a9e1986fad484476db8189ab4c9d072` — 1.1 → 1.0 + 0.0996 ZEC, fee 40k |
| Split #2 | `d553bedd441e0f888732fb5b4550b8a53e7f4f5f8f0cf350a2c1211a6eb93557` — 0.2 → 0.1 + 0.0996 ZEC, fee 40k |
| Post-split balance | **1.29920000 ZEC** — notes: 1.0, 0.1, 0.0996, 0.0996 |
| `ironwood plan --save` | 4 source notes, **23** scheduled transfers, bucket **4143104** |
| `ironwood preflight` (before Path B) | **`split-required`** (dust notes — case F) |
| Path B gate + `ironwood migrate` | **`presigned-waiting-for-broadcast`** — TXID `ff31a7fd…` |

**Operator loop status after 2026-07-05**

```text
[done]  Poll + confirm split #1 (after mempool-drop recovery)
[done]  Witness rescan from 4008652 for split #2
[done]  Split #2 broadcast + confirm
[done]  ironwood plan --save (23 transfers)
[done]  Path B splittable-only split gate + exact-denom migration spend
[done]  Case 4 prebuild transfer #1 (presigned, broadcast disabled)
[open]  Fee-dust 0.0996 notes (non-blocking after Path B)
[next]  Phase 3.3 broadcast in window; prebuild transfers #2+
```

#### Case 3 — Path B: splittable-only split gate (2026-07-05)

The original gate set `note_split_required` when `transfer_count != note_count` (23 vs 4), blocking turnstile prebuild after composite splits were done.

| Layer | Change | Where |
|-------|--------|-------|
| Split gate | Block only when an unspent Orchard note is a **splittable** composite | `orchard_wallet_needs_note_split`, `orchard_note_needs_splittable_split` |
| Fee-dust | 0.0996 notes that cannot fit a split fee no longer block migrate | Same |
| Exact-denom spend | 1.0 ZEC note: deduct migration fee from Ironwood output | `select_migration_spend` |

**Case 3 decision (unchanged)**

Do not turnstile-migrate composite notes. Stop at `split-required` while any **splittable** composite Orchard note remains; fee-dust notes that cannot fit a split fee do not block turnstile prebuild (Path B gate, 2026-07-05). Splitting is Orchard→Orchard only; Ironwood outputs belong in the turnstile phase.

### Case 4 — Presigned transaction exists but broadcast is not implemented

Nozy can now persist richer scheduled-transfer state after a locked V6 prebuild:

- `source_nullifier_hex`
- `prepared_txid`
- `presigned_tx_hex`
- `prepared_at_height`
- `expires_at_height`

Evidence and risks:

- Persisting this state lets `preflight` and `migrate` distinguish "ready to prebuild" from "already presigned."
- Broadcast remains intentionally disabled until scheduled-window validation, retry, and confirmation reconciliation are implemented.
- A stale presigned transaction must be detectable so the remaining schedule can be rebuilt instead of trying to reuse an expired transfer.

Conclusion: `presigned-waiting-for-broadcast` is a real Phase 3 state, not a failure. It means the locked raw transaction exists, but the broadcast task is still out of scope for this slice.

#### Case 4 — what we did (implementation breakdown)

Case 4 separates **prebuild** (prove, sign, persist) from **broadcast** (submit inside a ZIP 318 window). Nozy must not treat a saved raw hex blob as permission to send immediately.

**Problem we were solving**

- Early migration scaffolds could rebuild schedules every run and lose in-flight work.
- Broadcasting without window checks would violate ZIP 318 timing and could fingerprint the wallet.
- Desktop/API need a durable answer to “is there already a locked tx waiting?” without re-running the prover.

**What we shipped**

| Layer | What landed | Where |
|-------|-------------|-------|
| Schedule schema | Per-transfer lifecycle: `pending` → `presigned` → `broadcast` → `confirmed` / `expired` | `MigrationTransferStatus` in `src/ironwood/migration.rs` |
| Presigned metadata | `source_nullifier_hex`, `prepared_txid`, `presigned_tx_hex`, `prepared_at_height`, `expires_at_height` | `MigrationScheduledTransfer` + `ironwood_migration_schedule.json` |
| V6 prebuild | Orchard V2 spend → Ironwood V3 output via PCZT prove/sign/extract (PostNu6_3 proving key) | `build_migration_transaction_for_transfer` |
| Readiness state | `presigned-waiting-for-broadcast` when a non-expired presigned transfer exists | `assess_orchard_migration_readiness` |
| Guard rails | `migrate` returns early if already presigned; does not prebuild a second transfer | `execute_orchard_migration` |
| Stale detection | Expired windows increment `stale_presigned_count`; schedule rebuild drops missed slots | `validate_orchard_migration_schedule`, `load_or_rebuild_orchard_migration_schedule` |
| CLI surfacing | `preflight` prints presigned txid + stale count; `migrate` prints “Broadcast: disabled…” after prebuild | `src/main.rs` `IronwoodCommand::Preflight` / `Migrate` |
| Tests | Presigned field round-trip, stale-window detection, pending/presigned field consistency | `migration.rs` unit tests |

**Prebuild pipeline (one eligible transfer)**

```text
next_eligible_pending_transfer (bucket window open)
  → select_single_spend_note (one Orchard note covers denomination + fee)
  → V6 Builder: add_orchard_spend + add_ironwood_output (+ Ironwood change if any)
  → PCZT: Orchard proof + Ironwood proof + sign + extract raw tx
  → write presigned fields on schedule transfer #N, status = presigned
  → readiness → presigned-waiting-for-broadcast
```

**What we deliberately did not ship (Phase 3.3)**

- ~~No `sendrawtransaction` from `migrate` or a background broadcast task.~~ **Shipped 2026-07-05** — see **Phase 3.3 — window-validated broadcast** below.
- ~~No in-window retry loop or confirmation reconciliation (`broadcast` / `confirmed` status transitions).~~ **Partial** — `ironwood broadcast --wait-confirm` polls Zebrad; schedule marks `confirmed`.
- No Tor/VPN network-privacy step before broadcast.
- Desktop migrate button remains read-only; Home card shows readiness only.

#### Phase 3.3 — window-validated broadcast (2026-07-05)

**Problem:** Prebuilt turnstile txs must only hit the network inside their ZIP 318 anchor bucket and before expiry.

**What we shipped**

| Layer | What landed | Where |
|-------|-------------|-------|
| CLI command | `nozy ironwood broadcast` (`--dry-run`, `--wait-confirm`) | `src/main.rs` |
| Window gate | `presigned_transfer_broadcastable` — bucket open, not expired, presigned fields present | `migration.rs` |
| Readiness | `ready-to-broadcast` when presigned + in window | `assess_orchard_migration_readiness` |
| Broadcast | `sendrawtransaction` of `presigned_tx_hex`; schedule → `broadcast` | `execute_orchard_migration_broadcast` |
| Note spend | Mark source Orchard note spent by `source_nullifier_hex` | `mark_wallet_notes_spent_by_nullifier_hex` |
| Confirmation | `--wait-confirm` polls `getrawtransaction` → schedule `confirmed` | `reconcile_migration_broadcast_confirmations`, `ZebraClient::transaction_in_active_chain` |

**Live validation (Ironwood Testnet, transfer #1 — 1.0 ZEC)**

| Step | Outcome |
|------|---------|
| `ironwood preflight` | **`ready-to-broadcast`** |
| `ironwood broadcast --dry-run` | Pass — TXID `ff31a7fd…` |
| `ironwood broadcast --wait-confirm` | **Confirmed on chain** at tip ~4143640 |
| Post-broadcast | Orchard balance **1.2992 → 0.2992 ZEC**; `plan --save` → **22 transfers**; next bucket **4144128** |

**Operator loop (updated)**

```powershell
$z = @("--testnet", "--zebra-url", "http://<wsl-ip>:18232")   # not 127.0.0.1 on Windows

nozy @z sync --to-tip             # immediately before migrate if you waited on a bucket
nozy @z ironwood migrate          # prebuild when ready-to-prebuild
nozy @z ironwood preflight        # ready-to-broadcast when bucket open
nozy @z ironwood broadcast --wait-confirm
nozy @z sync --to-tip
nozy @z ironwood plan --save      # ONLY after broadcast confirms — never while presigned
```

**Broadcast guardrails (why Phase 3.3 is separate from migrate)**

- ZIP 318 expects txs in anchor-height buckets; sending early or late weakens the privacy story.
- A stale presigned tx must trigger schedule rebuild, not silent rebroadcast.
- Confirmation handling must tie `broadcast_txid` back to the schedule before marking `confirmed`.
- `plan --save` must refuse to rebuild while a presigned transfer is waiting (`save_orchard_migration_plan_at` guard).

**CLI validation (testnet, 2026-07-04)**

On profile **Ironwood Testnet** (`5e7821be3f1d89d6`, 1.3 ZEC Orchard):

| Command | Case 4–relevant outcome |
|---------|-------------------------|
| `ironwood preflight` | Correctly reports `split-required` — prebuild never runs, so no presigned tx yet |
| `ironwood migrate` | Stops before prebuild with note-splitting blocker; does not write `presigned_tx_hex` |
| After Case 3 (split) + bucket window | Expect `migrate` to prebuild transfer #1, persist hex to schedule, print `Broadcast: disabled…` |
| Second `migrate` while presigned | Expect `presigned-waiting-for-broadcast`; no duplicate prebuild |

**Path to “Case 4 pass” on testnet**

1. **Case 3** — run `ironwood split` loop until `preflight` clears `split-required` (composite 1.1 + 0.2 splits **done** 2026-07-05; 4 notes vs 23 transfers + fee dust remain — see **Case 3 — automated split loop completion**).
2. **Prebuild** — `ironwood migrate` at/after bucket **4143104** presigns transfer #1.
3. **Verify** — schedule JSON shows `status: presigned` + populated metadata; `preflight` → `presigned-waiting-for-broadcast`.
4. **Phase 3.3** — implement broadcast-only task with window validation; then mark `broadcast` / `confirmed` and enable desktop.

#### Case 4 — migrate attempt (2026-07-05)

After split loop + `plan --save`, Path B gate, and bucket **4143616** open:

| Check | Outcome |
|-------|---------|
| First attempt (strict gate) | Blocked at **`split-required`** (23 notes vs 4 transfers) |
| After Path B | `preflight` → **`ready-to-prebuild`** |
| `ironwood migrate` | **Prebuild OK** — transfer #1 presigned |
| TXID | `ff31a7fd5f9aba09c41361cc3704d1f3c33ab68cee92ef5146a961af11d6dc44` |
| Readiness | **`presigned-waiting-for-broadcast`** (broadcast still disabled) |
| Expires | height **4143872** |

**Case 4 decision (unchanged)**

Prebuild + persist is a safe stopping point; `presigned-waiting-for-broadcast` is success, not failure. Broadcast is a separate operator step (`ironwood broadcast`) with window validation and confirmation reconciliation.

#### Case 4 — turnstile operator loop: transfers #1–#2 + power-outage recovery (2026-07-05 evening)

This session exercised the **full Phase 3.3 loop** on profile **Ironwood Testnet** (`5e7821be3f1d89d6`): first confirmed 1.0 ZEC turnstile, attempted 0.1 ZEC turnstile, lost presigned state to operator error, recovered after a power outage, and completed transfer #2.

**What we were trying to do**

```text
transfer #1: migrate → broadcast --wait-confirm → plan --save
transfer #2: poll bucket → sync → migrate → broadcast --wait-confirm → plan --save
```

**Case breakdown — what we hit (in order)**

| # | Case | Symptom | Root cause | Fix / outcome |
|---|------|---------|------------|---------------|
| A | **Transfer #1 confirmed** | Balance **1.2992 → 0.2992 ZEC** after broadcast | Phase 3.3 window-validated `sendrawtransaction` | TXID `ff31a7fd5f9aba09c41361cc3704d1f3c33ab68cee92ef5146a961af11d6dc44` confirmed; `plan --save` → **22 transfers** |
| B | **Bucket wait for #2** | ~20 min polling until bucket **4144128** | ZIP 318 `not_before_height` gate — normal testnet block time | Tip reached bucket; `migrate` prebuilt 0.1 ZEC |
| C | **Broadcast blocked** | `mempool is disabled since synchronization is behind the chain tip` | Zebrad RPC height looked caught up but mempool still disabled until internal sync finished | Retry after node healthy — but see case D |
| D | **Presigned tx lost** | Schedule back to all `pending`; no `presigned_tx_hex` | Retry script ran **`plan --save`** while presigned tx `282c9a5c…` was still waiting | **Never `plan --save` while presigned.** Added guard in `save_orchard_migration_plan_at` |
| E | **Power outage** | Testnet Zebrad RPC down; migration blocked | Host power loss mid-session | Manual recovery required after login |
| F | **Wrong node started (Windows)** | RPC on `127.0.0.1:18232` at block **~25k** (~0.8% sync), state **0.05 GB** | Recovery mistakenly used `nozy-zebra-testnet\start-testnet-zebra.ps1` (native **`zebrad.exe`**) instead of WSL | **Stop Windows zebrad.** Real node: WSL `/home/lowo/zebra-testnet/zebrad.toml`, **276 GB** state, tip **~4.14M** |
| G | **Autostart gap** | Testnet did not come back after reboot | `install-zebrad-autostart.ps1` only starts **mainnet** WSL zebrad (`:8232`), not testnet (`:18232`) | Start testnet manually after outage; **mainnet-only autostart confirmed 2026-07-06** (no testnet task by design) |
| H | **Witness lag during bucket poll** | `migrate` failed: witness **106 blocks** behind after ~11 min wait | Witnesses go stale while polling `getblockcount`; max lag gate is **50 blocks** | **Sync immediately before `migrate`**, not only at start of poll loop |
| I | **Transfer #2 recovered** | Re-sync 106 blocks → migrate → broadcast in one pass | Sync + migrate back-to-back after bucket open | TXID `a1ed6e83aa3db422dd2c7e5da278071a675b4cc07128166de0e6123d669de775`; balance **0.2992 → 0.1992 ZEC** |
| J | **Confirm poll timeout** | `broadcast --wait-confirm` reported "confirmation still pending" | Zebrad slow to return `in_active_chain`; tx had already mined | `sync --to-tip` showed spend landed; safe to `plan --save` |

**Live validation (2026-07-05 evening)**

| Transfer | Amount | TXID | Bucket | Outcome |
|----------|--------|------|--------|---------|
| #1 | 1.0 ZEC | `ff31a7fd…` | ~4143616 | **Confirmed** — first live V6 turnstile broadcast |
| #2 (lost) | 0.1 ZEC | `282c9a5c…` | 4143872 | Prebuilt; **never broadcast** — wiped by accidental `plan --save` |
| #2 (redo) | 0.1 ZEC | `a1ed6e83…` | 4144896 | **Confirmed** — balance 0.1992 ZEC; schedule **21 transfers** |

**Operator infrastructure lessons**

| Topic | Wrong | Correct |
|-------|-------|---------|
| Zebrad on Windows | `nozy-zebra-testnet\zebra-bin\bin\zebrad.exe` | WSL only — see `scripts/run-nozy.ps1` policy |
| Testnet RPC URL | `http://127.0.0.1:18232` from Windows | `http://<wsl-ip>:18232` (e.g. `172.20.199.206`; changes after reboot) |
| Testnet config | `C:\Users\User\nozy-zebra-testnet\zebrad.toml` | `/home/lowo/zebra-testnet/zebrad.toml` |
| Post-reboot zebrad | Expect testnet autostart | Only **mainnet** autostart today (`C:\Zebrad\scripts\install-zebrad-autostart.ps1`) |
| Schedule rebuild | `plan --save` anytime | Only after **confirmed** broadcast; guard refuses while presigned |

**Operator loop status after 2026-07-05 evening**

```text
[done]  Transfer #1 (1.0 ZEC) broadcast + confirm
[done]  Transfer #2 (0.1 ZEC) broadcast + confirm (after outage recovery)
[done]  plan --save guard (refuse rebuild while presigned)
[open]  WSL testnet autostart (declined — mainnet autostart only, verified 2026-07-06)
[next]  Fee-dust spendability — see Case 4 late session below
```

#### Case 4 — fee-dust spendability + third turnstile (2026-07-05 late)

After the evening recovery, the wallet held **0.1992 ZEC** in **2×0.0996 ZEC** fee-dust notes (split change). The saved ZIP 318 schedule still listed **transfer #1 = 0.1 ZEC (10_000_000 zat)** first. No single note could fund that slot, but **`ironwood split`** also cannot reshape 9_960_000 zat notes (fee does not fit). This session exposed a **preflight vs migrate mismatch** and shipped **spendability-aware transfer selection** (Path C).

**What we were trying to do**

```text
poll bucket 4145152 → sync → migrate → broadcast → plan --save   # expected 0.1 ZEC turnstile #3
```

**Case breakdown — what we hit (in order)**

| # | Case | Symptom | Root cause | Fix / outcome |
|---|------|---------|------------|---------------|
| A | **Missed bucket while blocked** | Bucket **4145152** opened; `migrate` failed on note coverage | Prior attempt failed before prebuild; **`plan --save`** rolled schedule forward to bucket **4145408** | Re-poll correct bucket from saved schedule |
| B | **Preflight vs migrate disagree** | `preflight` → **`ready-to-prebuild`** (transfer #1 10M); `migrate` → **`split-required`** | Readiness used **sequence order** only; migrate checked **note coverage** | Path C: spendability-aware eligibility in preflight + migrate |
| C | **Fee-dust cannot fund 0.1 ZEC slot** | `No Orchard note covers … 10000000 zat plus fee`; only **9_960_000 zat** notes remain | Exact **0.1 ZEC** note was spent in prior turnstile; dust notes are below canonical 0.1 + fee | Skip unfundable sequence slots; prebuild next **fundable** in-window transfer |
| D | **Split still impossible on dust** | `ironwood split --dry-run` → cannot fit split outputs + fee into 9_960_000 zat | Same fee economics as Case 3-F | Expected — turnstile must use **smaller canonical denominations** (e.g. 0.01 ZEC) from dust notes |
| E | **Stale CLI binary** | Code changes had no effect; old error strings persisted | `cargo build` in Cursor sandbox wrote to **`CARGO_TARGET_DIR` cache**, not `NozyWallet\target\release\nozy.exe` | Rebuild with `$env:CARGO_TARGET_DIR = "…\NozyWallet\target"` before operator runs |
| F | **Path C — spendability-aware selection** | — | Schedule sequence ≠ spendability order after fee-dust | **`next_spendable_eligible_pending_transfer`**, **`assess_orchard_migration_readiness_with_spendability`**, migrate picks fundable transfer |
| G | **Third turnstile confirmed** | Prebuilt **schedule #2** (1M zat = **0.01 ZEC**), not unfunded #1 (10M) | 9_960_000 zat note covers 1M + fee with Ironwood change | TXID `10a17d8885434723770d5992f3599eb88c99098b24801342236fd5c16ac0a1ee`; balance **0.1992 → 0.0996 ZEC** |
| H | **Confirm poll timeout (again)** | `broadcast --wait-confirm` → pending | Zebrad slow on `in_active_chain` | `sync --to-tip` showed spend; **`plan --save`** → **24 transfers**, next bucket **4145664** |

**What we shipped (Path C)**

| Layer | Change | Where |
|-------|--------|-------|
| Spendability check | `can_cover_transfer_with_note_values` — note ≥ transfer + fee, or exact-denom Path B | `migration.rs` |
| Next transfer | `next_spendable_eligible_pending_transfer` — lowest sequence **in window** that notes can fund | `migration.rs` |
| Preflight | `assess_orchard_migration_readiness_with_spendability` — warns when #1 unfundable, points at spendable #N | `migration.rs`, `main.rs` `Preflight` |
| Migrate | Uses spendable transfer, not blind sequence order | `execute_orchard_migration` |

**Live validation (2026-07-05 late)**

| Field | Value |
|-------|-------|
| Bucket | **4145408** (tip ~4145487) |
| Schedule slot | **#2** — 1_000_000 zat (0.01 ZEC); **#1** (10M) skipped as unfundable |
| TXID | `10a17d8885434723770d5992f3599eb88c99098b24801342236fd5c16ac0a1ee` |
| Source note | `0d52690a…` (9_960_000 zat fee-dust) |
| Post-tx balance | **0.0996 ZEC** (1 note); schedule **24 transfers** |
| Preflight hint | `Scheduled transfer #1 (10000000 zat) is not fundable … next spendable transfer is #2 (1000000 zat).` |

**Operator lessons**

| Topic | Detail |
|-------|--------|
| Fee-dust phase | After splits + large turnstiles, expect **sub-canonical notes**; migration proceeds at **0.01 / 0.001 ZEC** slots, not always 0.1 ZEC |
| Build path | Verify `nozy.exe` timestamp after code changes; set **`CARGO_TARGET_DIR`** to workspace `target` when sandbox redirects builds |
| Preflight | With Path C, read the **spendability blocker line** — it names the transfer `migrate` will actually prebuild |

**Operator loop status after 2026-07-05 late**

```text
[done]  Transfers confirmed on chain: 1.0 + 0.1 + 0.01 ZEC (3 turnstiles)
[done]  Path C spendability-aware preflight + migrate
[open]  Schedule #1 (10M zat) remains pending/unfundable until note set changes
[next]  0.01 ZEC turnstiles from remaining 0.0996 note — bucket 4145664
```

#### Case 4 — Ironwood scan gap + Phase 3.4 send routing (2026-07-06)

After the Path C turnstile loop, Orchard notes on profile **Ironwood Testnet** (`5e7821be3f1d89d6`) were **fully spent**, but `ironwood status` still reported **Ironwood: 0 zat** locally despite confirmed V6 turnstile txs on chain (e.g. `ff31a7fd…` at height **4143641** with 2 `ironwood.actions`). This session shipped **Ironwood note indexing** (Phase 2 completion on JSON-RPC) and **post-activation normal-send routing** (Phase 3.4), and verified **mainnet-only** WSL zebrad autostart.

**What we were trying to do**

```text
(1) Index Ironwood pool outputs from turnstile txs so local balance matches chain
(2) Route post-NU6.3 normal sends through Ironwood (not block Send forever)
(3) Confirm mainnet WSL zebrad autostart — no testnet autostart
```

**Case breakdown — what we hit (in order)**

| # | Case | Symptom | Root cause | Fix / outcome |
|---|------|---------|------------|---------------|
| A | **Ironwood scan decrypt failure** | Full rescan `4134000..tip` → balance **0 ZEC**; `ironwood status` Ironwood **0 zat** | `NoteScanner` used `OrchardDomain::for_compact_action` for all actions; Ironwood V3 note plaintexts require **`IronwoodDomain`** (lead byte `0x03`) | `decrypt_shielded_action` branches on `ShieldedPool`; Ironwood actions use `IronwoodDomain` |
| B | **On-chain proof, wallet blind** | `getrawtransaction` on `ff31a7fd…` shows `ironwood.actions: 2`; `getblock` at confirm height includes `ironwood` bundle | Parsing was fine; decryption returned `None` for every Ironwood output | Same as A — not a Zebra `getblock` shape issue on WSL testnet |
| C | **Spendable loader pool-blind** | Even if indexed, sends would not see Ironwood notes | `load_spendable_notes_from_wallet` only accepted `orchard_incremental_witness_hex` | Pool-aware `witness_hex_for_pool()` on `SerializableOrchardNote` / `SpendableNote`; Ironwood witness fields populated during scan |
| D | **Ironwood witness on spendables ignored** | Ironwood scan passed `orchard_sk: None` into `process_block_ironwood_actions` | Ironwood notes indexed but discarded from in-memory spendable list | Pass spending key; persist `ironwood_incremental_witness_hex` + tip height on spendables and in `notes.json` |
| E | **Phase 3.4 blocked** | `orchard_only_send_blocker` rejected all sends post-activation | No `add_ironwood_spend` path in normal send builder | New `src/ironwood_tx.rs`: V6 PCZT `add_ironwood_spend` + `add_ironwood_output`; `transaction_builder` routes when `is_ironwood_active` |
| F | **`wallet_ready` stuck false** | `ironwood status` always "no (see blockers)" after fixes | `wallet_ready` hardcoded `false` in CLI | Dynamic: `ironwood_zat > 0`, `orchard_zat == 0`, no blockers |
| G | **Mainnet autostart scope** | Operator asked for mainnet autostart only — no testnet | `install-zebrad-autostart.ps1` registers `ZebradWSL-*` → `start-zebrad-wsl.ps1` (mainnet `/home/lowo/zebra-mainnet/`, `:8232`) | Verified tasks **Ready**; testnet remains **manual** WSL start (by design) |
| H | **Stale `nozy.exe` after edits** | Prior sessions saw code changes with no operator effect | Cursor sandbox may redirect `CARGO_TARGET_DIR` away from `NozyWallet\target` | Rebuild with `$env:CARGO_TARGET_DIR = "C:\Users\User\NozyWallet\target"` before operator runs |

**What we shipped**

| Layer | Change | Where |
|-------|--------|-------|
| V3 decrypt | `IronwoodDomain::for_compact_action` for `pool: Ironwood` | `decrypt_shielded_action` in `src/notes.rs` |
| Scan spendables | Ironwood notes get witnesses + spending key during scan | `process_block_ironwood_actions`, post-scan `apply_ironwood_witnesses_from_tracker` |
| Cached spendables | Load Ironwood notes with `ironwood_incremental_witness_hex` | `load_spendable_notes_from_wallet` |
| Witness refresh | Ironwood witness catch-up at sync | `refresh_ironwood_cached_witnesses_to_tip` in `src/ironwood_tx.rs`; `wallet_sync.rs` |
| Send routing | Post-NU6.3 → Ironwood V6 single-spend send | `src/ironwood_tx.rs`, `src/transaction_builder.rs` |
| Readiness | Pool-aware witness lag messages | `src/send_readiness.rs` |
| Status | `wallet_ready` when Ironwood-funded, Orchard empty | `src/main.rs` `ironwood status` |
| Autostart | Mainnet WSL only (no testnet task) | `C:\Zebrad\scripts\install-zebrad-autostart.ps1` (unchanged — verified) |

**Ironwood scan pipeline (after fix)**

```text
getblock (verbosity 2) → parse ironwood.actions
  → append cmx to IronwoodWitnessTracker
  → decrypt with IronwoodDomain + IVK (External, then Internal)
  → tag note pool: Ironwood, persist rho/rseed + ironwood witness hex
  → load_spendable_notes_from_wallet includes Ironwood pool witnesses
```

**Ironwood send pipeline (Phase 3.4)**

```text
transaction_builder.build_send_transaction
  → if is_ironwood_active: filter spendable notes pool == Ironwood
  → ZebraJsonRpcIronwoodWitnessProvider (treestate + block catch-up)
  → V6 Builder: add_ironwood_spend + add_ironwood_output (+ change)
  → PCZT: Ironwood proof + sign_ironwood + extract
```

**Live validation (2026-07-06, WSL testnet `http://172.20.199.206:18232`)**

| Check | Outcome |
|-------|---------|
| `sync --start-height 4134000 --to-tip` | Balance **1.09880000 ZEC**; **3 new** Ironwood notes indexed |
| `ironwood status` | Orchard **0 zat** \| Ironwood **109_880_000 zat** |
| `ironwood status` | **Ironwood-ready: yes** |
| On-chain turnstile `ff31a7fd…` | Confirmed height **4143641**; `ironwood.actions: 2` |
| Mainnet autostart | `ZebradWSL-StartAtLogon` + `ZebradWSL-Watchdog` → **Ready** (mainnet config only) |

**Operator lessons**

| Topic | Detail |
|-------|--------|
| Ironwood decrypt | Never use `OrchardDomain` for Ironwood pool actions — V3 notes will silently fail to decrypt |
| Rescan after fix | Rescan from NU6.3 activation (`4134000`) or first turnstile height to backfill indexed notes |
| WSL RPC | Testnet: `http://<wsl-ip>:18232` — not `127.0.0.1` from Windows |
| Build path | Set **`CARGO_TARGET_DIR`** to workspace `target` when sandbox redirects builds |
| Autostart | **Mainnet** auto-starts at logon; **testnet** zebrad is manual after reboot |

**What we deliberately did not ship**

- Desktop Send button wiring to Ironwood path (CLI `send` / API still need surface validation).
- Live broadcast of a normal Ironwood send on testnet (builder path compiles; operator smoke pending).
- Testnet WSL autostart (explicitly out of scope — mainnet only).

**Operator loop status after 2026-07-06**

```text
[done]  Ironwood V3 note decrypt + local balance (1.0988 ZEC indexed)
[done]  Phase 3.4 CLI Ironwood send routing (transaction_builder)
[done]  Mainnet-only zebrad autostart verified (ZebradWSL-*)
[done]  Live Ironwood send smoke — see below
[open]  Desktop/API Send surface wiring (desktop blockers updated 2026-07-06)
```

#### Case 4 — live Ironwood send smoke (2026-07-06 afternoon)

First **normal** post-activation send through the Ironwood pool (not a turnstile migration tx).

| Field | Value |
|-------|-------|
| Amount | **0.001 ZEC** (100_000 zat) + **40_000 zat** fee |
| Memo | `ironwood-smoke` |
| TXID | `d67940924a0c4b1d3cb6e35161f6865140d101d9af85c12df0b97f247946d7af` |
| Confirmed | Block **4148837** |
| Bundle | **2 ironwood actions**, **0 orchard actions** (pure Ironwood V6 spend) |
| Post-send balance | **1.0888 ZEC** Ironwood (108_880_000 zat) |

**Blocker hit before success**

| Symptom | Cause | Fix |
|---------|-------|-----|
| Witness lag 5189 blocks | `ironwood_incremental_witness_hex` was **empty** on cached notes (tip height set but no blob) | Rescan from first Ironwood note block (`--start-height 4143640`); shipped auto-repair in `scan_notes_for_sending` when Ironwood witness hex missing |

**Operator command (validated)**

```powershell
$z = @("--testnet", "--zebra-url", "http://<wsl-ip>:18232")
nozy @z sync --start-height 4143640 --to-tip   # if witnesses empty
nozy @z send -r <utest1…> -a 0.001 --memo "ironwood-smoke"
# confirm with: yes
```

### Case 6 — Ironwood lightwalletd must expose NU6.3 compact fields

Compact sync (Zeaking / desktop / API) depends on lightwalletd streaming **Ironwood-aware compact blocks**. JSON-RPC treestate parsing alone is not enough to validate the LWD path.

**Problem we were solving**

- Upstream [`zcash/lightwalletd`](https://github.com/zcash/lightwalletd) does not ship `ironwoodActions` or `ironwoodCommitmentTreeSize`.
- Nozy’s operator stack assumed vanilla LWD on `:9067`; testnet Ironwood validation needs **ironwood-valar** against Windows Zebrad `:18232` with WSL LWD on `:9068`.
- Without a smoke test, “LWD works” could mean “gRPC responds” while still serving pre-Ironwood compact shapes.

**What we shipped**

| Layer | What landed | Where |
|-------|-------------|-------|
| WSL install/start | `ironwood-lwd-wsl.sh` + `ironwood-lwd-wsl.ps1`; thin wrapper `start-lightwalletd-wsl.ps1` | `scripts/` |
| Proto mirror | `CompactTx.ironwoodActions`, `ChainMetadata.ironwoodCommitmentTreeSize` | `zeaking/proto/compact_formats.proto` |
| Smoke binary | `GetLightdInfo` + `GetBlockRange`; warns if post-activation range has no Ironwood tree metadata | `src/bin/ironwood_lwd_smoke.rs` |
| Smoke script | Builds binary, probes WSL `:9068`, runs range near tip | `scripts/ironwood-lwd-smoke.ps1` |
| Readiness doc | Operator path + validation table | **Ironwood lightwalletd (ironwood-valar)** section below |

**Validation pipeline**

```text
Windows Zebrad testnet :18232 (Ironwood-capable)
  → WSL ironwood-valar lightwalletd :9068 (rpcbind = Windows host IP from resolv.conf)
  → ironwood_lwd_smoke GetBlockRange (heights ≥ 4_134_000, prefer near tip)
  → expect ironwoodCommitmentTreeSize > 0 and/or ironwoodActions when present
```

**Live validation (2026-07-04)**

| Check | Outcome |
|-------|---------|
| LWD branch | `zecrocks/lightwalletd` @ `ironwood-valar` built in WSL |
| Backend | `/Zebra:6.0.0-rc.0/` via `GetLightdInfo` |
| `GetBlockRange` 4136100..tip | **`ironwoodCommitmentTreeSize=4`** (matches on-chain Ironwood pool note count) |
| Activation-height-only range | Tree size **0** at exact activation blocks — normal; scan nearer tip for smoke pass |

**What we deliberately did not ship**

- No automatic “start LWD from Nozy CLI sync” yet — operator starts WSL LWD via scripts.
- No desktop LWD URL picker for Ironwood testnet — set `LIGHTWALLETD_GRPC` manually.
- Full compact-sync-to-tip through Zeaking on testnet Ironwood notes — smoke validates **gRPC shape**, not end-to-end wallet scan over Ironwood actions.

**Case 6 decision**

Treat Ironwood LWD as **unvalidated** until `ironwood-lwd-smoke.ps1` passes against `ironwood-valar`. JSON-RPC witness/sync fixes (Case 3 live run) and LWD compact validation (Case 6) are parallel prerequisites for a complete post-NU6.3 sync story.

### Case 5 — CLI readiness should be a shared state machine

`ironwood preflight` and `ironwood migrate` need to report the same answer for the same wallet state.

Evidence and rationale:

- The CLI now has shared readiness states:
  - `planning-only`
  - `no-orchard-notes`
  - `split-required`
  - `ready-to-prebuild`
  - `waiting-for-window`
  - `presigned-waiting-for-broadcast`
  - `blocked`
- These states combine activation, schedule validation, witness freshness, source-note consistency, and ZIP 318 window eligibility.
- The shared model makes later API and desktop wiring safer because those surfaces can display the same backend decision rather than reimplementing migration logic.

Conclusion: the core CLI is now the authoritative Phase 3 migration state machine. Desktop/API should consume this behavior later instead of bypassing it.

### Phase 3 core decision

NozyWallet should keep the current Phase 3 slice conservative:

1. Validate or rebuild the saved schedule before prebuild.
2. Prepare only the next eligible scheduled transfer.
3. Persist exact presigned transfer metadata.
4. Stop with `split-required` until note splitting is complete; use `nozy ironwood split` (Phase 3.1) rather than faking split via turnstile change outputs.
5. Keep broadcast disabled until retry and confirmation handling are explicit.

This gives Nozy a safe, testable CLI foundation for Ironwood migration without prematurely exposing unsafe migration shapes to desktop or API users.

---

## Phase 3 case breakdown — active work (2026-07-06)

This section records **why** Phase 3 is underway, **what we are facing on testnet today**, and **how each case is being handled**. It complements the design cases above with operator-facing status.

### Why we are doing this

Ironwood (NU6.3) is **active on testnet** (activation height `4_134_000`). After activation, Orchard-only normal sends are no longer the correct shielded path — funds should move into the Ironwood pool via **ZIP 318**, which splits migration into:

1. **Note splitting** — Orchard send-to-self into canonical power-of-ten denominations (privacy normalization).
2. **Turnstile migration** — Orchard spend → Ironwood output (V6 txs), scheduled in shared anchor-height buckets.
3. **Broadcast** — presigned txs sent only inside their ZIP 318 windows.
4. **Normal sends** — post-activation recipient + change through Ironwood pool (Phase 3.4).

Nozy builds the **core CLI state machine first** so desktop, API, and extension surfaces can display the same backend decision instead of reimplementing migration logic.

### What we are facing (live testnet wallet)

Active profile: **Ironwood Testnet** (`5e7821be3f1d89d6`).

| Observation | Detail |
|-------------|--------|
| Balance | **1.09880000 ZEC** Ironwood (109_880_000 zat) — **0** Orchard after turnstile loop |
| Chain tip | ~4146541+ (**WSL** testnet Zebrad); RPC **`http://<wsl-ip>:18232`** |
| Zebrad | **WSL only** — `/home/lowo/zebra-testnet/zebrad.toml`, ~276 GB state |
| Ironwood | Active; Orchard → Ironwood **migration loop complete** on this profile |
| Schedule | Rebuilt after last confirmed turnstile; Orchard notes exhausted |
| Normal Send (CLI) | Routes through **Ironwood** when NU6.3 active (`ironwood_tx.rs`) |
| `ironwood status` | **Ironwood-ready: yes** |
| LWD | **ironwood-valar** on WSL `:9068` smoke-validated |
| Autostart | **Mainnet only** (`ZebradWSL-*` tasks **Ready**); testnet manual after reboot |

**Current gate:** Desktop/API Send UX parity; Keystone Ironwood hardware path (software send works).

**Resolved 2026-07-06 (afternoon):** Live Ironwood send smoke confirmed on testnet (`d6794092…`).

### Case-by-case: decision, status, fix

| Case | What we decided | Why | Status | How we fix / validate |
|------|-----------------|-----|--------|------------------------|
| **1 — Schedule validation** | Load and validate schedule before prebuild; rebuild only when drift | Orphan presigned txs | **Done** | `validate_orchard_migration_schedule` + `plan --save` guard |
| **2 — Next eligible transfer only** | Prebuild only in-window next transfer | ZIP 318 timing | **Done** | `next_eligible_pending_transfer` |
| **3 — Note splitting required** | Splittable composite gate; fee-dust non-blocking | ZIP 318 privacy | **Done** | Path B + Path C |
| **4 — Presigned + broadcast** | Window-validated `ironwood broadcast` | Timing + reconciliation | **Done** | 3+ turnstiles confirmed; see evening + late + **2026-07-06 scan** cases |
| **5 — Shared state machine** | `preflight` and `migrate` agree | Surface parity | **Done** | Path C spendability alignment |
| **6 — Ironwood LWD compact fields** | Validate `ironwood-valar` before compact sync | Proto shape | **Smoke validated** | `ironwood-lwd-smoke.ps1` |
| **7 — WSL node + autostart** | Zebrad in WSL; mainnet autostart only | Split-brain RPC | **Done (mainnet)** | `ZebradWSL-*` → mainnet; testnet **manual** by operator choice |
| **8 — Fee-dust spendability (Path C)** | Skip unfundable slots | Dust vs canonical plan | **Done** | `next_spendable_eligible_pending_transfer` |
| **9 — Ironwood scan + Phase 3.4 send** | Index Ironwood V3 notes; route normal sends | Turnstile outputs invisible locally; Send blocked forever | **Done + smoke validated** | See **Case 4 — 2026-07-06** + **live send smoke** |

### Execution order (remaining slices)

```text
3.0  State machine + V6 prebuild     [done]
3.1  ZIP 318 note splitting           [done]
3.2  Turnstile prebuild at bucket    [done — migration loop complete on profile]
3.3  Broadcast in anchor windows    [done]
3.3b Spendability-aware transfer pick [done — Path C]
3.4  Post-activation Send → Ironwood  [done — CLI smoke validated]
3.5  Ironwood JSON-RPC scan/index     [done — IronwoodDomain decrypt]
3.6  Ironwood LWD compact smoke       [done — ironwood-valar validated]
3.7  WSL testnet autostart            [declined — mainnet autostart only]
4.x  Desktop/API Send + migrate UX    [next]
```

### Operator commands (testnet validation loop)

```powershell
# WSL testnet Zebrad — resolve IP after reboot:
$wslIp = (wsl -d Ubuntu -- hostname -I).Split()[0].Trim()
$z = @("--testnet", "--zebra-url", "http://${wslIp}:18232")

nozy @z sync --to-tip
nozy @z ironwood status
nozy @z ironwood plan --save
nozy @z ironwood preflight    # expect split-required until split loop complete

# Case 3 note split loop (witness rescan if split fails treestate check):
nozy @z sync --to-tip
# If witness mismatch persists: rescan from note block, e.g.:
# nozy @z sync --start-height 4130900 --to-tip
nozy @z ironwood split --dry-run
nozy @z ironwood split            # repeat per note; wait for mempool confirm between splits
nozy @z sync --to-tip
nozy @z ironwood plan --save
nozy @z ironwood preflight

# Case 6 LWD smoke (parallel / before compact sync):
powershell -ExecutionPolicy Bypass -File scripts\ironwood-lwd-smoke.ps1

# Turnstile loop complete — verify Ironwood balance after rescan:
nozy @z sync --start-height 4134000 --to-tip
nozy @z ironwood status           # expect Ironwood zat > 0, Ironwood-ready: yes

# Normal send (CLI routes to Ironwood post-activation — smoke pending):
# nozy @z send <ua> <amount>

# Rebuild CLI after code changes (sandbox may redirect CARGO_TARGET_DIR):
#   $env:CARGO_TARGET_DIR = "C:\Users\User\NozyWallet\target"
#   cargo build --release -p nozy
```

---

## Ironwood lightwalletd (ironwood-valar)

NozyWallet compact sync depends on lightwalletd exposing **Ironwood compact fields** from NU6.3 blocks. Upstream [`zcash/lightwalletd`](https://github.com/zcash/lightwalletd) does not yet ship `ironwoodActions` / `ironwoodCommitmentTreeSize`; use the Ironwood branch maintained by Zec.rocks:

- **Repo:** [zecrocks/lightwalletd @ `ironwood-valar`](https://github.com/zecrocks/lightwalletd/tree/ironwood-valar)
- **Proto additions:** `CompactTx.ironwoodActions`, `ChainMetadata.ironwoodCommitmentTreeSize` (mirrored in `zeaking/proto/compact_formats.proto`)
- **Backend:** Ironwood-capable **zebrad** JSON-RPC (testnet `:18232`)

### WSL operator path (testnet)

When zebrad runs on **Windows** and lightwalletd runs in **WSL**, RPC must target the Windows host IP (not `127.0.0.1` from inside WSL). The helper script resolves this from `/etc/resolv.conf`:

```powershell
# First time: build ironwood-valar in WSL
powershell -ExecutionPolicy Bypass -File scripts\start-lightwalletd-wsl.ps1 -Testnet -Install

# Start testnet LWD (gRPC :9068 → Windows zebrad :18232)
powershell -ExecutionPolicy Bypass -File scripts\start-lightwalletd-wsl.ps1 -Testnet

# Status / stop
powershell -ExecutionPolicy Bypass -File scripts\start-lightwalletd-wsl.ps1 -Testnet -Status
powershell -ExecutionPolicy Bypass -File scripts\start-lightwalletd-wsl.ps1 -Testnet -Stop
```

From Windows, point Nozy at WSL’s IP:

```powershell
$env:LIGHTWALLETD_GRPC = "http://<wsl-ip>:9068"
```

### Validation smoke test

`GetBlockRange` over heights at or after NU6.3 activation (`4_134_000`) must return blocks whose compact txs may include **`ironwoodActions`** and chain metadata with **`ironwoodCommitmentTreeSize`**:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\ironwood-lwd-smoke.ps1 -InstallLwd -StartLwd
# or, if LWD already running:
powershell -ExecutionPolicy Bypass -File scripts\ironwood-lwd-smoke.ps1
```

Direct binary:

```powershell
cargo build --release --bin ironwood_lwd_smoke
.\target\release\ironwood_lwd_smoke.exe http://<wsl-ip>:9068 4134000 4134002
```

| Check | Pass | Fail / warn |
|-------|------|-------------|
| `GetLightdInfo` | Returns testnet chain + tip | gRPC unreachable → start LWD / fix firewall |
| `GetBlockRange` ≥ activation | ≥1 block; `ironwoodCommitmentTreeSize` > 0 near tip (or `ironwoodActions` when present) | Empty range → LWD behind zebrad; tree=0 at exact activation height is normal — scan nearer tip |

Until this smoke passes, treat **Ironwood LWD compact sync** as unvalidated even if JSON-RPC treestate parsing works.

**Validated (2026-07-04):** `ironwood-valar` LWD on WSL `:9068` against Windows testnet Zebrad `:18232`; `GetBlockRange` 4136100..tip returned `ironwoodCommitmentTreeSize=4` (4 Ironwood notes on chain).

### Side issues noted (non-blocking)

- **`sent_transactions.json` parse warnings** on desktop startup (EOF) — history file intermittently read empty; app continues; file on disk is valid JSON.
- **Send password toast** was misleading for Ironwood blockers — fixed in desktop error mapping + SendForm Ironwood pre-check banner.

---

## Progress snapshot — 2026-07-03

### Done

- Added Ironwood / NU6.3 dependency pins for the core Zcash crate cohort.
- Added `.cargo/config.toml` with `zcash_unstable="nu6.3"`.
- Updated Orchard proving / verifying key setup for the current `orchard` API.
- Updated Orchard transaction builder calls for `BundleVersion` / flags API drift.
- Added `ShieldedPool::{Orchard, Ironwood}` and persisted pool tagging on notes.
- Added Ironwood status / plan / migrate CLI scaffolding under `nozy ironwood`.
- Added ZIP 318-aware migration planning scaffolding:
  - canonical power-of-ten denominations
  - anchor bucket boundary helper
  - schedule summary model
  - draft per-transfer anchor-bucket schedule persisted as `ironwood_migration_schedule.json`
  - locked V6 prebuild path for Orchard V2 spend → Ironwood V3 output via PCZT proving/signing/extraction
- Added pre-activation readiness controls: clearer `ironwood status`, `ironwood preflight`, `ironwood plan --save`, and missed-window schedule refresh before migration prebuild.
- Fixed the CLI incremental sync cache/reporting path after the operator caught a false `0 ZEC` result following a valid saved Ironwood migration plan.
- Added pool-aware Zebra treestate parsing for `z_gettreestate`.
- Added `ZebraClient` accessors for Ironwood treestate and tree state.
- Added Ironwood tree codec and witness tracker wrappers.
- Added separate Orchard and Ironwood witness persistence fields on cached notes.
- Updated block parsing / scan plumbing so `orchard.actions` and `ironwood.actions` stay separate.
- Added parser tests for Ironwood treestate and Ironwood action extraction.
- Added desktop Home Ironwood readiness card with activation/RPC/wallet-pool status and disabled migration controls.
- Added runtime safety gates that block Orchard-only normal sends after Ironwood activation; CLI routes to Ironwood via Phase 3.4 (`ironwood_tx.rs`).
- **2026-07-06:** Ironwood V3 scan fix (`IronwoodDomain` decrypt); pool-aware spendable load; mainnet-only WSL autostart verified.

### Validated

```bash
cargo fmt --all
cargo test -p nozy --lib
cargo build
```

Latest full library test run: **86 passed, 8 ignored**.

### Partially done / blocked

- Phase 2 scan code compiles and has live NU6.3 testnet validation; **Ironwood note receive/index validated** on profile `5e7821be3f1d89d6` (1.0988 ZEC Ironwood after rescan from `4_134_000`).
- Ironwood notes are indexed **and** loadable as spendable via pool-aware witness fields; live normal Ironwood send broadcast still pending operator smoke.
- Normal Orchard-only sends are blocked after Ironwood activation; CLI **`transaction_builder`** routes to Ironwood when NU6.3 active (Phase 3.4).
- Local testnet Zebra setup was started in `C:\Users\User\nozy-zebra-testnet`:
  - `zebrad.toml` configured for Testnet.
  - RPC configured on `127.0.0.1:18232`.
  - state directory configured under `C:\Users\User\nozy-zebra-testnet\state`.
  - helper scripts created: `start-testnet-zebra.ps1`, `check-testnet-zebra.ps1`.
- Installed `zebrad 4.3.1` was tested and synced from genesis, but it is **not Ironwood-capable** (`getblockchaininfo` exposed upgrades only through NU6.1 and no `ironwood` value pool).
- `zebrad 5.2.0` compiled successfully with Rust stable `1.96.1`, but the final binary copy was interrupted / incomplete after the power outage. The testnet config survived; reinstall the isolated binary before restarting testnet.

### Next immediate steps

1. Restore the isolated Ironwood-capable Zebra binary:

   ```powershell
   rustup run stable cargo install --locked zebrad --version 5.2.0 --root "C:\Users\User\nozy-zebra-testnet\zebra-bin" --force
   ```

2. Start testnet Zebra:

   ```powershell
   C:\Users\User\nozy-zebra-testnet\start-testnet-zebra.ps1
   ```

3. Verify RPC reports Ironwood:

   ```powershell
   C:\Users\User\nozy-zebra-testnet\check-testnet-zebra.ps1
   ```

4. Confirm `getblockchaininfo.valuePools` contains `ironwood`, then run read-only Nozy scan / status checks against `http://127.0.0.1:18232`.

---

## Operator / miner timeline

This is the schedule ZF/ZODL shared with miners for feedback:

| Date | Milestone | Nozy implication |
|------|-----------|------------------|
| 2026-07-01 | Ironwood / NU6.3 testnet deployment | Dependency and testnet stack must track Ironwood branches |
| 2026-07-03 | Ironwood / NU6.3 testnet activation | Use testnet to validate scan, witness, send, migration |
| 2026-07-09 | Ironwood / NU6.3 mainnet deployment | Mainnet-compatible release candidate needed |
| 2026-07-15 | `zcashd 6.20.0` end of support | Nozy must assume Zebra-only node operators |
| 2026-07-28 | Ironwood / NU6.3 mainnet activation (height 3,428,143) | Migration UX and post-activation send routing must be ready |

Current `zcash_protocol` mainline facts:

- **Testnet NU6.3 height:** `4_134_000`
- **Mainnet NU6.3 height:** not configured yet (`None`)
- **NU6.3 branch ID:** `0x37a5165b`

Nozy now reads activation heights from `zcash_protocol` instead of hard-coding a speculative mainnet height.

---

## Implementation phases

### Phase 1 — NU6.3 dependency stack (done)

Nozy currently tracks the `librustzcash` mainline commit that includes the Ironwood/V6 builder APIs:

| Dependency | Pin |
|------------|-----|
| `zcash_primitives`, `zcash_protocol`, `zcash_address`, `zcash_keys`, `zcash_proofs`, `zcash_transparent`, `pczt` | `zcash/librustzcash@4d9a68dc80508e7644aa99e1b4add7c831057bba` |
| `orchard` | `0.15.0-pre.1` |

`.cargo/config.toml` enables the required unstable cfg:

```toml
[build]
rustflags = ["--cfg", "zcash_unstable=\"nu6.3\""]
```

Fixes applied:

- `ProvingKey::build(OrchardCircuitVersion::FixedPostNu6_2)` in `orchard_tx.rs`, `keystone.rs`
- `VerifyingKey::build(OrchardCircuitVersion::FixedPostNu6_2)` in `keystone.rs`
- `Pczt::serialize()?` error handling in `keystone.rs`
- `FeeRule::fee_required` includes `_ironwood_action_count`
- Direct Orchard builder usage now passes `BundleVersion::orchard_v2()` and version default flags
- Single active Orchard crate (`0.15.0-pre.1`)

Validation:

```bash
cargo check -p nozy
```

### Phase 2 — Ironwood tree + scan (started, compiles)

| File | Work |
|------|------|
| `zebra_tree_rpc.rs` | Parse `z_gettreestate` for `pool=ironwood` |
| `zebra_integration.rs` | `get_ironwood_treestate_parsed`, `get_ironwood_tree_state` |
| `ironwood_tree_codec.rs` | Witness serde (mirror `orchard_tree_codec.rs`) |
| `ironwood_witness.rs` | `IronwoodWitnessTracker` during scan |
| `notes.rs` / `NoteScanner` | Decrypt Ironwood actions; tag `pool: Ironwood` |
| `wallet_sync.rs` | Dual-tree witness refresh |

Current Phase 2 progress:

- `z_gettreestate` parsing is pool-aware and supports `ironwood.commitments`.
- `ZebraClient` exposes `get_ironwood_treestate_parsed` and `get_ironwood_tree_state`.
- `SerializableOrchardNote` now has separate Orchard and Ironwood witness fields.
- `NoteScanner` parses separate `ironwood.actions` / `ironwoodActions` streams, decrypts with **`IronwoodDomain`** for V3 notes, and tags discovered notes as `pool: Ironwood`.
- Ironwood notes are indexed with pool-specific witnesses and loadable via `load_spendable_notes_from_wallet`.
- Live NU6.3 testnet validation: Orchard restore/sync; **Ironwood turnstile outputs indexed** (1.0988 ZEC on profile `5e7821be3f1d89d6`); normal Ironwood send broadcast smoke pending.

### Phase 3 — ZIP 318 turnstile migration + send routing (3–5 days)

ZIP 318 PR #1317 changes this from "send all now" to a privacy-preserving scheduled migration:

| ZIP 318 requirement | Nozy work |
|---------------------|-----------|
| Dedicated migration entry point with user consent | CLI scaffold done; API/desktop needed |
| Show Orchard-pool-specific balance at risk | CLI `status`/`plan` scaffold done |
| Phase 1 note splitting as wallet-internal send-to-self | **CLI shipped** — `nozy ironwood split` |
| Phase 2 canonical power-of-ten denominations | Plan scaffold in `src/ironwood/migration.rs` |
| Shared anchor-height buckets | Draft interval constant `256` blocks in `migration.rs` |
| Persisted schedule + pre-signed txs | Schedule state + first raw V6 tx prebuild started |
| Broadcast-only background task; no sync in broadcast task | Needed for desktop/mobile/extension |
| Tor/VPN network-privacy step | Needed |
| Fallback on app open for missed windows | Needed |

Transaction builder work:

- `add_orchard_spend` (legacy V2 note) + `add_ironwood_output` (V3) for migration
- `add_ironwood_spend` + `add_ironwood_output` for **normal sends** post-activation (`src/ironwood_tx.rs`)
- `BuildConfig::Standard { ironwood_anchor: Some(...), .. }`
- `TxVersion::V6` after NU6.3 activation height
- Runtime guard: Orchard-only normal sends blocked after Ironwood activation; CLI routes to Ironwood builder (Phase 3.4).
- Wire `src/ironwood/migration.rs::execute_orchard_migration`
- Persist schedule state and pre-signed raw transactions
- Detect expired/invalid transfers and rebuild the remaining schedule

References: [librustzcash #2498](https://github.com/zcash/librustzcash/pull/2498), [ZIP 318 PR #1317](https://github.com/zcash/zips/pull/1317), Valar `zodl_ironwood_migration` (iOS SDK #1793).

Post-activation **normal sends**: route recipient + change through Ironwood builder (`ironwood_tx.rs`) — **CLI shipped 2026-07-06**; desktop/API wiring remains.

### Phase 4 — Surfaces (1–2 days)

| Surface | Command / route |
|---------|-----------------|
| CLI | `nozy ironwood status` · `plan` · `migrate` (**status/plan shipped**) |
| API | `GET /api/ironwood/status`, `POST /api/ironwood/migrate` |
| Desktop | Read-only Home readiness card added; one-click migrate remains disabled until Phase 3 |
| Extension | WASM core parity (separate `wasm-core` dep bump) |

---

## Shipped today (scaffolding)

- `src/shielded_pool.rs` — `ShieldedPool::Orchard | Ironwood`
- `src/ironwood/` — status, ZIP 318-aware migration plan, CLI commands
- `ironwood_migration_schedule.json` — draft per-wallet migration queue once `migrate` is invoked after activation
- `notes.json` — optional `pool` field (defaults Orchard)
- `get_ironwood_pool_stats()` — when node reports ironwood `valuePools`
- `ENHANCEMENT_ROADMAP.md` — Ironwood track #0

---

## Operator checklist (activation week)

1. Upgrade **Zebrad** to NU6.3 release (watch [Zebra releases](https://github.com/ZcashFoundation/zebra/releases)).
2. Upgrade **lightwalletd** for Ironwood compact blocks.
3. Run `nozy ironwood status` — confirm activation height matches node.
4. `nozy sync --to-tip` before migrate.
5. `nozy ironwood plan` — review ZIP 318 denomination/schedule preview.
6. `nozy ironwood migrate` (once Phase 3 lands).
7. Re-sync; verify Ironwood balance.

---

## Testnet wallet setup

Use a dedicated profile for Ironwood validation so testnet notes, witnesses, and schedules do not mix with mainnet state:

```powershell
nozy testnet-wallet new --rpc-url http://127.0.0.1:18232
nozy profile list
nozy --testnet receive
nozy --testnet sync --to-tip
nozy --testnet ironwood plan
nozy --testnet ironwood migrate
```

If an existing profile already holds tZEC, select it instead:

```powershell
nozy profile list
nozy testnet-wallet use --profile-id <profile-id> --rpc-url http://127.0.0.1:18232
nozy --testnet sync --to-tip
```

---

## Activation height

`src/ironwood/mod.rs` uses `zcash_protocol::consensus::{MAIN_NETWORK, TEST_NETWORK}`:

- Mainnet currently reports `None`; show date target only until the final height lands.
- Testnet currently reports `Some(4_134_000)`.

---

## Testing

- [ ] Regtest / testnet with NU6.3-enabled zebrad
- [ ] ZIP 318 note-splitting send-to-self confirms on testnet (CLI `ironwood split` shipped; live broadcast pending sync + proving run)
- [ ] Migration single-denomination turnstile tx broadcast
- [ ] Scheduled/pre-signed migration retry after missed window
- [ ] Post-migration send to existing UA → Ironwood receive
- [x] Ironwood turnstile outputs indexed locally after rescan (JSON-RPC, `IronwoodDomain`)
- [x] Live normal Ironwood send broadcast on testnet (CLI `send` — TXID `d6794092…`, block 4148837)
- [ ] Post-activation send rejected if building Orchard-only outputs (Orchard path still pre-activation only)
- [ ] Desktop smoke extended (Ironwood banner + migrate flow)

---

## Release wrap-up — v2.4.0 Ironwood (CLI)

**Published:** 2026-07-06  
**Tag:** [v2.4.0](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/v2.4.0)  
**Download:** [nozy-windows.exe](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/nozy-windows.exe) · Linux · macOS (see release assets)

This section closes the case-breakdown arc that started as a design question — *how should an Orchard-first wallet move users into Ironwood without harming privacy?* — and ends with a production CLI release on testnet-validated Ironwood infrastructure.

### How the cases led here

The migration case breakdown (Cases 1–4 at the top of this doc) was not academic. Each case became an engineering constraint that showed up again in live testnet work:

| Case | Design question | What v2.4.0 proves |
|------|-----------------|---------------------|
| **1 — Do nothing** | Can we lag after NU6.3? | No — post-activation sends route to Ironwood; Orchard-only normal sends are blocked once active |
| **2 — Migrate everything at once** | One-shot migration? | Rejected — ZIP 318 turnstiles, canonical denominations, and anchor buckets instead |
| **3 — Scheduled turnstile migration** | Privacy-aware migration? | **Shipped** — `ironwood plan` / `migrate` / `broadcast` state machine; 3+ confirmed turnstiles on profile `5e7821be3f1d89d6` |
| **4 — Post-migration normal sends** | Life after migration? | **Shipped** — V3 `IronwoodDomain` scan, pool-aware witnesses, V6 `ironwood_tx` send path; live smoke `d6794092…` block **4148837** |

Below those product cases, the operator case log (restore sync, treestate parsing, split loops, presigned broadcast, witness repair, LWD smoke) was the day-to-day evidence that the design could survive real nodes, real proving time, and real wallet state — not just compile checks.

```text
Cases 1–4 (product)     → ZIP 318 posture + Ironwood as the new shielded default
Cases A–H (2026-07-06)  → IronwoodDomain decrypt, witness repair, Phase 3.4 routing
Live send smoke         → normal Ironwood spend confirmed on testnet
v2.4.0 release          → CLI binaries for Windows, Linux, macOS; CI-green master at tag
```

### What shipped in v2.4.0

| Area | Delivered |
|------|-----------|
| **NU6.3 stack** | `orchard` 0.15-pre, librustzcash git pins, `.cargo/config.toml` (`zcash_unstable="nu6.3"`) |
| **Scan / balance** | Ironwood V3 note decrypt, pool-tagged `notes.json`, Ironwood witness fields |
| **Migration** | ZIP 318-aware schedule, split, turnstile prebuild, window broadcast |
| **Send** | Post-activation routing through `ironwood_tx.rs` (V6 PCZT) |
| **Operator tooling** | `ironwood status` / `plan` / `preflight` / `migrate` / `broadcast`; LWD smoke scripts |
| **Docs** | This readiness log — case breakdown + live validation trail |

**Not in v2.4.0 (by design):** desktop Send wiring, Keystone Ironwood hardware sign path, API server promotion on releases. The CLI remains the authoritative state machine; surfaces should consume it, not reimplement it.

### Validation snapshot at release

| Check | Result |
|-------|--------|
| Orchard → Ironwood migration loop | Complete on testnet profile; Orchard balance **0** |
| Ironwood balance indexed | **1.0988 ZEC** after rescan (`IronwoodDomain` fix) |
| `ironwood status` | **Ironwood-ready: yes** |
| Normal Ironwood send | TXID `d67940924a0c4b1d3cb6e35161f6865140d101d9af85c12df0b97f247946d7af` |
| ironwood-valar LWD | Smoke-validated (`ironwoodCommitmentTreeSize` at tip) |
| Mainnet zebrad autostart | WSL `ZebradWSL-*` tasks **Ready** (mainnet only) |

### Reflection — implementing Ironwood as a first-year wallet developer

Shipping Ironwood in NozyWallet was one of the most meaningful stretches of work in the project's first year — not because every surface is finished, but because the work sits on a real protocol transition in Zcash history.

NU6.3 / Ironwood is not a cosmetic upgrade. It is a new shielded pool, a new note version, new transaction semantics, and a wallet migration story (ZIP 318) written explicitly to protect user privacy during the move. Being able to take NozyWallet — an Orchard-first, Zebra-backed wallet — through that transition on testnet, from "Orchard notes still spendable" through turnstile migration to a confirmed normal Ironwood send, felt like participating in the ecosystem's next chapter rather than reading about it.

A few things made the experience genuinely good, even when it was hard:

- **Clear product cases.** Cases 1–4 gave a vocabulary for trade-offs (do nothing, migrate all at once, scheduled turnstiles, post-migration sends). When something failed in testnet — empty Ironwood balance after turnstiles, witness hex missing, send blocked post-activation — we could name which case broke and fix the right layer instead of guessing.
- **Evidence over optimism.** Every milestone in this doc ties to a command, a TXID, a block height, or a script output. That discipline matters for a young wallet team building on pre-release `librustzcash` pins.
- **Standing on real infrastructure.** WSL Zebrad, ironwood-valar lightwalletd, local witness derivation, and the existing Nozy proving stack meant Ironwood was integrated into a wallet that already cared about shielded-first usage — not bolted onto a demo.
- **Release as closure.** Publishing **v2.4.0** after CI green and a retagged release turned months of case notes into something other developers and operators can download and run.

As a first-year wallet developer, Ironwood was the right level of difficulty: hard enough to force deep learning (domains, pools, V6 PCZT, migration windows), bounded enough to ship a CLI milestone before mainnet activation (~2026-07-28 / height 3,428,143). The remaining work — desktop UX, hardware signers, compact sync end-to-end — is real, but it extends a validated core instead of replacing a guess.

### What comes next (post–v2.4.0)

1. **Mainnet week** — upgrade Zebrad + LWD, confirm activation height, run the same operator loop on mainnet profiles.
2. **Desktop / API** — wire Send and migration UX to the CLI state machine (read-only Home card exists; actions still gated).
3. **Keystone** — Ironwood hardware sign path (software send already validated).
4. **Compact sync** — full Zeaking scan over Ironwood actions once LWD path is exercised beyond smoke.

### Closing

The case breakdown started as a privacy decision document. It ends as a release note backed by testnet proof: NozyWallet can scan Ironwood, migrate Orchard funds through ZIP 318-shaped turnstiles, and send normally in the Ironwood pool from the CLI.

That is a good place to be one year into wallet development — and a good foundation for mainnet.

---

## Local-only / untracked material (indexed 2026-07-14)

### Priority 2 — cover estimator (local artifacts)

Public-chain cover trial writeup + data (untracked / local):

| Path | Role |
|------|------|
| [`COVER_ESTIMATOR_TESTNET_RESULTS.md`](COVER_ESTIMATOR_TESTNET_RESULTS.md) | Testnet scan 4134000→tip; Cover(D,B) tables |
| `docs/reference/cover_estimator_*.json` / `cover_estimator_analysis.txt` | Raw scan outputs |
| `scripts/cover_estimator_scan.py`, `cover_estimator_analyze.py`, `cover_estimator_dual_report.py` | Estimator tooling |
| Forum post draft / generators | `IRONWOOD_FORUM_POST.docx`, `generate_ironwood_forum_doc.py`, [`SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md`](SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md) |

### API / desktop WIP (untracked or modified)

| Path | Role |
|------|------|
| `api-server/src/ironwood_handlers.rs` | HTTP Ironwood status/plan/migrate/broadcast surface |
| `api-server/src/wallet_profile_handlers.rs` | Multi-profile / network wallet status |
| `src/ironwood/migration.rs`, `network_privacy.rs` | Migration + safer-migration modes |
| `scripts/ironwood-lwd-wsl.*`, `install-zebra-v6-wsl.sh` | Operator WSL scripts |
| Local `lightwalletd/` tree, `zebrad` binary | Runtime copies — not source of truth |

### Session omnibus

Also see [`SESSION_NYM_IRONWOOD_DESKTOP_CASE_BREAKDOWN.md`](SESSION_NYM_IRONWOOD_DESKTOP_CASE_BREAKDOWN.md) for Jul 11 desktop MVP + Nym wiring.

---

*Last updated: 2026-07-14 (local artifact index). Prior wrap-up: 2026-07-06 (v2.4.0 CLI).*
