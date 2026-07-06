# Ironwood — protocol formal verification (not wallet updates)

**Wallet / NU6.3 network upgrade work:** see [`IRONWOOD_WALLET_READINESS.md`](IRONWOOD_WALLET_READINESS.md).

**For Nozy team — revisit when we engage with Zcash proof work.**

Ironwood is **also** the name of the proposed **NU6.3 network upgrade** (new shielded pool + turnstile). This file tracks the **Lean 4 formal verification** repo (`zcash/ironwood`). Nozy implements Orchard spends/sync in Rust; the proof repo asks “is the protocol argument sound?” — a different layer from wallet migration code.

| Layer | Examples |
|-------|----------|
| **Ironwood** | Binding-signature balance, spend authority proofs, spec §4.14 formalization |
| **Nozy** | `orchard_tx`, `wallet_sync`, api-server, extension, ZIP-317 pilot |

---

# Ironwood PR #4 — knowledge notes

**Source:** [zcash/ironwood#4](https://github.com/zcash/ironwood/pull/4)  
**Author:** Daira-Emma Hopwood (`daira`)  
**Branch:** `binding-signature-balance` → `main`  
**Status (Jun 2026):** **Draft** again after review feedback (was briefly ready-for-review)  
**Reviewers:** TalDerei (comments); ebfull requested, awaiting

---

## What is Ironwood?

Public Zcash repo for **protocol formal verification** alongside the existing **mdbook** spec (`book/`). Goal: machine-checked proofs in **Lean 4** that match the protocol spec, starting with security-critical arguments Orchard/Ironwood care about.

**Not** a wallet or node implementation — this is math/proof infrastructure.

---

## What PR #4 does (one sentence)

Adds Lean 4 project scaffolding + CI, then formalizes the **binding-signature balance soundness** argument from protocol spec **§4.13 (Sapling)** and **§4.14 (Orchard)** — proving that a valid binding signature implies the transaction’s value sum is zero (no inflation).

---

## Why binding signature balance matters

Shielded pools (Sapling/Orchard) use **homomorphic value commitments** and a **binding signature** over the bundle. Soundness requires:

1. **Modulo field order:** `vSum ≡ 0 (mod r)` where `r` is the scalar field order (Pallas for Orchard, Jubjub for Sapling).
2. **As integers:** `vSum = 0` over ℤ (no silent wrap-around that could fake balance).

The spec’s pencil-and-paper proof is two stages; this PR formalizes both stages for both pools.

---

## Repo layout after merge

| Path | Role |
|------|------|
| `book/` | Existing mdbook spec (unchanged by this PR) |
| `Zcash/` | Lean package + lib namespace `Zcash` |
| `Zcash/Security/BindingSignature/Balance.lean` | Shared algebraic core |
| `Zcash/Security/BindingSignature/Orchard.lean` | Orchard no-overflow + capstone |
| `Zcash/Security/BindingSignature/Sapling.lean` | Sapling no-overflow + capstone |
| `lakefile.toml`, `lean-toolchain`, `lake-manifest.json` | Lake build |
| `.github/workflows/lean.yml` | Lean CI |
| `.github/workflows/book.yml` | Book CI (path-gated) |

**Toolchain:** Lean `v4.30.0-rc2`, pinned Mathlib, **CompElliptic** dep (git rev) for Pallas/Jubjub scalar field orders with primality certs.

---

## The proof architecture (two stages)

### Stage 1 — Algebraic / mod `r` balance

Over abstract `F`-module (later `ZMod r`):

- `V`, `R` — value and randomness bases
- `valueCommit`, `bindingVK`, `bsk` — commitments and binding key
- **`relation_of_imbalance`** — if bundle doesn’t balance (`A ≠ 0`), you get an explicit **discrete-log relation** between `V` and `R`
- **`bundle_mod_balances`** — under binding + extractability, `vSum ≡ 0 (mod r)`

**Binding model:** reduction to **DLR** (discrete-log relation), equivalent to DL (Jaeger–Tessaro). Not assumed as “no relation exists” — imbalance **exhibits** one.

### Stage 2 — Integer balance (no overflow)

- **`intBalance_eq_zero_of_lt`** — if `vSum ≡ 0 (mod r)` and `|vSum| < r`, then `vSum = 0` in ℤ
- **`natAbs_lt_of_vSumBound`** — shared helper: `vSumBound N = N·(2^64−1) + 2^63`
- Per-pool wrappers prove `|vSum| < r` from consensus/size bounds

**Capstone theorems:**

- `orchard_bundle_balances` — Orchard integer balance end-to-end
- `sapling_bundle_balances` — Sapling integer balance end-to-end

---

## Orchard-specific bounds (our main interest)

- Each action net value: `v = v_spend − v_output ∈ [−2^64+1, 2^64−1]`
- Action count: `n ≤ 2^16 − 1` (direct consensus rule)
- `orchardVSumBound = 1208916596242592319864833`
- Field: **Pallas** scalar order via `CompElliptic.Fields.Pasta.PALLAS_SCALAR_CARD`
- Theorem: `orchardVSumBound_lt_pallasScalarOrder`

---

## Sapling (included, thin wrapper)

- Bounds from **2 MB tx size** vs min spend/output description sizes (v4 + v5)
- Conservative unified bound: `saplingMinSpendBytes = 352` → max spends 5681
- Field: **Jubjub** scalar order via CompElliptic

Daira notes Sapling adds little code; **Orchard + future Ironwood** are the verification focus.

---

## Assumptions vs proved

| Item | Status in PR |
|------|----------------|
| Algebraic binding reduction (`relation_of_imbalance`) | **Proved** |
| Integer no-overflow bounds per pool | **Proved** (`*_natAbs_lt`) |
| Field order `< r` gap | **Proved** via CompElliptic |
| RedDSA extractability (`hExtract`: `bvk = bsk • R`) | **Assumed** (ROM/forging proof is separate) |
| Binding / DLR hardness closing the reduction | **Documented**, AGM/DLR wrapper **not yet built** |

Balance argument is **ROM-free** (extractability is an explicit hypothesis).

---

## Spec crosswalk (§4.13 / §4.14)

| Lean | Spec |
|------|------|
| `V`, `R` | 𝒱, ℛ |
| `valueCommit V R v rcv` | ValueCommit_rcv(v) |
| `bindingVK` / `bsk` | bvk / BindingPrivate |
| `bindingVK_decomp` | bvk = [vSum]𝒱 + [bsk]ℛ |
| `hExtract` | binding sig proves knowledge of bsk |
| `vSum` | Σ v_in − Σ v_out − vBalance |
| `bundle_mod_balances` | vSum ≠ 0 mod r ⇒ break binding |
| `intBalance_eq_zero_of_lt` | no overflow ⇒ vSum = 0 in ℤ |

---

## CI pattern (worth copying)

**Gate-job path gating:** workflows always run; internal filter decides if heavy build runs; still reports a **single required check** (`Lean CI` / `Ironwood book CI`). Avoids `on.paths` skipping leaving required checks pending.

- Lean build: `leanprover/lean-action` + Mathlib olean cache + `lake build`
- Lean skipped only if change is provably book-only; ambiguous changes run both

---

## Review thread takeaways (Jun 15–16)

**TalDerei** — cross-checked against spec §4.13/4.14:

- ACK: binding as **reduction** (not assuming independence)
- ACK: `hExtract` as right abstraction boundary
- ACK: likes `relation_of_imbalance` counterfactual + DLR in docs
- **Open question:** `relation_of_imbalance` / `rand_log_of_imbalance` are **not yet consumed** by any downstream theorem — scaffolding only?
- **Spec typo:** Orchard section cites Jubjub prime instead of Pallas — Daira will fix

**Daira response:**

- Wants to prove RedDSA strong unforgeability separately (key-rerandomization / Spend authority work exists for Sapling but may need redo)
- **Put PR back in Draft** until AGM/DLR wrapper wires `relation_of_imbalance` into the main theorem chain

---

## Related repos / PRs

| Link | Notes |
|------|------|
| [daira/zcash-lean#2](https://github.com/daira/zcash-lean/pull/2) | Original binding-signature work (cherry-picked into ironwood) |
| [daira/CompElliptic#1](https://github.com/daira/CompElliptic/pull/1) | Jubjub scalar field + primality (merged) |
| [eprint 2020/1213](https://eprint.iacr.org/2020/1213) | DLR ↔ DL equivalence (cited in docs) |

---

## What’s left before merge (from PR description)

1. **AGM / DLR wrapper** consuming `relation_of_imbalance` (close the reduction in the proof chain)
2. Wire or justify isolated lemmas vs capstones
3. Fix Orchard/Pallas spec typo reference
4. ebfull review

---

## Relevance to Nozy / Zeaking (when we engage)

- **Orchard-only wallet stack** — Orchard binding balance is directly relevant to “can a malformed tx inflate shielded supply?”
- Does **not** replace ZIP-317, expiry, wallet sync, or speed-up work — different layer (cryptographic soundness vs implementation)
- Future Ironwood verification may cover more of Orchard spend/auth path; watch this repo for proof APIs and statement of assumptions
- If we contribute: likely review clarity, test vectors alignment, or tooling — not Rust wallet code

---

## Quick start checklist (when we begin)

```bash
git clone https://github.com/zcash/ironwood.git
cd ironwood
git fetch origin pull/4/head:pr-4
git checkout pr-4
# Install Lean 4.30.0-rc2 (see lean-toolchain)
lake build
```

Read in order:

1. PR description + spec §4.14 (Orchard binding)
2. `Zcash/Security/BindingSignature/Balance.lean` module doc
3. `Orchard.lean` → `orchard_bundle_balances`
4. Review comments on GitHub for open wiring gap

---

## Local tracking

| Field | Value |
|-------|--------|
| First captured | 2026-06-15 |
| Added to NozyWallet | 2026-06-15 |
| Our intent | Monitor / possibly contribute when Ironwood Orchard proofs expand |
| Blocker to follow | PR #4 draft until DLR wrapper lands |

*Update this file when PR merges or scope changes.*
