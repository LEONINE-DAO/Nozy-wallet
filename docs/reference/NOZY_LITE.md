# Nozy Lite — operator uptime & data checks

**Status:** Product surface on the existing `nozy` CLI (not a separate wallet crate)  
**Audience:** Node operators, self-hosters, VPS wallets, power users next to Zebrad/WSL  
**Related:** [`ZEBRA_DEV_CLI_POSITIONING.md`](../ZEBRA_DEV_CLI_POSITIONING.md) · [`NOZY_LITE_BENCHES.md`](NOZY_LITE_BENCHES.md) · [`SESSION_NYM_IRONWOOD_DESKTOP_CASE_BREAKDOWN.md`](SESSION_NYM_IRONWOOD_DESKTOP_CASE_BREAKDOWN.md)

---

## What Nozy Lite is

**Nozy Lite** is the **headless / light** way to run NozyWallet: the production **`nozy` CLI** (and optional live TUI) for **uptime and data checks**, sync, balance, send, and Ironwood ops — without the Tauri desktop WebView.

Same crypto core and wallet files as desktop. Lite does **not** replace the full desktop app for day-to-day GUI users.

| Surface | Best for |
|---------|----------|
| **Nozy Lite (CLI + TUI)** | Cron/health, SSH, low RAM, Zebrad sidekick, scripts |
| **Desktop (Tauri)** | Send UX, Settings, Ironwood card, Keystone QR |

---

## Best uses

1. **Watch Zebrad + wallet health** — tip, RPC scan gap, witness lag, LWD tip, Ironwood RPC.
2. **Uptime loops** — `nozy health` exit codes + `--json` for systemd / Task Scheduler / Nagios-style checks.
3. **Quick data peek** — balance, status, history without cold-starting a GUI.
4. **Ops send / Ironwood** — same proven CLI paths when you need them.
5. **Old hardware / VPS** — no Chromium/WebView tax.

**Not best for:** mouse-first beginners who want a full Settings/Send GUI — use desktop for that.

---

## Commands (ops pack)

```text
nozy health [--max-scan-gap N] [--require-lwd] [--require-ironwood-rpc] [--json]
nozy status [--watch] [--interval SECS] [--json]   # global --json also works
nozy balance [--json]
nozy tui [--interval SECS]                        # live dashboard (Phase 2)
```

### `nozy health` exit codes

| Code | Meaning |
|------|---------|
| 0 | OK |
| 1 | Zebra RPC unreachable |
| 2 | RPC scan gap above `--max-scan-gap` |
| 3 | Required LWD or Ironwood RPC check failed |
| 4 | Local wallet data unreadable (balance/notes) |

Example cron (every minute):

```bash
nozy health --json || logger -t nozy-health "unhealthy exit $?"
```

Prefer a **release** build for ops on Windows; debug profiles can hit small default thread stacks when loading wallet crypto paths.
---

## Size & honesty

Lite’s story is **useful light**: working sync/send/Ironwood + monitorable health. Publish measured numbers in [`NOZY_LITE_BENCHES.md`](NOZY_LITE_BENCHES.md); do not invent RSS.

---

## Release naming

Ship/document the CLI artifact as **Nozy Lite (CLI)** in release notes. Desktop remains a separate download when production-ready.

---

## AI disclosure

Positioning assisted by Cursor Agent. Human review before forum paste.
