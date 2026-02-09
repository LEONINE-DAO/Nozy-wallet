# Philosophy

***

Nozy is built on a few principles that shape every feature and default.

#### Privacy is not optional

If the wallet can do something that leaks information, that path is removed. Transparent addresses are not supported. There is no “use shielded when you remember”—you always use shielded, because people be nozy that’s all the wallet does. The goal is to make the private path the only path.

#### The server should learn as little as possible

The API server is there to serve block data and broadcast signed transactions. It does not need your password, your keys, or your balance. The client does the decryption, the scanning, and the proving; the server stays dumb. That keeps the threat model small and keeps you in control.

#### Complexity should stay under the hood

Orchard, zero-knowledge proofs, and sync are complex. The interface should not be. Nozy aims for clear flows: create/restore wallet, see balance, send, receive, backup. Advanced options exist for those who need them, but the default path is simple.

#### Open and auditable

NozyWallet is open-source. You can see how it handles keys, how it talks to Zebra, and how it builds transactions. Documentation and code are there so you can verify what the wallet does instead of trusting a slogan.

***

Next: [Privacy model](privacy-model.md) — how privacy works in practice.
