# Set Up Your Own Node

Run **Zebrad** so NozyWallet talks to infrastructure you control.

Full reference: [Zebrad ↔ NozyWallet connectivity](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md).

## 1. Install Zebrad

Follow [Zebra Foundation docs](https://zebra.z.cash/). Build or install `zebrad` for your OS.

## 2. Enable RPC

```toml
# Linux: ~/.config/zebrad.toml
# Windows: %LOCALAPPDATA%\zebrad.toml

[rpc]
listen_addr = "127.0.0.1:8232"
enable_cookie_auth = false
```

Windows port **8232** conflict → use **18232** and match wallet config.

## 3. Start and wait for sync

```bash
zebrad start
```

Initial sync takes hours to days on mainnet depending on hardware.

## 4. Point NozyWallet

```bash
nozy config --set-zebra-url http://127.0.0.1:8232
nozy test-zebra
```

## 5. WSL + Windows wallet

Zebrad in WSL, desktop on Windows:

```powershell
wsl hostname -I   # use first IP
nozy config --set-zebra-url http://<ip>:8232
```

Or: `. .\scripts\zebra-wsl-rpc.ps1`

## 6. Optional: lightwalletd

For compact sync (extension / LWD paths):

- Run lightwalletd against your Zebrad
- Default gRPC `http://127.0.0.1:9067`
- Set `LIGHTWALLETD_GRPC` if non-default

## 7. Verify stack

```powershell
.\scripts\test-zebrad-nozywallet.ps1   # Windows smoke test
```

```bash
nozy sync --to-tip
nozy status
```

## Checklist

- [ ] `getblockcount` increases over time
- [ ] `nozy test-zebra` passes from wallet host
- [ ] `config.json` URL matches live node
- [ ] Wallet sync completes to tip

See [Zebra Node Setup](../advanced/zebra-node.md) | [Network Configuration](../advanced/network-config.md)
