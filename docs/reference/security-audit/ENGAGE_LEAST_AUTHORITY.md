# Engage Least Authority (or peer firm) — NozyWallet SOW paste

**Date:** 2026-08-01  
**Org:** LEONINE DAO · NozyWallet  
**Security contact:** Nozywallet.support@leoninedao.org  
**Repo:** https://github.com/LEONINE-DAO/Nozy-wallet  

This is an **engagement brief**, not an audit report. In-repo AI-assisted prep: [`SCOPE.md`](SCOPE.md), [`THREAT_MODEL.md`](THREAT_MODEL.md), [`FINDINGS.md`](FINDINGS.md).

---

## Ask

Professional security review of **NozyWallet** as a Zcash **wallet** (not a consensus node), preferably using an AI-assisted + human-triage workflow similar to Least Authority’s [ZCG ecosystem engagement](https://leastauthority.com/blog/ai-assisted-security-auditing-in-the-zcash-ecosystem/).

## Proposed scope (v1)

| Include | Exclude |
|---------|---------|
| CLI (`nozy`) + shared Rust core | Orchard / Ironwood **circuit** soundness (upstream) |
| Desktop Tauri shell + IPC | Browser extension MV3 / WASM (phase 2) |
| Ironwood ZIP 318 migrate/broadcast path | Mobile FFI / Expo (phase 2) |
| Note cache, vault KDF, Zebrad RPC client | Zebra / lightwalletd themselves |
| Optional: api-server companion bind/auth | Full ecosystem crate re-audit |

## Trust model (short)

- Keys encrypted at rest; prove/sign locally; prefer local Zebrad JSON-RPC  
- Ironwood turnstile amounts are **public**; network privacy (IP↔submit) is a first-class concern  
- Desktop unlock session and companion HTTP are high-value boundaries  

## Prior work for auditors

| Item | Link / note |
|------|-------------|
| Internal checklist | [`SECURITY_REVIEW.md`](../../../SECURITY_REVIEW.md) |
| Self-audit guide / results | [`SELF_SECURITY_AUDIT_GUIDE.md`](../../../SELF_SECURITY_AUDIT_GUIDE.md), [`SELF_AUDIT_RESULTS.md`](../../../SELF_AUDIT_RESULTS.md) |
| AI-assisted prep findings | [`FINDINGS.md`](FINDINGS.md) — treat as **leads**, re-verify |
| Mainnet send evidence | [`MAINNET_SEND_READINESS_EVIDENCE.md`](../MAINNET_SEND_READINESS_EVIDENCE.md) |
| Mainnet Ironwood turnstile | [`MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md`](../MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md) TXID `ea2fa4e6…7048fd` @ height 3430663 |
| Ironwood readiness | [`IRONWOOD_WALLET_READINESS.md`](../IRONWOOD_WALLET_READINESS.md) |

## Suggested focus areas (from prep)

1. Migration broadcast **privacy policy vs HTTP transport**  
2. Vault KDF + plaintext note cache  
3. Desktop session / IPC overrides for migrate  
4. api-server bind + mnemonic-on-wire  
5. ZIP 318 schedule integrity / witness-lag parity with send  

## Deliverables we want

- Written report with severity, citations, remediations  
- Distinction: confirmed vs informational  
- Optional: retest after remediation window  
- Clear statement that **circuit soundness was out of scope** (unless separately contracted)

## Commit pin

Auditors should pin a **release tag or commit SHA** at kickoff (record here when engaging):

```text
Repo: LEONINE-DAO/Nozy-wallet
Branch / tag: ________________
Commit SHA:   ________________
Date:         ________________
```

## Honest claims

- Nozy has **not** completed a third-party audit as of this date.  
- Prep findings are **self-triaged AI-assisted leads**, not LA-validated.  
- Desktop remains **beta** until external review + remediations for High findings.

## Contact

Email: **Nozywallet.support@leoninedao.org**  
Responsible disclosure: see [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) / [`SECURITY.md`](../../../SECURITY.md)
