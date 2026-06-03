# Release v2.3.0 — CLI (after merging PR #35)

## Tag and publish

```bash
git checkout master
git pull origin master
git tag -a v2.3.0 -m "v2.3.0: ZIP-317 dynamic fees, CLI --priority"
git push origin v2.3.0
```

GitHub Actions [release.yml](../.github/workflows/release.yml) builds and attaches:

- `nozy-windows.exe`
- `nozy-linux`
- `nozy-macos-intel` / `nozy-macos-arm`
- `nozywallet-api-*` (same tag)

## Release notes (paste into GitHub Release)

**Title:** v2.3.0 — Dynamic fee pilot (CLI)

**Body:**

### Summary

- ZIP-317 client-side fees for Orchard sends (Zebrad has no `estimatefee`)
- `nozy send --priority` for opt-in ×4 fee (pilot)
- ~2-block transaction expiry at build time
- Desktop/api-server send surfaces updated (binaries in same workflow)

### CLI usage

```bash
nozy send -r u1... -a 0.1 --mainnet
nozy send -r u1... -a 0.1 --mainnet --priority
```

### Verify

- `cargo test fee_policy`
- Testnet send with and without `--priority`; compare on-chain fee

### Follow-ups

Extension WASM, expiry polling, speed-up — tracked in Phase A RFC.
