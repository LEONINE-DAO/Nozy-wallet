# NozyWallet — production-ready checklist (≈$80k retroactive grant gate)

**Purpose:** Single tracker for when CLI, desktop, extension, mobile, companion API, and shared core are **production-ready enough** to support a **~USD $80k** Zcash-style **retroactive** grant ask.

**How to use:** Check boxes as you finish. Put PR / release / TXID / URL after the item when useful. Do not mark a surface **Done** on the scoreboard until its Definition of done is complete.

**Related (do not duplicate work — link out):**
| Doc | Role |
|-----|------|
| [`PRODUCTION_CHECKLIST.md`](PRODUCTION_CHECKLIST.md) | Short cross-repo status table |
| [`SECURITY_REVIEW.md`](SECURITY_REVIEW.md) | Pre-release security review |
| [`nozy-mobile/STORE-CHECKLIST.md`](nozy-mobile/STORE-CHECKLIST.md) | Mobile store phases |
| [`browser-extension/store-assets/README.md`](browser-extension/store-assets/README.md) | Extension store assets |
| [`docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md`](docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md) | Mainnet send field evidence |
| [`docs/reference/IRONWOOD_WALLET_READINESS.md`](docs/reference/IRONWOOD_WALLET_READINESS.md) | Ironwood / NU readiness |
| [`docs/reference/MOBILE_HOSTED_ZEBRAD_FUNDING_CASE_BREAKDOWN.md`](docs/reference/MOBILE_HOSTED_ZEBRAD_FUNDING_CASE_BREAKDOWN.md) | Hosted Zebrad funding gate |
| `docs/grant/` (local) | Dollar budgets / Lockbox text — keep private |

**Honest product stance (grant copy):** Nozy is a **wallet + companion stack**, not a consensus node. Shielded-first on Zebrad + lightwalletd with local witness derivation. Optional Nym network privacy is **opt-in**, not default full routing. Desktop remains **beta until Ironwood GA**. No third-party audit claimed until one exists. No Chrome Web Store / Play publication claimed until published.

---

## Verification log

