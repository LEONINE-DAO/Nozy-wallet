# AGENTS.md — guidance for AI coding agents (NozyWallet)

This file is **machine-readable project policy**. Tools (Cursor, Copilot, Claude Code, etc.) should load it **before** generating patches or advising opening a PR. Humans: see also [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Scope

- **This repository is a wallet and related services**, not a Zcash consensus node. Do **not** treat it as Zebra; do not propose consensus-rule changes here.
- **Default privacy product stance:** shielded-first; respect existing docs on Zebrad vs zcashd capabilities.

---

## Contribution gate (reduce maintainer churn)

Before **large or behavior-changing** work (new pool, new RPC surface, sync architecture, crypto, or multi-crate refactors):

1. **Open or reference a GitHub issue** describing intent and approach; prefer alignment with maintainers for anything that touches protocol, security, or release surfaces.
2. **Disclose AI assistance** in the PR description: which tool(s), and how (e.g. draft implementation vs review-only). The **human author remains responsible** for correctness and security.
3. **Be ready to explain** design trade-offs and how changes interact with **Zebra JSON-RPC**, **lightwalletd**, and **witness/anchor** flows.

Agents should **warn the user** if they are about to open a drive-by PR with no issue and no disclosure.

---

## Repository map (where code belongs)

| Area | Path | Notes |
|------|------|--------|
| Core library + CLI | Repo root (`src/`, `Cargo.toml` package `nozy`) | Orchard/Sapling wallet logic, `ZebraClient`, transaction building |
| Compact sync / LWD | `zeaking/` | Workspace member; `zeaking::lwd` shared by desktop, api-server, FFI |
| HTTP API | `api-server/` | Workspace member; localhost companion for extension |
| Mobile bindings | `zeaking-ffi/` | Workspace member (UniFFI) |
| Desktop UI | `desktop-client/` | Tauri app under `desktop-client/src-tauri/` (own `Cargo.toml`) |
| Extension WASM | `browser-extension/wasm-core/` | **Excluded** from root workspace; build with its own `Cargo.toml` / `wasm-pack` as documented |
| Book / site docs | `book/`, `landing/` | User and contributor docs |

Root **`[workspace]`** members: `zeaking`, `api-server`, `zeaking-ffi`. Crates under **exclude** must not be assumed part of `cargo build` at the root unless documented.

---

## Zebrad (Zebra) and shielded sends

- **Source of truth:** [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](ZEBRAD_SHIELDED_SEND_LIMIT.md).
- Wallet flows must **not** rely on zcashd-only witness RPCs (`z_findnoteposition`, `z_getauthpath`) for Orchard/Sapling proves; use the **documented** treestate + client-side witness approach and JSON-RPC where required.
- If suggesting new node RPCs or “just use zcashd,” **stop** and re-read that doc and current `ZebraClient` / witness providers.

---

## Build, format, and test (agents must run or preserve these)

From repository root unless a sub-crate README says otherwise:

```bash
cargo fmt --all -- --check
cargo build
cargo test
```

CI enforces **`cargo fmt --all -- --check`** on pushes/PRs. After edits, run **`cargo fmt --all`** so CI does not fail.

Subprojects (desktop Tauri, `browser-extension/wasm-core`) have their own manifests; follow each crate’s README when changing those trees.

---

## Code conventions (match existing code)

- **Minimal diffs:** Do not refactor unrelated modules, rename public APIs, or “clean up” files outside the task.
- **Errors:** Use existing `NozyResult` / `NozyError` patterns; avoid panics in library paths for recoverable failures.
- **Crypto:** Use **librustzcash / orchard / sapling-crypto** and existing crate patterns; **no** custom ciphers or hand-rolled proofs.
- **Secrets:** Prefer existing patterns (`zeroize`, secure storage); do not log seeds, keys, or raw mnemonics.
- **Imports and style:** Follow `rustfmt` and existing module layout; see [`CONTRIBUTING.md`](CONTRIBUTING.md).

---

## Security and privacy

- Treat all wallet code as **high impact**: keys, addresses, transaction data, and RPC parameters can harm users if mishandled.
- **Do not** add telemetry that exfiltrates addresses, amounts, or wallet state without explicit project direction and user consent.
- For suspected vulnerabilities, follow **responsible disclosure** in [`CONTRIBUTING.md`](CONTRIBUTING.md) (do not file public issues for undisclosed exploits).

---

## Pull request expectations

- Link related **issues** when applicable.
- State **what changed**, **how tested**, and **AI disclosure** (if any).
- If CI fails on format, run `cargo fmt --all` and push again.

---

## When to stop and ask a human

- Consensus behavior, network forks, or Zebra protocol bugs → **Zebra / ZIP process**, not this repo.
- Ambiguous requirements or conflicting docs → **issue first**, then code.
- Large dependency upgrades → follow existing Dependabot / lockfile practices; do not bump Zcash crates casually without building all affected surfaces.
