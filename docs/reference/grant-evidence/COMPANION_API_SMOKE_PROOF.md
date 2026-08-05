# Companion API — grant evidence pointers

## Live smoke screenshot (localhost production proof)

File in-repo:

- [`../../api-server/scripts/companion-api-smoke-proof.png`](../../api-server/scripts/companion-api-smoke-proof.png)

Shows `SMOKE PASSED` for `/health`, `/api/wallet/exists`, `/api/config` against `http://127.0.0.1:3000`.

### How to download

1. **GitHub (browser):** open  
   https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/api-server/scripts/companion-api-smoke-proof.png  
   → **Download** (or raw URL below → Save As).

2. **Raw URL (curl / wget / browser Save As):**  
   https://raw.githubusercontent.com/LEONINE-DAO/Nozy-wallet/master/api-server/scripts/companion-api-smoke-proof.png

3. **Local clone:**  
   `api-server/scripts/companion-api-smoke-proof.png`

### Re-run smoke (optional)

```powershell
# with nozywallet-api already on :3000
.\api-server\scripts\smoke-companion.ps1
# optional HTML proof page: api-server/scripts/proof-smoke.html
```
