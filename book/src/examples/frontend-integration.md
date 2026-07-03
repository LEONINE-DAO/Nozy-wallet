# Integrate with Frontend

NozyWallet exposes wallet logic through **Tauri invoke** (desktop), **HTTP api-server** (browser extension companion), and **UniFFI** (mobile). Pick the surface that matches your app.

## Architecture

```text
┌──────────────┐     invoke      ┌─────────────────┐
│ React / UI   │ ───────────────►│ Tauri commands  │──► nozy crate
└──────────────┘                 └─────────────────┘

┌──────────────┐     HTTP        ┌─────────────────┐
│ Extension /  │ ───────────────►│ api-server      │──► nozy + zeaking
│ web dashboard│   localhost:3000└─────────────────┘
└──────────────┘
```

## Desktop (Tauri + React)

From `desktop-client/src/lib/api.ts`:

```typescript
import { invoke } from "@tauri-apps/api/core";

const balance = await invoke<{ balance_zec: number }>("get_balance");
const tx = await invoke("send_transaction", {
  request: { recipient: "u1…", amount: 0.001, password: "…" },
});
```

Common commands: `wallet_exists`, `create_wallet`, `unlock_wallet`, `get_balance`, `sync_wallet`, `send_transaction`, `get_sync_status`, `test_zebra_connection`.

Full list: [`desktop-client/README.md`](../../../desktop-client/README.md).

**Important:** Run inside the **Tauri webview**, not a standalone browser at `localhost:5173` — IPC is unavailable there.

## HTTP api-server (extension companion)

Start locally:

```bash
cd api-server
cargo run
# http://127.0.0.1:3000
```

Example routes:

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness |
| GET | `/api/wallet/exists` | Wallet present |
| POST | `/api/wallet/unlock` | Unlock |
| GET | `/api/balance` | Balance |
| POST | `/api/sync` | Scan blocks |
| POST | `/api/transaction/send` | Shielded send |
| GET | `/api/config` | `zebra_url`, `last_scan_height` |
| GET | `/api/lwd/chain-tip` | lightwalletd tip |

Browser extension setup: [`browser-extension/COMPANION.md`](../../../browser-extension/COMPANION.md).

## Mobile (zeaking-ffi)

UniFFI bindings wrap the same sync and wallet primitives. Build from `zeaking-ffi/` per that crate’s README.

## Configuration shared by all surfaces

- **Zebrad RPC** — `zebra_url` in config or `ZEBRA_RPC_URL`
- **lightwalletd** — optional compact sync; default `http://127.0.0.1:9067`
- **Proving params** — download once per machine (`nozy proving --download`)

## Error handling

Desktop maps Rust errors to codes (`NET_001`, `SEND_001`, …) in `desktop-client/src/utils/errors.ts`. HTTP API returns JSON `{ success, message, code? }`.

## Next steps

- [API Overview](../api/overview.md)
- [Set Up Your Own Node](own-node.md)
- [Zebra Node Setup](../advanced/zebra-node.md)
