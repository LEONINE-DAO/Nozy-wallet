# Case breakdown: “Half my ZEC disappeared” — ZIP 318 twin-note merge bug

**Status:** Root cause fixed locally · funds recovered · first Ironwood turnstile confirmed  
**Date:** 2026-07-29 → 2026-07-30 (mainnet, profile `5a5c81bda1fa6343`)  
**Related code:** [`src/notes.rs`](../../src/notes.rs) (`merge_scanned_notes`) · [`src/note_index.rs`](../../src/note_index.rs) · [`src/ironwood/migration.rs`](../../src/ironwood/migration.rs) (ZIP 318 split)  
**Audience:** article / postmortem / funding honesty — what went wrong, who pushed back, what we fixed

---

## One-line verdict

The wallet did **not** send ~$10 of ZEC to a stranger. ZIP 318 note-splits create **two equal-value Orchard notes in one tx**; sync **found both**, then `merge_scanned_notes` keyed on `(txid, height, value)` and **kept only one**. Balance looked halved. The operator kept saying “we didn’t spend that.” The assistant kept inventing other stories. The operator was right.

---

## What the operator saw

| Moment | What it looked like | Reality |
|--------|---------------------|---------|
| Before Ironwood work | ~**0.045 ZEC** (~**$21** at ~$470/ZEC) in Orchard | Real unspent notes including a 0.0454 parent later split |
| After sync / “migrate prep” | ~**0.0227 ZEC** (~**$10.70**) | Twin notes from the split were **dropped from `notes.json`**, not spent externally |
| Assistant narratives | “You sent it,” “display glitch,” “pre-NU6.3 lost it,” “desktop white-screen did it” | Wrong until deep dive |
| After fix + re-scan | **0.0447 ZEC** restored (9 unspent) | Both twins back under nullifier identity |
| After bucket window | **Ironwood ~0.0196** + **Orchard ~0.0027** | First ZIP 318 turnstile **confirmed** (`ea2fa4e64a5c…`) |

On-chain fee for those split txs was **0.0006 ZEC** each — not “ten dollars of fees.”

---

## Timeline (honest)

1. **Orchard note-splits** at heights **3428651** / **3428662** (`9b3b2f…`, `e0e233…`) — ZIP 318 canonical denominations:
   - `4,540,000` → **2×2,000,000 + 480,000** (+ 60,000 fee)
   - `480,000` → **2×200,000 + 20,000** (+ 60,000 fee)
2. Sync decrypted **all** outputs belonging to the wallet.
3. **`merge_scanned_notes`** treated `(txid, block_height, value)` as unique → second equal-value note never persisted.
4. CLI / Ironwood status showed **half** the Orchard balance. Operator: *we didn’t send $10*.
5. Assistant argued external send / coincidence / UI — **without proving a recipient** (shielded; not in `sent_transactions.json`; not in migrate schedule as broadcast).
6. Deep dive matched the math **exactly** to twin collapse.
7. Fix: merge (and spend-by-identity) on **nullifier** (and `rho` when ambiguous). Unit test: equal-value twins must both survive.
8. Re-scan from `3428650` restored **0.0447 ZEC**. An old migrate watcher racing an old binary briefly re-collapsed the cache — stopped watcher, re-synced with fixed binary.
9. Bucket **3430656** opened → first turnstile **confirmed** to Ironwood.

---

## Where the assistant was wrong (and the operator corrected it)

This section is intentional. Shipping wallets means **not gaslighting the person who holds the keys**.

| Claim from the session | Why it was wrong | What the operator said |
|------------------------|------------------|-------------------------|
| “You sent ~0.022 ZEC to another Orchard address” | Missing value matched **self-split twins**, not a logged send; no recipient in history/code | “We have not spent shit” / “receive wallet has no new ZEC” |
| “0.0447 was a duplicate display / plan+notes mix-up” | 0.0447 ≈ real pre-collapse total; twins are real notes with **different nullifiers** | “I had over $20 in ZEC” |
| “Lost during migrate because builder thought pre-NU6.3” | `IronwoodBuilderNotAvailable` **aborts before a tx exists**; those splits were Orchard-only earlier | Asked if migrate ate it — fair question; answer is no for *that* bug |
| “Lost when we fixed the desktop blank white screen” | Fix was `api` → `walletApi` + logos; desktop then showed Create/Restore (**unlocked empty UI**), not a broadcast | Tied the scare to that moment; the spend story was still wrong |
| Confident tone without chain+cache proof | Sounded like lying even when it was sloppy inference | Called it out |

