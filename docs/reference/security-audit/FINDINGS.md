# Findings — triaged (2026-08-01)

**Method:** Human triage of [`LEADS.md`](LEADS.md) after AI lead collection (LA-style).  
**Severity:** OWASP-inspired Impact × Likelihood (wallet context).  
**Not a Least Authority report.** Do not cite as third-party audit.

---

## Confirmed findings

### F-01 — Migration privacy mode does not bind the submit HTTP client
| | |
|--|--|
| **Lead** | L01 |
| **Severity** | **High** → **Fixed (2026-08-01)** |
| **Impact** | High — IP ↔ public turnstile amount linkage despite “Tor detected” |
| **Likelihood** | Medium — only when Zebrad URL is remote and proxy auto-detect succeeds |

**Description.** `require_migration_network_privacy` may allow broadcast under `DetectedPrivacyProxy`, but `execute_orchard_migration_broadcast` builds `ZebraClient::new(url)` with `privacy_proxy_url: None` and then `broadcast_transaction` on that client.

**Evidence.** [`src/ironwood/migration.rs`](../../../src/ironwood/migration.rs) (~1676, ~1794); [`src/zebra_integration.rs`](../../../src/zebra_integration.rs) `ZebraClient::new` (~306–338).

**Remediation (landed).** After privacy gate succeeds, rebuild submit client via `zebra_client_for_migration_broadcast` / `ZebraClient::with_migration_privacy` so Tor/I2P proxy or Nym mixnet flag matches the assessed mode. Unit tests cover proxy + mixnet client modes.

**Test idea.** Mock assess → DetectedPrivacyProxy; assert client builder receives proxy URL (or broadcast refuses).

---

### F-02 — Note cache persisted in plaintext beside encrypted vault
| | |
|--|--|
| **Lead** | L02 |
| **Severity** | **High** → **Fixed (2026-08-01)** |
| **Impact** | High — spend material / targeting metadata on disk |
| **Likelihood** | Medium — disk theft / malware reading AppData |

**Description.** `notes.json` / NoteIndex stores note bytes, optional `rho`/`rseed`, witnesses, values, nullifiers without encryption.

**Evidence.** [`src/notes.rs`](../../../src/notes.rs) `SerializableOrchardNote`, save paths.

**Remediation (landed).** [`src/notes_vault.rs`](../../../src/notes_vault.rs): on-disk `NZN1` + AES-256-GCM; Argon2id from wallet password + persistent `notes.salt`; session key unlocked with wallet. Legacy plaintext JSON still loads; next save upgrades. Unlock wired from CLI, api-server, and desktop session.

---

### F-03 — api-server default bind all interfaces + optional auth + mnemonic restore
| | |
|--|--|
| **Lead** | L13 |
| **Severity** | **High** → **Fixed (2026-08-01)** |
| **Impact** | High — seed on HTTP if restore used without key |
| **Likelihood** | Low–Medium — depends on operator deploy |

**Description.** HTTP listener binds `0.0.0.0`. `NOZY_API_KEY` optional. Restore accepts mnemonic on the wire. Grant checklist claimed default `127.0.0.1` / `NOZY_BIND_ADDR` — **current `main.rs` still hard-binds `0.0.0.0`**.

**Evidence.** [`api-server/src/main.rs`](../../../api-server/src/main.rs); restore handlers; [`SECURITY_REVIEW.md`](../../../SECURITY_REVIEW.md) §4.

**Remediation (landed).** Default `NOZY_BIND_ADDR=127.0.0.1`. Binding `0.0.0.0`/`::` without `NOZY_API_KEY` **refuses to start**. Docs updated in [`SECURITY_CONFIG.md`](../../../api-server/SECURITY_CONFIG.md).

---

### F-04 — CLI prints full mnemonic on wallet create
| | |
|--|--|
| **Lead** | L12 |
| **Severity** | **High** → **Hardened (2026-08-01)** (still prints once by design) |
| **Impact** | High — terminal history, screenshare, shoulder-surf |
| **Likelihood** | High on every create |

**Description.** Intentional UX (`SECURITY_REVIEW` notes create print). Still a primary leak vector.

**Remediation (landed partial).** Pre-print warnings + clear-terminal guidance on mainnet and testnet create. Full “write to file / no stdout” remains optional follow-up.

---

