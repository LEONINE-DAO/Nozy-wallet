# Glossary of Terms

## A

### Address
A string of characters used to receive cryptocurrency. In Zcash, addresses can be transparent (public) or shielded (private). NozyWallet only uses shielded Orchard addresses for complete privacy.

### Argon2
A key derivation function used for password hashing in NozyWallet. Argon2 is designed to be memory-hard and resistant to both GPU and custom hardware attacks.

## B

### BIP39 (Bitcoin Improvement Proposal 39)
The standard for mnemonic seed phrases. NozyWallet uses 24-word BIP39 mnemonics to generate and recover wallets.

### BIP32 (Bitcoin Improvement Proposal 32)
The standard for hierarchical deterministic (HD) wallets. Allows generating multiple keys from a single seed.

### Block
A collection of transactions that have been validated and added to the blockchain. Zcash blocks are created approximately every 75 seconds.

### Blockchain
A distributed ledger that records all transactions in blocks linked together cryptographically. Zcash uses a blockchain similar to Bitcoin but with privacy enhancements.

## C

### Commitment
A cryptographic commitment that hides the value and recipient of a transaction while still allowing verification. In Zcash, commitments are used in shielded transactions.

### Confirmation
The process of a transaction being included in a block and added to the blockchain. Each subsequent block adds another confirmation.

## D

### Decryption
The process of converting encrypted data back into readable form. In Zcash, notes are encrypted and can only be decrypted by the recipient using their viewing key.

## E

### Encryption
The process of converting readable data into an unreadable format. In Zcash, transaction notes are encrypted so only the recipient can decrypt them.

## F

### Fungibility
The property of money where all units are interchangeable and indistinguishable. Shielded Zcash (ZEC) is fungible because transaction history is hidden.

## G

### Groth16
A zero-knowledge proof system used in Zcash. NozyWallet uses Groth16 proofs for transaction privacy.

## H

### HD Wallet (Hierarchical Deterministic Wallet)
A wallet that can generate multiple addresses from a single seed phrase. All addresses are derived deterministically, making backup and recovery easier.

## I

### I2P (Invisible Internet Project)
A privacy network that provides anonymous communication. NozyWallet can be configured to route transactions through I2P for additional privacy.

## K

### Key Derivation
The process of generating cryptographic keys from a master seed or password. NozyWallet uses BIP32 for hierarchical key derivation.

## M

### **MOE (Medium of Exchange)**
One of the three primary functions of money. MOE refers to money's ability to be used as an intermediary in trade to avoid the inconveniences of a barter system. Zcash functions as a medium of exchange, allowing users to buy goods and services privately.

### Mnemonic Phrase
A sequence of words (typically 12, 18, or 24 words) that can be used to recover a wallet. NozyWallet uses 24-word BIP39 mnemonic phrases.

### Memo
An optional text field in Zcash transactions. Memos in shielded transactions are encrypted and can only be read by the recipient.

## N

### Note
A unit of value in Zcash's shielded pools. Notes contain encrypted information about the amount, recipient, and memo of a transaction.

### Nullifier
A cryptographic value that proves a note has been spent without revealing which note was spent. Prevents double-spending while maintaining privacy.

### NU 6.1 (Network Upgrade 6.1)
The latest Zcash network upgrade, activated at block 3,146,400 (November 23, 2025). NozyWallet is fully compatible with NU 6.1 and protocol version 170140.

## O

### Orchard
Zcash's newest and most efficient shielded pool. Orchard provides better performance and scalability than previous pools (Sapling, Sprout). NozyWallet uses exclusively Orchard addresses.

## P

### Privacy
The ability to keep financial information confidential. NozyWallet enforces privacy by default by only supporting shielded transactions.

### Proving Parameters
Large parameter files needed to generate zero-knowledge proofs for Zcash transactions. NozyWallet manages the download and storage of these parameters automatically.

## S

### Shielded Transaction
A Zcash transaction that hides the sender, receiver, and amount using zero-knowledge proofs. All NozyWallet transactions are shielded.

### **SOV (Store of Value)**
One of the three primary functions of money. SOV refers to money's ability to maintain its purchasing power over time. Zcash, like Bitcoin, can function as a store of value due to its fixed supply cap and scarcity. With privacy guarantees, Zcash may offer superior store of value properties as it cannot be "tainted" or blacklisted.

### Seed Phrase
See "Mnemonic Phrase"

### Spending Key
A private key that allows you to spend notes. Must be kept secret and secure.

## T

### Tor (The Onion Router)
A privacy network that routes internet traffic through multiple encrypted layers. NozyWallet can be configured to route transactions through Tor for additional anonymity.

### Transaction
A transfer of value from one address to another. In Zcash, transactions can be transparent or shielded. NozyWallet only supports shielded transactions.

### Transparent Transaction
A Zcash transaction where the sender, receiver, and amount are visible on the blockchain. NozyWallet does **not** support transparent transactions to ensure complete privacy.

## U

### **UOA (Unit of Account)**
One of the three primary functions of money. UOA refers to money's use as a standard measure for valuing goods, services, and assets. ZEC (Zcash) can function as a unit of account, with prices denominated in ZEC. This is particularly useful for private commerce where transactions are hidden but still need a standard valuation.

### Unified Address
A Zcash address format that can contain multiple address types. NozyWallet extracts Orchard addresses from unified addresses.

## V

### Viewing Key
A key that allows viewing incoming transactions but not spending them. Useful for monitoring balances without the ability to send funds.

## Z

### ZEC
The ticker symbol for Zcash, the cryptocurrency. 1 ZEC = 100,000,000 zatoshis (the smallest unit).

### Zcash
A privacy-focused cryptocurrency that uses zero-knowledge proofs to hide transaction details. NozyWallet is a wallet for Zcash.

### Zero-Knowledge Proof
A cryptographic proof that allows you to prove you know something without revealing what it is. Zcash uses zero-knowledge proofs (specifically Groth16) to prove transactions are valid without revealing amounts or addresses.

### Zatoshis
The smallest unit of Zcash. 1 ZEC = 100,000,000 zatoshis (10^8). Named after Zcash founder Zooko Wilcox-O'Hearn.

## Economic Functions of Money

Money traditionally serves three primary functions, and Zcash (ZEC) can fulfill all three:

### **SOV (Store of Value)**
Zcash maintains purchasing power over time. With a fixed supply cap and privacy guarantees that prevent blacklisting, ZEC serves as an effective store of value.

### **MOE (Medium of Exchange)**
Zcash enables private transactions for goods and services. Shielded transactions make ZEC an ideal medium of exchange for privacy-conscious commerce.

### **UOA (Unit of Account)**
Zcash can be used as a standard unit for pricing and valuing goods, services, and assets. Private transactions can still be denominated in ZEC as a standard measure.

## Privacy-Specific Terms

### Shielded Pool
A collection of notes that use the same privacy protocol (Orchard, Sapling, or Sprout). NozyWallet uses the Orchard pool exclusively.

### Note Commitment Tree
A Merkle tree structure that commits to all notes in a shielded pool. Allows verification without revealing specific notes.

### Value Commitment
A cryptographic commitment that hides the amount in a transaction while allowing verification that amounts balance.
