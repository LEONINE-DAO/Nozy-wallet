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

`nozy-ffi` + Settings → **Coinholder vote**:

| Step | Where |
|------|--------|
| Export Ironwood notes | `vote_export_notes` (on-device FFI) |
| Sign delegation request | `vote_sign_delegation` (on-device FFI) |
| Prepare / prove / cast | Desktop Vote or `nozy-vote` CLI |

Requires rebuilt `libnozy_ffi` + UniFFI Kotlin/Swift bindgen wired to `NativeModules.NozyFfi`. Companion-only installs should use Desktop for the full ballot.

## Extension

Popup **Vote** tab → companion `nozywallet-api` `/api/vote/*` (companion wallet, not WASM). Export/sign in api-server; prepare/cast via `nozy-vote` sidecar (`NOZY_VOTE_BIN`). See [`browser-extension/COMPANION.md`](../../browser-extension/COMPANION.md).

## Next round — vote in Nozy (do this before polling opens)

Source: [Q3 results thread](https://forum.zcashcommunity.com/t/q3-coinholder-directed-retroactive-grants-program-results/57909) (FPF, 2026-09-29, corrected 2026-09-30). Q3 polling is closed. People cannot cast the next ballot in Nozy until the work below ships.

**Q4 window** (FPF; times 20:00 UTC):

| Phase | When |
|-------|------|
| Call for proposals | 2026-10-30 → 2026-11-13 |
| Review | 2026-11-16 → 2026-12-16 |
| Polling / voting | 2026-12-17 → 2026-12-29 |

Build the ballot during review. It has to be in Desktop and the extension before 2026-12-17.

### Already done

Delegation is in Desktop, the extension Vote tab, and `nozy-vote`: export Ironwood notes → prepare → sign → submit. `POST /api/vote/cast` already accepts `choices` as proposal id → option id and runs `nozy-vote cast --choices`. The screens still stop after delegation and send people to Valar to mark the ballot.

### Ballot choices (one per proposal)

Use the option ids that round publishes. Do not hardcode 0–3.

| Choice shown in Nozy | Q3 result label if it wins |
|----------------------|----------------------------|
| Accept | Funded |
| Reject | Do not support the proposed project |
| Lower amount | Would reconsider in a later round at a lower amount |
| Abstain | Not a result. Leave it out of the count |

### ZIP 1016 check (show this after the window closes)

From the corrected Q3 tally and the [OpenZcash verify export](https://openzcash.org/api/coinholders/verify?round=q3-2026&format=csv):

1. ZEC = vote-chain `total_value / 1e8`.
2. Drop abstain.
3. Funded only if both are true: non-abstaining ZEC ≥ **420,000**, and **Accept > Reject + Lower amount**.
4. Otherwise rejected. Say which reject reason had the votes, or that quorum was missed.

Also offer a CSV of proposal, the four ZEC totals, and that outcome. That is the wallet export the thread asked for. Q3 reference file: `https://openzcash.org/api/coinholders/verify?round=q3-2026&format=csv`.

### Ship list

1. After delegation confirms, load the round’s proposals and option ids.
2. One screen (Desktop and extension) with the four choices above. Submit the whole map through the existing cast API.
3. After 2026-12-29 20:00 UTC, results screen: apply the ZIP 1016 check locally and export CSV. Compare against the published FPF outcome before calling a row final.
