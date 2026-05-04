# NozyWallet — security review checklist

Use this for a **structured review** before a major release or store submission. It does not replace a professional audit for high-value wallet software.

## 1. Secrets and key material

- [ ] **Mnemonic / seed:** never logged, never in URLs, never sent to companion API or third-party hosts (verify `api-server` handlers and extension message paths).
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

- [ ] **Bind address:** default `0.0.0.0` vs `127.0.0.1` — confirm intended deployment; firewall story for LAN.
- [ ] **CORS / auth:** if exposed beyond localhost, require auth token or mTLS (not implemented by default — document risk).
- [ ] **No wallet seed on wire:** companion routes must not accept mnemonics (verify handlers).

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
