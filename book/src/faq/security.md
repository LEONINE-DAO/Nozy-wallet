# Security Questions

## Is NozyWallet custodial?

No. Keys and mnemonic stay on your device. Nozy does not hold funds or recovery phrases on your behalf.

## What is encrypted on disk?

Wallet files (mnemonic-derived material, notes index) are encrypted with a password-derived key. See [Wallet Storage](../security/wallet-storage.md).

## Should I use a password?

Yes. Use a strong unique password for wallet unlock. It protects encrypted storage if someone copies your data directory.

## How should I back up?

**Primary:** 24-word mnemonic on paper, offline.  
**Optional:** encrypted export from desktop.  
Details: [Backup Strategies](../security/backup-strategies.md).

## Does NozyWallet support transparent addresses?

No. Orchard shielded only — reduces accidental privacy leaks.

## Is the desktop app safe to run from a browser tab?

No. Use the **NozyWallet desktop window** from the taskbar. Opening `localhost:5173` in Chrome/Cursor has no wallet backend.

## What RPC do I trust?

Your **Zebrad** (and optional **lightwalletd**) node sees broadcast metadata and sync queries. Run your own node or trust your VPS explicitly. See [Zebra Node Setup](../advanced/zebra-node.md).

## Has the code been audited?

Dependency auditing is documented in [Security Audits](../security/audits.md). No third-party audit certificate is claimed for all surfaces.

## How do I report a vulnerability?

Private responsible disclosure — see [Contributing Guide](../contributing/guide.md) and root `CONTRIBUTING.md`. Do not post exploits publicly before coordination.

## Are experimental features (Secret, Monero, swap) production-ready?

Treat optional CLI features as **experimental** unless listed in release notes. Default production path is **Orchard ZEC** via CLI and desktop.

## Does NozyWallet support Keystone?

Yes — **Zcash mainnet** Orchard sends with air-gapped PCZT signing. Pair in **Settings → Keystone** (desktop) or the **Keystone** screen (mobile). Full guide: [Keystone Hardware Wallet](../security/keystone-hardware-wallet.md).

## Is UFVK export safe?

UFVK is **read-only** — it reveals shielded activity to whoever holds it, but cannot spend. Use only for Keystone pairing or trusted watch-only setups. Never confuse UFVK with spending keys.

## Can I use Keystone on testnet?

No. Keystone integration in NozyWallet is **mainnet only**. Use normal local signing on testnet if needed.
