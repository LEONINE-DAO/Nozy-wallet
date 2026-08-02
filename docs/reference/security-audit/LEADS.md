# Leads table — AI-assisted audit (2026-08-01)

**Method:** LA-style S1 broad + S2 threat-model + S3 scenarios  
**Scope:** CLI + `nozy` core + Desktop Tauri ([`SCOPE.md`](SCOPE.md))  
**Status legend:** `open` → not yet triaged · `confirmed` · `mitigated` · `duplicate` · `info` · `false_positive`

Cap: ~20 leads. Citations verified against tree on 2026-08-01.

| ID | Strategy | Title | Area | Sev cand. | Status | Citations |
|----|----------|-------|------|-----------|--------|-----------|
| L01 | S1 | Privacy gate ≠ submit transport (proxy allow then clearnet `ZebraClient::new`) | privacy | High | fixed | `migration.rs` + `zebra_client_for_migration_broadcast` (2026-08-01) |
| L02 | S1 | Plaintext `notes.json` stores note/rseed/rho/witnesses | keys | High | fixed | `notes_vault.rs` NZN1 AES-GCM (2026-08-01) |
| L03 | S1 | Custom iterated-SHA256 vault KDF (not Argon2) | keys | Medium | fixed | `storage.rs` NZK2 Argon2id (2026-08-01) |
| L04 | S1/S2 | Desktop unlock password in process `Mutex<String>` | session | Medium | hardened | scrub on clear + migrate step-up (2026-08-01) |
| L05 | S1 | `HDWallet` Clone + mnemonic/XPrv not zeroizing | keys | Medium | fixed | Drop zeroize + no Clone; share via `&`/`Arc` (2026-08-01) |
| L06 | S1/S2 | RFC1918 / link-local counted as local for Priority 1 | privacy | Medium | fixed | loopback-only LocalNode (2026-08-01) |
| L07 | S1/S3 | Desktop IPC: `force_clearnet` / `attest_*` / `skip_broadcast_hygiene` | ipc/privacy | Medium | hardened | force/skip ignored on desktop IPC (2026-08-01) |
| L08 | S1 | Desktop send `amount: f64` → zatoshis cast | tx_correctness | Medium | fixed | `zec_to_zatoshis_exact` + `amount_zatoshis` (2026-08-01) |
| L09 | S1/S3 | Backup / LWD paths unsandboxed | ipc | Medium | hardened | allowlist + restore step-up (2026-08-01) |
| L10 | S1/S3 | Empty-identity merge append residual | state | Low | fixed | skip empty nullifier+note_bytes (2026-08-01) |
| L11 | S1 | Presigned turnstile hex on disk (schedule JSON) | privacy | Medium | fixed | wipe + NZS1 schedule encrypt (2026-08-01) |
| L12 | S2 | CLI full mnemonic print on create | keys | High | hardened | create warnings + clear-terminal guidance (2026-08-01) |
| L13 | S2 | api-server binds `0.0.0.0`; optional auth; mnemonic restore | rpc | High | fixed | default `127.0.0.1`; refuse `0.0.0.0` without API key (2026-08-01) |
| L14 | S3 | Schedule rebuild wipes `broadcast_txid` / Confirmed history | state | Medium | fixed | merge-preserve history (2026-08-01) |
| L15 | S3 | Migrate prebuild skips witness-lag hard gate (`None, None`) | tx_correctness | Medium | fixed | lag args in `execute_orchard_migration` (2026-08-01) |
| L16 | S3 | Desktop migrate/broadcast: no step-up re-auth | session | Medium | fixed | `load_wallet_for_migrate` (2026-08-01) |
| L17 | S3 | Twin-note merge by nullifier | state | — | mitigated | `notes.rs` ~560–622; `wallet_sync` twin tests |
| L18 | S3 | Zero Ironwood change on canonical crossing | tx_correctness | — | mitigated | `migration.rs` change≠0 reject; funding ExactCover/FeeFromOutput |
| L19 | S2 | Privacy gate exists; clearnet is explicit override | privacy | — | mitigated | `require_migration_network_privacy`; residual = L01/L07 |
| L20 | S3 | Loose `>= transfer+fee` cover helper vs strict funding | tx_correctness | Low | fixed | cover helper ≡ `select_canonical_zip318_funding` (2026-08-01) |

## Dedup notes

- L17 / historical twin-note bug: duplicate of known mainnet postmortem (fixed).  
- L13 overlaps `SECURITY_REVIEW.md` §4 (still open in checklist; code confirms bind).  
- L19 does not cancel L01 — gate without matching transport remains High.

## Strategy coverage

| Strategy | Lead IDs |
|----------|----------|
| S1 Broad | L01–L11 |
| S2 Threat-model | L04, L06, L07, L12, L13, L19 |
| S3 Scenarios | L07, L09, L10, L14–L18, L20 |
