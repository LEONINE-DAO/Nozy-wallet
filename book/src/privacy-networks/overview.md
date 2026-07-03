# Privacy Networks Overview

NozyWallet can route **some** traffic through privacy networks (Tor, I2P) for metadata resistance when connecting to nodes or services.

> **Status:** Experimental. Production ZEC path today assumes direct or VPN-protected Zebrad RPC.

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

- [Tor Integration](tor.md)
- [I2P Integration](i2p.md)
- [Setup Guide](setup.md)

## Related

- [Network Configuration](../advanced/network-config.md)
- [Security Features](../features/security.md)
