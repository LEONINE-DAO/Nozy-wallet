# Keystone Hardware Wallet

NozyWallet supports **[Keystone](https://keyst.one/)** as an air-gapped hardware signer for **Zcash mainnet Orchard** sends. Keystone holds spend authority; NozyWallet builds proved transactions (PCZT), shows QR codes for signing, and broadcasts the signed result.

**Network:** Keystone integration is **mainnet only**. Testnet wallets cannot enable Keystone pairing or PCZT sends.

---

## Where to find it in NozyWallet

| Surface | Location |
|---------|----------|
| **Desktop** | **Settings → Keystone** — pair UFVK, enable signing |
| **Desktop sends** | **Send** tab — when Keystone is enabled, uses prepare → sign → broadcast |
| **Mobile** (companion API) | **Keystone** screen from dashboard — same PCZT flow via `nozywallet-api` |
| **API** | `GET/POST /api/keystone/*` on localhost companion — see [API endpoints](#api-endpoints) below |

---

## What Keystone does (and does not do)

| Keystone role | Details |
|---------------|---------|
| **Signs spends** | Adds Orchard spend authorization to a proved PCZT |
| **Can hold seed** | Typical setup: mnemonic generated or imported on Keystone |
| **View-only pairing** | UFVK import lets Nozy sync balance and build unsigned PCZTs |

| Not supported | Details |
|---------------|---------|
| **Transparent (`t1`) sends** | Orchard shielded only — recipients must be `u1…` unified addresses |
| **Testnet** | Mainnet config and mainnet Keystone device required |
| **In-wallet QR scan (desktop)** | Paste signed PCZT / UR frames, or scan on Keystone device |
| **On-chain multisig** | This is PCZT co-signing, not Bitcoin-style m-of-n |

---

## Prerequisites

1. **Zcash mainnet** wallet in NozyWallet (`network: mainnet` in config — default).
2. **Zebrad + sync** — wallet synced to tip before sending ([Zebra Node Setup](../advanced/zebra-node.md)).
3. **Keystone device** set to **Zcash mainnet** with firmware that supports Zcash PCZT (`zcash-pczt` UR type).
4. **Matching keys** — Keystone must control the same Orchard account as NozyWallet (see setup paths below).

---

## Setup (pairing)

### Path A — Seed on Keystone (recommended)

1. Create or restore your Zcash wallet **on Keystone** (mainnet).
2. On Keystone, export the **UFVK** (unified full viewing key).
3. In NozyWallet, import or restore the **same mnemonic** so sync and PCZT building work locally.  
   *Alternatively*, store only the UFVK in Nozy config for watch-only sync if your workflow uses Keystone as the sole signer — spending still requires the seed on Keystone.
4. **Settings → Keystone** → **Export UFVK** (or confirm stored UFVK matches Keystone). Mainnet UFVKs start with **`uview1`**.
5. Import that UFVK on Keystone if not already paired; confirm the **same unified receive address** (`u1…`) on both devices.
6. **Enable Keystone**.

### Path B — Seed in NozyWallet first

1. Create or restore wallet in NozyWallet (mainnet).
2. **Settings → Keystone** → **Export UFVK** → import on Keystone.
3. Import the **same mnemonic** into Keystone so it can sign PCZTs.
4. Confirm matching `u1…` receive address.
5. **Enable Keystone**.

> **UFVK is read-only.** It can reveal shielded activity to whoever holds it. It cannot spend funds. See [Private Key Management](key-management.md).

---

## Sending with Keystone (desktop)

1. Ensure **Settings → Keystone** shows **Enabled** and **mainnet**.
2. Open **Send** — you should see *Keystone signing enabled (mainnet)*.
3. Enter recipient (`u1…`), amount, optional memo → **Review & prepare**.
4. **Prepare for Keystone** — proving may take several minutes on first send.
5. Scan the **PCZT QR** on Keystone (multiple UR frames if shown — scan all).
6. Sign on Keystone.
7. Paste **signed PCZT hex** or **UR frames** back into NozyWallet.
8. **Broadcast signed tx** — transaction is submitted via your Zebrad node.

Success shows a txid and a link to [mainnet.zcashexplorer.app](https://mainnet.zcashexplorer.app).

---

## Sending with Keystone (mobile + API)

When using the mobile app with `nozywallet-api`:

1. Open **Keystone** from the dashboard.
2. Enable Keystone and export UFVK for pairing (same as desktop).
3. **Prepare for Keystone** → scan UR on device → paste signed data → **Broadcast signed tx**.

API calls mirror the desktop Tauri commands — useful for automation or custom frontends.

---

## Receiving ZEC

- **Into NozyWallet:** Share your Nozy **Receive** address (`u1…`). Anyone (including Keystone) can send shielded ZEC to it.
- **From Keystone to NozyWallet:** Use Keystone’s send UI with your Nozy receive address, or use the PCZT flow above with Nozy as the builder/broadcaster.

If Keystone and Nozy share the same wallet, synced balance appears in Nozy after compact sync — no separate “receive from Keystone” step.

---

## API endpoints

Local companion only (`nozywallet-api`, default `http://127.0.0.1:3000`):

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/api/keystone/status` | Enabled, UFVK paired, pending send, network |
| `POST` | `/api/keystone/enable` | Enable/disable Keystone |
| `POST` | `/api/keystone/export-ufvk` | Export and store Orchard UFVK |
| `POST` | `/api/keystone/prepare-send` | Build proved PCZT + UR frames |
| `POST` | `/api/keystone/complete-send` | Decode signed PCZT, broadcast |

See [API Server Setup](../advanced/api-server.md) for running the companion.

---

## Troubleshooting

| Problem | What to check |
|---------|----------------|
| **“Keystone requires mainnet”** | Set `network` to `mainnet` in wallet config; disable testnet. |
| **Invalid recipient** | Use mainnet Orchard unified address (`u1…`), not `t1` or Sapling-only. |
| **Keystone won’t sign** | Seed must match UFVK; device on Zcash mainnet; scan all UR frames. |
| **Broadcast fails** | Sync to tip; Zebrad reachable; signed PCZT not expired (re-prepare if needed). |
| **UFVK mismatch** | Re-export from the wallet that holds the seed; confirm `uview1` on mainnet. |
| **Balance zero after pair** | Run sync; UFVK alone does not import notes without scan. |

More: [Common Issues](../troubleshooting/common-issues.md), [Desktop Troubleshooting](../desktop-app/troubleshooting.md).

---

## Security notes

- Treat UFVK export like sharing a read-only copy of your shielded history.
- Verify recipient and amount on **Keystone’s screen** before signing.
- Air-gapped signing reduces hot-wallet exposure; proving still runs on the Nozy machine.
- Back up your mnemonic — Keystone and/or Nozy depending on where the seed lives ([Backup Strategies](backup-strategies.md)).

---

## Related

- [Security Best Practices](best-practices.md)
- [Private Key Management](key-management.md)
- [Sending ZEC](../user-guide/sending-zec.md)
- [Using the GUI](../desktop-app/using-gui.md)
- [Security FAQ](../faq/security.md#does-nozywallet-support-keystone)
