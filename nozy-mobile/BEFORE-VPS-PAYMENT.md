# Before you pay for the VPS — store prep path

You can finish most App Store / Play prep **without** a paid VPS. Pay for a **new mobile-only subscription** when you are ready to go live on `nozywallet.leoninedao.org`.

> **Important:** Do **not** deploy the mobile API on `zec.leoninedao.org`. That box is for operator uptime / Zec.rocks. See [`MOBILE-HOSTED-ACCOUNT.md`](MOBILE-HOSTED-ACCOUNT.md).

---

## What works without VPS (do now)

| Step | Command / doc | Outcome |
|------|----------------|---------|
| 1. Production app UI | `$env:EXPO_PUBLIC_APP_VARIANT="production"; npx expo start` | Hosted-only UI, no experimental screens |
| 2. Generate API key | `powershell -File ..\scripts\prepare-mobile-store-credentials.ps1` | `hosted-api.env` + key for app |
| 3. Production-mode API locally | `powershell -File ..\scripts\run-production-api-for-mobile-qa.ps1` | API requires `X-API-Key` like VPS |
| 4. Free HTTPS tunnel (optional) | [`HTTPS-TUNNEL-QA.md`](HTTPS-TUNNEL-QA.md) | Test **production APK** against HTTPS before DNS |
| 5. EAS production build | `eas build --platform android --profile production` | Store candidate AAB (Expo account) |
| 6. Listing copy | [`STORE-LISTING.md`](STORE-LISTING.md) | Play / App Store text draft |
| 7. Checklist | [`STORE-CHECKLIST.md`](STORE-CHECKLIST.md) | Track phases |

---

## What needs a new VPS subscription (mobile only)

| Item | Est. cost | Notes |
|------|-----------|--------|
| **New** VPS for `nozywallet.leoninedao.org` | ~$5–12/mo | Separate account/project from operator node |
| DNS A record | Usually free | `nozywallet` → new mobile VPS IP |
| TLS | Free | Let's Encrypt on **mobile** VPS |

**Existing (unchanged):** `zec.leoninedao.org` — Zebrad uptime / Zec.rocks. Mobile API **calls** it as RPC client; does not run on that server.

---

## Day the new mobile VPS is paid (≈30 min)

1. Create **new** Ubuntu VPS (new provider account or project — not `zec` server).
2. DNS: `nozywallet.leoninedao.org` → **new** IP only.
3. SSH into **new** box, deploy API (see [`MOBILE-HOSTED-ACCOUNT.md`](MOBILE-HOSTED-ACCOUNT.md)):

   ```bash
   export NOZY_API_KEY="paste-from-hosted-api.env"
   sudo -E bash scripts/deploy-mobile-api-vps.sh
   ```

4. Verify: `curl -H "X-API-Key: …" https://nozywallet.leoninedao.org/health`
5. Phone / production app: Welcome → hosted URL + same API key.

Full detail: [`VPS-DEPLOY.md`](VPS-DEPLOY.md)

---

## Blockers found (fix before submit)

| Issue | Status | Action |
|-------|--------|--------|
| `https://leonine-dao.github.io/Nozy-wallet/privacy/` | **Ready in repo** | Deploy landing (`landing/`) to GitHub Pages |
| `https://nozywallet.leoninedao.org/health` | **Down** | Expected until VPS paid |
| Privacy policy in store console | Open | Use URL from `src/constants/links.ts` |

---

## Recommended order

```text
Now     → credentials script + production API QA + tunnel test (optional)
        → EAS production build on physical Android
        → screenshots from production UI
When ready → pay **new mobile VPS** (not zec server) → deploy → E2E on hosted URL
Submit  → Play internal track → then production
```

See also [`docs/reference/NOZY_MOBILE_CASE_BREAKDOWN.md`](../docs/reference/NOZY_MOBILE_CASE_BREAKDOWN.md) Case I1.
