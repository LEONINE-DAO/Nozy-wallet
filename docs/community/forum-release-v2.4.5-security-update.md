# Forum reply — CLI v2.4.5 / v2.4.6 + Desktop beta.6 (+ companion API)

**Thread (reply here, don’t start a new topic):**  
https://forum.zcashcommunity.com/t/nozywallet-development-roadmap/53745

Paste-ready below. Tone matches prior roadmap updates. Adjust freely.

---

## Body (paste)

NozyWallet update — **Teriyaki Hot v2.4.5** + **Mango Habanero v2.4.6** + **Hot Lemon Pepper Sprinkles beta.6**

Short follow-up on what we cut fresh **CLI**, **Desktop**, and now **localhost companion API** builds so the security / supply-chain work — and the companion packaging — are what people actually download.

### What we fixed

**Rust / wallet binaries** (shipped on **v2.4.5** / desktop **beta.6**; carried forward on **v2.4.6**)

* **RUSTSEC-2026-0204** — `crossbeam-epoch` → **0.9.20** (CLI + desktop locks)
* **RUSTSEC-2026-0194 / 0195** — desktop `plist` → **1.10.0** → `quick-xml` **0.41**
* Broader **cargo-audit** cleanup: dropped yanked `core2`, bumped `anyhow` / `spin` / `getset`; remaining UniFFI/Tauri/ark noise is documented ignore-only
* GitHub Dependabot security alerts for those crates are cleared on `master`

**Companion API (from v2.4.6)**

* GitHub Release attach of **`nozywallet-api-*`** for same-machine extension / local apps (`http://127.0.0.1:3000` by default)
* Seed / bind policy documented; optional `NOZY_PRODUCTION` + `NOZY_API_KEY` for locked-down local runs — not a public hosted wallet API

### Releases

| **Surface** | **Tag** | **Notes** |
|:---|:---|:---|
| CLI | [v2.4.5 — Teriyaki Hot](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/v2.4.5) | Supply-chain clears |
| CLI + companion API | [v2.4.6 — Mango Habanero](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/v2.4.6) | **Latest** CLI; `nozywallet-api-*` localhost companion |
| Desktop | [desktop-v1.0.0-beta.6 — Hot Lemon Pepper Sprinkles](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/desktop-v1.0.0-beta.6) | Pre-release; same food name |

If you’re on **v2.4.4** / **desktop beta.5**, please upgrade — older installers won’t pick up these lockfile fixes by themselves. Prefer **v2.4.6** for CLI (includes companion binaries).

### What this is *not*

* Not hosted / public companion GA — localhost beta only
* Not “we’re done with audits” — third-party review remains optional / community-driven
* Desktop remains **beta** until GA

### Links

* Roadmap home (this thread): https://forum.zcashcommunity.com/t/nozywallet-development-roadmap/53745  
* All releases: https://github.com/LEONINE-DAO/Nozy-wallet/releases  
* Companion notes: https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/browser-extension/COMPANION.md  
* Prefer local **Zebrad + lightwalletd**; verify downloads with `HASHES.txt` / `.sha256`

No Delays

---

## Shorter variant

Upgrade note: **CLI [v2.4.6 — Mango Habanero](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/v2.4.6)** (latest; includes **localhost companion API** `nozywallet-api-*`), prior security cut **[v2.4.5 — Teriyaki Hot](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/v2.4.5)**, and **Desktop [beta.6 — Hot Lemon Pepper Sprinkles](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/desktop-v1.0.0-beta.6)**. Supply-chain clears (`crossbeam-epoch`, desktop `quick-xml`/`plist`, cargo-audit + Dependabot). Please move off 2.4.4 / beta.5.

Full context stays in the roadmap thread: https://forum.zcashcommunity.com/t/nozywallet-development-roadmap/53745

No Delays
