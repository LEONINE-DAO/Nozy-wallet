# NozyWallet white paper — outline & production guide

**Purpose:** Blueprint for a **white paper** (architecture, phases, challenges, trade-offs, security, lessons learned) distinct from but complementary to the generated **technical paper** (`NozyWallet_Technical_Paper_v2.docx`).

**Audience:** Shielded Labs, Zcash forum, grants, lectures, serious contributors.

**How to produce:**

1. Edit content in `scripts/generate-nozy-whitepaper.py` (or pull from linked source docs below).
2. Run `python scripts/generate-nozy-whitepaper.py` → **`docs/NozyWallet_Whitepaper.docx`** (~12–16 pages).
3. For the longer technical report (~20 pages): `python scripts/generate-nozy-paper.py`.
4. Attach evidence: [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md), TXIDs, forum post draft in [`../community/shielded-lab-pilot-send-readiness-update-2026-06.md`](../community/shielded-lab-pilot-send-readiness-update-2026-06.md).

---

## White paper vs technical paper

| | **White paper** (this outline) | **Technical paper** (generator) |
|---|-------------------------------|----------------------------------|
| **Goal** | *Why* we built it this way; decisions & lessons | *What* Orchard/Zcash is; protocol depth |
| **Length** | ~12–15 pages | ~20+ pages |
| **Reader** | Mark/Zooko, operators, grant reviewers | Engineers implementing wallet crypto |
| **Hero sections** | Architecture decisions, phases, trade-offs, incidents | §2–3 protocol, §4 viewing keys |
| **Evidence** | Mainnet tables, BUG registry | Same, but appendix |

---

## Part A — Executive summary (1 page)

**Write:** Problem → solution → what ships today → one mainnet lesson.

| Source | Pull from |
|--------|-----------|
| Product thesis | [`NOZYWALLET_PROPOSAL.md`](../NOZYWALLET_PROPOSAL.md) §Executive summary |
| Pilot alignment | [`PILOT_EXPIRY_PROVING_LATENCY.md`](PILOT_EXPIRY_PROVING_LATENCY.md) executive summary |
| Mainnet proof | [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md) test matrix |

**One paragraph to include:** Wallet ≠ node; Zebrad + lightwalletd + **local witness derivation**; shielded-first Orchard-only sends.

---

## Part B — Architecture decisions (2–3 pages)

Document **decisions as ADR-style blocks**: Context → Decision → Consequences.

### B.1 Stack: Zebrad-only (no zcashd)

| Decision | Zebrad JSON-RPC + lightwalletd compact sync; no consensus node in wallet repo |
|----------|-------------------------------------------------------------------------------|
| **Why** | Operator alignment, Zebra team direction, JSON-RPC for broadcast/treestate |
| **Trade-off** | No `estimatefee`; witnesses not served by node — wallet must derive locally |
| **Source** | [`ARCHITECTURE.md`](../ARCHITECTURE.md), [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md), [`rfcs/DYNAMIC_FEE_PILOT_PLAN.md`](../rfcs/DYNAMIC_FEE_PILOT_PLAN.md) §1 |

### B.2 One Rust core, many surfaces

| Decision | `nozy` crate + `zeaking` sync + thin surfaces (CLI, api-server, extension WASM, desktop) |
|----------|----------------------------------------------------------------------------------------|
| **Why** | Single Orchard send path; avoid duplicated fee/expiry logic |
| **Trade-off** | WASM vs native feature split; extension excluded from root workspace |
| **Source** | [`ARCHITECTURE.md`](../ARCHITECTURE.md), [`AGENTS.md`](../../AGENTS.md) |

### B.3 Local Orchard witnesses (not node witness RPC)

| Decision | Incremental witness in `notes.json`; catch-up via Zebra blocks + `z_gettreestate` verification |
|----------|-----------------------------------------------------------------------------------------------|
| **Why** | Zebra does not offer spend-ready witness RPC; documented in send-limit doc |
| **Trade-off** | Stale witness → long send unless sync-first; parallel `getblock` batches |
| **Source** | [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md), `src/orchard_witness.rs`, [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md) |

### B.4 Client-side ZIP-317 fees (dynamic-fee pilot)

| Decision | `fee_policy.rs` + optional ×4 priority; no node fee estimator |
|----------|--------------------------------------------------------------|
| **Why** | Zebrad unimplemented `estimatefee`; pilot needs deterministic client policy |
| **Trade-off** | Ecosystem must agree on “standard fee” definition eventually |
| **Source** | [`rfcs/DYNAMIC_FEE_PILOT_PLAN.md`](../rfcs/DYNAMIC_FEE_PILOT_PLAN.md), [`rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md`](../rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md) |

### B.5 Five-block pilot expiry (not fifteen)

| Decision | `PILOT_EXPIRY_DELTA_BLOCKS = 5`; rebuild/retry for slow proves |
|----------|----------------------------------------------------------------|
| **Why** | Fast expire-and-replace for speed-up UX (~6 min vs ~19 min) |
| **Trade-off** | Slow hosts need late tip refresh + rebuild, not longer expiry |
| **Source** | [`PILOT_EXPIRY_PROVING_LATENCY.md`](PILOT_EXPIRY_PROVING_LATENCY.md), BUG-2026-011 |

