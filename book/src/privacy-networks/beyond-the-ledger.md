# Lesson: Privacy besides the ledger

**Audience:** Anyone who already knows Zcash has a shielded pool and wants the rest of the story.  
**Time:** ~15 minutes.  
**Goal:** Be able to say, honestly, what Orchard/Ironwood hide — and what they do **not**.

***

## Learning goals

After this lesson you should be able to:

1. Split Zcash privacy into **ledger** vs **network / wallet**.
2. Name what a remote node operator can still see.
3. Rank the real mitigations (own node first; Tor/Nym/VPN second; hygiene always).
4. Avoid the slogan “Nym integrated” / “completely untraceable” when talking about Nozy.

***

## 1. Two layers, not one slogan

Zcash privacy is **two jobs**:

| Layer | What it hides | What it does not hide |
|-------|----------------|------------------------|
| **Ledger** (Sapling / Orchard / Ironwood) | Sender, receiver, amount on the public chain | That *your computer* talked to a node |
| **Beside the ledger** | Your IP, timing, scan pattern, “this host just submitted a tx” | On-chain math (already done by proofs) |

Zero-knowledge proofs let the network **verify** a spend without learning who paid whom. They do **not** hide the TCP connection that carried `sendrawtransaction`.

Nozy’s product stance: **shielded-first on the ledger**, plus **your own Zebrad** (or a privacy network) so an operator is not watching you. See [Privacy model](../nozy/privacy-model.md).

***

## 2. What the ledger actually does

On a **fully shielded** Orchard or Ironwood send:

- Your address is not printed on the chain.
- The recipient’s address is not printed on the chain.
- The payment amount is not printed on the chain.
- Observers cannot reconstruct a Bitcoin-style transaction graph from those fields alone.

That is the cryptographic half. Nozy also **rejects transparent `t1` addresses** on user send/receive so you cannot leak by accident.

**Exceptions you should not paper over:**

- **ZIP 318 / Ironwood turnstile** — migrate amounts are **public by design**. The mixnet cannot hide them.
- **Fees** — ZIP-317 still moves a public value (typical Nozy priority send: **0.0004 ZEC** for a 2-action single-note shape).
- **The recipient** — they know you paid them. Selective revelation is the point, not invisibility from the person you chose.

***

## 3. What still leaks off-chain

Even with perfect Orchard/Ironwood proofs, a **remote RPC or lightwalletd** can log:

| Signal | Why it hurts |
|--------|----------------|
| **Your IP** | Ties a house / phone / VPS to a scan or a broadcast |
| **Connect times** | When the wallet is online |
| **Sync range / birthday** | Rough wallet age; unique start heights fingerprint sessions |
| **The instant you submit a raw tx** | Strongest leak: *this IP just spent* |
| **RPC errors and retries** | Fingerprints the client |

The transaction bytes can be fully shielded. The node still knows **your IP submitted those bytes**.

Tor, I2P, Nym, and a consumer VPN hide **the path**. They do **not** hide **the payload**. The destination still sees JSON-RPC: block ranges, `getblock`, `sendrawtransaction`.

***

## 4. How you complete the model

Ranked. Do the first thing that applies.

### A. Run a node you control (best)

Local **Zebrad** or **Zakura** on loopback or your LAN/WSL. Submit never leaves your box. Mixnet stays **idle on purpose** — that is correct, not a bug.

Checklist: [Set up your own node](../examples/own-node.md) · [Zebra node setup](../advanced/zebra-node.md).

```bash
nozy test-zebra
nozy privacy-network send-egress
```

The send-egress command answers: *what hop will the **next submit** use?* It does **not** describe compact sync.

### B. If you cannot run a node — split the hops

Nym’s recommended hybrid is **two different jobs**, not one global “privacy mode”:

```text
Compact sync (lots of bytes)  →  dVPN / Fast mode, or a VPN
Submit / sendraw (tiny)       →  mixnet
Local/LAN Zebrad              →  neither (direct)
```

**Do not** sync and broadcast through the same hosted lightwalletd a minute later.

Consumer [zcash.nym.com](https://zcash.nym.com) is an **OS VPN stopgap** for shielded ZEC holders (Fast mode to sync; Mixnet mode and a **new exit** to send). It is **not** in-app mixnet/dVPN.

Tor and I2P: [Privacy Networks overview](overview.md).

### C. Wallet hygiene (transport cannot do this)

The destination still sees request **content** and **wall-clock arrival**. Nozy’s Ironwood baseline hygiene (on by default) is wallet work:

1. **Start-height obfuscation** — auto-resume rewinds a random overlap and snaps to a checkpoint so exact resume heights are not a session fingerprint.
2. **Randomized migrate-broadcast delay** — real broadcast sleeps 30–300s.
3. **Tip-sync guard** — refuse broadcast if you just caught tip (default 120s).

Also: fresh receive addresses when unlinkability matters; never paste the seed into a website.

***

## 5. Honest Nozy status (so we do not lie)

Use this table when you talk in public.

| Claim | Truth |
|-------|--------|
| Shielded send/receive, no `t1` by design | **Yes** |
| Local Zebrad is the default privacy end-state | **Yes** |
| Send-egress badge (Local / Mixnet / Tor / I2P / Clearnet) | **In the wallet** |
| “Nym integrated” as a product slogan | **No** — do not use it |
| Live remote `sendraw` over mixnet from this operator box | **Not claimed** (needs an exit-reachable public Zebrad) |
| Fresh dVPN Fast-mode sync numbers | **Not claimed** until Nym publishes a current pin |
| Invisibility from a nation-state | **Not promised** |
| Privacy after KYC + withdraw to a known identity | **Broken** by that choice |

Engineering maps (repo): [`NYM_SEND_EGRESS_CASE_BREAKDOWN.md`](../../../docs/reference/NYM_SEND_EGRESS_CASE_BREAKDOWN.md), [`NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md`](../../../docs/reference/NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md).

***

## 6. Self-check

Answer before you tell someone “this wallet is private”:

1. Am I talking about **the chain** or **the operator who served my RPC**?
2. Is my Zebrad **local**, or is a stranger logging my IP?
3. If remote: did I **split** sync vs submit, or did I use one hosted LWD for both in the same minute?
4. Did I just catch tip and broadcast immediately?
5. Did I confuse **OS NymVPN** with **in-app mixnet**?

If (1) is only the chain, you are describing **half** of Zcash privacy.

***

## 7. One sentence to keep

**zk-SNARKs hide the ledger. Your node (or Tor/Nym/VPN + hygiene) hides you from the operator.** Both have to be true before “I have true privacy” is an honest sentence.

***

## Next

- [Privacy model](../nozy/privacy-model.md) — short product statement  
- [Privacy Networks overview](overview.md) — Tor / I2P / Nym hops  
- [Absolute Privacy](../features/absolute-privacy.md) — shielded-only stance  
- [Set up your own node](../examples/own-node.md) — do the strongest layer  
- [Privacy FAQ](../faq/privacy.md)
