# Privacy Guarantees

NozyWallet provides the same level of privacy as Monero, with better performance.

## Privacy by Default

NozyWallet **enforces** privacy - you cannot send a transparent transaction. This makes it functionally equivalent to Monero in terms of privacy guarantees.

## How Orchard Privacy Works

### 1. Zero-Knowledge Proofs (zkSNARKs)

Orchard uses zero-knowledge proofs to hide:
- **Sender identity**: Who sent the transaction
- **Receiver identity**: Who received the transaction  
- **Transaction amount**: How much was sent
- **Transaction linkability**: Transactions cannot be linked together

### 2. Action Description Encryption (ADE)

Similar to Monero's RingCT, Orchard encrypts:
- Recipient addresses (one-time addresses)
- Transaction amounts
- Memo fields (if used)

### 3. Nullifiers

Like Monero's key images, Orchard uses nullifiers to:
- Prevent double-spending
- Maintain privacy (doesn't reveal which note was spent)
- Ensure security without compromising anonymity

## Comparison with Monero

| Privacy Feature | Monero | NozyWallet (Orchard) |
|----------------|--------|---------------------|
| **Default Privacy** | ✅ Yes | ✅ Yes (enforced) |
| **Sender Hidden** | ✅ Ring Signatures | ✅ zkSNARKs |
| **Receiver Hidden** | ✅ Stealth Addresses | ✅ One-time Addresses |
| **Amount Hidden** | ✅ RingCT | ✅ Encrypted |
| **Untraceable** | ✅ Yes | ✅ Yes |
| **Fungible** | ✅ Yes | ✅ Yes |

## Why NozyWallet is Like Monero

### 1. Privacy is Mandatory

- Monero: All transactions are private (no transparent option)
- NozyWallet: All transactions are private (transparent addresses blocked)

### 2. Untraceable Transactions

- Monero: Ring signatures hide the sender
- NozyWallet: zkSNARKs hide the sender

### 3. Fungible Currency

- Monero: No blacklisted coins possible
- NozyWallet: All shielded coins are fungible

### 4. Complete Transaction Privacy

- Monero: Sender, receiver, and amount are hidden
- NozyWallet: Sender, receiver, and amount are hidden

## Advantages Over Monero

### 1. Faster Block Times

- Zcash: ~75 seconds per block
- Monero: ~2 minutes per block
- **Result**: Faster transaction confirmations

### 2. Lower Fees

- Zcash transaction fees are typically lower
- Especially for larger transactions
- **Result**: More cost-effective

### 3. Better Scalability

- zkSNARKs are more efficient than Ring Signatures
- Smaller transaction sizes
- **Result**: Better network performance

## Privacy Enforcement

NozyWallet enforces privacy at multiple levels:

1. **Address Validation**: Blocks transparent addresses (t1)
2. **Transaction Building**: Only Orchard shielded transactions
3. **Wallet Design**: No transparent address generation

You **cannot** accidentally compromise your privacy with NozyWallet.

## Trust and Verification

Orchard privacy is:
- **Cryptographically proven** (not probabilistic like Ring Signatures)
- **Mathematically sound** (zero-knowledge proofs)
- **Audited** (by security researchers)

## Conclusion

NozyWallet provides **Monero-level privacy** with:
- Same privacy guarantees
- Better performance
- Faster transactions
- Lower fees

If you want Monero's privacy but need Zcash's speed - NozyWallet is for you.
