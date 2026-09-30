# Desktop companion (Chrome + Microsoft Edge)

## Which product should users install?

1. **Nozy Desktop (Tauri)** — **[recommended main entry](https://github.com/LEONINE-DAO/Nozy-wallet/releases)** for a full wallet UX, Zebra-first flows, and ongoing Orchard send work.
2. **Browser extension (MV3)** — **lighter mode**: WASM wallet in the browser + this **companion HTTP API** for **lightwalletd / zeaking** compact sync **without** running Zebrad on the same machine. Use the popup **API** tab to hit `http://127.0.0.1:3000`.

The static **`landing/`** site is marketing only, not the wallet.

The MV3 extension does **not** embed `zeaking`, gRPC, or SQLite. For **lightwalletd compact sync** and parity with the desktop wallet, use the **Nozy API server** (or Tauri app hosting the same Rust stack) on the same machine.

**Sending shielded ZEC** from the extension against **Zebrad-only** still depends on RPC capabilities (witness / anchor); Desktop + updated Nozy core target the supported Zebrad tree APIs. The companion fixes **compact sync**, not every prove path in the extension. See repo **`ZEBRAD_SHIELDED_SEND_LIMIT.md`** when relevant.

## Connect your node (extension users)

The extension prefers **Zebrad JSON-RPC** for in-extension scan/send. If you don’t have a node yet, use **Public sync** — the same model as **zec.rocks**:

1. Open the extension → Welcome / **Connect your node**.
2. We try local Zebrad (this PC, WSL, Desktop companion config).
3. If none is found, you’ll see: **“Connect to Nozy’s public lightwalletd?”**
   - **Yes** → `https://lwd.nozywallet.org:443` (keys stay in the extension; compact-sync style public node).
   - **No** → stay offline until you start Zebrad / paste a Remote URL.
4. You can also pick **Public sync** in the mode chips at any time.
5. When the pill shows **Connected** or **Public sync**, **Restore** or **Create** a wallet.
6. **Create** shows the 24-word phrase (reveal or copy), then a short **Word #N** quiz before you enter the wallet. Later: **Settings → Recovery phrase** (password again) to view/copy another backup.

**Not** `api.nozywallet.org` — that companion is operator-only. Public shared infra is **LWD only**.

For your own node:

1. **Start Zebrad** (this PC, WSL, or your VPS).
2. Pick your setup:
   - **Find automatically** — tries localhost, WSL IP, and Nozy Desktop `zebra_url` if `nozywallet-api` is running.
   - **This PC** — `http://127.0.0.1:8232`
   - **WSL / Linux VM** — auto-detects `http://172.x.x.x:8232` (127.0.0.1 does not work from Chrome on Windows).
   - **Remote VPS** — paste your RPC URL (e.g. `https://your-server.example.com:443`).
3. When the pill shows **Connected**, **Restore Desktop wallet** with the same 24-word phrase as Nozy Desktop. **Create** mints a new address that will not match Desktop / CLI / phone.

The Desktop wallet, CLI (`nozy`), and local `nozywallet-api` already share `%APPDATA%\nozy\…\wallet.dat`. The extension keeps its own Chrome vault — restore that phrase so Receive matches Wallet 1. Unlock → **Wrong address? Restore Desktop wallet**, or Settings → **Replace with Desktop recovery phrase**. Reveal the words in Desktop **Settings → Account**.

Optional: run **`nozywallet-api`** for lightwalletd compact sync (Companion tab) — separate from Zebrad RPC above. After Public sync opt-in, companion LWD defaults to `lwd.nozywallet.org`.

### Network privacy / Nym (Settings)

Settings → **Network privacy (Nym)** talks to the same companion privacy APIs as desktop:

- Send egress badge + mixnet / dVPN toggles (helpers on this PC).
- Operator opt-in: point companion LWD at **`http://127.0.0.1:9068`** when `lwd-mixnet-client` is running (Joaco dialling half). Probe checks `:9070/health` and `GetLightdInfo` via companion.
- Chrome does **not** embed the Nym SDK. Do not market “Nym integrated.”

## Companion API key

`nozywallet-api` now **generates** `{wallet_data_dir}/companion_api_key` on first start unless `NOZY_API_KEY` is set. Paste that value into the extension **API** tab and the mobile Welcome screen (`X-API-Key`). `/health` and `/api/lwd/*` stay public. Set `NOZY_ALLOW_UNAUTHENTICATED=1` only for emergency/dev. On Windows, `SHOW-API-KEY.bat` copies the key to the clipboard.

## Localhost HTTP (recommended)

1. Run **Zebrad** (JSON-RPC reachable from the browser host — WSL IP if node is in WSL). The extension **will not create or restore a wallet** until Zebrad RPC responds (`getblockcount`). Use Welcome → **Auto-detect Zebrad**, or Settings after unlock.
2. Run **`nozywallet-api`** from the Nozy-wallet repo (default bind: `http://127.0.0.1:3000`). Set `LIGHTWALLETD_GRPC` if lightwalletd is not on `http://127.0.0.1:9067`.
   - **Windows + WSL footgun:** if lightwalletd listens inside WSL, `127.0.0.1:9067` on the Windows host will not reach it. Point `LIGHTWALLETD_GRPC` at the WSL IP (e.g. `http://172.x.x.x:9067`).
3. The extension calls:
   - `GET /health`
   - `GET /api/lwd/info`
   - `GET /api/lwd/chain-tip`
   - `POST /api/lwd/sync/compact`
   - `POST /api/lwd/sync/compact-to-tip`
   - `GET /api/sapling/status` — quiet legacy balance (companion wallet data dir)
   - `POST /api/sapling/scan` — scan compact cache for legacy notes
   - `POST /api/sapling/shield` — move legacy notes into Orchard/Ironwood (not in wasm-core)
   - `GET /api/vote/status`, `GET /api/vote/active` — NU7 vote status / ballot
   - `POST /api/vote/export-notes`, `prepare`, `delegate`, `sign-delegation`, `delegate-finish`, `cast` — companion wallet + `nozy-vote` sidecar
   - `GET /api/crosslink/status`, `positions`, `roster` — Crosslink Protocol Guardian (feature-net)
   - `POST /api/crosslink/stake`, `retarget`, `unbond`, `withdraw` — node-wallet staking via companion
   - `POST /api/zns/resolve` — resolve Zcash names (proxy to public indexer)
3. **Manifest**: `host_permissions` includes `http://127.0.0.1:3000/*` (and broader patterns as needed). This works in **Google Chrome** and **Microsoft Edge** (Chromium).

### Service worker API

Messages to the background (`NOZY_REQUEST`):

| `method` | `params` | Description |
|----------|----------|-------------|
| `wallet_reset` | `{}` | Wipe Chrome vault so Welcome can restore the Desktop phrase |
| `companion_status` | `{ baseUrl? }` | Health + optional chain-tip probe |
| `companion_lwd_info` | `{ baseUrl?, lightwalletd_url? }` | `GetLightdInfo` via companion |
| `companion_lwd_chain_tip` | `{ baseUrl?, lightwalletd_url? }` | Tip height via companion |
| `companion_lwd_sync_compact` | `{ baseUrl?, start, end?, lightwalletd_url?, db_path?, resume? }` | Sync on **desktop** DB |
| `companion_lwd_sync_compact_to_tip` | `{ baseUrl?, lightwalletd_url?, db_path?, start_floor?, persist_progress_every? }` | Sync from next missing height through tip |
| `companion_sapling_status` | `{ baseUrl? }` | Quiet legacy unspent balance from companion notes |
| `companion_sapling_scan` | `{ baseUrl?, password?, start_floor?, full? }` | Scan companion LWD compact for legacy notes |
| `companion_sapling_shield` | `{ baseUrl?, password?, dry_run?, no_broadcast? }` | Move legacy funds into shielded Orchard (HTTP only; no wasm Groth16) |
| `companion_vote_*` | see popup **Vote** tab | NU7 export → prepare → delegate → sign → finish → cast (companion wallet) |
| `companion_crosslink_*` | see popup **XL** tab | Season 1 Guardian status / roster / stake / retarget / unbond / withdraw |
| `companion_zns_resolve` | `{ name, network?, baseUrl? }` | Resolve a Zcash name via companion `POST /api/zns/resolve` |

Default `baseUrl` is `http://127.0.0.1:3000`. Override per-call for non-default ports.

The popup **API** tab shows a quiet “Legacy funds / Move to shielded” notice **only** when companion status reports unspent legacy balance &gt; 0 (shared nozy data dir via api-server — not the in-extension WASM wallet).

The popup **Vote** tab uses the **in-extension WASM wallet** for export + sign (seed stays in the browser). Prepare / PIR / cast still run on local **`nozywallet-api` + `nozy-vote`** (same split as Desktop — `zcash_voting` cannot link beside `zeaking`). Start companion on loopback, then: Export Ironwood notes → Prepare → Delegate → Sign → Submit.

The popup **XL** (Crosslink) tab drives `/api/crosslink/*`. Point the companion at a Season 1 node (`nozy config --use-crosslink --set-crosslink-url …`). Feature-net cTAZ only — not mainnet ZEC. The mobile app **More → Crosslink** screen uses the same routes.
## Native messaging (optional)

To talk to the **desktop binary** without HTTP, register a **native messaging host** for each browser you support. On **Windows**, Chrome and Edge use **different registry keys** for the host manifest; ship an installer or script that registers **both** so one Nozy desktop build can serve either browser.

Enterprise policies may block extensions or localhost; document that for locked-down PCs.

## Mobile-only users

Without a reachable companion (desktop API or tunnel), the extension **cannot** run full `zeaking` sync in the service worker. Use the **mobile app** path (`zeaking-ffi`) or accept RPC-only / limited flows.

## Distribution

GitHub Releases may attach **`nozywallet-api-*`** (default `http://127.0.0.1:3000`) for same-machine extension / local-app use. Prefer loopback; see [`api-server/SEED_POLICY.md`](../api-server/SEED_POLICY.md) and [`api-server/README.md`](../api-server/README.md). This is **not** a hosted / public wallet API.

Optional locked-down local runs: `NOZY_PRODUCTION=true` and `NOZY_API_KEY`.

Publish the same MV3 package to the **Chrome Web Store** and **Microsoft Edge Add-ons**; validate in both browsers during QA.
