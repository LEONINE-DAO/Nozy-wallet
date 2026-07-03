# Philosophy

***

Nozy is built on a few principles that shape every feature and default.

#### Privacy is not optional

If the wallet can do something that leaks information, that path is removed. Transparent addresses are not supported. There is no “use shielded when you remember”—shielded is all the wallet does. The goal is to make the private path the only path.

#### The server should learn as little as possible

The wallet is designed so **keys and decryption stay on your device**. When you use a **remote** RPC or api-server, that operator can still see connection metadata—timestamps, IPs, broadcast timing—even if they cannot read your Orchard notes. Running **your own Zebrad** is how you align infrastructure with the same minimal-trust goal.

For chain data and broadcast, the remote party should not need your password, your keys, or your balance. The client does decryption, scanning, and proving; a dumb RPC only serves blocks and accepts signed transactions you choose to submit. That keeps the threat model small and keeps you in control.

See [Privacy model](privacy-model.md) for the full on-chain vs off-chain split.

#### Complexity should stay under the hood

Orchard, zero-knowledge proofs, and sync are complex. The interface should not be. Nozy aims for clear flows: create/restore wallet, see balance, send, receive, backup. Advanced options exist for those who need them, but the default path is simple.

#### Open and auditable

NozyWallet is open-source. You can see how it handles keys, how it talks to Zebra, and how it builds transactions. Documentation and code are there so you can verify what the wallet does instead of trusting a slogan.

***

Next: [Privacy model](privacy-model.md) — how privacy works in practice.
