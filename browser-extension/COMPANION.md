# Desktop companion (Chrome + Microsoft Edge)

The MV3 extension does **not** embed `zeaking`, gRPC, or SQLite. For **lightwalletd compact sync** and parity with the desktop wallet, use the **Nozy API server** (or Tauri app hosting the same Rust stack) on the same machine.

**Sending shielded ZEC** with **Zebrad-only** is still limited by missing zcashd-style witness RPCs on Zebra; the companion fixes **sync**, not that prove path. See repo root **`ZEBRAD_SHIELDED_SEND_LIMIT.md`**.

## Localhost HTTP (recommended)

1. Run **`nozywallet-api`** from the Nozy-wallet repo (default bind: `http://127.0.0.1:3000`). Set `LIGHTWALLETD_GRPC` if lightwalletd is not on `http://127.0.0.1:9067`.
2. The extension calls:
   - `GET /health`
   - `GET /api/lwd/info`
   - `GET /api/lwd/chain-tip`
   - `POST /api/lwd/sync/compact`
3. **Manifest**: `host_permissions` includes `http://127.0.0.1:3000/*` (and broader patterns as needed). This works in **Google Chrome** and **Microsoft Edge** (Chromium).

### Service worker API

Messages to the background (`NOZY_REQUEST`):

| `method` | `params` | Description |
|----------|----------|-------------|
| `companion_status` | `{ baseUrl? }` | Health + optional chain-tip probe |
| `companion_lwd_info` | `{ baseUrl?, lightwalletd_url? }` | `GetLightdInfo` via companion |
| `companion_lwd_chain_tip` | `{ baseUrl?, lightwalletd_url? }` | Tip height via companion |
| `companion_lwd_sync_compact` | `{ baseUrl?, start, end?, lightwalletd_url?, db_path? }` | Sync on **desktop** DB |

Default `baseUrl` is `http://127.0.0.1:3000`. Override per-call for non-default ports.

## Native messaging (optional)

To talk to the **desktop binary** without HTTP, register a **native messaging host** for each browser you support. On **Windows**, Chrome and Edge use **different registry keys** for the host manifest; ship an installer or script that registers **both** so one Nozy desktop build can serve either browser.

Enterprise policies may block extensions or localhost; document that for locked-down PCs.

## Mobile-only users

Without a reachable companion (desktop API or tunnel), the extension **cannot** run full `zeaking` sync in the service worker. Use the **mobile app** path (`zeaking-ffi`) or accept RPC-only / limited flows.

## Distribution

Publish the same MV3 package to the **Chrome Web Store** and **Microsoft Edge Add-ons**; validate in both browsers during QA.