| Date | What | Result |
|------|------|--------|
| 2026-07-20 | Repo evidence audit vs this checklist ([agent pass](365e607a-dafa-4d00-8179-f91185aff1c9)) | Many engineering items DONE; store/sign-off/grant pack OPEN |
| 2026-07-20 | `cargo fmt --all -- --check` (local) | **PASS** |
| 2026-07-20 | `cargo test --lib --bins` (local) | **PASS** (exit 0) |
| 2026-07-20 | `cargo build --release --bin nozy` + `-p nozywallet-api` | **PASS** |
| 2026-07-20 | API default bind → `127.0.0.1` + `NOZY_BIND_ADDR`; seed policy documented | **Done** |
| 2026-07-20 | Security contact set to **Nozywallet.support@leoninedao.org** | **Done** |
| 2026-07-20 | Extension send recording + Zingo receive + CLI fee TXID cataloged | [`docs/reference/grant-evidence/`](docs/reference/grant-evidence/README.md) |
| 2026-07-27 | Nym × Ironwood network privacy (PR #178) | **Shipped:** baseline hygiene, opt-in mixnet broadcast, opt-in dVPN sync; desktop Settings → Network privacy; Case A1 local/LAN Zebrad stays direct. Still opt-in — not fully productized for all sync |
| 2026-07-27 | Ironwood docs / landing / book (PRs #175/#177) | **Shipped** narrative + readiness docs; mainnet activation still **2026-07-28** @ height **3,428,143** (upcoming as of log date) |
| 2026-07-27/28 | Dependabot security cleanup (PRs #179–#182) | **Progress:** quinn-proto, mobile overrides, wasm-poc removed (GHSA Orchard soundness), libcrux-chacha in dVPN spike; Axum 0.8 route param fix (#180). `cargo audit` hard-zero still open |
| 2026-07-27 | Grant one-pager / evidence catalog refreshed | [`ONE_PAGER.md`](docs/reference/grant-evidence/ONE_PAGER.md) dated 2026-07-27; Ironwood + Nym called out honestly |

**Next P0 blockers (do these next):**
1. ~~Publish extension GitHub Release zip~~ — **Done:** [extension-v0.1.8](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/extension-v0.1.8) (store submit still open — [`STORE_SUBMISSION_CHECKLIST.md`](browser-extension/STORE_SUBMISSION_CHECKLIST.md))
2. Finish Android EAS / device smoke; keep grant scope **Android-first** — [`ANDROID-EAS-SMOKE.md`](nozy-mobile/ANDROID-EAS-SMOKE.md)
3. Pick ZCG / Lockbox / forum target; hours sketched in [`BUDGET_80K.md`](docs/reference/grant-evidence/BUDGET_80K.md) ($80k baseline / ~$100k stretch)
4. Run dynamic-fee A′ testnet soak; counters live via `GET /api/pilot/metrics` — [`DYNAMIC_FEE_A_PRIME_SOAK.md`](docs/rfcs/DYNAMIC_FEE_A_PRIME_SOAK.md). **A2 Zeaking move remains blocked.**
5. *(Narrative)* Ironwood readiness + Nym opt-in + ZNS/merchant — do not overclaim GA, full mixnet, or in-wallet ZNS claim

---

## Scoreboard (roll-up)

Mark each row **Done** only when that section’s Definition of done is met.

| # | Surface / gate | Status | Notes |
|---|----------------|--------|-------|
| 0 | Shared core + CI | ☐ Partial | fmt/test/release build/CI strong; Dependabot #179–#182 hygiene progress; cargo audit hard-zero still open |
| 1 | CLI (`nozy`) | ☐ Partial | Commands + mainnet evidence + dynamic-fee TXID; ZNS `--to` name resolve; grant-window release polish open |
| 2 | Companion API (`api-server`) | ☐ Partial | Engineering: seed policy, CI smoke, `NOZY_PRODUCTION`⇒API key, release assets. **Do not public-claim GA yet** (see §2 note). Chain parity sign-off open; hosted live open |
| 3 | Desktop (Tauri) | ☐ Partial | Beta until Ironwood GA; Network privacy Settings shipped (PR #178); code signing + path allowlist before GA |
| 4 | Browser extension (MV3) | ☐ Partial | Release zip **extension-v0.1.8** shipped; store icons/screenshots/submit still open |
| 5 | Mobile (Expo) | ☐ Partial | Production profile + Sell mode; EAS device smoke + store still open — Android-first |
| 6 | Security & privacy | ☐ Partial | Internal SECURITY_REVIEW signed; contact **Nozywallet.support@leoninedao.org**; Nym opt-in + dep cleanup; **no** third-party audit |
| 7 | Docs & operator runbooks | ☐ Partial | Connectivity + Ironwood + Nym + vendor Sell guide; start-here matrix + birthday runbook thin |
| 8 | Releases & distribution | ☐ Partial | Extension zip ✅; mobile artifact / desktop signed publish / store submit open |
| 9 | Grant evidence pack ($80–100k) | ☐ Partial | One-pager + budget hours refreshed 2026-08-01; pick forum/ZCG process; EAS smoke evidence |

**Go / no-go for $80–100k ask:** All of **0–8** Done (or explicitly scoped out with written justification). **9** Done with links ready to paste into the grant form.

---

## 0. Shared core, Zebrad/LWD, CI

**Definition of done:** A stranger can clone, build, and run tests; mainnet scan/send invariants are documented and CI-green for workspace members.

### Core library (`nozy`)
- [x] `cargo fmt --all -- --check` clean — **2026-07-20 local PASS**
- [x] `cargo build --release` for `nozy` + `nozywallet-api` — **2026-07-20 local PASS**
- [x] `cargo test --lib --bins` green — **2026-07-20 local PASS**
- [x] Orchard-only product stance documented — `src/privacy.rs` + `book/src/security/best-practices.md` (Orchard-only; Sapling not supported). Env `NOTE_SCAN_INCLUDE_SAPLING` not used; stance is hard Orchard-only.
- [x] Clippy: `zeaking` + `nozywallet-api` stay `-D warnings` in CI; `nozy` clippy debt tracked in [`PRODUCTION_CHECKLIST.md`](PRODUCTION_CHECKLIST.md)
- [x] No seed / mnemonic / spending key in logs or error strings (grep pass) — **2026-07-20** SECURITY_REVIEW §1 (CLI create print is intentional UX)

### Zebrad / lightwalletd
- [x] Supported versions / connectivity documented — [`docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md`](docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md) + [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](ZEBRAD_SHIELDED_SEND_LIMIT.md)
- [x] Scan path uses documented treestate + local witness approach — [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](ZEBRAD_SHIELDED_SEND_LIMIT.md)
- [x] Fresh mainnet sync → balance → send smoke on operator stack — June 2026 runs in [`MAINNET_SEND_READINESS_EVIDENCE.md`](docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md) (TXID prefixes `5a03fbd1…`, `902cf006…`)
- [x] Witness lag / sync-before-send guards verified — same evidence paper + BUG-2026-011 notes
- [x] One-node policy in ops docs — [`AGENTS.md`](AGENTS.md) + connectivity doc

### `zeaking` / compact sync
- [x] `zeaking` builds in workspace CI — clippy + workspace members
- [x] LWD compact sync documented for desktop / API / FFI — [`zeaking/README.md`](zeaking/README.md) “Nozy integration”
- [x] Stale-cache / prune behavior documented — CLI `nozy lwd prune` + `zeaking` prune helpers (README integration section is thin but command/docs exist)

### Supply chain
- [x] `cargo audit` run posture documented — CI soft-fail + accepted transitive `RUSTSEC` note in `ci.yml` / [`PRODUCTION_CHECKLIST.md`](PRODUCTION_CHECKLIST.md) *(hard-zero advisories still open)*
- [x] Extension `npm audit` at moderate in CI — [`extension-ci.yml`](.github/workflows/extension-ci.yml)
- [ ] Mobile `npm audit` at agreed severity (document + run)
- [x] Lockfile bumps of Zcash crates only with full matrix build note — policy in [`AGENTS.md`](AGENTS.md)
- [x] Late-July Dependabot hygiene — PRs #179–#182 (quinn-proto, mobile overrides, **wasm-poc removed** for GHSA Orchard soundness, libcrux-chacha in dVPN spike). Progress only; hard-zero `cargo audit` still open

**Evidence:** 2026-07-20 fmt/test · CI workflows · mainnet evidence TXIDs · PRs #179–#182

---

## 1. CLI (`nozy`)

**Definition of done:** Operators can create/restore, sync to tip, receive, send shielded Orchard, and recover from common failures without private tribal knowledge.

### Wallet lifecycle
- [x] Create wallet — `nozy new` (`src/main.rs` + book CLI docs)
- [x] Restore from mnemonic — `nozy restore`
- [x] Password KDF path — password at create/restore (CLI is password-on-demand; no separate `unlock`/`lock` session commands)
- [ ] Backup / export guidance (no seed in plaintext logs) — confirm book chapter + grep

### Sync & balance
- [x] `sync` / `sync --to-tip` against Zebrad — commands + June 2026 field runs
- [x] Balance / NoteIndex v2 fix documented — BUG-2026-012 marked fixed on master ([`docs/issues/bugs/2026-06-cli-balance-v2-noteindex.md`](docs/issues/bugs/2026-06-cli-balance-v2-noteindex.md)); re-verify on latest tag before claiming
- [x] `status` shows tip / scan gap / useful errors — `nozy status` (v2.3.1+)

### Send
- [x] Shielded Orchard send on mainnet after sync — [`MAINNET_SEND_READINESS_EVIDENCE.md`](docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md)
- [x] ZIP-317 / priority / expiry documented — `src/fee_policy.rs` + book transaction commands + pilot docs
- [x] Expired pending notes release balance — shipped v2.3.6.2+ (roadmap / changelog)
- [x] Speed-up / rebuild path — API + CLI/desktop/extension alignment noted in roadmap
- [x] Failure modes documented — connectivity + common-issues troubleshooting

### Release
- [x] Tagged GitHub Release — README points at **v2.4.1.1** (and prior)
- [x] Install / usage in book or README
- [ ] Changelog entry specifically framed for **grant-window** production cut (tag when 0–8 green)

**Evidence:** v2.4.1.1 · TXIDs in mainnet evidence paper

---

## 2. Companion API (`api-server`)

**Definition of done:** Localhost companion is safe by default; hosted path (if claimed) has auth + TLS; seed handling is **documented and intentional** (not accidental).

**Internal — when to call companion GA (do not put in release/forum copy until you decide):**

1. Chain rows in [`api-server/PARITY.md`](api-server/PARITY.md) signed off (sync / balance / send / LWD compact-to-tip) against a real node  
2. CI Companion API Smoke green on `master`  
3. You intentionally update public README / release.yml / COMPANION to say production-ready localhost companion  
4. Hosted remains a separate decision  

Until then: keep shipping binaries + engineering hardening; public stance stays “CLI is GA; companion is attached for same-machine use.”

### Localhost (desktop / extension)
- [x] Default bind story documented and aligned — default `127.0.0.1`; `NOZY_BIND_ADDR=0.0.0.0` for hosted ([`SECURITY_CONFIG.md`](api-server/SECURITY_CONFIG.md), COMPANION.md, README)
- [x] CORS / rate limits documented — [`api-server/SECURITY_CONFIG.md`](api-server/SECURITY_CONFIG.md)
- [x] Seed handling policy (honest) - [`SEED_POLICY.md`](api-server/SEED_POLICY.md): loopback create/restore may accept mnemonic; `NOZY_PRODUCTION` or `0.0.0.0`/`::` requires `NOZY_API_KEY` (refused otherwise); responses masked (`display_mnemonic_safe`); never log. Also [`SECURITY_REVIEW.md`](SECURITY_REVIEW.md) section 4 + [`SECURITY_CONFIG.md`](api-server/SECURITY_CONFIG.md).
- [x] CI Companion API Smoke — boot `nozywallet-api` + [`scripts/smoke-companion.sh`](api-server/scripts/smoke-companion.sh) in [`.github/workflows/ci.yml`](.github/workflows/ci.yml)
- [ ] Sync / balance / send / speed-up / LWD routes parity with CLI - routes exist; formal **chain** field sign-off open ([`PARITY.md`](api-server/PARITY.md)); non-chain helper: [`scripts/parity-local.sh`](api-server/scripts/parity-local.sh)
- [x] Ironwood / readiness endpoints present in API — `ironwood_handlers` + status/readiness gates in-repo (PR trail / desktop alignment); keep claims matched to [`IRONWOOD_WALLET_READINESS.md`](docs/reference/IRONWOOD_WALLET_READINESS.md); formal “Done” still needs post-activation field check
- [x] Axum 0.8 route param compatibility — PR #180
### Hosted (mobile companion — only if in grant scope)
- [ ] Public HTTPS health check — not live yet per mobile store checklist
- [ ] API key enforced on public VPS
- [x] Product copy says own node / wait for funding — [`nozy-mobile/src/lib/connectionPresets.ts`](nozy-mobile/src/lib/connectionPresets.ts) + funding case breakdown
- [x] Deploy runbook exists — [`nozy-mobile/VPS-DEPLOY.md`](nozy-mobile/VPS-DEPLOY.md) (includes `NOZY_BIND_ADDR=0.0.0.0`)

**Evidence:** SECURITY_CONFIG | SEED_POLICY | PARITY | smoke + parity-local scripts | CI `api-smoke` | `release.yml` attaches `nozywallet-api-*` (localhost; no public GA claim) | connectionPresets | VPS-DEPLOY

---

## 3. Desktop (Tauri)

**Definition of done:** Windows users can install a signed (or clearly beta-unsigned) build, sync, send, and manage settings without the CLI.

### Product
- [x] Create / restore / unlock / lock — described in release notes / app surface
- [x] Sync feedback (header / banner / panel) — release notes
- [ ] Balance + history accurate vs CLI on same wallet data — **needs explicit QA note**
- [x] Shielded send with progress stages — release notes
- [x] Settings: network, security, display, Keystone (as shipped) — release notes
- [x] Settings → **Network privacy** (opt-in Nym) — `NetworkPrivacySettings`; mixnet broadcast + dVPN sync toggles; Case A1 local/LAN Zebrad stays direct (PR #178). **Not** fully productized routing for all sync
- [x] Ironwood readiness UI present — `IronwoodReadinessCard`; keep claims matched to [`IRONWOOD_WALLET_READINESS.md`](docs/reference/IRONWOOD_WALLET_READINESS.md)

### Engineering / release
- [x] IPC / capability allowlist reviewed — SECURITY_REVIEW §5 (2026-07-20); residual path args documented
- [x] Beta clearly labeled (GA deferred until Ironwood) — [`DESKTOP_RELEASE.md`](desktop-client/DESKTOP_RELEASE.md) / [`DESKTOP_BETA_2_RELEASE.md`](desktop-client/DESKTOP_BETA_2_RELEASE.md)
- [x] Updater channel policy — **manual download only** (no updater plugin)
- [x] Release workflow exists — [`.github/workflows/desktop-release.yml`](.github/workflows/desktop-release.yml)
- [x] Release notes: not a third-party audit (honest)
- [x] GA vs beta gate clear

**Evidence:** desktop release docs + workflow

---

## 4. Browser extension (MV3 + WASM)

**Definition of done:** Store-submittable build; scan/send path matches core fee/expiry policy; no debug exfil.

### Product
- [x] Create / restore / lock — implemented in popup / service worker
- [x] Zebrad RPC scan (`getblockhash` → `getblock`) — `browser-extension/background/rpc-utils.js` + LOCAL_RPC.md
- [x] Fee / expiry / speed-up alignment with core — CHANGELOG / WASM fee_policy (A′1)
- [x] Companion API optional path documented — [`COMPANION.md`](browser-extension/COMPANION.md)
- [x] Clear errors for RPC fail / scan abort — service worker + popup `lastRpcError` behavior

### Security
- [x] [`SECURITY_REVIEW.md`](SECURITY_REVIEW.md) §2 checked and signed — **2026-07-20**
- [x] `host_permissions` justified — [`browser-extension/HOST_PERMISSIONS.md`](browser-extension/HOST_PERMISSIONS.md)
- [x] No debug `fetch` of wallet payloads — background fetches = user RPC / companion only (re-grep at store submit)
- [x] Session mnemonic cleared on lock — `walletLock()` in service-worker
- [x] WASM CI path — [`extension-ci.yml`](.github/workflows/extension-ci.yml) + pinned lockfile in wasm-core

### Store / distribution
- [x] Icons (16/32/48/128) final — `browser-extension/icons/`
- [ ] Screenshots + store listing copy — placeholders remain in store-assets
- [x] Privacy policy content exists — landing privacy page (wire store listing URL at submit)
- [ ] Chrome Web Store / Edge **or** public GitHub Release zip published for reviewers
- [x] Version / changelog discipline — [`browser-extension/CHANGELOG.md`](browser-extension/CHANGELOG.md) (0.1.6 — 2026-07-20)

**Evidence:** icons · rpc-utils · COMPANION · CHANGELOG

---

## 5. Mobile (Expo / companion)

**Definition of done:** Production profile build on a physical device; store checklist Phase 3–4 complete for the **claimed** v1 strategy (companion-only unless FFI is explicitly in scope).

Follow [`nozy-mobile/STORE-CHECKLIST.md`](nozy-mobile/STORE-CHECKLIST.md); roll-up here:

### Config & safety
- [x] Production profile: HTTPS only, experimental FFI hidden — STORE-CHECKLIST Phase 1
- [x] API key required for hosted URL — Phase 1
- [x] Privacy policy linked in-app — `src/constants/links.ts`
- [ ] No test mnemonics / secrets in repo — confirm before submit

### Infra (honest)
- [x] Documented own-node / wait-for-funding path (hosted Zebrad deferred) — funding case breakdown + presets
- [x] No fake public Zebrad presets — presets emptied / honest copy

### Store
- [ ] Screenshots (Welcome, Dashboard, Send, Receive, Settings)
- [ ] Listing text + Data safety / privacy labels finalized
- [ ] EAS production Android build on physical device
- [ ] E2E: connect → create/restore → sync → send
- [ ] Play submit **or** internal track with clear status
- [ ] iOS path **or** explicitly “Android-first” in grant scope — **recommend scoping Android-first for $80k**

**Evidence:** STORE-CHECKLIST Phase 0–1 [x] items

---

## 6. Security & privacy

**Definition of done:** Structured internal review signed; disclosure path live; no audit overclaims in public copy.

- [x] Full pass of [`SECURITY_REVIEW.md`](SECURITY_REVIEW.md) with sign-off row filled — **2026-07-20 internal / beta**
- [x] Self-audit artifacts exist — [`SELF_AUDIT_RESULTS.md`](SELF_AUDIT_RESULTS.md) (labeled self, Dec 2025)
- [x] Responsible disclosure in [`CONTRIBUTING.md`](CONTRIBUTING.md) — **Nozywallet.support@leoninedao.org**
- [x] Honest “not third-party audit” in desktop release notes / book audits page
- [x] Telemetry: none by default / no silent exfil — [`AGENTS.md`](AGENTS.md) + Privacy page
- [ ] Plan for professional audit written into grant budget (§9) — reserve line exists in [`BUDGET_80K.md`](docs/reference/grant-evidence/BUDGET_80K.md); engagement not started
- [x] Keystone path documented at current maturity — [`book/src/security/keystone-hardware-wallet.md`](book/src/security/keystone-hardware-wallet.md)
- [x] Supply-chain critical cleanup progress — wasm-poc removal + quinn-proto / related Dependabot PRs #179–#182 *(does not replace third-party audit)*

**Evidence:** SECURITY_REVIEW sign-off · SELF_AUDIT · CONTRIBUTING · Keystone book · AGENTS telemetry rule · cargo audit 2026-07-20 · Dependabot #179–#182

---

## 7. Docs & operator runbooks

**Definition of done:** One clear “start here” per surface; failure runbooks exist; grant reviewers can navigate without a maintainer call.

- [ ] Start-here matrix: CLI vs desktop vs extension vs mobile — landing ProductSurfaces helps; still want one explicit matrix page
- [x] Runbook: RPC unreachable — `book/src/troubleshooting/common-issues.md` + connectivity doc
- [ ] Runbook: wrong network — only scattered mentions; add short section
- [ ] Runbook: scan range / birthday — thin; add short section
- [x] Runbook: witness mismatch / lag — connectivity + common-issues “send blocked”
- [x] Ironwood / fee / privacy expectations linked — reference docs set + PRs #175/#177
- [x] Connectivity doc current — [`ZEBRAD_NOZYWALLET_CONNECTIVITY.md`](docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md)
- [x] Case breakdown papers as engineering evidence — `docs/reference/*_CASE_BREAKDOWN.md` including Nym × Ironwood set (see grant-evidence README)

**Evidence:** book troubleshooting · docs/reference case papers · Ironwood + Nym docs

---

## 8. Releases & distribution

**Definition of done:** Reproducible public artifacts for every surface in scope; landing download path works.

| Surface | Artifact | Done |
|---------|----------|------|
| CLI | GitHub Release binary / crates instructions | [x] v2.4.1.1 referenced |
| API | GitHub Release `nozywallet-api-*` beta + Docker/docs | [~] `build-api` enabled in release.yml; tag a release to publish; Dockerfile/docs still confirm |
| Desktop | NSIS/MSI (or beta) on Releases | [ ] workflow ready; publish beta tag if claiming |
| Extension | Store and/or Release zip + WASM | [ ] |
| Mobile | Play/EAS artifact | [ ] |
| Landing | Download / product pages live | [ ] verify Pages deploy |

- [x] CI workflows present — `ci.yml`, `extension-ci.yml`, `release.yml`, `desktop-release.yml`, `extension-release.yml`
- [ ] Version bump + changelog discipline called out for grant cut
- [ ] Checksums / provenance note for binaries on grant-window release

**Evidence:** workflow files · README release links

---

## 9. Grant evidence pack (≈$80k retroactive)

**Definition of done:** Application can be filled in one sitting from this pack. Budget math lives in local `docs/grant/`; public repo stays free of private Lockbox drafts if that is your policy.

### Delivered public good (narrative)
- [x] One-page summary — [`docs/reference/grant-evidence/ONE_PAGER.md`](docs/reference/grant-evidence/ONE_PAGER.md) *(refreshed 2026-07-27: Ironwood + Nym opt-in)*
- [x] Differentiation vs existing wallets — same one-pager (fees, Ironwood readiness, optional Nym hybrid)
- [x] Timeline raw material exists — [`NOZYWALLET_2025_CASE_BREAKDOWN.md`](docs/reference/NOZYWALLET_2025_CASE_BREAKDOWN.md) + 2026 case papers + late-July Nym/Ironwood + one-pager compression
- [x] Surfaces in scope listed (CLI, API, extension, mobile, desktop beta; Nym/Ironwood wallet-side retroactive; out-of-scope named)

### Proof
- [x] GitHub org/repo with releases + commits
- [x] Mainnet evidence table — [`MAINNET_SEND_READINESS_EVIDENCE.md`](docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md)
- [x] Per-surface demo — [`docs/reference/grant-evidence/`](docs/reference/grant-evidence/README.md): extension MP4 (local Videos), Zingo receive proof, CLI dynamic-fee TXID + privacy banner; mobile memo on Zingo Jul 14
- [x] CI posture documented (this checklist + workflows)
- [x] Security posture paragraph — in [`ONE_PAGER.md`](docs/reference/grant-evidence/ONE_PAGER.md)

### Budget story (≈$80k)
Keep dollar detail in `docs/grant/` if private. Public checklist only tracks readiness of the story:

- [x] Hours / milestone map skeleton — [`BUDGET_80K.md`](docs/reference/grant-evidence/BUDGET_80K.md) *(fill hour estimates before submit)*
- [x] Suggested split draft — same file (A–G totaling $80k)
- [x] What $80k does **not** buy — one-pager + budget
- [x] AI assistance disclosure norm — [`AGENTS.md`](AGENTS.md)

### Submission hygiene
- [ ] Forum / ZCG / Lockbox target process identified
- [ ] Links scrubbed of secrets, private IPs, seed material
- [x] Maintainer contact + real responsible disclosure URL/email — **Nozywallet.support@leoninedao.org**
- [ ] “Production-ready” claim matches this checklist scoreboard

**Evidence:** [`docs/reference/grant-evidence/`](docs/reference/grant-evidence/README.md) · mainnet paper · case breakdowns · AGENTS disclosure rule

---

## Suggested order of work (minimize thrash)

1. **P0** — §4 extension GitHub Release zip (+ store assets path); link YouTube demo  
2. **P0** — §5 mobile EAS device QA; keep grant scope **Android-first**  
3. **P0 for ask** — §9 fill budget hours + pick ZCG / Lockbox / forum target (one-pager already refreshed)  
4. **P1** — §7 thin runbook gaps + §8 publish remaining artifacts  
5. **P1** — optional further `cargo audit` / RUSTSEC hard-zero (Dependabot #179–#182 already progressed)  
6. Before GA — desktop remains beta until Ironwood; tighten `db_path` / `backup_path` allowlist + signing  

---

## Out of scope unless explicitly added to the $80k ask

- Running a second full-node product or Windows native Zebrad sync on the maintainer machine  
- Zakura as default backend  
- Multichain (Namada / Penumbra)  
- Claiming third-party audit complete without engagement  
- “Hosted = no node needed” without a real Nozy Zebrad  

---

## Sign-off (human)

| Role | Name | Date | Verdict |
|------|------|------|---------|
| Engineering | | | ☐ Ready for $80k ask / ☐ Not yet |
| Security review | Internal pass | 2026-07-20 | ☑ Passed internal / ☐ Blockers (residuals noted in SECURITY_REVIEW) |
| Grant narrative | | | ☐ Pack complete / ☐ Gaps |

When all three are Ready/Passed/Complete, treat the checklist as **green** for submission.
