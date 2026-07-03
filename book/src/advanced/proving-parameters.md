# Proving Parameters

Orchard shielded spends require **Halo2 proving parameters** (large files, downloaded once per machine).

## Download

```bash
nozy proving --download
nozy proving --status
```

Desktop: **Settings → Proving** or automatic download on first send.

## Size and location

Parameters are cached on disk in the platform cache directory (see `proving.rs` / OS-specific paths). Expect **hundreds of MB** download.

## First send latency

First prove in a session loads/builds proving keys — often **1–3+ minutes** additional wall time. Warm subsequent sends are faster.

CLI and api-server may call `warm_orchard_proving_key()` at unlock/startup in some builds.

## Offline / airgap

Download on a networked machine first, then copy cache directory per operator docs for your OS version.

## Failure

| Symptom | Fix |
|---------|-----|
| PROVE_001 | Re-run `--download` |
| Disk full | Free space, retry |
| Corrupt cache | Delete cache dir, re-download |

## Related

- [Quick Send Tutorial](../examples/quick-send.md)
- [Mainnet send timings](../../../docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md)