### F-05 — Custom vault KDF (iterated SHA-256)
| | |
|--|--|
| **Lead** | L03 |
| **Severity** | **Medium** → **Fixed (2026-08-01)** |
| **Impact** | High if weak passwords |
| **Likelihood** | Medium offline |

**Description.** `derive_key_from_password` uses 100k iterated SHA-256, not Argon2id/scrypt. AES-GCM encryption itself is fine.

**Evidence.** [`src/storage.rs`](../../../src/storage.rs).

**Remediation (landed).** New writes use `NZK2` + Argon2id. Legacy blobs still decrypt; next save upgrades. Unit tests cover roundtrip + legacy read.

---

### F-06 — Desktop session holds unlock password; migrate/broadcast without step-up
| | |
|--|--|
| **Leads** | L04, L16 |
| **Severity** | **Medium** → **Hardened (2026-08-01)** |
| **Impact** | High if process memory compromised |
| **Likelihood** | Low–Medium |

**Description.** `UNLOCK_PASSWORD` stores password for unlock lifetime. Ironwood migrate/broadcast use session wallet load; reveal path requires re-auth.

**Evidence.** [`desktop-client/src-tauri/src/session.rs`](../../../desktop-client/src-tauri/src/session.rs); ironwood commands.

**Remediation (landed partial).** Session clear scrub-overwrites password; notes vault key cleared on lock. Split / migrate / broadcast require explicit password step-up for password-protected wallets (`load_wallet_for_migrate`). Full Zeroize + idle timeout remain follow-up.

---

### F-07 — Desktop IPC privacy overrides are client-trusted bools
| | |
|--|--|
| **Lead** | L07 |
| **Severity** | **Medium** → **Hardened (2026-08-01)** |
| **Impact** | Medium (privacy policy bypass) |
| **Likelihood** | Medium if renderer compromised |

**Description.** `force_clearnet`, `attest_private_network`, `skip_broadcast_hygiene` accepted from Tauri request body.

**Remediation (landed partial).** Desktop `ironwood_broadcast` / status path force `force_clearnet=false` and `skip_broadcast_hygiene=false` regardless of IPC. Attestation checkbox still honored (Settings). Native confirm dialog for clearnet remains optional follow-up (CLI still has `--force-clearnet`).

---

### F-08 — RFC1918 treated as “local” for migration privacy
| | |
|--|--|
| **Lead** | L06 |
| **Severity** | **Medium** → **Fixed (2026-08-01)** |
| **Impact** | Medium — LAN node sees IP↔turnstile |
| **Likelihood** | Medium on WSL/LAN Zebrad setups |

**Evidence.** [`zebra_integration.rs`](../../../src/zebra_integration.rs) `url_is_loopback`; [`network_privacy.rs`](../../../src/ironwood/network_privacy.rs).

**Remediation (landed).** Priority 1 auto-allow is **loopback only**. Private LAN Zebrad requires Tor/I2P, Nym, attestation, or `--force-clearnet`.

---

### F-09 — Schedule rebuild drops confirmed migration history
| | |
|--|--|
| **Lead** | L14 |
| **Severity** | **Medium** → **Fixed (2026-08-01)** |
| **Impact** | Medium (integrity / forensics; not direct fund theft) |

**Description.** `load_or_rebuild_orchard_migration_schedule` rebuilds from plan when validation fails or expired transfers exist, clearing `broadcast_txid` / Confirmed rows. Observed on operator mainnet profile after preflight refresh.

**Evidence.** [`migration.rs`](../../../src/ironwood/migration.rs); [`MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md`](../MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md) schedule caveat.

**Remediation (landed).** Rebuild merges preserve `Broadcast`/`Confirmed` rows; validation compares open (Pending/Presigned) transfers to the current plan only. Explicit plan save also preserves history. Confirmed rows drop `presigned_tx_hex`.

---

### F-10 — Migrate prebuild skips witness-lag readiness gate
| | |
|--|--|
| **Lead** | L15 |
| **Severity** | **Medium** → **Fixed (2026-08-01)** |

**Description.** `assess_orchard_migration_readiness(..., None, None)` omits lag args that normal send uses. Anchor root still checked at prove time; residual risk is failed or poorly timed prebuilds / operator confusion.

**Remediation (landed).** `execute_orchard_migration` now passes `max_witness_lag_blocks(spendable)` + `MAX_SEND_WITNESS_LAG_BLOCKS` into readiness (same gate as send/CLI preflight).