### B.6 Note index v2 (`NoteIndex`)

| Decision | Serialized index with nullifier/height maps; atomic save |
|----------|---------------------------------------------------------|
| **Why** | Fast load; merge sent + received history |
| **Trade-off** | All code paths must load v2 (mark-spent bug if legacy parser used) |
| **Source** | `src/note_index.rs`, BUG-2026-001/002 fixes |

---

## Part C — Phased development approach (2 pages)

Use a **phase table** with gate criteria (what must be true before next phase).

| Phase | Name | Shipped | Gate to next |
|-------|------|---------|--------------|
| **0** | Foundation | HD wallet, Orchard scan, CLI | Mainnet scan works |
| **1** | Zebrad + LWD stack | `zeaking::lwd`, compact SQLite | Operator can sync to tip |
| **2** | Mainnet send | Witness pipeline, ZIP-225 v5, broadcast | Successful mainnet tx |
| **3** | NU6.2 + pilot A1 | librustzcash 0.28, 5-block expiry, ZIP-317 fix | Branch ID + fee shape correct |
| **4** | Surfaces + API | api-server, extension companion, desktop WIP | Parity on send/sync |
| **5** | Reliability (2026-06) | BUG-2026-001–011, send-readiness, unified sync | Mainnet evidence doc PASS |
| **6** | Pilot A2 / observatory | Zeaking fee observatory (Shielded Labs gate) | Shared metrics schema |
| **7** | Business / web / mobile | [`ENHANCEMENT_ROADMAP.md`](../../ENHANCEMENT_ROADMAP.md) | Per surface |

**Sources:** [`CHANGELOG.md`](../../CHANGELOG.md), [`NOZYWALLET_PROPOSAL.md`](../NOZYWALLET_PROPOSAL.md) milestones, generator §6.5–6.6, [`RELEASE_v2.3.0_NOTES.md`](../RELEASE_v2.3.0_NOTES.md).

**Diagram for slide/paper:**

```text
Phase 0–2: Core + infra truth  →  Phase 3–5: Mainnet + pilot  →  Phase 6+: Ecosystem metrics
```

---

## Part D — Challenges & how we addressed them (3 pages)

### D.1 Zebrad integration

| Challenge | Response |
|-----------|----------|
| No fee RPC | Client ZIP-317 in `fee_policy.rs` |
| No witness for spends | Local incremental witness + block catch-up |
| Privacy/Tor vs VPS direct RPC | `trusted_zebra_urls`, structured connect errors |
| Node behind tip → mempool disabled | Document sync-to-tip before send; status UX |
| NU6.2 branch ID mismatch | Dependency bump PR #58 |

**Sources:** [`ZEBRA_DEV_CLI_POSITIONING.md`](../ZEBRA_DEV_CLI_POSITIONING.md), BUG-2026-004 in CHANGELOG, forum draft June 2026.

### D.2 Orchard shielded send pipeline

| Challenge | Response |
|-----------|----------|
| Prove latency >> block time | Late expiry encode + rebuild loop |
| Pre-broadcast `-25` | BUG-2026-011 |
| Anchor / Merkle path errors | `ZebraJsonRpcOrchardWitnessProvider`, treestate RPC |
| Halo2 cold start | `warm_orchard_proving_key()` |
| History expiry metadata wrong | On-chain `expiry_height` from signed tx |

**Sources:** [`PILOT_EXPIRY_PROVING_LATENCY.md`](PILOT_EXPIRY_PROVING_LATENCY.md), `src/orchard_tx.rs`, [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md).

### D.3 Sync protocols (dual path)

| Path | Role | Challenge |
|------|------|-----------|
| **JSON-RPC scan** | `nozy sync`, note discovery | Slow on large ranges; bounded incremental default |
| **Compact LWD** | `zeaking::lwd`, extension resume | gRPC reachability; compact DB size/RAM |
| **Unified orchestrator** | `wallet_sync` (v2.3.6.2+) | api-server vs CLI drift — merged |

**Sources:** [`rfcs/WALLET_SYNC_UNIFIED_ARCHITECTURE.md`](../rfcs/WALLET_SYNC_UNIFIED_ARCHITECTURE.md), [`MEMORY_CASE_STUDY.md`](../MEMORY_CASE_STUDY.md), `zeaking/README.md`.

### D.4 Surface parity

| Challenge | Response |
|-----------|----------|
| api-server old binary vs CLI | Document rebuild; `run-nozy-api.ps1` |
| WASM extension separate workspace | Own `Cargo.toml`; companion API on :3000 |
| 50k rescan on send (BUG-2026-001) | Cache-first `notes.json` |

**Sources:** [`issues/bugs/`](../issues/bugs/), [`ARCHITECTURE.md`](../ARCHITECTURE.md).

---

## Part E — Trade-offs (1–2 pages)

