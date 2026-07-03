# Privacy Network Setup

Experimental checklist for Tor/I2P with NozyWallet.

## 1. Base wallet works without proxy

```bash
nozy test-zebra
nozy sync --to-tip
```

Fix [Zebra connectivity](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md) first.

## 2. Start privacy network

**Tor:** system Tor service or Tor Browser with SOCKS on 9050.

**I2P:** I2P router with documented SOCKS/http tunnel ports.

## 3. Configure Nozy

```bash
nozy privacy-network --help
```

Apply flags or config per subcommand output — interfaces evolve between releases.

## 4. Verify

- RPC still reaches Zebrad (may be slower)
- Sync completes
- Send dust on testnet before mainnet

## 5. Operational notes

- Backup mnemonic before experimenting
- Expect higher latency and timeouts (adjust patience / NET_002)
- Document your RPC URL scheme for lectures

## Related

- [Tor Integration](tor.md)
- [I2P Integration](i2p.md)
