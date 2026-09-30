# nozy-vote

Isolated CLI helper for **Zcash coinholder voting** (Valar Shielded Vote / NU7).

Tracking: [#273](https://github.com/LEONINE-DAO/Nozy-wallet/issues/273)  
Forum: [NU7 Coinholder Vote](https://forum.zcashcommunity.com/t/nu7-coinholder-vote/56912)

## Eligibility

| Event | UTC |
|-------|-----|
| Ironwood snapshot | **2026-08-24 19:00** |
| Voting opens | 2026-08-25 |
| Voting closes | 2026-09-14 19:00 |

Migrate Orchard → Ironwood **before** the snapshot.

## Desktop

Nozy Desktop has a **Vote** tab (`VITE_ENABLE_NU7_VOTE`, on unless `false`). Export/sign run in the desktop process; prepare/delegate/cast **shell out** to this `nozy-vote` binary (cannot link `zcash_voting` beside `zeaking` — sqlite `links` conflict). Set `NOZY_VOTE_BIN` or place `nozy-vote.exe` next to the desktop app; in dev it looks under `tools/nozy-vote/target/release/`.

## End-to-end (when a round is ACTIVE)

```powershell
# 1) From repo root — export Ironwood notes (unlocks wallet)
nozy vote-export-notes --out vote-notes.json

# 2) Vote helper (tools/nozy-vote)
cd tools\nozy-vote
cargo build --release
.\target\release\nozy-vote.exe --env prod status
.\target\release\nozy-vote.exe --env prod active
.\target\release\nozy-vote.exe hotkey-init
.\target\release\nozy-vote.exe --env prod init-round
.\target\release\nozy-vote.exe --env prod import-notes --file ..\..\vote-notes.json
.\target\release\nozy-vote.exe --env prod delegate --notes-file ..\..\vote-notes.json
# writes signing-request-<round>.json under the data dir (see status)

# 3) Sign with wallet seed (stays in nozy — not in nozy-vote)
cd ..\..
nozy vote-sign-delegation --request <path-to-signing-request.json> --out delegation-sig.json

# 4) PIR + prove + submit delegation
cd tools\nozy-vote
.\target\release\nozy-vote.exe --env prod delegate-finish --notes-file ..\..\vote-notes.json --sig ..\..\delegation-sig.json
```

`cast` (ballot choices + helper shares) is wired:

```powershell
.\target\release\nozy-vote.exe --env prod active
.\target\release\nozy-vote.exe --env prod cast --choices 1=0,2=1,3=0,4=0,5=0
# optional: --delegation-tx <hash>  --single-share  --no-wait
```

Use `--env stage` against Valar staging while testing; production NU7 uses `--env prod`.

## Build

```powershell
cd tools\nozy-vote
cargo build --release
```

`zcash_voting` 3.0.0-rc.3 is a **required** dependency (always linked). First compile is heavy.

If Cursor sets `CARGO_TARGET_DIR` to a sandbox cache, the binary lands there — not under `tools/nozy-vote/target/`. Run the exe from `$env:CARGO_TARGET_DIR\release\nozy-vote.exe`, or unset `CARGO_TARGET_DIR` before `cargo build`.

## AI disclosure

Implementation assisted by Cursor Agent. Human author remains responsible for correctness and security.
