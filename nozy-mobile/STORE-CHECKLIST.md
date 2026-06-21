# Mobile store release checklist

**Status:** Pre-release — app not yet on App Store or Google Play.  
**Bundle ID:** `com.leoninedao.nozywallet`

---

## Before submission

- [ ] Production API on HTTPS VPS ([`VPS-DEPLOY.md`](VPS-DEPLOY.md)) with API key enforced
- [ ] Privacy policy URL live (see `src/constants/links.ts`)
- [ ] No test mnemonics or secrets in repo
- [ ] `eas.json` profiles reviewed for production
- [ ] Icons and splash (`assets/`) finalized
- [ ] End-to-end test: create wallet → sync → send on testnet or small mainnet amount

---

## Build (EAS)

```bash
npm install -g eas-cli
eas login
eas build --platform android --profile production
eas build --platform ios --profile production
```

---

## Store assets (prepare)

| Item | Notes |
|------|--------|
| Short description | Shielded ZEC companion wallet |
| Screenshots | Dashboard, Send, Receive, Settings |
| Support URL | GitHub issues or project site |
| Content rating | No gambling; crypto wallet |

---

## After launch

- [ ] Update [`landing/`](../landing/) mobile card from “soon” to store links
- [ ] Note release in [`CHANGELOG.md`](../CHANGELOG.md)

See also [`ENHANCEMENT_ROADMAP.md`](../ENHANCEMENT_ROADMAP.md).
