# Tor Integration

Route selected NozyWallet connections over **Tor** (experimental).

## Requirements

- Tor daemon or Tor Browser SOCKS proxy running locally
- `nozy privacy-network` subcommands supported in your build

## Typical SOCKS

Default Tor SOCKS: `127.0.0.1:9050`

Configuration varies by platform — see CLI help output for current flags.

## Zebrad over Tor

Running Zebrad as a hidden service or connecting to `.onion` RPC is an **advanced operator** setup. Document your onion URL in `zebra_url` if used:

```bash
nozy config --set-zebra-url http://youronion.onion:8232
```

Latency and sync time increase significantly.

## Limitations

- Not all builds test Tor paths in CI
- lightwalletd gRPC over Tor may need additional proxy configuration

See [Setup Guide](setup.md).
