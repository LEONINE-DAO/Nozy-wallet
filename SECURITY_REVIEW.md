# NozyWallet — security review checklist

Use this for a **structured review** before a major release or store submission. It does not replace a professional audit for high-value wallet software.

**Related (2026-08 AI-assisted prep):** [`docs/reference/security-audit/`](docs/reference/security-audit/) — threat model + triaged findings. Prefer remediating High items in [`FINDINGS.md`](docs/reference/security-audit/FINDINGS.md) before claiming GA.

## 1. Secrets and key material

- [ ] **Mnemonic / seed:** never logged, never in URLs, never sent to **third-party** hosts. Companion create/restore may accept mnemonic on **loopback only** under [`api-server/SEED_POLICY.md`](api-server/SEED_POLICY.md) — verify handlers do not log it and responses stay masked (`display_mnemonic_safe`).
- [ ] **Passwords:** only used for local encryption; verify Argon2 / storage paths in `WalletStorage` and extension `encrypt_for_storage`.
- [ ] **Memory:** confirm `zeroize` / `SecureSeed` on sensitive buffers where the codebase already uses them; no new `String` copies of mnemonics without clear lifecycle.
- [ ] **Grep (local):** `rg -i "mnemonic|seed phrase|private_key|spending_key" --glob '!**/target/**' --glob '!**/node_modules/**'` — inspect hits for logging or error strings that could leak.

## 2. Browser extension (MV3)

- [ ] **`manifest.json`:** `host_permissions` — justify each pattern; document for store reviewers.
- [ ] **Service worker:** mnemonic only in `chrome.storage.session` for scan resume; cleared on lock (verify `walletLock` / `clearScanResumeForBackground`).
- [ ] **Content script / provider:** validate origin handling for `eth_requestAccounts` / Zcash provider; no arbitrary script injection into privileged context.
- [ ] **No debug exfil:** no `fetch` to non-user URLs with chain or wallet payload (grep `fetch(` in `browser-extension/background/`).
- [ ] **WASM boundary:** built from pinned `wasm-core` / `Cargo.lock`; reproducible `wasm-pack` release build in CI.

## 3. JSON-RPC (Zebra)

- [ ] **Cookie / TLS:** document safe defaults (`LOCAL_RPC.md`); never recommend disabling auth on public networks.
- [ ] **MITM:** user-educated on HTTPS / VPN for non-localhost RPC; extension cannot fix hostile network alone.

## 4. `api-server` (companion)

**Policy:** [`api-server/SEED_POLICY.md`](api-server/SEED_POLICY.md) — localhost companion may accept mnemonic for create/restore; all-interfaces bind requires `NOZY_API_KEY`.

- [x] **Bind address:** default `127.0.0.1`; `NOZY_BIND_ADDR=0.0.0.0` only for intentional LAN/hosted (`main.rs` refuses without API key).
- [x] **CORS / auth:** documented in [`api-server/SECURITY_CONFIG.md`](api-server/SECURITY_CONFIG.md); API key required when binding all interfaces **or** when `NOZY_PRODUCTION` is set (even on loopback); production CORS via `NOZY_PRODUCTION` + `NOZY_CORS_ORIGINS`.
- [x] **Seed on wire (honest):** create/restore accept mnemonic on loopback by design; responses masked; do not claim “never on wire.” Re-verify before each major release (handlers + extension companion path).

## 5. Desktop (Tauri)

- [ ] **IPC:** review Tauri `allowlist` / commands for path traversal or arbitrary file read.
- [ ] **Updates:** signer identity and update channel policy.

## 6. Supply chain

- [ ] **`cargo audit`:** resolve or document accepted risk for open `RUSTSEC` items (see CI `security-audit` job).
- [ ] **Lockfiles:** PRs that change `Cargo.lock` / `package-lock.json` get extra scrutiny.

## 7. Disclosure

- [ ] **Vulnerabilities:** follow `CONTRIBUTING.md` responsible disclosure; do not file public issues for undisclosed exploits.

## Sign-off

| Reviewer | Date | Scope (e.g. extension only) | Notes |
|----------|------|-------------------------------|--------|
|          |      |                               |        |
