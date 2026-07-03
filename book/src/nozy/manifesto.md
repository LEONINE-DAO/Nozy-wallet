# NozyWallet Manifesto

***

**NozyWallet** is a privacy-first Zcash wallet: shielded Orchard only, self-custodial keys, and a stack meant to run against **infrastructure you control**—starting with your own Zebrad node.

***

## Cypherpunks write code

The [Cypherpunk Manifesto](https://www.activism.net/cypherpunk/manifesto.html) (Eric Hughes, 1993) argued that privacy in the electronic age will not be handed down by governments or corporations. It must be **built**:

- *Privacy is the power to selectively reveal oneself to the world.*
- *We must defend our own privacy if we expect to have any.*
- *Cypherpunks write code.*

NozyWallet is an attempt to live up to that—not as nostalgia, but as engineering:

| Manifesto idea | Nozy practice |
|----------------|---------------|
| Selective revelation | Orchard hides public ledger details; you reveal payment only to the recipient |
| Don’t trust benevolent intermediaries | Self-custodial keys; run your own Zebrad instead of opaque hosted RPC |
| Write the software | Open-source wallet + documented node path; no “privacy toggle” that leaks by default |

“Cypherpunk” (with a **y**) is the movement for cryptography and privacy tools. It is not the same as the “cyberpunk” sci-fi aesthetic—though both distrust centralized control.

***

## Freedom is privacy

On a transparent blockchain, the world sees your graph. On Orchard, the math hides the graph—but **your RPC provider can still watch you use the wallet** unless you run your own node.

Nozy’s stance:

1. **On-chain:** No transparent escape hatch. Shielded-only.
2. **Off-chain:** Zebrad-first. Learn to operate your node; verify with `nozy test-zebra`.
3. **Culture:** Read the manifesto. Understand what you’re defending. Then backup your seed.

> *If I say something, I want it heard only by those for whom I intend it.*  
> — Cypherpunk Manifesto

Your transaction intent should not be a free feed for every RPC operator on the internet.

***

**[View the NozyWallet codebase on GitHub →](https://github.com/LEONINE-DAO/Nozy-wallet)**

Next: [Getting started](../getting-started/installation.md) — or [What is Nozy?](what-is-nozy.md) for the full node + privacy model.
