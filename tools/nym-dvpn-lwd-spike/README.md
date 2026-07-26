# nym-dvpn-lwd-spike

Isolated engineering spike for **Nozy Priority 1 (sync path)**: Zcash **lightwalletd compact-block sync** through Nym **2-hop smoldvpn**.

> **Product priority update:** The **biggest win** for IP privacy is routing **outgoing tx submit** over Nym (smolmix), not sync alone. See [`NYM_IP_PRIVACY_CASE_BREAKDOWN.md`](../../docs/reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md) and [issue #147](https://github.com/LEONINE-DAO/Nozy-wallet/issues/147). This dVPN spike remains useful for sync metadata / censorship.

- Tracking: [issue #146](https://github.com/LEONINE-DAO/Nozy-wallet/issues/146)
- Based on Nym’s [`zcash-sync`](https://github.com/nymtech/nym/blob/develop/smoldvpn/examples/zcash-sync.rs) example
- **Deps (Mark / 2026-07):** `feature/nym-sdk-dvpn` retired → `smoldvpn` + `nym-sdk-session` from `nymtech/nym` **`develop`**. Mixnet spike uses crates.io `smolmix = "1.21.4"`.
- **Not** wired into default `nozy` / desktop builds (crate is excluded from the root workspace)

## What this is / is not

| This spike | Product today |
|------------|----------------|
| Proves dVPN + gRPC compact sync throughput | Safer-migration gate still uses local Zebrad / Tor SOCKS / attestation |
| Opt-in CLI under `tools/` | No desktop toggle yet |
| Needs funded mainnet (or sandbox) Nyx mnemonic + ticketbooks | End-user credential-proxy / gifted zk-nyms are follow-up |

**Live status (2026-07-26):** clearnet LWD sync **PASS**; mainnet ticketbooks recoverable after issuance FAIL; smol-dvpn gateway register still **FAIL** (timeouts). See [`NYM_DVPN_SYNC_CASE_BREAKDOWN.md`](../../docs/reference/NYM_DVPN_SYNC_CASE_BREAKDOWN.md).

Next spikes (not this crate): `zeaking` `connect_with_connector`, then **smolmix** for migrate-broadcast, then **mix-fetch** for the extension.

## Prerequisites

1. Rust toolchain (same as Nozy root).
2. Funded Nym mnemonic with enough `$NYM` for WireGuard ticketbooks (~225 NYM / ticketbook PAYG class).
3. **Network defaults:** spike uses **mainnet** when `NETWORK_NAME` + `NYM_API` are unset. Only source sandbox env if your funds are sandbox:

   ```bash
   # Sandbox only:
   set -a; source envs/sandbox.env; set +a
   export MNEMONIC="<funded sandbox mnemonic>"
   ```

   Mainnet (typical Keplr / swap.nym.com `$NYM`):

   ```powershell
   $env:MNEMONIC = "<funded mainnet mnemonic — shell only>"
   # optional: $env:DVPN_DIRECTORY_URL = "https://nymvpn.com/api/public/v1/directory/gateways?show_vpn_only=true"
   ```

4. Build **`--release`** — boringtun is much slower in debug and dominates tunnel timings.
5. Disconnect the consumer **NymVPN** app (`nym-vpnd`) while measuring the SDK spike.

### Dependency note

Nym currently pulls `libcrux-psq 0.0.8`, which fails to compile on rustc 1.88 (`E0716`). This spike vendors a one-line fix under `vendor/libcrux-psq` via `[patch.crates-io]`. Remove the patch when upstream Nym bumps past the broken crate.

## Run

```powershell
cd tools/nym-dvpn-lwd-spike
$env:MNEMONIC = "<funded mainnet or sandbox mnemonic — shell only>"
# For sandbox only: also source Nym envs/sandbox.env (NETWORK_NAME + NYM_API)
cargo run --release -- --blocks 1000
```

Useful flags:

```text
--blocks N          compact blocks to stream (default 10000)
--lwd URL           override LWD (default https://zec.rocks:443 or LIGHTWALLETD_GRPC)
--two-hop           default
--one-hop           single gateway
--quic              QUIC bridge on entry (two-hop only)
--entry / --exit    random | CC | base58 identity
```

Example output compares **direct** vs **tunnel** blocks/s.

### Fallback if git deps fail

`nym-smol-dvpn` is not on crates.io yet. If `cargo` cannot resolve the git branch:

```bash
git clone -b feature/nym-sdk-dvpn https://github.com/nymtech/nym.git
cd nym
set -a; source envs/sandbox.env; set +a
export MNEMONIC="..."
cargo run --release -p nym-smol-dvpn --example zcash-sync -- --blocks 1000
```

Then keep this README / issue as the Nozy tracking surface.

## AI disclosure

Spike scaffold assisted by Cursor Agent (implementation draft). Human review required before any product wiring.

## License notes

Helpers adapted from Nym `smol-dvpn` examples are Apache-2.0 (Nym Technologies SA). Spike packaging under Nozy is MIT like the rest of the wallet tree; do not relicense Nym code.