---

### F-11 — Unsandboxed backup / LWD path arguments
| | |
|--|--|
| **Lead** | L09 |
| **Severity** | **Medium** → **Hardened (2026-08-01)** |

**Remediation (landed).** Desktop backup/restore paths must resolve under wallet data / home allowlist (`resolve_allowlisted_user_path`). LWD `db_path` must stay under the active wallet data dir. Restore over an existing wallet requires step-up password.

---

### F-12 — Desktop send amount via `f64`
| | |
|--|--|
| **Lead** | L08 |
| **Severity** | **Medium** → **Fixed (2026-08-01)** |

**Remediation (landed).** `zec_to_zatoshis_exact` / `resolve_send_amount_zatoshis`; desktop prefers `amount_zatoshis` from exact decimal-string parse in SendForm; legacy `amount` f64 still accepted only when exact. **Residual (2026-08-01):** Keystone/cosign and api-server send paths now use the same resolver.

---

### F-13 — Presigned turnstile hex on disk
| | |
|--|--|
| **Lead** | L11 |
| **Severity** | **Medium** → **Fixed (2026-08-01)** |

**Remediation (landed).** Wipe `presigned_tx_hex` on broadcast/confirm and on every schedule save for Broadcast/Confirmed/Expired. Schedule file encrypted at rest (NZS1, notes vault session key); plaintext JSON still loads and upgrades on next save.

---

### F-14 — Empty-identity merge append residual
| | |
|--|--|
| **Lead** | L10 |
| **Severity** | **Low** → **Fixed (2026-08-01)** |

**Remediation (landed).** `merge_scanned_notes` skips rows lacking both nullifier and `note_bytes` (logs warning) instead of appending opaque cache rows.

---

## Mitigated / closed (keep for auditors)

| ID | Topic | Notes |
|----|-------|-------|
| M-01 | Twin-note merge | Nullifier/`note_bytes` identity + tests (L17) |
| M-02 | Zero Ironwood change on ZIP 318 crossing | Enforced at build (L18) |
| M-03 | Privacy gate exists | Clearnet must be explicit — but see **F-01** transport gap (L19) |

---

## Deferred / open (not elevated)

| Lead | Reason |
|------|--------|
| — | None — F-* findings and deferred leads L05/L20 closed; optional residuals landed 2026-08-01 |

Residuals completed: `HDWallet` no longer `Clone` (`&` / `Arc`); schedule NZS1 encryption; Keystone/cosign/api exact zatoshis.

---

## Summary counts

| Severity | Count |
|----------|-------|
| High fixed / hardened | F-01, F-02, F-03, F-04 (partial) |
| High still open | — |
| Medium fixed / hardened | F-05–F-13 |
| Medium still open | — |
| Low fixed | F-14 |
| Mitigated baselines | 3 |

---

## Recommended remediations order

1. ~~F-01 privacy transport bind~~ **done**
2. ~~F-03 api-server loopback default + auth for seed routes~~ **done**
3. ~~F-02 encrypt note cache~~ **done**
4. ~~F-05 Argon2id vault~~ **done**
5. ~~F-06/F-07 Desktop session + IPC privacy~~ **hardened**
6. ~~F-09 schedule history preservation~~ **done**
7. ~~F-10–F-14 remaining Medium/Low~~ **done / hardened**

---

## Update log

| Date | Note |
|------|------|
| 2026-08-01 | Initial LA-style triage from S1–S3 leads |
| 2026-08-01 | Fixed F-01, F-03, F-05, F-08; hardened F-04 |
| 2026-08-01 | Fixed F-02, F-09; hardened F-06, F-07 |
| 2026-08-01 | Fixes: F-01 submit client bind, F-03 bind+API key gate, F-04 create warnings, F-05 NZK2 Argon2id, F-08 loopback-only LocalNode |
| 2026-08-01 | Fixed/hardened F-10–F-14 (witness lag, path sandbox, exact zatoshis, wipe presigned hex, skip empty-identity merge) |
| 2026-08-01 | L05 HDWallet Drop/zeroize; L20 cover helper aligned to strict ZIP 318 funding |
| 2026-08-01 | Residuals: HDWallet no Clone; NZS1 schedule encrypt; Keystone/cosign/api exact zatoshis |
