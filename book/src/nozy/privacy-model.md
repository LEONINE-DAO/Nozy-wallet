# Privacy model

***

Nozy’s privacy has **two parts**: what the **chain** hides, and what **your infrastructure** might still leak.

***

#### What Orchard hides (on-chain)

- **Sender** — Your address and identity are not visible on the chain.
- **Receiver** — The recipient’s address is hidden.
- **Amount** — The value of the transaction is hidden.
- **Linkability** — Observers cannot tie transactions together from ledger data alone.

Zero-knowledge proofs let the network verify validity without learning who sent, who received, or how much.

***

#### What a third-party node can still see (off-chain)

Even with perfect Orchard math, a **remote RPC operator** is not blind to *you as a user*. They may log:

| Signal | Risk |
|--------|------|
| Your IP / connection times | Correlation, geolocation, deanonymization |
| Sync and scan patterns | When your wallet is active; approximate wallet use |
| Broadcast timing | When you spend; mempool association |
| RPC errors / retries | Fingerprinting client behavior |

That is why the privacy model treats **your own Zebrad** as part of the product—not an advanced appendix. Hosted RPC is a convenience trade-off, not maximum privacy.

***

#### How Nozy keeps ledger privacy strong

- **Shielded-only** — No transparent addresses; no accidental public sends.
- **Orchard-only** — Current shielded pool; no legacy transparent-first flows.
- **Local cryptography** — Decryption, scanning, and proving on your device; keys are not sent to a server for spending.
- **Local witnesses** — Orchard spend witnesses are derived on the wallet using treestate from RPC—not served as a trusted “witness API” by the node. Your node supplies **facts about the chain**; your wallet supplies **secrets**.

***

#### How you complete the model (operator checklist)

For the statement “I have true privacy” to hold in practice—not just on paper:

1. **Run Zebrad you control** (home, VPS, or WSL you operate). Point Nozy at it: [Zebra Node Setup](../advanced/zebra-node.md).
2. **Verify connectivity** — `nozy test-zebra` before sync or send.
3. **Sync to tip** before spending — stale witnesses block send and increase risk.
4. **Prefer local lightwalletd** if you use compact sync; avoid remote LWD you do not trust.
5. **Network layer** — Tor/VPN if hiding your IP from the P2P/RPC path matters ([Privacy Networks](../privacy-networks/overview.md)).
6. **Address hygiene** — New receive addresses when linkability matters.
7. **Backup mnemonic offline** — Privacy is meaningless if you lose funds or leak the seed.

***

#### What Nozy does not promise

- Invisibility from nation-state adversaries without broader OPSEC.
- Privacy if you paste your seed into a website or use a compromised machine.
- Protection from a malicious **recipient** who knows you paid them.
- Anonymity if you voluntarily KYC at an exchange and withdraw to a known identity.

The model is: **strong ledger privacy by design, plus infrastructure privacy when you run your own node and practice basic hygiene.**

***

#### Cypherpunk framing

Privacy is *selective revelation* ([Cypherpunk Manifesto](https://www.activism.net/cypherpunk/manifesto.html)). Orchard handles revelation to the network (minimal). You choose revelation to people (recipient). Your node choice controls revelation to **operators**. Nozy removes the “transparent by mistake” path; **you** remove the “someone else’s server watches me” path.

***

For more detail, see [Absolute Privacy](../features/absolute-privacy.md), [Security best practices](../security/best-practices.md), and [Zebrad connectivity](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md).

***

Next: [Manifesto](manifesto.md) — principles in brief — or [Getting started](../getting-started/installation.md).
