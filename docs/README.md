# Docs

| Path | Purpose |
|------|---------|
| [`MEMORY_CASE_STUDY.md`](MEMORY_CASE_STUDY.md) | Compact sync peak RAM (`chunk_max_blocks`), phase metrics, secrets/zeroize — **no invented RSS numbers** |
| [`rfcs/README.md`](rfcs/README.md) | RFCs: multichain, NEAR/Secret, spikes, extension ports |
| [`rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md`](rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md) | **Dynamic fee pilot — Phase A checklist** (review before coding) |
| [`rfcs/DYNAMIC_FEE_PILOT_PLAN.md`](rfcs/DYNAMIC_FEE_PILOT_PLAN.md) | Dynamic fee pilot architecture (Shielded Labs alignment) |
| [`reference/DYNAMIC_FEE_CASE_BREAKDOWN.md`](reference/DYNAMIC_FEE_CASE_BREAKDOWN.md) | **Dynamic-fee pilot case breakdown** — Cases 1–6 + cross-ref to Ironwood labels |
| [`reference/IRONWOOD_WALLET_READINESS.md`](reference/IRONWOOD_WALLET_READINESS.md) | **Ironwood NU6.3** — migration case breakdown, v2.4.0 wrap-up (links to dynamic-fee doc) |
| [`reference/KYC_INBOUND_PRIVACY_CASE_BREAKDOWN.md`](reference/KYC_INBOUND_PRIVACY_CASE_BREAKDOWN.md) | **KYC / `t` inbound privacy** — throwaway UAs, quarantine, mix warnings, Zodl/Zingo/Zkool research cases |
| [`reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md`](reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md) | **Nym IP / egress** — IP↔tx submit as biggest win; smolmix broadcast vs smol-dvpn sync cases |
| [`reference/NOZY_LITE.md`](reference/NOZY_LITE.md) | **Nozy Lite** — CLI/TUI for operators: uptime, health, data checks |
| [`reference/NOZY_LITE_BENCHES.md`](reference/NOZY_LITE_BENCHES.md) | **Lite benches** — CLI vs desktop size / cold start (measured, not invented) |
| [`reference/SESSION_NYM_IRONWOOD_DESKTOP_CASE_BREAKDOWN.md`](reference/SESSION_NYM_IRONWOOD_DESKTOP_CASE_BREAKDOWN.md) | **2026-07-11 session** — Nym D2a–D2e + desktop Ironwood MVP + UX (scoreboard + leftover checklist) |
| [`ZEBRA_DEV_CLI_POSITIONING.md`](ZEBRA_DEV_CLI_POSITIONING.md) | **Positioning + roadmap:** Nozy CLI as the go-to **Zebra + Orchard** dev / debug tool |
| [`TESTNET_CLI_WALKTHROUGH.md`](TESTNET_CLI_WALKTHROUGH.md) | **Zebrad testnet + Nozy CLI:** start node, config, sync, TZEC send |
| [`BUSINESS_ZEC_ZNS_TODO.md`](BUSINESS_ZEC_ZNS_TODO.md) | **Business & POS + ZNS** — phased backlog (vendor Sell mode, name resolve) |
| [`BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md`](BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md) | Phase 0 locked decisions (profile model, indexer URLs, MVP criteria) |
| [`GITHUB_ISSUE_DRAFT_BUSINESS_ZNS.md`](GITHUB_ISSUE_DRAFT_BUSINESS_ZNS.md) | Paste-ready GitHub issue for maintainer alignment |
| [`journal/README.md`](journal/README.md) | **Development journal** — local-only (lecture / research; **not in git**) |
| [`ENHANCEMENT_ROADMAP.md`](../ENHANCEMENT_ROADMAP.md) | **Product roadmap** — web app, mobile, extension, multichain |
| [`web-app/README.md`](../web-app/README.md) | **Web app** — browser dashboard (starting) |
| [`landing/README.md`](../landing/README.md) | **Launchpad** — Vite/React product hub (ZEC-only surfaces today) |
| [`journal/research/2026-06-21-nozy-super-wallet-stack.md`](journal/research/2026-06-21-nozy-super-wallet-stack.md) | **Research:** Best stack for privacy super wallet (extension + api + future web-app) |
| [`journal/research/2026-06-21-cipherpay-nozywallet-integration.md`](journal/research/2026-06-21-cipherpay-nozywallet-integration.md) | **Research:** CipherPay merchant gateway vs native Sell mode — complement, defer impl |
| [`issues/README.md`](issues/README.md) | **Bugs & features** — registry, templates, RCA writeups |
| [`ZCASH_RESIDENCY_FACILITATOR_DAYPACK.md`](ZCASH_RESIDENCY_FACILITATOR_DAYPACK.md) | **Zcash Developers Residency** — 20-day facilitator guide (NozyWallet build path) |
| [`issues/BUG_REGISTRY.md`](issues/BUG_REGISTRY.md) | Master bug list (ID, status, fix version) |
| [`MAINNET_SEND_EXPIRY_TEST.md`](MAINNET_SEND_EXPIRY_TEST.md) | **Mainnet test** — BUG-2026-011 send/expiry verification (api-server + CLI) |
| [`reference/PILOT_EXPIRY_PROVING_LATENCY.md`](reference/PILOT_EXPIRY_PROVING_LATENCY.md) | **Paper reference** — pilot expiry vs proving latency, 5 vs 15, Zodl FAQ |
| [`reference/MAINNET_SEND_READINESS_EVIDENCE.md`](reference/MAINNET_SEND_READINESS_EVIDENCE.md) | **Paper / lecture** — recorded mainnet sync+send timings, TXIDs, witness guard |
| [`reference/CLI_BALANCE_NOTEINDEX.md`](reference/CLI_BALANCE_NOTEINDEX.md) | **Paper / lecture** — CLI balance bug (v2 NoteIndex), `wallet_balance_snapshot()`, dual-parser lesson |
| [`reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md`](reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md) | **Paper / lecture** — verify Zebrad ↔ NozyWallet RPC, WSL/Windows, config, diagnostics |
| [`../book/README.md`](../book/README.md) | **User book (mdBook)** — `SUMMARY.md` TOC; build with `mdbook build` |
| [`reference/WHITEPAPER_OUTLINE.md`](reference/WHITEPAPER_OUTLINE.md) | **White paper blueprint** — architecture decisions, phases, trade-offs, lessons learned |
| [`NozyWallet_Whitepaper.docx`](NozyWallet_Whitepaper.docx) | **White paper** (Word) — run `python scripts/generate-nozy-whitepaper.py` |
| [`reference/NozyWallet_Whitepaper.md`](reference/NozyWallet_Whitepaper.md) | Same white paper as **Markdown** (readable in Cursor) |
| [`PILOT_MAINNET_EVIDENCE.md`](PILOT_MAINNET_EVIDENCE.md) | Dynamic-fee pilot mainnet evidence (expire + speed-up) |
| [`issues/FEATURE_REGISTRY.md`](issues/FEATURE_REGISTRY.md) | Feature / enhancement tracker |
