# Absolute Privacy

NozyWallet enforces **complete privacy by default**. This means every transaction is automatically private - you cannot accidentally send a transparent transaction that would reveal your financial information.

## How Privacy Works

### Shielded Transactions Only

Unlike other Zcash wallets that support both transparent and shielded transactions, NozyWallet **only supports shielded transactions**. This ensures:

- **No transparent addresses** - You cannot receive or send from transparent addresses
- **All transactions are private** - Every transaction uses zero-knowledge proofs
- **No privacy mistakes** - You cannot accidentally compromise your privacy

### Zero-Knowledge Proofs

All NozyWallet transactions use Zcash's **Orchard protocol**, which provides:

- **Sender privacy**: Your address is hidden
- **Receiver privacy**: Recipient address is hidden
- **Amount privacy**: Transaction amounts are hidden
- **Cryptographic proofs**: Privacy is mathematically guaranteed, not just obfuscated

### Untraceable Transactions

Every transaction is completely untraceable:

- **No transaction graph** - Cannot link transactions together
- **No address reuse** - New addresses for every transaction
- **No metadata leakage** - No information exposed on blockchain

## Comparison with Other Wallets

### vs. Standard Zcash Wallets

| Feature | Standard Zcash Wallet | NozyWallet |
|---------|----------------------|------------|
| Transparent transactions | ✅ Supported | ❌ Not supported |
| Shielded transactions | ✅ Supported | ✅ Only option |
| Privacy by default | ❌ User must choose | ✅ Always private |
| Privacy mistakes | ⚠️ Possible | ✅ Impossible |

### vs. Monero

| Feature | Monero | NozyWallet |
|---------|--------|------------|
| Privacy level | ✅ High | ✅ Equivalent |
| Block time | ~2 minutes | ~75 seconds |
| Transaction fees | Medium | Lower |
| Transaction size | Larger | Smaller |
| Protocol | Ring signatures | Zero-knowledge proofs |

## Privacy Guarantees

When you use NozyWallet, you get these guarantees:

### 1. Sender Privacy

- Your address is never revealed on the blockchain
- Cannot determine who sent a transaction
- Cannot link your address to transactions

### 2. Receiver Privacy

- Recipient address is hidden
- Cannot determine who received funds
- Only the recipient can see incoming transactions

### 3. Amount Privacy

- Transaction amounts are encrypted
- Only sender and receiver know the amount
- Cannot determine value from blockchain

### 4. Fungibility

- All coins are identical
- No coin history tracking
- No blacklisted or tainted coins
- Every coin is as good as any other

## Best Practices

To maintain maximum privacy:

1. **Use unique addresses** - Generate a new address for each transaction
2. **Don't reuse addresses** - Address reuse can reduce privacy
3. **Run your own node** - Use your own Zebra node for maximum privacy
4. **Keep software updated** - Security updates protect your privacy
5. **Backup securely** - Protect your mnemonic phrase offline

## Technical Details

### Orchard Protocol

NozyWallet uses Zcash's **Orchard** shielded pool, which provides:

- **Better performance** - Faster proofs than Sapling
- **Better scalability** - More efficient than previous pools
- **Future-proof** - Active development and improvements
- **NU 6.1 support** - Latest protocol features

### Zero-Knowledge Proofs

Zero-knowledge proofs allow you to:

- Prove you have the right to spend funds without revealing your identity
- Prove transaction validity without revealing amounts
- Maintain privacy while ensuring security

## Frequently Asked Questions

**Q: Can I send transparent transactions?**  
A: No. NozyWallet only supports shielded transactions to ensure complete privacy.

**Q: Is my privacy guaranteed?**  
A: Yes, as long as you follow best practices (unique addresses, secure backups, etc.).

**Q: How does this compare to Monero?**  
A: Privacy levels are equivalent, but Zcash offers faster block times and lower fees.

**Q: What if I need transparent transactions?**  
A: For transparent transactions, you would need to use a different wallet. However, transparent transactions compromise privacy.

## Learn More

- [Security Best Practices](security/best-practices.md)
- [Network Configuration](advanced/network-config.md)
- [Privacy Network Options](advanced/privacy-networks.md) (if implemented)
