# Zebra Node Setup

NozyWallet is an Orchard wallet — **not** a Zcash consensus node. For sync, broadcast, and treestate you need a running **[Zebrad](https://github.com/ZcashFoundation/zebra)** instance with JSON-RPC enabled.

For a full operator guide (lectures, papers, Windows + WSL checklists), see **[Zebrad ↔ NozyWallet connectivity](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md)** in the repo docs.

---

## Quick setup

### 1. Install and configure Zebrad

```toml
# ~/.config/zebrad.toml  (Linux/macOS)
# %LOCALAPPDATA%\zebrad.toml  (Windows — authoritative path)

[rpc]
listen_addr = "127.0.0.1:8232"
```

On Windows, if port **8232** is already in use, pick another port (e.g. **18232**) and use that in the wallet config.

Start the node:

```bash
zebrad start
```

### 2. Point NozyWallet at RPC

**CLI:**

```bash
nozy config --set-zebra-url http://127.0.0.1:8232
nozy test-zebra
```

**Desktop:** Settings → Network → set node URL → **Test Connection**.

Config file locations:

| OS | Path |
|----|------|
| Windows | `%APPDATA%\nozy\nozy\config\config.json` |
| Linux / macOS | `~/.config/nozy/config.json` |

Optional override for one session: `ZEBRA_RPC_URL=http://host:port`.

### 3. Zebrad in WSL, wallet on Windows

Zebrad often runs inside WSL while the desktop app runs on Windows. Localhost on Windows is **not** the same as localhost inside WSL unless you set up forwarding.

```powershell
wsl hostname -I          # note the IP
nozy config --set-zebra-url http://<wsl-ip>:8232
nozy test-zebra
```

Or dot-source `scripts/zebra-wsl-rpc.ps1` from the repo to set `ZEBRA_RPC_URL` automatically.

---

## Verify connectivity

| Check | Command |
|-------|---------|
| Nozy → Zebrad | `nozy test-zebra` |
| Raw RPC | `curl` / PowerShell `getblockcount` (see reference doc) |
| Full smoke test (Windows) | `.\scripts\test-zebrad-nozywallet.ps1` |
| Sync | `nozy sync --to-tip` or desktop **Sync** |

**Pass:** block height returned and increasing over time; `last_scan_height` catches up to tip after sync.

---

## What Zebrad does not do

Orchard **witnesses** are derived in the wallet using treestate from RPC — Zebrad does not serve wallet witnesses. See [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../../ZEBRAD_SHIELDED_SEND_LIMIT.md).

---

## Troubleshooting

| Problem | What to try |
|---------|-------------|
| “Cannot connect to node” (NET_001) | Is Zebrad running? Is RPC enabled? Is `zebra_url` correct? |
| Wrong port / silent fallback | Re-read `config.json`; avoid UTF-8 BOM when editing with PowerShell |
| Desktop works in CLI but not GUI | Use the **NozyWallet desktop window**, not `localhost:5173` in a browser |
| Send blocked after “synced” | Check **witness lag** — scan at tip ≠ witnesses fresh |

Detailed RCA and Windows port notes: [desktop pre-release debug session](../../../docs/issues/bugs/2026-06-desktop-pre-release-debug-session.md).
