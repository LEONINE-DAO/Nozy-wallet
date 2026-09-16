# NozyWallet Mobile — contributor handoff

**Next session (Nym / public LWD session split):** [`docs/reference/NEXT_SESSION_NYM_LWD_SESSION_SPLIT.md`](../docs/reference/NEXT_SESSION_NYM_LWD_SESSION_SPLIT.md)

Read this before changing `nozy-mobile/` or the hosted API story.

---

## What it is

Expo / React Native **light wallet** (Zodl-style): keys on the phone, compact sync to **`https://lwd.nozywallet.org:443`**. Optional companion HTTP (`nozywallet-api` on the user’s PC) for self-hosters and Expo Go.

**`api.nozywallet.org` is not a public multi-user host.** It is LEONINE’s operator companion. Store/default clients must not auto-join that wallet.

---

## Read order

1. [`README.md`](README.md) — quick start
2. [`STORE-CHECKLIST.md`](STORE-CHECKLIST.md) — App Store / Play
3. [`../nozy-ffi/README.md`](../nozy-ffi/README.md) — UniFFI + Android `.so`
4. [`../ENHANCEMENT_ROADMAP.md`](../ENHANCEMENT_ROADMAP.md)
5. [`../api-server/README.md`](../api-server/README.md) — companion HTTP (optional)

---

## Key files

| File | Role |
|------|------|
| `modules/nozy-wallet/` | Expo module wrapping `libnozy_ffi` |
| `src/services/onDeviceWallet.ts` | Create / restore / unlock / compact sync |
| `src/services/api.ts` | Optional companion HTTP |
| `src/context/WalletSessionContext.tsx` | Session + backend mode |
| `src/screens/Welcome.tsx` | Create / restore / unlock |

---

## Native build

From repo root:

```powershell
.\scripts\build-nozy-ffi.ps1 -Target host -Bindgen kotlin
.\scripts\build-nozy-ffi.ps1 -Target android
```

Then rebuild the Android app (`npx expo run:android`). Expo Go cannot load `libnozy_ffi`.

---

## Out of scope (this slice)

- iOS `libnozy_ffi` staticlib (needs macOS)
- On-device Orchard send proving (receive + compact sync land first)

---

*Update when mobile milestones shift on the enhancement roadmap.*
