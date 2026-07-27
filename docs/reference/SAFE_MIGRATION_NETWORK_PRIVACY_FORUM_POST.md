# Safer Ironwood migration: three priorities after ZIP 318

**Draft for Zcash Community Forum**  
**NozyWallet · LEONINE-DAO · July 2026**  
**Status:** Ready to paste / edit before posting  
**Implementation:** Started in-repo — see `src/ironwood/network_privacy.rs` + `nozy ironwood preflight` / `broadcast --attest-private-network`

---

## Suggested title

Safer Ironwood migration: IP → shared cover → amount/timing algorithms — NozyWallet priorities

## Suggested thread

Reply under [Ironwood: Verifying the Soundness of Zcash's Circulating Supply](https://forum.zcashcommunity.com/t/ironwood-verifying-the-soundness-of-zcash-s-circulating-supply/56044), or a short standalone post that links that thread and ZIP 318.

---

## Body (paste below)

Hi all —

This is a short product/engineering note from **NozyWallet** on how we are thinking about **safe Orchard → Ironwood migration** at the stage we are at now.

### What we already agree with in ZIP 318

We are building against the wallet migration draft in **[ZIP 318 PR #1317](https://github.com/zcash/zips/pull/1317)** (*Orchard to Ironwood migration*).

That draft is doing the hard on-chain privacy work:

- note splitting into **canonical denominations**
- **scheduled / bucketed** migration transactions (cohorts, shared anchors)
- sync **decoupled** from broadcast
- explicit **user consent**
- and — importantly — a required **network-privacy step** before the schedule is committed (Tor toggle, VPN fallback, IP-correlation disclaimer)

We treat that as the right shape for migration. A one-shot “move everything now” path is not good enough for a privacy-first wallet.

On Nozy’s side, the **CLI Ironwood path** (scan, split, turnstile schedule/prebuild, Ironwood send) is already testnet-validated in **v2.4.0**. Desktop/API migration UX is still being wired. So we are past “can we migrate at all?” and into “can we migrate **safely** in the sense users actually care about?”

For us, safer migration has **three priorities** after the ZIP 318 mechanism itself:

1. **Protect the broadcasting IP** (Nym / Tor / local node)
2. **Coordinate users so migration cover traffic is shared** (real cohorts, not lonely turnstiles)
3. **Get the amounts and timing selection algorithm right** (Zooko-style / ZIP 318 amount–timing research, as in our coordination-migration writeup)

### Priority 1 — IP / network metadata (Nym / Tor / local node)

ZIP 318 correctly notes a residual leak:

> Without network-level privacy, the server or network operator that receives a broadcast can correlate the broadcasting IP address with the on-chain pool-crossing event.

Canonical amounts and shared timing buckets reduce **on-chain** fingerprinting. They do **not** by themselves stop:

- ISP / local network observers seeing wallet egress
- a lightwalletd or submit endpoint logging **IP ↔ migration broadcast**
- timing joins between “this IP synced / opened the wallet” and “this cohort got a turnstile”

If we only ship beautiful turnstiles over clearnet, we have improved the chain graph and still left a track for attackers who watch the wire.

At this stage we are **not** proposing to replace ZIP 318. We are proposing to take its network-privacy step seriously in product:

1. **Prefer a local Zebrad** for desktop when possible  
   Strongest default for Nozy’s architecture: wallet → local JSON-RPC, not a public light server, for sensitive migration traffic.

2. **Offer / require private egress for migration broadcast**  
   Align with ZIP 318’s Tor opt-in — and treat **Nym** and **Tor** as first-class ways to protect the broadcasting IP.  
   Practical v1 today: guidance + a hard warn / gate before migrate-broadcast unless local node is healthy **or** the user attests they are on Nym/Tor (SOCKS detection for Tor/I2P where possible).

   **What we will try first (Nym engineering, after Mark / Harry / Nym guidance — [zcash-sdk.nym.com](https://zcash-sdk.nym.com/)):**  
   - **Hygiene first (landed 2026-07-25):** Start-height obfuscation, randomized migrate-broadcast delay, tip-sync guard — see [NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md](NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md) and forum draft [NYM_ZCASH_HYBRID_FORUM_ARTICLE.md](NYM_ZCASH_HYBRID_FORUM_ARTICLE.md).  
   - **Biggest transport win (next):** Route **all outgoing transactions** (send, Ironwood migrate-broadcast, splits) over Nym so remote lightwalletd / Zebrad cannot link **IP ↔ tx**. Prefer **smolmix** (mixnet) for submit; see [NYM_IP_PRIVACY_CASE_BREAKDOWN.md](NYM_IP_PRIVACY_CASE_BREAKDOWN.md) (living checklist D2a–D2e; IP-relocate **PASS** 2026-07-11) and [issue #147](https://github.com/LEONINE-DAO/Nozy-wallet/issues/147).  
   - **Also in flight:** Nym **2-hop dVPN** (`smol-dvpn`) for **compact block sync** — isolated crate [`tools/nym-dvpn-lwd-spike`](../../tools/nym-dvpn-lwd-spike/) and [issue #146](https://github.com/LEONINE-DAO/Nozy-wallet/issues/146). Recommended hybrid bulk path; **does not** replace broadcast-over-Nym.  
   - **Later:** **mix-fetch** / mix-websocket for the **browser extension**.  

   We are **not** treating system-wide NymVPN attestation as the end state — that is a bridge until in-app smolmix (submit) and smol-dvpn (sync) land.

3. **Do not default “skip network privacy”**  
   ZIP 318 says no network-privacy option may be pre-selected. We agree. For high-risk flows we may go further than “MAY proceed without” and make clearnet an explicit, discouraged exception.

4. **Keep claims honest**  
   Nym/Tor reduce IP↔session linkage; they do not erase KYC history, subset-sum on revealed migration amounts, or a compromised device. Orchard/Ironwood hide note plaintexts; mixnets hide **who talked to the network**.

### Priority 2 — Shared migration cover traffic across users

ZIP 318’s privacy story for pool-crossing amounts is **value collision + cohorts**: many wallets emit the same canonical denominations into the same anchor-height buckets, so one user’s turnstile is hidden among others.

That only works if **other people are actually there**.

A perfectly scheduled, Nym-routed migration that lands alone in an empty bucket is still a lonely fingerprint. So our **second priority** is making cover traffic real:

- **Implement ZIP 318 bucketing faithfully** so Nozy users land in the same network-wide cohorts as other ZIP 318 wallets (shared boundaries, canonical denoms, bounded multiplicity).
- **Avoid “migrate whenever the app is open” behavior** that pulls users out of shared windows and into wallet-specific timing.
- **Explore light coordination** so users (and ideally multiple wallet implementations) can prefer the same upcoming buckets when privacy matters — without building a centralized “migration server” that becomes a new metadata honeypot.
- **Be honest about thin cohorts** early after activation: if a bucket is sparse, the wallet should warn, delay to a fuller window, or increase multiplicity carefully within `K_MAX` — not pretend one isolated turnstile is private.

### Priority 3 — Amounts and timing selection algorithm

Even with private egress and other users in the cohort, **which amounts you publish and when** still decides whether an observer can reassemble *you*.

Pool-crossing transfers reveal migrated value on-chain. A naive schedule (full balance, unique leftovers, wallet-specific jitter) hands attackers:

- **subset-sum / amount fingerprinting** against a known prior withdrawal or KYC’d size
- **timing fingerprints** that separate one wallet’s schedule from the cohort
- **whale / residual shapes** that do not collide with anyone else

This is where we want to stay aligned with:

- **ZIP 318** amount-selection and anchor-bucket timing (canonical denominations, shared boundaries, bounded multiplicity)
- **Zooko’s migration amount/timing proposal** and the **coordination-migration writeup** we have been working from — including ideas such as `{1,2,5}×10^k`-style chunking, concurrency / same-decade overlap checks before large rounds, and never one-shotting a full tagged balance

Priority 3 is the **algorithm layer**: given a balance and a threat model, choose a sequence of `(denomination × time bucket)` pairs that maximize collision and cover, not just “empty the Orchard pool somehow.”

Community feedback sharpened three load-bearing points we now treat as first-class:

1. **The ladder must be normative in ZIP 318, not per-wallet.**  
   The anonymity set for a migration is the union of every wallet’s turnstile crossings at the same denomination in the same time window. If Nozy uses `{1,2,5}×10^k` and another wallet uses plain powers of ten, amounts distinguish the cohorts and they never merge — N small anonymity sets instead of one large one. So denomination choice is a **cross-wallet** decision. Nozy currently ships ZIP 318 **power-of-ten** so we collide with other ZIP 318 implementations; we treat `{1,2,5}×10^k` as **planned only if/when the ZIP (or clear ecosystem consensus) makes that ladder normative**.

2. **Schedule shape is part of the fingerprint, not just amount.**  
   Two wallets on the same ladder with different cadences (migrate-on-activation vs spread over weeks) stay separable by temporal distribution. Shared **anchor-bucket boundaries** and shared **cadence rules** (schedule start, multiplicity fill, missed-window behavior) must align too.

3. **Remainder / toxic change is where linkability hides.**  
   Hard invariant we now argue for: **only canonical amounts ever cross the turnstile.** Orchard→Orchard splits reveal no net value (pool balance zero; note amounts stay hidden), so rearrange privately into exact canonical sizes *before* any Orchard→Ironwood crossing. That shrinks the residual to irreducible dust below the smallest denomination. For that dust: **accumulate-and-wait** (roll into the next round until it sums to a canonical unit) — never broadcast a one-off size to “finish.” A leftover sitting in Orchard is a small bounded signal (pool sealing); a unique on-chain amount is permanent. Same structural problem as equal-output CoinJoin toxic change.

4. **Cover / concurrency: mandate what wallets can enforce unilaterally; standardize the estimator.**  
   Cover is **not locally verifiable** without a coordination channel — and that channel is the metadata honeypot we refuse. So MUST/normative should stay limited to mergeable, unilateral invariants: canonical ladder, bucket boundaries, bounded multiplicity, **no one-off turnstile amounts**, no pre-selected clearnet. **Cover-awareness** (warn/delay on a sparse bucket) fits as **SHOULD**. Higher leverage: standardize a **public-chain cover estimator** (recent crossings per denomination per bucket) so wallets converge on the same “is this bucket full enough?” without anyone running a coordinator. Standardize the measurement, recommend the behavior, mandate only locally enforceable invariants.

In product terms for Nozy that means:

- Prefer **collision-prone denominations** over high-entropy custom sizes
- Keep **split separate from turnstile** — hard rule, not a suggestion — so we never invent non-canonical Ironwood crossings to burn dust
- **Accumulate** sub-denomination remainder in Orchard rather than emitting a unique turnstile size
- Align **cadence** with ZIP 318 buckets — no wallet-private “migrate whenever open” jitter that pulls users out of shared windows
- Treat thin-bucket / cover gates as **SHOULD warn/delay**, not a fake local “privacy lock” that pretends we can see other wallets
- Build toward a **shared cover estimator** from public crossings (P2 scaffold today is local-only)
- Do **not** freestyle a `{1,2,5}` ladder alone; converge with ZIP 318 so the ecosystem shares one set

### How the three priorities fit together

| Priority | Layer | What it hides | Failure mode if skipped |
|----------|--------|----------------|-------------------------|
| 1 | Nym / Tor / local node | IP ↔ broadcast session | Observer joins your network identity to the turnstile |
| 2 | Shared cohorts / cover traffic | Your turnstile among other migrations | You migrate alone even if IP is hidden |
| 3 | Amount + timing algorithm | Your balance shape inside the cohort | Unique amounts/timing still fingerprint you among cover |

None replaces the others. **IP protection without cover** still leaves on-chain loneliness. **Cover without a good amount/timing algorithm** still leaves subset-sum and schedule fingerprints. **A perfect algorithm on clearnet** still paints your IP on the wire.

### Why this focus *now*

Mainnet NU6.3 is close. Wallets will migrate real funds under real observation.

- ZIP 318 is the right **on-chain** migration privacy framework.
- **Priority 1** decides whether carefully bucketed turnstiles still paint a target on the user’s **IP**.
- **Priority 2** decides whether those turnstiles actually land in a **shared anonymity set**.
- **Priority 3** decides whether the **amounts and windows** inside that set resist fingerprinting and subset-sum — per Zooko / coordination-migration research, not ad hoc wallet logic.
- Nozy’s next engineering focus, alongside finishing desktop/API Ironwood UX, is therefore: wire **Nym/Tor (and local-node)** for real — **smolmix for all outgoing tx submit first** (biggest IP↔tx win), then **smol-dvpn for compact sync**, then **mix-fetch for the extension** — harden **cross-user cover**, then lock the **amount/timing selection** path — not treat any of these as a docs footnote.

We would rather ship a slightly stricter migration UX that is **safe enough to recommend**, than a fast clearnet migrate button that “works” but fails the privacy goal.

### Asks / discussion

**On IP protection**

- Does the ZIP 318 network-privacy step need stronger language for **desktop full-node** wallets (local node as RECOMMENDED / default) vs light clients (Tor/VPN MUST be offered)?
- Are people treating **NymVPN / mixnet** as an acceptable peer to Tor for that step, or should the ZIP stay Tor-primary with VPN as fallback only?
- For “safer migration,” should wallets **block** clearnet broadcast for migration by default, or only warn?

**On shared cover traffic**

- Should thin-bucket behavior be **SHOULD warn/delay** only (not MUST), so ZIP 318 does not mandate what wallets cannot verify alone?
- Can we standardize a **cover estimator** from public chain data (crossings per denom per bucket) without creating a coordinator / honeypot?
- Keep cover emergent from shared ZIP parameters — no private “migration server.”

**On amounts and timing**

- Should the ZIP 318 denomination ladder be **normative and singular** (power-of-ten *or* `{1,2,5}×10^k`), rather than allowing per-wallet choice that fragments cohorts?
- Should ZIP 318 also norm **cadence** (schedule start relative to activation, bucket fill rules, missed-window behavior) so temporal fingerprints do not separate otherwise-aligned wallets?
- Agree that **only canonical amounts MAY cross the turnstile**, with **accumulate-and-wait** for dust below the smallest denom (never emit one-off sizes)?
- Which equal-output CoinJoin prior art on denomination selection and toxic change should we treat as required reading for turnstile design?

Happy to take feedback. Links:

- ZIP 318 draft: https://github.com/zcash/zips/pull/1317  
- Ironwood forum thread: https://forum.zcashcommunity.com/t/ironwood-verifying-the-soundness-of-zcash-s-circulating-supply/56044  
- Nozy Ironwood readiness notes: `docs/reference/IRONWOOD_WALLET_READINESS.md`  
- Nozy KYC / coordination-migration notes (amount–timing + cover): `docs/reference/KYC_INBOUND_PRIVACY_CASE_BREAKDOWN.md`

— NozyWallet / LEONINE-DAO

---

## Notes for the poster (not for the forum)

- Tone: collaborative with ZIP authors; ZIP 318 already has Tor + value collision/cohorts/amount selection — Nozy is stating product priority order for implementing those layers well.
- Priority order: (1) IP / Nym-Tor, (2) shared cover traffic, (3) amount/timing algorithm (Zooko + coordination writeup). Do not swap unless product priority changes.
- Do **not** ship `{1,2,5}×10^k` in Nozy alone while ZIP 318 is still power-of-ten — that fragments the anonymity set. Switch only with ZIP/ecosystem normative change.
- Do not overclaim Nym, active multi-user coordination, or a specific denom set as consensus unless the community agrees.
- Cover-traffic coordination must not imply a centralized migration coordinator that logs identities.
- If linking the Zooko proposal externally, add the public URL when you have it; the internal writeup is referenced via the KYC/coordination case breakdown for now.
- Optional: attach or link prior Ironwood case-breakdown forum material if already posted.
- AI disclosure: if you used Cursor/AI to draft this, say so briefly in the post or PR style disclosure per project norms.

---

## Reply draft (paste under forum feedback on ladder / schedule / remainder)

Thanks — this is the right place to push.

**On the ladder being normative in ZIP 318**  
We agree. Anonymity is the union of same-denomination crossings in the same time window *across wallets*. If Nozy ships `{1,2,5}×10^k` while others stay on plain powers of ten, the cohorts never merge — you get N small sets instead of one large one. So the ladder shouldn’t be a per-wallet “privacy flavor”; it needs to be **the** ZIP 318 denomination set (or an ecosystem-normative successor), with implementations converging on that, not freelancing.

Today Nozy follows ZIP 318’s **power-of-ten** split/schedule so we land in the same buckets as other ZIP 318 wallets. We’re treating `{1,2,5}×10^k` as a **planned** path only if/when the ZIP (or a clear ecosystem consensus) moves there — otherwise we’d be fragmenting the set on purpose.

**Schedule is part of the fingerprint**  
Also agreed. The set is over `(denomination × time bucket)`, not amount alone. Migrate-on-activation vs spread-over-weeks with the same ladder still separates cohorts by temporal shape. Shared **anchor-bucket boundaries** and shared **cadence rules** (when the schedule starts, how multiplicity fills buckets, what happens on missed windows) have to align too — not only the denomination table.

**Remainder / toxic change**  
This matches what we’ve already hit on testnet: real balances aren’t clean sums of canonical notes; fee dust and leftovers are where linkability hides. Hard rule we want: **only canonical amounts cross the turnstile**; Orchard→Orchard split rearranges privately first; sub-denom dust **accumulates until it forms a canonical unit** — never emit a one-off size to finish.

**CoinJoin prior art**  
The equal-output / toxic-change literature maps cleanly here: anonymity ≈ count of identical outputs in a round; toxic change is the classic fingerprint. We should borrow denomination selection and change-handling lessons rather than reinvent them for turnstiles.

**What we’re taking into the analysis**  
1. Prefer **normative ladder + normative cadence** in ZIP 318 over wallet-local heuristics.  
2. Treat **schedule shape** as first-class in any amount/timing writeup (not a footnote).  
3. Put **remainder policy** (canonical-only turnstile + accumulate-and-wait) next to denomination selection as a required ZIP / wallet requirement.  
4. Keep Priority 1 (IP) and Priority 2 (cover) — cover as SHOULD + shared public estimator, not a coordinator.

Happy to help pressure-test remainder rules against live ZIP 318 schedules if useful.

---

## Reply draft 3 (paste under /6 cover estimator — keep short)

Glad to keep this in-thread so other wallets and ZIP authors can converge on one reading.

Starting definition looks right to us:

- **Cover(D, B)** = count of turnstile crossings of amount **D** in comparable recent buckets **B**
- Crossing ≈ tx with **orchard.valueBalance > 0** and **ironwood.valueBalance < 0**; amount = revealed value
- Pin so wallets agree: **bucket boundaries**, **lookback / recency weighting**, **multiplicity (+ cap)**, and **only exact canonical amounts count** (non-canonical = noise / toxic leftover, not cover)
- Public chain data only — no coordinator, no shielded secrets

We ran that on local Ironwood testnet Zebrad for heights **4134000 → ~4170495** (activation → tip; ~36.5k blocks): **82** Orchard→Ironwood crossings.

One pin already showed up: **fee placement splits “revealed D”**. With `orchard.valueBalanceZat` as D we see **1** exact power-of-ten cover event (1 ZEC). With `|ironwood.valueBalanceZat|` as D we see more canonical dens (1 / 10 / 0.01 ZEC clusters). Same crossings, different Cover(D,B). Details + JSON: `docs/reference/COVER_ESTIMATOR_TESTNET_RESULTS.md`.

Drop your **amount-key choice** + block range (or tally JSON) and we’ll diff counts 1:1. Still open: lookback/recency, multiplicity(+cap).
