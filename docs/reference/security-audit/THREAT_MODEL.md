# Threat model — NozyWallet CLI + core + Desktop

**Audience:** AI-assisted audit strategies + human triage (LA-style)  
**Date:** 2026-08-01  
**Scope:** [`SCOPE.md`](SCOPE.md)

---

## Assets

| Asset | Sensitivity |
|-------|-------------|
| BIP-39 mnemonic / spending keys | Critical |
| Wallet encryption password | Critical |
| Unspent note plaintexts (`notes.json`) | High (spend material + targeting) |
| Presigned Ironwood raw txs | High (windowed broadcast + metadata) |
| Migration schedule / TXIDs | Medium (forensics, privacy linkage) |
| Turnstile amounts on chain | Public by design (NU6.3) |

## Adversaries

1. **Disk thief** — steals `%APPDATA%\nozy` without process access  
2. **Malicious / compromised renderer** — Tauri webview XSS or hostile IPC caller  
3. **LAN observer** — shared Wi‑Fi; can reach mis-bound api-server or LAN Zebrad  
4. **Remote LWD / Zebrad operator** — sees IP ↔ sync / submit timing; turnstile amounts are public  
5. **Malicious dApp** — out of scope for this pass (extension)  
6. **Supply-chain** — malicious crate / Dependabot lag (`cargo audit`)

## Primary threats (mapped to LA focus areas)

### Wallet key material
- Offline crack of `wallet.dat` (KDF strength)
- Mnemonic printed to stdout / terminal scrollback on create
- Password held in Desktop process memory while unlocked
- Plaintext note cache beside encrypted vault

### Signing authority
- Anyone who can invoke unlocked Desktop IPC can migrate/send without step-up auth
- Presigned turnstile bytes on disk submitted by another process in-window

### Transaction correctness
- Wrong fee / Ironwood change on turnstile
- Witness lag / wrong anchor → failed or unsafe spend
- `f64` amount conversion on Desktop send
- Note merge dropping or duplicating equal-value notes

### Session / auth integrity
- Long-lived unlock session
- Privacy attestation / `force_clearnet` as client-supplied bools
- api-server optional API key + mnemonic restore on HTTP

### Privacy (Ironwood-specific)
- Clearnet submit after “Tor detected” policy pass (gate ≠ transport)
- RFC1918 Zebrad treated as “local” (LAN sees IP↔amount)
- Tip-sync → immediate broadcast correlation (hygiene skip)

### State corruption / forensics
- Schedule rebuild wipes `broadcast_txid` / Confirmed history
- Twin-note merge regressions

## Assumptions we rely on

- Operator runs Zebrad they trust (preferably loopback)
- Desktop webview is not hostile unless XSS / local malware
- Upstream `orchard` / `librustzcash` proofs are sound (out of scope)
- Users who choose `--force-clearnet` / attestation accept IP linkage risk

## Controls already present (baseline)

- Encrypted `wallet.dat` (AES-GCM + iterated SHA-256 KDF — see findings)
- Migration network privacy gate + tip-sync / delay hygiene
- ZIP 318 zero Ironwood-change invariant at turnstile build
- Twin-note merge by nullifier (+ tests)
- Mainnet CLI confirm (`SEND`) for some send paths
- Optional api-server API key / rate limit

## Desired hardening (product backlog)

See [`FINDINGS.md`](FINDINGS.md) remediations. Highest leverage: bind privacy policy to actual HTTP client; encrypt note cache; Argon2id for vault; preserve migration history; Desktop step-up for migrate/broadcast; default api-server to loopback.
