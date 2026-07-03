# Desktop App Troubleshooting

## Cannot connect to node (NET_001)

The desktop app talks to Zebrad over JSON-RPC using `zebra_url` in your wallet config.

1. Confirm Zebrad is running (local machine, WSL, or remote VPS).
2. **Settings → Network** — check the URL and click **Test Connection**.
3. From a terminal: `nozy test-zebra` (same config the CLI uses).

If CLI succeeds but the browser tab at `http://localhost:5173` fails, you opened the Vite dev server — use the **NozyWallet taskbar window** instead.

Full Zebrad ↔ NozyWallet guide (WSL, Windows ports, config paths): **[connectivity reference](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md)**.

---

## Sync says success but send is blocked

Scan height at tip does not guarantee **witness freshness**. Check the sync status on Home (blocks behind, witness lag). Run **Sync** again and wait until witness lag is within the send guard (≤ 50 blocks).

---

## Sync / blocks not moving

- Verify the node tip is advancing: `nozy test-zebra` or raw `getblockcount`.
- WSL IP may change after reboot — update `zebra_url` if Zebrad runs in WSL.
- Run `.\scripts\test-zebrad-nozywallet.ps1` from the repo for a consolidated check (Windows).

---

## Getting help

- [Common issues](../troubleshooting/common-issues.md)
- [GitHub Issues](https://github.com/LEONINE-DAO/Nozy-wallet/issues)
- [Zebrad node setup (book)](../advanced/zebra-node.md)
