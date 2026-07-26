# Ironwood breaks a privacy assumption Zcash users were taught to trust

**NozyWallet · LEONINE-DAO · July 2026**  
**Status:** Draft for site / forum / X long-form — edit before publishing  
**Related:** [Shielded Labs — Security issues in migrating Orchard → Ironwood](https://docs.google.com/document/d/1z4Aj7tO34RKk0SXZYkNXtswxdBXKbR_IJ_Xw5EJljkc/edit) · [SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md](SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md) · [IRONWOOD_WALLET_READINESS.md](IRONWOOD_WALLET_READINESS.md)

---

## Suggested titles (pick one)

1. Ironwood breaks a privacy assumption Zcash users were taught to trust  
2. When migration itself becomes the leak: Orchard → Ironwood and wallet responsibility  
3. NozyWallet on Ironwood: notifying users is not enough — migration must be honest about risk  

---

## Article body (paste below)

Ironwood is going to activate at **block height 3,428,143** on **July 28**. At that point, **all Orchard transactions will be frozen** besides transactions that migrate to Ironwood.

That sentence is easy to read and hard to live with.

For years, Zcash users have been told a simple story: inside a shielded pool, amounts stay private; your light wallet syncs compact blocks; your node or lightwalletd does not learn what you sent or how much you hold. That story is still true for *ordinary* shielded activity **within** a pool.

Ironwood migration is not ordinary shielded activity.

### What changes at activation

After NU6.3 / Ironwood activates:

- New Orchard outputs are rejected. The Orchard pool is sealed.
- Value that remains in Orchard can move only by **exiting through a turnstile** into Ironwood.
- Until a user migrates, their Orchard balance may look “there” in a wallet while being **temporarily unavailable** for normal sends.

This is intentional consensus design. Turnstiles exist so the public can account for how much ZEC entered and left each pool. Sealing Orchard bounds the circulating supply story after the Orchard vulnerability remediation. Those goals matter.

They also create a **product and privacy event** that wallets cannot paper over with a quiet UI update.

### Why this is a major departure from typical privacy expectations

Within Orchard (or Ironwood), a shielded spend does not publish the amount on the public ledger as a clear turnstile crossing. Migration does.

Two facts follow, and both are permanent enough that users deserve plain language:

1. **The migrated amount is revealed on the public blockchain** because of the turnstile. That is a feature of pool-crossing accounting, not a wallet bug.
2. **That amount can be linked to the user’s network identity** — especially their IP address — if migration is broadcast over clearnet to a lightwalletd or remote node. The indexer or peer that receives the submission can join “this IP / session” to “this on-chain pool-crossing amount.”

Under normal shielded operation, lightwalletds and peers do not see senders, recipients, or amounts. Migration raises the stakes: the wire metadata is no longer “someone synced”; it can become “someone with this IP moved *this much* ZEC out of Orchard.”

That is not how most users mentally model Zcash privacy. Pretending otherwise would be a trust failure.

Shielded Labs (Zooko Wilcox and Taylor Hornby) spelled out the threat model and defenses in detail here:

[Security issues in migrating user funds from Orchard to Ironwood](https://docs.google.com/document/d/1z4Aj7tO34RKk0SXZYkNXtswxdBXKbR_IJ_Xw5EJljkc/edit)

Their primary defense is blunt and correct: **strengthen network-level privacy** (appropriately configured Nym or Tor) before migration, so approximate balance is not linked to identity. Their secondary defense is mixing concurrent migrations among cohorts — amount and timing choices that collide with other honest users instead of fingerprinting a single wallet.

Wallets that only ship a “Migrate all now” button over clearnet are optimizing for convenience while failing the privacy story Zcash is supposed to sell.

### What wallet developers should do

The ecosystem ask to wallets is not ambiguous. At minimum:

#### 1. Notify users that funds may become temporarily unavailable

Before activation — and again when activation is detected — the wallet UI should say, in words a non-developer can understand:

- Orchard spends will freeze except migration.
- Balances may remain visible while send is blocked until migration completes.
- Waiting until after activation without a plan is a usability trap.

Silent breakage (“why won’t Send work?”) is worse than an amber banner.

#### 2. Offer migration — with factual risk warnings

Users need a path to move funds. They also need honesty:

- Turnstile **amounts are public**.
- Clearnet broadcast can link those amounts to **IP / lightwalletd session**.
- One-shot migration of a full balance can enable linkage attacks against tagged history (KYC trails, known unshields, subset-sum style analysis). See the motivating story in the Shielded Labs document.

“Migrate” must never be presented as a privacy-preserving equivalent of a normal shielded send. Consent should be explicit. Clearnet should be discouraged, not pre-selected.

#### 3. Implement a privacy-preserving migration path

Notification and a warned button are the floor. The real work is shipping a migration design that matches the research:

- Network privacy first (local full node where possible; otherwise Nym / Tor for broadcast — and ideally for sync metadata too).
- Canonical / bucketed amounts that collide across users (Shielded Labs Appendix A uses `{1,2,5}×10^k`; ZIP 318 discusses related scheduling).
- Timing and cover so lonely turnstiles are not the default.
- No “finish” with unique one-off turnstile sizes for dust; residual below the floor should be abandoned or accumulated, not fingerprinted.

The detailed algorithm and threat model live in the Shielded Labs document linked above. Wallets should treat that as required reading, not optional blog flavor.

### What NozyWallet is doing

NozyWallet is Orchard-first. Ironwood does not change that posture; it moves the shielded default forward. Our product decision is that migration is a **privacy operation**, not a balance transfer with a spinner.

Concretely:

1. **Notify** — Activation notices in CLI, desktop, and mobile status surfaces: height **3,428,143**, target **2026-07-28**, Orchard freeze except migration, temporary unavailability for normal sends.
2. **Offer migrate with warnings** — Plan → Split → Migrate → Broadcast, with factual copy that turnstile amounts are public and clearnet can link them to IP/session. Broadcast is gated behind safer-migration network privacy (local Zebrad, detected Tor/I2P, Nym mixnet helper, or explicit user attestation). Clearnet is an explicit exception, not a quiet default.
3. **Privacy-preserving path** — Active amount ladder aligned with Shielded Labs `{1,2,5}×10^k`; residual below 0.001 ZEC abandoned rather than emitted as unique sizes; shared anchor-bucket scheduling; network-privacy assessment before submit. Memoryless randomized timing and consolidation rounds from Appendix A remain on the roadmap so cover traffic gets stronger, not weaker, after activation.

We would rather ship a slightly stricter migration UX that is honest enough to recommend than a fast clearnet migrate that “works” and teaches users the wrong lesson about Zcash.

### Closing

Ironwood is necessary for supply soundness after a serious Orchard-class failure. It is also a forced moment where **pool-crossing leaks what in-pool privacy hid**.

Users will judge wallets by whether we told them early, warned them factually, and built migration that respects network identity and on-chain fingerprinting — not by whether we buried a migrate button under Settings.

If you are a wallet developer still catching up: start with the three asks above, then read the Shielded Labs document end to end before you ship anything that spends Orchard into Ironwood on mainnet.

Activation does not wait for polish. Neither should user notices.

---

## Optional short pull-quotes (for X / headers)

- “Migration is not a normal shielded send. Turnstiles publish amounts; clearnet can attach an IP.”  
- “Notify. Warn. Then ship a privacy-preserving path — in that order.”  
- “We would rather delay a migrate button than teach users the wrong privacy model.”  

## Suggested links to include when posting

- Shielded Labs doc: https://docs.google.com/document/d/1z4Aj7tO34RKk0SXZYkNXtswxdBXKbR_IJ_Xw5EJljkc/edit  
- Forum thread: https://forum.zcashcommunity.com/t/ironwood-verifying-the-soundness-of-zcash-s-circulating-supply/56044  
- ZIP 318 draft: https://github.com/zcash/zips/pull/1317  
- Nozy readiness notes: `docs/reference/IRONWOOD_WALLET_READINESS.md`  
