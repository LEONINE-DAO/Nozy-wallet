# NozyWallet Mobile

Expo / React Native companion app for [NozyWallet](https://github.com/LEONINE-DAO/Nozy-wallet). Connects to **nozywallet-api** on your PC or a hosted VPS to manage a private (Orchard) Zcash wallet.

---

## Quick start (Windows)

1. **Start the API** from repo root (keep the window open):
   - Run [`RUN-API.bat`](../RUN-API.bat) or `cargo run -p nozywallet-api`
   - Wait for `listening on http://0.0.0.0:3000`

2. **Start the app on Android emulator**:
   - Run [`RUN-ON-EMULATOR.bat`](RUN-ON-EMULATOR.bat) or `npm run android`

3. On **Welcome**, use API URL `http://10.0.2.2:3000` (emulator → host PC). Leave API key blank for local dev.

**Web preview:** `.\start-web.ps1` or `npm run web`

---

## Architecture

```text
Mobile app  →  nozywallet-api (API URL)  →  Zebra node (Zebra URL)
```

| Context | API URL |
|---------|---------|
| Android emulator | `http://10.0.2.2:3000` |
| Browser | `http://localhost:3000` |
| Phone / public VPS | `https://nozywallet.leoninedao.org` + API key |

---

## Scripts

| Command | Purpose |
|---------|---------|
| `npm start` | Expo dev server |
| `npm run android` | Expo + Android |
| `npm run web` | Web preview |
| `npm run typecheck` | TypeScript check |

---

## Project layout

```text
App.tsx                 Entry
src/navigation/         Stack navigator
src/screens/            All UI screens
src/services/api.ts     API client (X-API-Key when set)
src/context/            Session (password, API URL, key, auto-sync)
src/theme.ts            Landing-page colors
src/constants/links.ts  Privacy policy & support URLs
```

---

## Docs

| File | When to read |
|------|----------------|
| **[HANDOFF.md](./HANDOFF.md)** | New agent or contributor — read first |
| **[VPS-DEPLOY.md](./VPS-DEPLOY.md)** | Deploy public API for mobile data |
| **[STORE-CHECKLIST.md](./STORE-CHECKLIST.md)** | App Store / Play release |

---

## Release builds (EAS)

Requires [Expo account](https://expo.dev) and `eas-cli`:

```powershell
npm install -g eas-cli
eas login
eas build --platform android --profile production
```

See **STORE-CHECKLIST.md** for full store submission steps.

---

## Stack

- Expo SDK 52 · React Native 0.76 · TypeScript
- React Navigation · AsyncStorage

Bundle ID: `com.leoninedao.nozywallet`
