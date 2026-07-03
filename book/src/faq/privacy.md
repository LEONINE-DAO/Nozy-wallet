# Privacy Questions

## Are my transactions private?

Orchard shielded transactions hide amount, sender, and receiver on the public ledger. NozyWallet does not add transparent leakage by design.

## Does NozyWallet phone home?

No telemetry that exfiltrates addresses, amounts, or seed by default project policy. Your RPC node still sees query/broadcast metadata.

## Can I use a public RPC?

You can point `zebra_url` at a trusted VPS. Operator sees IP and timing — use Tor/VPN or own node for stronger metadata privacy.

## Address reuse?

Reusing one unified address works but may reduce unlinkability. Generate fresh receive addresses when privacy matters.

## Viewing keys?

Advanced / future documentation — default UI is spend/receive focused, not full viewing-key export.

## Zcash vs Monero privacy model?

Nozy targets **Monero-like default privacy** using Zcash Orchard — see [Privacy model](../nozy/privacy-model.md).

## Related

- [Security Questions](security.md)
- [Absolute Privacy](../features/absolute-privacy.md)
