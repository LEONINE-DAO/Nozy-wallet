# Local RPC (Zebra / zcashd) with the extension

For **lightwalletd compact sync** via the running Nozy desktop API (instead of raw gRPC in the MV3 worker), see **[COMPANION.md](./COMPANION.md)** (`http://127.0.0.1:3000` + `companion_*` messages).

## “Failed to fetch” on Scan or RPC

That message means the browser **could not open a TCP connection** to the URL (not a JSON-RPC error yet).

Checklist:

1. **Node is running** and RPC is enabled for the port you configured (e.g. `8232`).
2. **Port matches config** — Zebra’s default `listen_addr` may differ from zcashd; verify in `zebrad.toml` / logs.
3. **Windows + WSL / Docker** — If Zebra runs inside WSL or a container, `http://127.0.0.1:8232` from Windows sometimes does **not** reach that process. Use the VM/container **LAN IP** (e.g. from `wsl hostname -I` or `docker inspect`) instead of `127.0.0.1`.
4. **Zebra cookie auth** — If JSON-RPC requires authentication, unauthenticated `fetch` calls can fail. For **local development only**, you can disable cookie auth in Zebra config (see Zebra docs: `enable_cookie_auth = false` or env `ZEBRA_RPC__ENABLE_COOKIE_AUTH=false`). **Do not** disable auth on exposed networks.

## Quick test outside the extension

From the same machine where the browser runs:

```bash
curl -s -X POST http://127.0.0.1:8232 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}'
```

If this fails, fix the node / network / URL before retrying in Nozy.

## Extension permissions

The manifest includes `host_permissions` for `http://127.0.0.1:3000/*` (companion API), `http://*/*`, and `https://*/*`, so `http://127.0.0.1:…` is allowed once the host is actually reachable.
