# Least Authority–style AI-assisted audit — Scope (NozyWallet)

**Engagement type:** In-repo AI-assisted self-review (mirrors [LA methodology](https://leastauthority.com/blog/ai-assisted-security-auditing-in-the-zcash-ecosystem/))  
**Date:** 2026-08-01  
**Surfaces in scope:** CLI (`nozy`) + shared `nozy` core + Desktop Tauri  
**Not a Least Authority certificate.** Their ZCG engagement audited zebra / zcashd / orchard / halo2 / librustzcash. Nozy *consumes* those crates.

---

## In scope

| Area | Paths / surfaces |
|------|------------------|
| Key material | `src/storage.rs`, `src/hd_wallet.rs`, `src/key_management.rs`, CLI create/reveal |
| Signing / prove | `src/orchard_tx.rs`, `src/ironwood_tx.rs`, PCZT builders |
| Ironwood / ZIP 318 | `src/ironwood/` (plan, split, migrate, broadcast, network privacy, hygiene) |
| Note cache integrity | `src/notes.rs`, `src/note_index.rs`, `src/wallet_sync.rs` |
| Zebrad RPC client | `src/zebra_integration.rs` (proxy, local detection, broadcast) |
| Desktop IPC / session | `desktop-client/src-tauri/src/` (session, ironwood, backup, transaction, lwd) |
| Companion API (desktop-adjacent) | `api-server/` bind, auth, mnemonic-on-wire — only as operator risk |

## Out of scope (this pass)

- Orchard / Ironwood **circuit soundness** and halo2 (upstream / LA ZCG reports)
- Browser extension WASM / MV3 (separate threat model: [`browser-extension/THREAT_MODEL.md`](../../../browser-extension/THREAT_MODEL.md))
- Mobile FFI (`zeaking-ffi`, `nozy-ffi`, Expo)
- Consensus node behavior (Zebra)
- Third-party penetration testing / formal verification

## Trust boundaries

```text
[ User / OS ] --password--> [ wallet.dat encrypted blob ]
[ Unlocked process ] ------> [ notes.json plaintext cache ]
[ CLI / Tauri UI ] --------> [ nozy core prove/sign ]
[ nozy core ] -------------> [ Zebrad JSON-RPC ]  (local preferred)
[ Optional ] --------------> [ api-server HTTP ]  (companion)
```

## Non-goals

- Claiming “Least Authority audited NozyWallet”
- Shipping exploit PoCs (prefer adversarial unit tests)
- Replacing [`SECURITY_REVIEW.md`](../../../SECURITY_REVIEW.md) checklists

## Related evidence

- Mainnet Ironwood turnstile: [`MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md`](../MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md)
- Internal review: [`SECURITY_REVIEW.md`](../../../SECURITY_REVIEW.md)
- Engagement paste: [`ENGAGE_LEAST_AUTHORITY.md`](ENGAGE_LEAST_AUTHORITY.md)
