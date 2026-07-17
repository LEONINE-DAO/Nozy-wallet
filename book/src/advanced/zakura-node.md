# Zakura Node Setup

NozyWallet works with **[Zakura](https://zakura.com/)** — a Zebra-fork full node — using the **same config** as Zebrad. You do **not** need Zakura’s zcashd compatibility mode.

Full operator guide: **[Zakura ↔ NozyWallet connectivity](../../../docs/reference/ZAKURA_NOZYWALLET_CONNECTIVITY.md)**.

---

## Quick setup

### 1. Configure Zakurad

```toml
# ~/.config/zakurad.toml

[rpc]
listen_addr = "127.0.0.1:8232"
enable_cookie_auth = false   # lightwalletd + local dev
```

```bash
zakurad start
```

### 2. lightwalletd

Point [lightwalletd](https://github.com/zcash/lightwalletd) at Zakura RPC (`127.0.0.1:8232`). See Zakura’s [lightwalletd book chapter](https://github.com/zakura-core/zakura/blob/v1.0.0/book/src/user/lightwalletd.md).

### 3. Point NozyWallet

```bash
nozy config --set-zebra-url http://127.0.0.1:8232
nozy test-zebra    # detects Zakura + probes z_gettreestate
nozy sync --to-tip
```

---

## Zebrad or Zakura?

| Node | When to use |
|------|-------------|
| **Zebrad** | Zcash Foundation default; well-trodden with NozyWallet CI |
| **Zakura** | Faster sync, snapshots, pruned nodes; Ironwood from Project Tachyon / Valar |

Both use port **8232** (mainnet) and the same `zebra_url` setting.

---

## Troubleshooting

See the [connectivity guide](../../../docs/reference/ZAKURA_NOZYWALLET_CONNECTIVITY.md) and [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../../ZEBRAD_SHIELDED_SEND_LIMIT.md).
