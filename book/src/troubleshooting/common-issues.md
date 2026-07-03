# Common Issues

Quick fixes for the problems operators hit most often. For Zebrad connectivity in depth, see [Zebra Node Setup](../advanced/zebra-node.md) and the [connectivity reference](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md).

## Cannot connect to node (NET_001)

**Symptoms:** Settings test fails; sync errors; `nozy test-zebra` fails.

**Checks:**

1. Zebrad running? `Get-Process zebrad` (Windows) or `ps aux | grep zebrad` (Linux).
2. RPC enabled in `zebrad.toml` — Windows: **`%LOCALAPPDATA%\zebrad.toml`**.
3. Correct URL in config — WSL users need **WSL IP**, not `127.0.0.1`, unless forwarding is set up.
4. Port conflict — Windows may use **8232** for IP Helper; try **18232**.

```bash
nozy test-zebra
nozy config   # show zebra_url
```

## Desktop opened in browser instead of app

**Symptoms:** Every action fails; “use desktop window” style errors.

**Fix:** Close the browser tab at `localhost:5173`. Use the **NozyWallet** taskbar window from `cargo tauri dev` or the installed `.exe`.

## Sync succeeds but send blocked

**Symptoms:** “Witness N blocks behind…” or “sync to tip before sending”.

**Fix:** Run sync to tip; wait for witness refresh. Scan height at tip ≠ witnesses fresh. Use desktop sync status or `nozy status`.

## Balance shows zero (CLI)

**Symptoms:** `nozy balance` is 0 but notes exist.

**Fix:** Known v2 NoteIndex parser issue — see [CLI balance reference](../../../docs/reference/CLI_BALANCE_NOTEINDEX.md). Try `nozy list-notes` or desktop balance; update to fixed build.

## Send takes many minutes

**Expected** on first shielded send: Orchard proving (often 2–4 minutes on laptop-class CPU). Subsequent sends faster if proving key is warm.

Ensure witness lag ≤ 50 blocks before send — stale wallets spend time on witness catch-up.

## Invalid recipient address

Unified Orchard addresses start with **`u1`** (mainnet) or **`utest1`** (testnet). Transparent `t1` addresses are not supported.

## Proving parameters missing

```bash
nozy proving --download
nozy proving --status
```

## lightwalletd / compact sync timeout

Desktop send can hang on “Checking sync status…” if lightwalletd on `127.0.0.1:9067` is down. Start lightwalletd or wait for timeout; Zebrad RPC sync may still work via CLI.

## Multi-wallet / wrong profile

Desktop Welcome lists profiles. Ensure the active profile matches the wallet you expect before send.

## Keystone pairing or PCZT send fails

**Symptoms:** “Keystone requires mainnet”; UFVK export disabled; prepare/broadcast errors; Keystone rejects QR.

**Checks:**

1. Wallet config `network` must be **`mainnet`** (Keystone is not supported on testnet).
2. Keystone device set to **Zcash mainnet** — not testnet.
3. UFVK should start with **`uview1`** on mainnet; re-export from Nozy **Settings → Keystone**.
4. Same mnemonic (or matching UFVK + seed on Keystone) — UFVK alone cannot sign.
5. Recipient must be **`u1…`** Orchard unified address.
6. Sync to tip before prepare; re-prepare if the signed PCZT expired.

Full guide: [Keystone Hardware Wallet](../security/keystone-hardware-wallet.md).

---

More: [Error Messages](error-messages.md) | [Desktop Troubleshooting](../desktop-app/troubleshooting.md) | [Getting Help](getting-help.md)
