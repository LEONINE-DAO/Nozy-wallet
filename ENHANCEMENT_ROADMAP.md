# NozyWallet enhancement roadmap

**Status:** Living document — ZEC-first, privacy super wallet.  
**Tagline:** Your gateway to shielded value. *Private by default.*  
**Launchpad:** [`landing/`](landing/) · **Latest CLI:** [Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest)

Full problem / solution / impact narrative: [`docs/NOZYWALLET_PROPOSAL.md`](docs/NOZYWALLET_PROPOSAL.md) (local working doc).

---

## Product surfaces

| Surface | Path | Status | Notes |
|---------|------|--------|--------|
| **CLI + core** | `src/`, `nozy` | **Mainnet** | Orchard + Ironwood; Zebrad / Zakura + LWD / Nozy Sync Engine |
| **Launchpad** | `landing/` | **Active** | GitHub Pages product hub |
| **Web app** | [`web-app/`](web-app/) | **Starting** | Full dashboard; extension + `nozywallet-api` |
| **Browser extension** | `browser-extension/` | **Chrome Web Store live** | [NozyWallet 2.0.0 (Sweet Chili)](https://chromewebstore.google.com/detail/nozywallet/kjbpoimgafbikmachlilkhoamdlnhooj) — keys on device; public sync via `lwd.nozywallet.org` when no local node |
| **Operator API** | `api-server/` | Production (localhost) | Companion for extension / desktop / web |
| **Desktop** | `desktop-client/` | Production download (beta tag) | Tauri; no formal GA label |
| **Mobile** | [`nozy-mobile/`](nozy-mobile/) | In development | Expo; see [VPS-DEPLOY](nozy-mobile/VPS-DEPLOY.md) |
| **Nozy Agent** | `nozy-agent/` (planned) | Planning | Self-hosted Rust agent; see section below |

**Privacy chains (later):** Namada, Penumbra — [`docs/rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md`](docs/rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md)

**Business / POS:** ZNS + Sell mode — [#85](https://github.com/LEONINE-DAO/Nozy-wallet/issues/85), [`docs/BUSINESS_ZEC_ZNS_TODO.md`](docs/BUSINESS_ZEC_ZNS_TODO.md)

---

## Shipped milestones

Releases and features already on mainline or tagged CLI releases. See [`CHANGELOG.md`](CHANGELOG.md) and [`docs/issues/FEATURE_REGISTRY.md`](docs/issues/FEATURE_REGISTRY.md).

### Core wallet & consensus

| Milestone | Version | Impact |
|-----------|---------|--------|
| **Dynamic fee pilot — Phase A1** | v2.3.0+ | Client-side ZIP-317; `--priority` ×4; short expiry; live fee estimates (CLI, API, desktop) — [RFC](docs/rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md) |
| **NU6.2 mainnet compatibility** | v2.3.2 | Correct consensus branch ID; librustzcash alignment |
| **ZIP-317 fee shape fix** | v2.3.2 | Actions = `max(spends, outputs)` (not spends + outputs) |
| **Orchard spend detection (#61)** | v2.3.3 | Nullifiers at discovery; fixes second-send double-spend |
| **Single-note coin selection** | v2.3.4 | Reliable broadcast when wallet holds multiple notes |
| **Unified `wallet_sync`** | v2.3.6.2 | Shared scan → merge → persist for API + CLI |

### Dynamic fee pilot — follow-on (A1 completion)

| Milestone | Version | Impact |
|-----------|---------|--------|
| **`Expired` status + balance release** | v2.3.6.2 | Pending notes unlocked when tx passes expiry unmined |
| **Speed-up rebuild (not rebroadcast)** | v2.3.6.2 | `POST /api/transaction/speed-up`; desktop + extension WASM |
| **Extension `fee_policy` alignment** | v2.3.4+ | WASM uses shared ZIP-317 policy with core |

### Sync & operator stack

| Milestone | Version | Impact |
|-----------|---------|--------|
| **Compact sync (`zeaking::lwd`)** | Shipped | lightwalletd → SQLite; extension/API LWD path |
| **`nozy sync --to-tip` + `lwd prune`** | v2.3.1 | Find deposits without multi-run sync; stale cache cleanup |
| **`nozy status` sync diagnostics** | v2.3.1 | Zebra tip, LWD tip, scan gap, stale-cache warning |
| **Remote VPS Zebra + `trusted_zebra_urls`** | v2.3.6.3–6.4 | Hosted infra without disabling privacy policy globally |
| **Structured sync errors** | v2.3.6.x | JSON phases, 503 `ZEBRA_UNAVAILABLE`, connect error codes |

### API & product (recent)

| Milestone | Status | Impact |
|-----------|--------|--------|
| **Chrome Web Store — extension 2.0.0** | **Live (2026-10-05)** | [Listing](https://chromewebstore.google.com/detail/nozywallet/kjbpoimgafbikmachlilkhoamdlnhooj); public compact sync; create/restore/backup; Ironwood-aware send; [extension-v2.0.0](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/extension-v2.0.0) |
| **Public LWD sync** | Shipped | `https://lwd.nozywallet.org:443` when no local node; keys stay in extension |
| **CLI v2.4.7+ / Desktop beta.7** | Released | See [Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases) |
| **Send reuses note cache (no 50k rescan)** | Branch / unreleased | Fast send after sync; see [CHANGELOG Unreleased](CHANGELOG.md) |
| **History merges received deposits** | Branch / unreleased | `Received` + `Sent` in `/api/transaction/history` |
| **Launchpad + product roadmap** | Active | Product hub, web-app/mobile plans |

---

## Dynamic fee pilot — remaining (Phase A′ & A2)

Parent plan: [`docs/rfcs/DYNAMIC_FEE_PILOT_PLAN.md`](docs/rfcs/DYNAMIC_FEE_PILOT_PLAN.md) · Checklist: [`docs/rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md`](docs/rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md)

| Track | Status | Next |
|-------|--------|------|
| **A′1 Extension lifecycle parity** | In progress | Full speed-up/expiry parity with core per RFC §8 |
| **A′2 Testnet soak + pilot metrics** | Planned | Opt-in counters: priority sends, speed-ups, expired-unmined |
| **A2 Zeaking shared `fee_policy`** | **Blocked** | Shielded Labs go-ahead — move policy to `zeaking/`, compact expiry detection, observatory |

**Not in scope for A2 until SL approval:** changing Zebra/lightwalletd; Phase B standard-fee swap from Shielded Labs.

---

## Current focus: web app (2026)

The next major surface is **`web-app/`** — a browser dashboard (community-shaped super wallet), not the static launchpad.

| Phase | Goal |
|-------|------|
| **W0** | Scaffold Vite + React; shared theme with `landing/`; README + env template |
| **W1** | Connect to `nozywallet-api` (health, unlock, balance) on localhost |
| **W2** | Send / receive ZEC flows via companion API |
| **W3** | Extension messaging bridge (optional unlock path) |
| **W4** | Deploy to `/Nozy-wallet/app/` or custom domain |

Details: [`web-app/README.md`](web-app/README.md)

---

## Mobile (parallel track)

| Milestone | Doc |
|-----------|-----|
| Local dev + emulator | [`nozy-mobile/README.md`](nozy-mobile/README.md) |
| Public API on VPS | [`nozy-mobile/VPS-DEPLOY.md`](nozy-mobile/VPS-DEPLOY.md) |
| Store release | [`nozy-mobile/STORE-CHECKLIST.md`](nozy-mobile/STORE-CHECKLIST.md) |

Business profile + Sell mode: Phase 0 decisions locked — [`docs/BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md`](docs/BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md)

---

## Planned: Nozy Agent (self-hosted)

Separate product under `nozy-agent/` (not inside `web-app/`). It sits on top of the stack Nozy already ships: **CLI + Desktop + Chrome extension + localhost companion**, with optional public compact sync. Local LLM (Ollama default / localhost vLLM optional), Nym-mandatory egress, companion-controlled wallet actions, merchant desk, bounty hunter, protocol debug, Zcash research, optional Pet / Legacy Guardian.

- Medium draft (Markdown): [`docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md`](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md)
- Medium draft (PDF): [`docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.pdf`](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.pdf)
- Chrome extension (live): [NozyWallet on Chrome Web Store](https://chromewebstore.google.com/detail/nozywallet/kjbpoimgafbikmachlilkhoamdlnhooj)

### Wallet action catalog (locked)

The agent does anything the user can do in the wallet by calling the localhost companion, one typed action per user-facing route. Full write-up: [The whole wallet, as a catalog](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md#the-whole-wallet-as-a-catalog).

| Class | Rule |
|-------|------|
| Look | Balance, history, status, sync, invoices, and the other read routes run after unlock |
| Change | Send, migrate, vote, stake, invoices, local gift cards, ZNS, and viewing-key export each wait for a confirmation card |
| Hands off | Create, restore, and unlock stay on the user’s screen. The agent never takes a seed or password |
| Refused | Transparent addresses, disabling Nym, skipping approval, raw broadcast of a foreign transaction, spend-key export |

### Voice (locked)

Voice is push-to-talk on the island, using local speech-to-text ([whisper.cpp](https://github.com/ggml-org/whisper.cpp)) and local speech ([Piper](https://github.com/OHF-Voice/piper1-gpl)). On the phone, the small on-device models are used. Spoken answers use the same catalog. A spoken “yes” does not approve a spend. Secrets stay on screen. Audio does not leave the device. Full write-up: [Voice on the island](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md#voice-on-the-island).

Desktop: an **Agent** tab in the top bar, immediately after Home. Mobile: an **Agent** tab in the bottom bar, immediately after Home, with hold-to-talk as the main control. Settings moves under More. Until that seed’s agent exists, the tab is the pay-and-create gate. The same seed is the same agent on desktop, extension, and mobile. The extension does not create a second agent. Full write-up: [Where the agent sits in the wallet](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md#where-the-agent-sits-in-the-wallet).

### Threshold spend — FROST (locked for the later build)

Recorded so the agent scaffold does not invent custody rules later. **Not part of the first skeleton.** Full write-up: [FROST spend authority](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md#frost-spend-authority-planned-after-the-proposer-loop) in the agent draft. Reference: [ZIP-312](https://zips.z.cash/zip-0312), [ZF FROST status](https://zfnd.org/the-state-of-frost-for-zcash/), [ZF FROST Book](https://frost.zfnd.org/zcash/technical-details.html).

| Decision | Status |
|----------|--------|
| When | After the single-sig proposer loop (local model, Nym, human approval, companion build) is tested |
| What it splits | Orchard spend-authorizing key (`ask`) via rerandomized RedPallas FROST. Ironwood waits on a compatibility check |
| Default | Agent holds **no** spend share. It may coordinate. Shares stay in the companion share store |
| Key generation | Distributed. A complete `ask` is never assembled on one machine |
| Personal | User device + second device or hardware, threshold 2 |
| Merchant till | Owner + hardware, threshold 2. Invoices and note matching do not require a spend share |
| Legacy Guardian | Later, dedicated wallet only: founder / heir / trustee, 2-of-3. A missed check-in does not sign |
| Agent co-signer | Off by default; separate review if ever proposed |
| Relay | `frostd` is an untrusted encrypted relay. Off-box session traffic uses the Nym gate and fails closed |
| Restore | Share + verifying shares + participant ids + full viewing key. A seed alone does not restore it |
| Viewing / disclosure | Unchanged: UFVK or scoped incoming viewing key and the Phase 5 grant log |

### Milestone verify inbox (locked for Legacy Guardian)

School and college records can arrive by email to an address the agent owns. The heir asks the institution to send the transcript there. Uploading a screenshot is the low-value path only. Full write-up: [Milestone verify inbox](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md#milestone-verify-inbox).

| Decision | Status |
|----------|--------|
| Address | One inbound-only address per milestone, on a domain the operator controls. No replies |
| Who may send | Exact domains the founder wrote when the milestone was created. The model cannot add one |
| Check | Raw RFC 5322 message. Agent verifies SPF, DKIM, and DMARC with Rust `mail-auth`. DMARC must pass and align with that allowlist |
| Receive | Stalwart (or equivalent) on the operator host when port 25 is open. Otherwise a raw-preserving front (Cloudflare Email Worker or a receive-only VPS) that posts the original bytes to the agent. Consumer mailboxes are out |
| Identity | Extracted fields must match the heir details the founder recorded. A real transcript for someone else fails |
| Strength | Inbox plus a trustee is medium. Inheritance still needs an issuer-signed credential or institutional verification, plus the trustee quorum |
| Privacy | Encrypted local evidence store. No cloud model, no verification SaaS |

### Privacy town (locked for the later build)

A place where agents practice private ordering and are checked on user privacy. **Not part of the first skeleton.** User agents do not share a world or a VPS. Full write-up: [The privacy town](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md#the-privacy-town-planned-after-a-single-user-agent-works).

| Decision | Status |
|----------|--------|
| Shape | Mainland is the VPS. Each user agent lives on a private island (their machine). Shared AI Town-style memory is rejected |
| Tasks | The user seals the task on the island. On the mainland the agent does only that task. The dock drops anything that is not the receipt |
| Practice | Synthetic orders only. Reward is a local practice mark (software + suite version), not ZEC and not a public scoreboard |
| Real orders | Town gate carries fixed invoice messages. Gift-card codes and other bearer secrets are encrypted to the buyer’s island and not stored on the VPS. Merchant Desk confirmation still releases goods |
| Mainland stalls | Shops, invoice desk, encrypted delivery, practice yard, Warden rules, a public bulletin, merchant-published stats, and the market stall. No wallets or keys on the VPS |
| Warden | Higher-up checker of agent builds. Sees pass/fail reports with user content removed. Cannot spend or open an island |
| Break-in control | One OS user, process, and database per homestead. Signed envelopes only. Other agents’ text is untrusted. Nym for off-box gate traffic |
| Host | The VPS is the town agents visit. Homes stay on the user machine. Shared on the VPS: practice mark and fixed order envelopes only. Gate host: 1984, Njalla, or Privex |
| Create / buy | One agent per seed, derived by the wallet on a non-spend path. The same seed is the same paid agent on desktop, extension, and mobile. Memory syncs as ciphertext only the seed can open |
| Learn | On the island only: episodes and playbooks the user can read, correct, or wipe. The next similar task is proposed from that playbook. Approval, tools, and Nym stay fixed |

### Agent spending wallet (locked, separate from Nozy)

A second Zcash wallet with its own seed, so the agent can receive bought ZEC and spend it under a cap. The human Nozy wallet stays shielded-only. Full write-up: [The agent’s own wallet](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md#the-agents-own-wallet-separate-from-nozy).

| Decision | Status |
|----------|--------|
| Keys | Own seed. Not derived from the Nozy seed. Neither wallet can spend the other |
| Buy ZEC | Exchange withdrawal to the agent wallet’s transparent address. That history is public |
| Then | Shield into the agent wallet’s own Orchard. Change stays there |
| Cap | User-set. Under the cap the agent may spend this wallet on allowed tasks. Over the cap it asks. It cannot raise the cap or spend the human wallet |
| Home | Each payout uses a new diversified shielded address from the human wallet, paid once from the agent’s shielded funds. No standing Nozy address. No transparent input on that payment |
| Restore | The user backs up and restores this seed on each device. It is not inside the Nozy backup |

### Market adapters (locked)

The Zcash agent wallet does not trade on Solana, Ethereum, Namada, or Osmosis. Each chain is a separate key and a separate adapter. The agent does these tasks at the mainland market stall. The island keeps the keys and signs. Full write-up: [Wiring markets beyond Zcash](docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md#wiring-markets-beyond-zcash).

| Order | Adapter |
|-------|---------|
| 1 | Zcash agent wallet: `t1` receive, shield into its own Orchard, capped spend, fresh address home |
| 2 | Namada MASP sidecar. No ZEC bridge inside this adapter |
| 3 | Penumbra shielded DEX sidecar, before any public DEX |
| 4 | Osmosis, Ethereum, and Solana, each opted in alone: own key, own cap, public-trade warning |

Bridges and IBC wait on their own review. NEAR is not the router. The human Nozy seed is never an adapter key.

### Bounded trading — NEAR Intents demoted (locked)

| Decision | Status |
|----------|--------|
| Product stance | Prefer the **private model** (shielded ZEC, local agent, Nym egress). Private→transparent exits are a trust demotion, not “still private.” |
| Trading wallet | Separate capped companion process; never main-wallet keys in the agent |
| NEAR Intents / 1Click | **Paper + tiny human-approved canary only** — **not** a safe first venue for size or bounded automation |
| Why | [Forum #57497](https://forum.zcashcommunity.com/t/my-experience-exiting-shielded-zec-from-zodl-to-near-intents-589k-usdt-still-held-50-days-despite-a-written-compliance-clearance/57497): ~$589k USDT held ~50 days after written compliance clearance was later reopened; “trustless” wallet UX vs compliance-gated hold power; recovery burned privacy via tickets/public posts |
| Private→transparent harm | Exit edge reveals amount/timing; third parties can hold/freeze; marketing that blurs this boundary is unacceptable |
| Required copy | Never call NEAR trustless / same privacy as shielded ZEC; pre-trade trust-boundary warning in UI |
| Circuit breakers | Uncredited deposit, compliance hold, clearance-then-reopen, missing refund tx hash → freeze venue |
| Promotion past canary | Separate issue + review: settle/refund SLA, reclaim path, no unresolved #57497-class holds, owner-signed mandate |

Other venues: one adapter per security review; prefer clearer settlement/refund stories and privacy-preserving routes before raising any compliance-gated rail past canary.

---

## Backlog (ordered)

1. Web app W0–W2 (ZEC dashboard)
2. Dynamic fee A′1–A′2 (extension parity + pilot metrics)
3. Edge Add-ons / Firefox AMO listing (Chrome store already live)
4. Business profile + mobile Sell mode (issue #85)
5. Desktop formal GA label (downloads already published under beta tags)
6. Dynamic fee A2 in Zeaking (when Shielded Labs approves)
7. **Nozy Agent** RFC + scaffold (parallel; Nym + local LLM gates). FROST, the verify inbox, and the privacy town are specified in the agent draft and stay out of this scaffold
8. Multichain sidecars (Penumbra smoke → Namada spike per RFC)
9. Mobile App Store / Play (STORE-CHECKLIST)
10. Bounded trading adapters (**NEAR paper/canary only** until promotion criteria)

---

## References

- [`AGENTS.md`](AGENTS.md) — contribution gate
- [`docs/NOZYWALLET_PROPOSAL.md`](docs/NOZYWALLET_PROPOSAL.md) — problem, solution, milestones, impact
- [`docs/issues/FEATURE_REGISTRY.md`](docs/issues/FEATURE_REGISTRY.md) — feature IDs and status
- [`browser-extension/COMPANION.md`](browser-extension/COMPANION.md)
- [`CHANGELOG.md`](CHANGELOG.md)

---

*Update this file when a surface ships, a release tags, or web-app phases complete.*