**Lesson for agents and maintainers:** if the user says “I didn’t send that,” treat **wallet accounting bugs** as first-class suspects — especially after ZIP 318 splits that *intentionally* emit repeated denominations.

---

## Root cause (engineering)

```text
ZIP 318 split  →  two notes, same txid, same height, SAME value, different nullifiers
                         ↓
              merge_scanned_notes (old)
                         ↓
         find by (txid, height, value) → update first, skip second
                         ↓
              notes.json under-reports balance
```

**Primary bug:** [`merge_scanned_notes`](../../src/notes.rs) identity key.

**Related hazard:** `mark_note_spent_by_spend_metadata` / `tag_spent_in_txid_by_identity` matching the same triple — with twins, “first match wins” can mark the **wrong** twin spent. Disambiguate with **`rho` / nullifier**.

**Not the bug:** Ironwood migrate creating a new wallet seed; white-screen CSS; fee markets; “ZEC left to a hardcoded address in code.”

Migrate pays **the same FVK**, Ironwood **internal** receiver (`Scope::Internal`, diversifier 0) — same wallet, new pool.

---

## Fix

1. **Merge by nullifier** (fallback: `note_bytes` if nullifier empty). Never collapse on value alone.
2. **Spend identity:** if multiple equal-value candidates, require `rho` match; refuse to guess.
3. **Test:** `wallet_sync::tests::merge_keeps_equal_value_zip318_twin_notes`.
4. **Recovery:** `nozy sync --start-height 3428650 --to-tip` with the fixed binary (watchers on **old** binaries can overwrite a good `notes.json` — stop them first).

---

## Article angle (suggested narrative)

**Working title:** *We thought users spent half their ZEC. It was our sync merge.*

**Arc:**

1. **Hook** — Operator sees ~$21 → ~$10 during Ironwood week. No send. No inbound on any other wallet. Assistant insists otherwise.
2. **Conflict** — Shielded chain doesn’t show a cleartext payee; easy to invent an “external send” story. User refuses it.
3. **Turn** — Reconstruct split outputs: `2M+2M+480k+fee` and `200k+200k+20k+fee`. Wallet only stored one of each equal pair.
4. **Payoff** — One-line bug, fix, balance restored, then a real turnstile lands in Ironwood with a txid.
5. **Trust close** — Builders owe users the benefit of the doubt when numbers move without a signed send history. AI copilots must too.

**Pull quote:**

> The fee was 0.0006 ZEC. The “missing” 0.022 ZEC was still ours — deleted by a merge key that assumed one note per value per transaction.

---

## Evidence checklist (for editors)

| Item | Evidence |
|------|----------|
| Split txs | `9b3b2f819a3fef8a…` (h 3428651), `e0e233e8119da732…` (h 3428662) |
| Orchard `valueBalanceZat` | `60000` (= fee shape), not ~2.2M leaving the pool |
| No send log for those txids | `sent_transactions.json` |
| Collapse pattern | 9 notes / 4,470,000 zat → 7 notes / 2,270,000 zat |
| Recovery | Re-scan → 24 notes / 4,470,000 zat unspent before turnstile |
| First Ironwood turnstile | `ea2fa4e64a5ca3f588dea58f38feb2a72a8d4e30292ac012d983e23bde7048fd` (confirmed) |
| Post-turnstile snapshot (example) | Orchard ~270,000 zat · Ironwood ~1,960,000 zat |

---

## What we still owe

- Land the merge/nullifier fix on `master` / release notes (if not already in a PR).
- Finish second ZIP 318 transfer (0.002) if still pending on the schedule.
- Never restart migrate watchers against a **stale** `nozy.exe` while repairing `notes.json`.
- **Hardening landed:** sync **refuses to persist** if any scanned nullifier is missing after merge; `nozy notes-doctor` audits equal-value twin groups; sync prints when ZIP 318 twin groups are preserved.

---

## Closing

Ironwood migration is hard enough without the wallet lying about balances. This incident was a **cache identity bug** dressed up as a spend. The operator’s refusal to accept a false narrative is what forced the correct diagnosis. That is how it should work — human with keys first, confident stories second.
