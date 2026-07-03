# Address Management

NozyWallet uses **Orchard unified addresses** for receiving. Transparent addresses are not supported.

## Generate address

### Desktop

**Receive** tab → copy `u1…` address.

### CLI

```bash
nozy receive
```

Each wallet can derive multiple addresses (account / index) — default flow uses the primary receiving address.

## Address book (Contacts)

Desktop **Contacts** tab:

1. Add **name** + **shielded address** (`u1…` or testnet equivalent).
2. Use **Pick contact** on Send tab to fill recipient.

CLI:

```bash
nozy address-book add --name Vendor --address u1…
nozy address-book list
```

Only shielded addresses are accepted — transparent `t1` rejected.

## Unified addresses

- **Mainnet:** starts with `u1`
- **Testnet:** starts with `utest1`

Always verify network matches your wallet configuration.

## Labels and reuse

- Reusing one address is OK for privacy trade-offs; fresh addresses per receive improves unlinkability.
- Contacts are **local metadata** — not on-chain labels.

## Related

- [Receiving ZEC](receiving-zec.md)
- [Sending ZEC](sending-zec.md)
