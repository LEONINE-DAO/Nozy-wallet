# Forum post — NozyWallet AI-assisted security review (2026-08-01)

Paste-ready. Adjust tone / links as needed. **Do not claim this is a Least Authority certificate.**

---

## Title options

1. NozyWallet: Least Authority–style AI-assisted self-review (CLI + Desktop) — findings closed
2. What we audited in NozyWallet (and what we fixed) — August 2026
3. Ironwood / ZIP 318 wallet security pass: scope, findings, remediations

---

## Body

**NozyWallet — AI-assisted security self-review (2026-08-01)**

We ran a Least Authority–style **AI-assisted self-review** of NozyWallet (CLI + shared `nozy` core + Desktop Tauri), then triaged and **remediated** the findings in-tree. This is **not** a Least Authority engagement or certificate. It follows the spirit of their [AI-assisted auditing writeup for the Zcash ecosystem](https://leastauthority.com/blog/ai-assisted-security-auditing-in-the-zcash-ecosystem/): broad lead collection → threat-model pass → scenario pass → human triage → fixes with tests where practical.

Full pack in the repo:  
https://github.com/LEONINE-DAO/Nozy-wallet/tree/main/docs/reference/security-audit  
(`SCOPE.md`, `THREAT_MODEL.md`, `LEADS.md`, `FINDINGS.md`)

### What was in scope

| Area | Focus |
|------|--------|
| Key material | Vault KDF, mnemonic handling, HD wallet memory |
| Ironwood / ZIP 318 | Plan, split, migrate, broadcast, network privacy, schedule integrity |
| Note cache | Disk encryption, merge / identity bugs |
| Zebrad RPC | Proxy binding, “local” node definition |
| Desktop IPC / session | Password step-up, path sandboxing, amount parsing |
| Companion API | Bind address + auth for seed-bearing routes |

**Out of scope this pass:** Orchard/Ironwood circuit soundness (upstream), browser extension WASM, mobile FFI, Zebra consensus, third-party pen-test.

### Method (short)

1. **S1 — Broad scan** of high-impact surfaces (keys, IPC, migration broadcast, disk).
2. **S2 — Threat model** against disk theft, compromised renderer, remote Zebrad, LAN operators.
3. **S3 — Scenarios** (migrate/broadcast, schedule rebuild, note merge, funding helpers).
4. Cap ~20 leads → triage into **F-01…F-14** findings + a few mitigated baselines.
5. Ship remediations + unit tests; document residuals honestly.

### Results at a glance

| Bucket | Count / status |
|--------|----------------|
| Leads collected | 20 (L01–L20) |
| Elevated findings | F-01 … F-14 |
| High | 4 — **fixed / hardened** (F-01–F-04; F-04 still prints mnemonic once by design) |
| Medium | F-05–F-13 — **fixed / hardened** |
| Low | F-14 — **fixed** |
| Already mitigated baselines | Twin-note merge, zero Ironwood change on ZIP 318 crossing, privacy gate exists |
| Open elevated findings | **None** |

### Highlights — what we found and fixed

**Privacy / network**

- **F-01 (High):** Privacy gate could allow Tor/I2P, but broadcast still used a clearnet `ZebraClient`. Fixed: submit client now binds to the assessed privacy mode (proxy / Nym).
- **F-08:** “Local” node for safer migration was too broad (RFC1918). Fixed: **loopback only**; LAN needs Tor/I2P, Nym, attestation, or explicit clearnet override (CLI).
- **F-07:** Desktop IPC could force clearnet / skip hygiene. Hardened: those flags are ignored on desktop; attestation remains a Settings choice.

**Keys / disk**

- **F-02 (High):** `notes.json` was plaintext beside the vault. Fixed: **NZN1** AES-GCM + Argon2id (session unlock with wallet password).
- **F-05:** Vault KDF was iterated SHA-256. Fixed: **NZK2** Argon2id (legacy still decrypts; upgrades on save).
- **F-13:** Presigned turnstile hex on disk. Fixed: wipe after broadcast/confirm + **NZS1** encrypt the schedule file with the same session key.
- **L05:** `HDWallet` no longer `Clone`s secrets; Drop zeroizes mnemonic; share via `&` / `Arc`.

**Desktop / API**

- **F-03 (High):** api-server default bind was all interfaces. Fixed: default `127.0.0.1`; refuse `0.0.0.0` without `NOZY_API_KEY`.
- **F-06 / F-16:** Migrate/broadcast needed step-up password (not silent session reuse).
- **F-11:** Backup / LWD paths allowlisted; restore over existing wallet requires password.
- **F-12:** Send / Keystone / cosign prefer integer **zatoshis** (reject inexact `f64`).

**Migration correctness**

- **F-09:** Schedule rebuild used to drop confirmed turnstile history. Fixed: merge-preserve Broadcast/Confirmed.
- **F-10:** Migrate prebuild now uses the same witness-lag gate as send.
- **L20:** “Can we fund this transfer?” helper now matches strict ZIP 318 funding (no false “ready” on headroom notes).
- **F-14:** Empty-identity note rows are skipped on merge instead of polluting the cache.

### What we are *not* claiming

- This is **not** “Least Authority audited NozyWallet.”
- Upstream Orchard / Ironwood / halo2 soundness was **not** re-audited here.
- CLI still prints the mnemonic **once** on create (warned + clear-terminal guidance) — UX tradeoff, not closed as “no print ever.”
- Extension / mobile have their own threat models and were out of this pass.

### Where to read more

- Audit index: `docs/reference/security-audit/`
- Findings detail: `FINDINGS.md`
- Mainnet Ironwood migration evidence (separate from the audit): `docs/reference/MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md`

Happy to take questions, challenge any severity call, or point reviewers at specific diffs. If the community wants a formal third-party review later, we also drafted an engagement paste in `ENGAGE_LEAST_AUTHORITY.md`.

---

## Shorter blurb (Discord / reply)

We finished a LA-style AI-assisted self-review of NozyWallet CLI+Desktop (not an LA cert): ~20 leads → F-01–F-14, all elevated items fixed or hardened — including migration privacy client bind, notes/schedule encryption, Argon2id vault, api-server loopback default, desktop step-up + IPC hardening, and ZIP 318 schedule/funding fixes. Pack: `docs/reference/security-audit/`.
