# Privacy model

***

Nozy’s privacy comes from Zcash’s Orchard protocol and from how the wallet is designed around it.

#### What is hidden

- **Sender** — Your address and identity are not visible on the chain.
- **Receiver** — The recipient’s address is hidden.
- **Amount** — The value of the transaction is hidden.
- **Linkability** — You can’t tie one transaction to another from the chain alone.

All of this is enforced by zero-knowledge proofs: the network checks that the transaction is valid without learning who sent, who received, or how much.

***

#### How Nozy keeps it that way

- **Shielded-only** — No transparent addresses. You can’t accidentally send or receive in the clear.
- **Orchard-only** — Only the current shielded pool; no legacy transparent or Sapling-only flows in this wallet.
- **Your keys, your device** — Decryption and scanning happen on your machine; the server only sees encrypted or public data and signed transactions you choose to broadcast.
- **No built-in leakage** — No optional “show on explorer” or “share address” that would tie your balance to an identity.

So the privacy model is: **default to the strongest Zcash gives (Orchard), and don’t offer weaker options.**

***

#### What you should do

- Back up your seed phrase securely and offline.
- Prefer generating a new address per receive when you can.
- If you need maximum opsec, run your own Zebra node and point the wallet at it.

For more detail, see [Absolute Privacy](../features/absolute-privacy.md) and [Security best practices](../security/best-practices.md).

***

Next: [Getting started](../getting-started/installation.md) — install and create your first wallet.