Present as a **decision matrix** (good for lectures).

| Topic | Option A | Option B | NozyWallet choice | Rationale |
|-------|----------|----------|-------------------|-----------|
| Expiry delta | 5 blocks | 15 blocks | **5** | Speed-up UX |
| Slow prove fix | Longer expiry | Rebuild/retry | **Rebuild** | Preserves pilot semantics |
| Stale witness | Catch-up at send | Reject if lag >50 | **Reject + sync** | Predictable latency |
| Fee source | Node estimator | ZIP-317 client | **ZIP-317** | Zebrad reality |
| Sync default | Always full rescan | Incremental + cache | **Incremental** | Operator bandwidth |
| Transparent sends | Allow `t1` | Orchard-only | **Orchard-only** | Privacy product |
| Multichain | Monolith wallet | ZEC-first modules | **ZEC-first** | [`rfcs/MULTICHAIN_*`](../rfcs/) |

---

## Part F — Security & privacy considerations (2 pages)

| Area | Practice | Doc |
|------|----------|-----|
| **Keys** | Encrypted `wallet.dat`, `zeroize`, no seed in API responses | generator §7.2, `src/storage.rs` |
| **Network** | Optional Tor; trusted Zebra allowlist for operators | `privacy_network` config |
| **Transaction** | Orchard-only; no transparent leakage by design | `input_validation.rs` |
| **API** | Localhost companion; optional API key in production | `api-server/README.md` |
| **Viewing keys** | UFVK export for Keystone; selective disclosure planned | generator §4 |
| **Supply chain** | librustzcash / orchard pins; NU upgrade discipline | PR #58, AGENTS.md |
| **Incident response** | BUG registry, RCA docs, mainnet evidence | [`BUG_REGISTRY.md`](../issues/BUG_REGISTRY.md) |

**Do not claim:** full formal audit unless completed; say “high-impact wallet code — responsible disclosure in CONTRIBUTING.md.”

---

## Part G — Lessons learned (1–2 pages)

Bullet list for Mark/Zooko / lecture close.

1. **Two clocks on shielded sends** — build-time expiry vs mempool expiry; pilot measures the second; VPS bugs often hit the first.
2. **Wallet ≠ node** — witnesses and fee policy are wallet responsibilities on Zebrad.
3. **Sync-before-send is product policy** — not just performance; guard at 50 blocks saved 7+ minute failures.
4. **Keep pilot knobs stable** — fix runtime (rebuild, warm prove) before changing `nExpiryHeight` policy.
5. **Operator stacks need first-class testing** — WSL Zebrad + Windows CLI matches real users, not laptop-only CI.
6. **Cache format migrations matter** — v2 `NoteIndex` vs legacy array caused subtle post-send bugs.
7. **Surface parity is expensive** — api-server, CLI, extension must share `build_and_broadcast_send_transaction`.
8. **Evidence wins trust** — TXIDs + timings in public docs beat ad-hoc forum claims.

**Primary evidence doc:** [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md).

---

## Part H — Appendix (optional)

| Appendix | Content |
|----------|---------|
| A | Glossary (generator Appendix A) |
| B | Mainnet TXID table |
| C | BUG-2026-001–011 one-liners |
| D | Environment diagram (ARCHITECTURE stack ASCII) |
| E | References (ZIP-317, ZIP-316, ZIP-225, Shielded Labs pilot post) |

---

## Production checklist

- [ ] Executive summary ≤ 1 page, no jargon without glossary
- [ ] Every architecture decision has **trade-off** column
- [ ] Phase table matches CHANGELOG (no aspirational phases marked “shipped”)
- [ ] Mainnet evidence cited with TXIDs and dates
- [ ] Security section: honest about localhost API, no audit overclaim
- [ ] Lessons learned tied to named bugs (BUG-2026-011, etc.)
- [ ] Regenerate technical paper: `python scripts/generate-nozy-paper.py`
- [ ] Forum post links to white paper PDF or `docs/reference/` paths on GitHub
- [ ] AI disclosure line if paper drafted with assistance (CONTRIBUTING policy)

---

## Suggested lecture flow (45 min)

1. Problem: shielded-first wallet on Zebrad (5 min)
2. Architecture stack diagram (5 min)
3. Orchard send pipeline + two clocks (10 min)
4. Dynamic-fee pilot mapping (5 min)
5. Mainnet evidence table (10 min)
6. Lessons + Q&A (10 min)

Slides: export diagrams from this doc + [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md) §Lecture outline.

---

## Next engineering step (optional)

Generate the white paper:

```powershell
pip install python-docx
python scripts/generate-nozy-whitepaper.py
```

Output: **`docs/NozyWallet_Whitepaper.docx`** (Word) and **`docs/reference/NozyWallet_Whitepaper.md`** (Cursor/IDE)

**Note:** Cursor cannot preview `.docx` files. Read the `.md` in the editor, or open the `.docx` in Microsoft Word / LibreOffice.

For the longer technical report (~20 pages): `python scripts/generate-nozy-paper.py`
