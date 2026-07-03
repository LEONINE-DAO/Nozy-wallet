# Security Best Practices

## Custody

- You control keys — no recovery if mnemonic is lost.
- Verify receive addresses on a second device before large deposits.

## Mnemonic

- Write at creation; never store in cloud notes or email.
- Two offline copies in separate locations.
- Test restore once before mainnet funds.

## Password

- Unique strong password for wallet encryption.
- Lock wallet when stepping away (desktop).

## Node trust

- Prefer **your own Zebrad** or a VPS you operate.
- RPC provider sees broadcast timing and IP metadata.

## Updates

- Build from tagged releases or verify release signatures when published.
- Run `cargo audit` if building from source.

## Phishing

- NozyWallet will never ask for your seed in chat or email.
- Download only from official GitHub releases.

## Shielded-only default

Orchard-only design avoids accidental transparent-address mistakes.

## Hardware wallet (Keystone)

- Use **mainnet** on both NozyWallet and Keystone.
- Verify amount and recipient on the **Keystone screen** before signing.
- UFVK export is read-only but privacy-sensitive — pair only with your device.
- Guide: [Keystone Hardware Wallet](keystone-hardware-wallet.md).

## Related

- [Private Key Management](key-management.md)
- [Keystone Hardware Wallet](keystone-hardware-wallet.md)
- [Backup Strategies](backup-strategies.md)
- [Security Audits](audits.md)
- [Security Features](../features/security.md)
