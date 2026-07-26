# Why Ironwood? (Orchard → NU6.3)

This page is for newcomers who see “Orchard” and “Ironwood” in NozyWallet and wonder what changed—and why the wallet cares.

***

## Short version

1. **Orchard** was Zcash’s modern shielded pool. Nozy shipped as an Orchard-first, shielded-only wallet.
2. In **2026**, Shielded Labs security researcher **Taylor Hornby** found a **soundness bug** in Orchard’s zero-knowledge circuit. In the worst case it could have allowed **undetectable counterfeit ZEC** inside the pool.
3. The circuit was **patched** in an emergency network upgrade. That stopped new abuse of the bug.
4. Because Orchard is **private**, nobody can prove from the public ledger alone that counterfeit notes were *never* minted while the bug was live. Supply soundness needed a stronger answer.
5. **Ironwood (NU6.3)** seals the old Orchard pool, opens a **new** shielded pool with the corrected circuit, and moves value through a **turnstile** so the circulating supply can be verified again.
6. **Nozy follows the chain.** After Ironwood activates, normal sends use Ironwood; Orchard balances migrate with Plan → Split → Migrate → Broadcast—not “Orchard forever.”

***

## What Taylor found

Orchard transactions hide sender, receiver, and amount using a **zk-SNARK** proof. The network accepts a transaction if the proof verifies. That only works if proofs are **sound**: a dishonest prover must not be able to convince the network of a false statement (for example, that a transaction balances when it does not).

Taylor Hornby, doing security research funded by **Shielded Labs**, discovered a flaw in Orchard’s circuit—the math the proof is supposed to enforce. Under the right conditions, an attacker could produce a proof the network would accept even though the underlying notes did not balance correctly. That is a **counterfeiting** risk *inside* the shielded pool: from the outside, fake notes look like real ones.

Important context for newcomers:

- The bug lived in a highly reviewed system for years before it was found.
- Ecosystem teams coordinated an **emergency fix** so the vulnerable circuit path could no longer be used going forward.
- Public messaging has generally been that exploitation is **believed unlikely**—but belief is not the same as **independent verification** of historical supply inside a private pool.

For primary sources, start with Shielded Labs’ [Ironwood overview](https://shieldedlabs.net/ironwood/) and [circulating-supply writeup](https://shieldedlabs.net/ironwood-verifying-the-soundness-of-zcashs-circulating-supply/), and the community forum thread [Ironwood: Verifying the Soundness of Zcash’s Circulating Supply](https://forum.zcashcommunity.com/t/ironwood-verifying-the-soundness-of-zcash-s-circulating-supply/56044).

***

## Why a patch alone was not enough

Patching the circuit stops **new** counterfeiting via that bug. It does **not** automatically restore the property users want next:

> “Anyone running a node can verify that no more ZEC is circulating than should be.”

In a transparent ledger, supply is visible. In Orchard, amounts are hidden. If a soundness bug *could* have minted notes before the fix, those notes would not announce themselves on-chain. So the ecosystem’s answer is not only “fix the math,” but also:

- **Seal** the old Orchard pool (no new ordinary Orchard activity after activation).
- Stand up **Ironwood**—a new pool using the **corrected** circuit.
- Allow value to leave Orchard only through a **turnstile** that accounts for how much can exit versus how much legitimately entered.

That design restores **verifiable supply soundness** without requiring every user to wait for a full migration before the network can make strong claims again.

***

## What Ironwood means for users

After **NU6.3 / Ironwood** activates (mainnet target **height 3,428,143**, **2026-07-28**):

| Before (typical Orchard use) | After Ironwood activation |
|------------------------------|---------------------------|
| Normal shielded sends inside Orchard | Orchard is **sealed** for ordinary sends |
| Balance spends like any other note | Remaining Orchard value moves only via **migration** (turnstile) into Ironwood |
| Supply story trusted via pool rules | Users can reason about supply with the sealed pool + turnstile + new pool |

Migration is **not** the same as a normal private payment:

- Turnstile crossings expose **amounts** on the public chain (by design, for accounting).
- Broadcasting a migration over clearnet can link that amount to **network identity** (IP / session). Shielded Labs (Zooko Wilcox and Taylor Hornby) documented this in *Security issues in migrating user funds from Orchard to Ironwood*.

Nozy treats migration as a **privacy operation**: notify, warn, then Plan / Split / Migrate / Broadcast with safer network guidance—not a silent “upgrade balance” button. See also [Privacy model](../nozy/privacy-model.md).

***

## Why NozyWallet changed

Nozy was built **Orchard-first and shielded-only**. That product stance does not go away—it **moves forward** with the active shielded pool.

If Nozy stayed Orchard-only after Ironwood:

- Users could not make normal payments once Orchard is sealed.
- Nozy would lag the network’s supply-integrity story.
- Holders of Orchard notes would have no first-class migrate path in the wallet they trust.

So Nozy’s change is deliberate:

1. **Scan and hold** both pools while migration is in progress.
2. **Route new sends** to Ironwood once the network (and wallet builder path) are active.
3. **Ship migrate tooling** in CLI Lite and Desktop (and companion API surfaces) so Orchard notes can move safely and explicitly.
4. Keep the old promise: **no transparent `t1` sends**—privacy by default, still.

Operator and readiness detail lives in the repo: [`docs/reference/IRONWOOD_WALLET_READINESS.md`](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/docs/reference/IRONWOOD_WALLET_READINESS.md) and the [Ironwood mainnet week runbook](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/docs/reference/IRONWOOD_MAINNET_WEEK.md).

***

## Timeline newcomers can remember

```text
Orchard era     →  Taylor finds Orchard circuit soundness bug
                →  Emergency network upgrade patches the circuit
                →  Ironwood (NU6.3) proposed: seal Orchard, new pool, turnstile
                →  Wallets (including Nozy) add migrate + Ironwood send
Ironwood era    →  Post-activation: live value in Ironwood; Orchard exits via migrate
```

***

## FAQ

**Did Orchard “fail” as a privacy system?**  
No. The bug was about **proof soundness / supply integrity**, not “everyone can see your amounts.” Privacy of parties and amounts inside a healthy pool is a different question from “can someone mint fake notes the chain cannot detect?”

**Was counterfeit ZEC definitely created?**  
Public ecosystem communication has generally said exploitation is **unlikely**. Ironwood exists so you do **not** have to take that on trust forever: the sealed pool + turnstile restore verifiable bounds.

**Do I lose my Orchard funds?**  
They remain yours as notes in the old pool until you migrate. After activation, **normal Orchard send is blocked**; use Nozy’s Ironwood migrate flow (or equivalent) to move value into the new pool.

**Is Ironwood a different coin?**  
No. It is still **ZEC**, in a new shielded pool with corrected circuit rules.

***

## Further reading

- Shielded Labs — [Ironwood](https://shieldedlabs.net/ironwood/)
- Shielded Labs — [Verifying the soundness of Zcash’s circulating supply](https://shieldedlabs.net/ironwood-verifying-the-soundness-of-zcashs-circulating-supply/)
- Forum — [Ironwood thread](https://forum.zcashcommunity.com/t/ironwood-verifying-the-soundness-of-zcash-s-circulating-supply/56044)
- Project Tachyon — [Auditing the Orchard pool’s supply](https://tachyon.z.cash/blog/auditing-orchard-supply/)
- Nozy — [What is Nozy?](../nozy/what-is-nozy.md) · [Privacy model](../nozy/privacy-model.md)
