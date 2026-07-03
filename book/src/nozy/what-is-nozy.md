# What is Nozy?

***

NozyWallet is a privacy-first Zcash wallet built for the Orchard protocol. It focuses on making every transaction private by default and removing the choice—or the mistake—of using transparent addresses.

#### In one sentence

NozyWallet is a Zcash wallet designed around **your own Zebrad node**, with shielded-only transactions and no transparent support—Monero-level privacy on the ledger, with a stack you can verify instead of a vendor you must trust.

***

#### Why it exists

Most Zcash wallets support both transparent and shielded transactions. That flexibility means users can accidentally leak information on the chain. Nozy removes that risk: you cannot send or receive on transparent addresses.

But **ledger privacy and infrastructure privacy are not the same thing.** Orchard hides sender, receiver, and amount from the public blockchain. A third-party node operator can still learn *when* you sync, *when* you broadcast, and often *which IP* asked—unless you control the node yourself. Nozy is built for people who want both: **cryptographic privacy on-chain** and **minimal trust off-chain.**

***

#### Two layers of privacy

| Layer | What it protects | How Nozy helps |
|-------|------------------|----------------|
| **On-chain (Orchard)** | Sender, receiver, amount on the public ledger | Shielded-only; zero-knowledge proofs; no transparent addresses |
| **Infrastructure (your node)** | Who you are when you sync, scan, and broadcast | Zebrad-first design; local witness derivation; you choose the RPC endpoint |

Orchard gives you the first layer. **Running your own Zebrad node** is how you take the second layer seriously.

***

#### Why running your own node matters

When you use someone else’s RPC—an exchange, a hosted wallet backend, a random public endpoint—they do not need your seed to learn a lot about you. They typically see:

- Your **IP address** (or VPN exit) when you connect
- **When** you sync and **when** you broadcast transactions
- **Chain queries** tied to your session (block ranges, treestate fetches, mempool traffic)

That is not the same as reading your Orchard notes—they usually cannot decrypt shielded amounts from the chain alone. It **is** metadata that can correlate your activity, throttle you, log you, or feed analytics. In the worst case, a malicious RPC could try to serve a wrong chain tip or censor your broadcasts.

**Your own Zebrad node** changes the trust model:

1. **You verify the chain** — You are not asking a company “what is the current history?” You run consensus software and validate blocks yourself.
2. **You shrink the observer** — Sync and broadcast metadata stay between your wallet and **your** machine (or your VPS), not a stranger’s logs.
3. **You align with how Nozy spends** — Orchard witnesses are derived **locally** from treestate your wallet obtains over RPC. With your node, that path does not depend on a hosted service’s honesty for spend readiness. See [Zebra Node Setup](../advanced/zebra-node.md) and the [Zebrad connectivity guide](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md).

**What your own node does *not* magically fix** (so the statement stays true):

- **Ledger privacy** still comes from Orchard math, not from Zebrad alone.
- **Network identity** — Your node still talks to the P2P network; use Tor/VPN if you need IP-level hiding ([Privacy Networks](../privacy-networks/overview.md)).
- **lightwalletd** — If you use a *remote* lightwalletd for compact sync, that service can see which block ranges you request. Prefer local lightwalletd paired with your Zebrad, or accept that trade-off.
- **Recipients, exchanges, and OPSEC** — KYC, address reuse, and device compromise are separate from node choice.

Nozy’s honest claim: **self-hosted Zebrad is the recommended way to get full-stack privacy**—not an optional power-user tweak.

***

#### The Cypherpunk idea

In 1993, Eric Hughes wrote the [Cypherpunk Manifesto](https://www.activism.net/cypherpunk/manifesto.html). It is not about aesthetics; it is about **who must build privacy**:

> *Privacy is necessary for an open society in the electronic age.*  
> *Privacy is the power to selectively reveal oneself to the world.*  
> *We cannot expect governments, corporations, or other large, faceless organizations to grant us privacy out of their beneficence.*  
> *Cypherpunks write code. We know that someone has to write software to defend privacy…*

Nozy sits in that tradition:

- **Code, not permission** — Shielded-by-default wallet logic instead of “please don’t log my transparent send.”
- **Selective revelation** — You choose who sees your balance (you); the chain does not broadcast amounts or addresses in the clear.
- **Skepticism of benevolent intermediaries** — Keys stay on your device; the node you run (or explicitly trust) replaces an opaque hosted backend.

We are not relics of the 1990s—Orchard and Zebrad are modern—but the **threat model is the same**: if you want privacy in an open society, you build systems that do not require trusting strangers with your financial metadata.

More in [Manifesto](manifesto.md) and [Philosophy](philosophy.md).

***

#### What you get

- **Shielded-only** — Every send and receive uses Orchard; no transparent t-addresses.
- **Zebrad-first** — Built for JSON-RPC to **your** full node; local Orchard witness derivation (not “trust the server’s witness”).
- **Self-custodial** — Mnemonic and keys stay on your device; no custodian can move your funds.
- **Verifiable stack** — Open source wallet + open source node; you can audit what runs, not just what marketing claims.
- **Multi-surface** — Desktop (Tauri), CLI, api-server companion, and mobile FFI—one Rust core, same privacy stance.

Optional compact sync via lightwalletd (Zeaking) exists for extension and fast catch-up; **Zebrad remains the trust anchor** for tip, broadcast, and treestate.

***

Next: [Purpose](purpose.md) — what Nozy is for.
