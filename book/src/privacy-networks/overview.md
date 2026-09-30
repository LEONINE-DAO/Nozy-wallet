# Privacy Networks Overview

NozyWallet can route **some** traffic through privacy networks (Tor, I2P) for metadata resistance when connecting to nodes or services.

> **Status:** Experimental. Production ZEC path today assumes direct or VPN-protected Zebrad RPC. Prefer a **local Zebrad**. Ironwood stopgap: free NymVPN for shielded ZEC holders at [zcash.nym.com](https://zcash.nym.com) — Fast mode for compact sync, Mixnet mode and a new exit for send/broadcast. Do not sync and migrate-broadcast through the same hosted lightwalletd a minute later.

## Nym hybrid (how Nozy actually uses it)

This is **not** one “Nym mode.” Three hops, only one of which is used at a time for send:

1. **Local Zebrad** (loopback or your LAN/WSL node) — default. Mixnet is **skipped on purpose**. IP never leaves your box for submit.
2. **Mixnet (smolmix)** — remote `sendraw` / broadcast only, when Zebrad is a **public** URL. Small packets. Not for sync.
3. **dVPN / Fast mode** — remote **compact-block sync** only. Not for submit.

Consumer NymVPN at [zcash.nym.com](https://zcash.nym.com) is a **stopgap OS VPN** if you have no local node. It is not the same as in-app mixnet/dVPN helpers.

Check what *this wallet* will do on the next send:

```bash
nozy privacy-network send-egress
```

Engineering map (git repo): [`docs/reference/NYM_SEND_EGRESS_CASE_BREAKDOWN.md`](../../../docs/reference/NYM_SEND_EGRESS_CASE_BREAKDOWN.md).

## Why privacy networks

- Hide wallet IP from RPC provider
- Reduce network-level correlation

## What they do not hide

- On-chain shielded cryptography (already private for amounts/parties in Orchard)
- Malicious RPC — use trusted nodes regardless of Tor

## CLI entry

```bash
nozy privacy-network --help
```

Subcommands test connectivity and configure proxies per build.

## Chapters

- [Lesson: Privacy besides the ledger](beyond-the-ledger.md) — IP, timing, hygiene, and what Nym cannot hide
- [Tor Integration](tor.md)
- [I2P Integration](i2p.md)
- [Setup Guide](setup.md)

## Related

- [Network Configuration](../advanced/network-config.md)
- [Security Features](../features/security.md)
