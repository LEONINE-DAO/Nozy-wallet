# Performance & Speed

NozyWallet is designed for speed and efficiency, providing fast transaction processing and low fees while maintaining complete privacy.

## Fast Block Times

Zcash's network has a block time of approximately **75 seconds**, which is faster than Monero's ~2 minutes. This means:

- **Faster confirmations** - Your transactions confirm in ~75 seconds
- **Lower wait times** - Less waiting for transaction finality
- **Better user experience** - Quick, responsive wallet operations

## Transaction Efficiency

### Compact Transactions

Orchard transactions are more efficient than Monero transactions:

- **Smaller size** - Orchard transactions are smaller on average
- **Lower fees** - Smaller transactions = lower fees
- **Better scalability** - Network can handle more transactions

### Proof Generation

- **Fast proving** - Modern zero-knowledge proof systems are efficient
- **GPU acceleration** - Optional GPU acceleration for faster proofs
- **Parameter caching** - Proving parameters cached for speed

## Network Performance

### Low Latency

NozyWallet connects directly to Zebra nodes for minimal latency:

- **Direct RPC connection** - No intermediate servers
- **Efficient protocol** - Optimized JSON-RPC calls
- **Connection pooling** - Reuse connections for speed

### Scalability

The Zcash network can handle:

- **High transaction throughput** - Thousands of transactions per block
- **Efficient note scanning** - Fast blockchain scanning
- **Quick wallet sync** - Rapid synchronization for new wallets

## Performance Benchmarks

(Note: Actual performance may vary based on hardware and network conditions)

- **Transaction building**: < 1 second
- **Proof generation**: 1-5 seconds (depends on hardware)
- **Blockchain scan**: ~1000 blocks/second
- **Address generation**: < 100ms
- **Wallet sync**: Varies by blockchain height

## Optimization Tips

### For Faster Performance

1. **Use SSD storage** - Faster disk I/O for wallet operations
2. **Run local node** - Reduce network latency
3. **Enable GPU acceleration** - Faster proof generation (if supported)
4. **Keep software updated** - Performance improvements in updates
5. **Adequate RAM** - More RAM = better performance

### For Lower Fees

- **Wait for low-fee periods** - Fees vary with network congestion
- **Use standard transaction size** - Smaller = lower fees
- **Batch transactions** - Group multiple operations when possible

## Comparison with Other Wallets

### Speed Comparison

| Operation | Standard Zcash | Monero | NozyWallet |
|-----------|---------------|--------|------------|
| Block time | 75 seconds | ~2 minutes | 75 seconds |
| Transaction build | Fast | Fast | Fast |
| Proof generation | Fast | N/A | Fast |
| Wallet sync | Medium | Slow | Fast |

### Fee Comparison

| Wallet Type | Average Fee | Notes |
|-------------|-------------|-------|
| NozyWallet | Low | Shielded transactions |
| Monero | Medium | Larger transaction size |
| Transparent Zcash | Very Low | No privacy |

## Technical Details

### Orchard Protocol Efficiency

The Orchard protocol provides:

- **Compact proofs** - Smaller proof sizes
- **Batch processing** - Multiple operations in one transaction
- **Efficient scanning** - Fast note commitment tree traversal

### Network Optimization

NozyWallet optimizes network usage:

- **Connection reuse** - Persistent RPC connections
- **Batch requests** - Multiple RPC calls in single request
- **Efficient queries** - Minimal data transfer

## Frequently Asked Questions

**Q: How fast are transactions?**  
A: Transactions typically confirm in 1-2 blocks (~75-150 seconds).

**Q: Are fees high?**  
A: No, Zcash fees are generally lower than Monero due to smaller transaction sizes.

**Q: Does performance degrade over time?**  
A: No, performance remains consistent as the blockchain grows.

**Q: Can I speed up transactions?**  
A: Transaction speed depends on block time, which is fixed. You cannot pay more for faster blocks.

## Learn More

- [Network Configuration](advanced/network-config.md)
- [Zebra Node Setup](advanced/zebra-node.md)
- [Proving Parameters](advanced/proving-parameters.md)
