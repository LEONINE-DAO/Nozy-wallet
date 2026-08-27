# NU7 coinholder vote — Nozy integration

**Issue:** [#273](https://github.com/LEONINE-DAO/Nozy-wallet/issues/273)  
**Helper:** [`tools/nozy-vote/`](../../tools/nozy-vote/)  
**Forum:** https://forum.zcashcommunity.com/t/nu7-coinholder-vote/56912

## Calendar

- **Snapshot:** 2026-08-24 19:00 UTC — spendable **Ironwood** notes only  
- **Vote:** 2026-08-25 → 2026-09-14 19:00 UTC  
- Tallies: https://tally.valargroup.org  

## Flow (delegation + cast)

1. `nozy vote-export-notes --out vote-notes.json`  
2. `nozy-vote --env prod` → `hotkey-init` → `init-round` → `import-notes` → `delegate`  
3. `nozy vote-sign-delegation --request signing-request-….json --out delegation-sig.json`  
4. `nozy-vote --env prod delegate-finish --notes-file vote-notes.json --sig delegation-sig.json`  
5. `nozy-vote --env prod cast --choices 1=0,2=1,3=0,4=0,5=0`  

See [`tools/nozy-vote/README.md`](../../tools/nozy-vote/README.md).

## Desktop

**Vote** tab in Nozy Desktop: export/sign in-process; prepare/cast via `nozy-vote` helper binary (sqlite conflict prevents linking `zcash_voting` beside `zeaking`).

## Mobile

`nozy-ffi` + Settings → **NU7 Vote**:

| Step | Where |
|------|--------|
| Export Ironwood notes | `vote_export_notes` (on-device FFI) |
| Sign delegation request | `vote_sign_delegation` (on-device FFI) |
| Prepare / prove / cast | Desktop Vote or `nozy-vote` CLI |

Requires rebuilt `libnozy_ffi` + UniFFI Kotlin/Swift bindgen wired to `NativeModules.NozyFfi`. Companion-only installs should use Desktop for the full ballot.

## Extension

Popup **Vote** tab → companion `nozywallet-api` `/api/vote/*` (companion wallet, not WASM). Export/sign in api-server; prepare/cast via `nozy-vote` sidecar (`NOZY_VOTE_BIN`). See [`browser-extension/COMPANION.md`](../../browser-extension/COMPANION.md).
