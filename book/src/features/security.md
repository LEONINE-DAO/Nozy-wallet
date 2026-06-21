# Security Features

NozyWallet treats wallet software as **high-impact**: keys, addresses, and transaction data must be handled carefully. Security is built into storage, defaults, and operational guidance—not optional hardening.

## Self-custodial by design

- **Your keys, your wallet** — Nozy does not custody funds or hold recovery phrases on your behalf.
- **Local wallet file** — Encrypted on disk; unlocked only with your password (Argon2-based protection in the core library).
- **No transparent fallback** — Shielded-first flows reduce accidental address-type mistakes that leak metadata.

## Password and storage

- **Argon2** hashing for wallet encryption at rest.
- **Zeroize** patterns for sensitive buffers in Rust paths where applicable (see project security docs and `AGENTS.md`).
- **Backup responsibility** — You must secure your BIP39 mnemonic; loss of mnemonic and password means loss of funds.

## Network and node security

- **Your node, your trust model** — Recommended stack: **Zebrad** + **lightwalletd** you control or explicitly trust.
- **Companion API** — `nozywallet-api` is intended for **localhost**; exposing it publicly requires TLS, API keys, and VPS hardening ([VPS deploy guide](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/nozy-mobile/VPS-DEPLOY.md)).
- **Privacy networks** — Optional Tor/I2P configuration for RPC (see Privacy Networks chapters in this book).

## Orchard and proving

- **Standard Zcash crates** — Orchard / librustzcash; no custom ciphers or hand-rolled proofs in the wallet core.
- **Proving parameters** — Orchard uses Halo 2; manage proving resources per installation docs (no legacy Sapling parameter download for Orchard-only paths).

## Extension and web surfaces

- **Browser extension** — MV3 + WASM; compact sync via companion API ([companion docs](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/browser-extension/COMPANION.md)).
- **Web app (in development)** — Dashboard talks to **`nozywallet-api`**; seeds are not stored in the static launchpad site ([web-app README](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/web-app/README.md)).

## Viewing keys and disclosure

- **UFVK export** (e.g. Keystone pairing) is **read-only** — can reveal shielded activity to whoever holds the viewing key; never equivalent to spending authority.
- Planned business **accountant disclosure** flows are documented separately in project RFCs; treat UFVK sharing as irreversible for privacy against that party.

## Open source and review

- Source: [LEONINE-DAO/Nozy-wallet](https://github.com/LEONINE-DAO/Nozy-wallet)
- Report suspected vulnerabilities responsibly per [`CONTRIBUTING.md`](../../CONTRIBUTING.md); do not file public issues for undisclosed exploits.

## Best practices

For day-to-day habits (backups, device security, node hygiene), see [Security Best Practices](../security/best-practices.md) and [Private Key Management](../security/key-management.md).
