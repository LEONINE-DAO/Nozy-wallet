# Private Key Management

NozyWallet derives keys from a **BIP39 mnemonic** using standard Zcash Orchard HD paths.

## What you hold

| Secret | Role |
|--------|------|
| **24-word mnemonic** | Root secret — restores entire wallet |
| **Wallet password** | Encrypts local `wallet.dat` / profile storage |
| **In-memory keys** | Active after unlock; cleared on lock |

Private keys are **not** displayed in normal UI. Optional CLI/settings paths may expose viewing keys for advanced debugging — treat as sensitive.

## Derivation

- HD wallet follows Bcash/Zcash conventions in `hd_wallet` / orchard integration.
- Optional **Secret Network** path shares mnemonic, different derivation — see [Secret Network](../advanced/secret-network.md).

## Storage

Encrypted at rest — see [Wallet Storage](wallet-storage.md).

- Argon2 / PBKDF-style password protection in core library
- `zeroize` for sensitive buffers where applicable

## Operational rules

1. Never paste mnemonic into websites.
2. Never photograph seed on a networked phone.
3. Hardware wallet: **Keystone** on Zcash mainnet — [Keystone Hardware Wallet](keystone-hardware-wallet.md). Seed stays on device; Nozy builds PCZT for air-gapped signing.
4. Compromised machine → move funds to new wallet on clean device with new seed.

## Recovery

Only via mnemonic or encrypted backup — no “forgot password” server reset.

## Related

- [Backup Strategies](backup-strategies.md)
- [Security Best Practices](best-practices.md)
- [Keystone Hardware Wallet](keystone-hardware-wallet.md)
