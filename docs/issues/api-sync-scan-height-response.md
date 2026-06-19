# GitHub issue: `/api/sync` scan height (response for Gilmore)

Use this as a **comment on Gilmore's issue** if one exists, or create a new issue with the title below.

---

## Title (if creating new issue)

`Docs: clarify POST /api/sync accepts start_height and end_height (no config edit required for rescan)`

---

## Body

### Question

Using the API server (`POST /api/sync`) rather than the CLI. Some API docs show `/api/sync` only accepts `password` in the body, with no `start_height` param.

Is there a way to set scan height through the API (body param or config endpoint), or do we need to edit `last_scan_height` directly in `config.json` before calling `/api/sync`?

### Answer

**You do not need to edit `config.json` for a targeted rescan.** The handler already accepts optional scan bounds in the POST body.

```json
POST /api/sync
Content-Type: application/json

{
  "password": "...",
  "start_height": 3050000,
  "end_height": 3051000,
  "zebra_url": "https://zec.leoninedao.org:443"
}
```

Implemented in `api-server/src/handlers.rs` (`SyncRequest`: `start_height`, `end_height`, `zebra_url`, `password`). Scan logic: `src/wallet_sync.rs` (`resolve_scan_range`).

Also documented in `api-server/FRONTEND_DEVELOPER_GUIDE.md`. The short line in `api-server/README.md` is incomplete — doc fix tracked here.

---

### Behavior summary

| Request | What happens |
|---------|----------------|
| `{ "password": "..." }` only | Incremental sync from `last_scan_height + 1`, **max 1000 blocks** per call. Repeat until `already_synced: true`. |
| `"start_height": N` | Starts at block **N** for this sync (overrides checkpoint for this request). |
| `"start_height": N` without `"end_height"` | Scans **N → chain tip** in one request (can be very long; watch proxy/API timeouts). |
| `"start_height": N, "end_height": M` | Scans inclusive range **N–M** only (**recommended** for controlled rescans). |

After a successful sync, the server updates `last_scan_height` to the end of the scanned range.

---

### Reading / resetting checkpoint

- **Read checkpoint:** `GET /api/config` → `last_scan_height`
- **No write endpoint** for `last_scan_height` today — only read via API

**Config file paths (Windows):**

| File | Path |
|------|------|
| Config | `%APPDATA%\nozy\nozy\config\config.json` |
| Note cache | `%APPDATA%\nozy\nozy\data\notes.json` |

Config is **not** under `wallet_data/config.json` — config dir and data dir are separate (XDG / `directories` crate).

---

### Recommended rescan approaches

**1. Chunked API rescan (preferred — no file edits):**

```bash
curl -X POST http://localhost:3000/api/sync \
  -H "Content-Type: application/json" \
  -d '{"password":"...","start_height":3050000,"end_height":3051000}'
```

Repeat with the next block range until caught up.

**2. Normal catch-up:** repeat `POST /api/sync` with password only (1000 blocks per call).

**3. Reset default incremental start (optional):** stop API, edit `last_scan_height` in config to one below desired start, restart, then sync without `start_height`.

**4. Full rebuild (last resort):** backup `notes.json`, delete it, set `"last_scan_height": null` in config, call `/api/sync`. Empty cache triggers full backfill.

---

### What not to do

- Do **not** set `last_scan_height` **ahead** of what was actually scanned — returns "already synced" with wrong balance.
- Avoid single `start_height` → tip requests on production nginx unless `proxy_read_timeout` is high enough (first full scan can exceed 60s default).

---

### Follow-up (doc fix)

- [ ] Expand `api-server/README.md` `POST /api/sync` section to match `FRONTEND_DEVELOPER_GUIDE.md`
- [ ] Optional: add `POST /api/config/last-scan-height` so operators never edit config by hand

---

**Bottom line:** Use `start_height` / `end_height` on `/api/sync` for rescans. Editing config is optional and only for resetting the default incremental checkpoint — not required for a correct rescan.
