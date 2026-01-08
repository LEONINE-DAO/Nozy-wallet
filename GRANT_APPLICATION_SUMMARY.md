# NozyWallet - ZCG Grant Application Summary

## 🎯 Core Value Proposition: Load-Bearing Elements

**NozyWallet's grant application centers on two load-bearing elements that anchor core Zcash invariants:**

1. **Deterministic Orchard Note Detection (NU6.1)** - Ensures wallet reliability and consistency
2. **Indexing Layer with Correctness Proof** - Enables fast queries without constant node access

These are not optional features—they are the **foundation** that enables everything else. Without deterministic scanning, wallets cannot reliably restore state. Without indexing, user experience degrades with constant network queries.

---

## 🔑 Why These Elements Are Load-Bearing

### Deterministic Behavior: The Foundation of Wallet Reliability

**Problem:** Non-deterministic note scanning leads to:
- Inconsistent wallet balances across restores
- Unreliable transaction history
- User confusion and loss of trust

**Solution:** Deterministic Orchard note detection under NU6.1 ensures:
- ✅ Identical results across wallet restores
- ✅ Consistent state regardless of scan order
- ✅ Reliable transaction history
- ✅ Trustworthy balance calculations

**Impact:** This is a **core Zcash invariant**. Every production wallet must have deterministic behavior. Without it, the wallet cannot be trusted.

### Indexing Layer: The Foundation of User Experience

**Problem:** Constant node queries lead to:
- Slow wallet operations (seconds per query)
- Network dependency for every action
- Poor user experience
- Scalability limitations

**Solution:** Local blockchain indexing enables:
- ✅ Fast queries (10x+ speedup vs direct queries)
- ✅ Offline-capable operations
- ✅ Scalable transaction history
- ✅ Better user experience

**Impact:** This is a **load-bearing element**. It enables fast, responsive wallet operations that users expect. Without it, the wallet feels slow and unreliable.

---

## 📋 Grant Request: Phase 1 Core Zcash Enhancements

### Primary Deliverables (Load-Bearing Elements)

#### 1. Deterministic Orchard Note Detection (NU6.1) ⭐
**Timeline:** 4-6 weeks  
**Budget:** $25K-35K  
**Priority:** HIGHEST

**Deliverables:**
- Deterministic scanning algorithm for NU6.1 (protocol version 170140)
- Test vectors for blocks 3,146,400-3,146,500
- Verification: Identical results across wallet restores
- NU6.1 compliance validation (ZIP 271, ZIP 1016)

**Verification Artifacts:**
- Test vector file: `test_vectors/deterministic_notes_3146400_3146500.json`
- Compliance report: `docs/nu61_compliance_report.md`
- Deterministic test results showing identical outputs

**Why This Matters:**
Deterministic behavior is a **core Zcash invariant**. Without it, wallets cannot be trusted. This deliverable ensures NozyWallet meets the reliability standards required for production use.

#### 2. Indexing Layer with Correctness Proof ⭐
**Timeline:** 6-8 weeks  
**Budget:** $30K-45K  
**Priority:** HIGHEST

**Deliverables:**
- Complete Zeaking indexing system
- Orchard action tracking
- Transaction history caching
- Correctness proof for blocks 3,000,000-3,100,000
- Performance benchmarks (target: >10x speedup)

**Verification Artifacts:**
- Correctness proof document: `docs/indexing_correctness_proof_3000000_3100000.md`
- Performance benchmarks showing speedup vs direct queries
- Index rebuild test results (deterministic behavior)

**Why This Matters:**
Indexing is a **load-bearing element**. It enables fast, responsive wallet operations. Without it, every wallet action requires a network query, leading to poor user experience. This deliverable provides the foundation for scalable wallet operations.

#### 3. Security Hardening (Remove Unsafe Patterns)
**Timeline:** 3-4 weeks  
**Budget:** $15K-25K  
**Priority:** HIGH

**Deliverables:**
- Complete removal of all `unwrap()` calls
- Comprehensive error handling
- Mutex poisoning protection
- Security audit preparation

**Why This Matters:**
Security hardening moves the privacy surface in the right direction. Removing unsafe patterns ensures the wallet is production-ready and secure.

#### 4. Privacy Network Integration (Tor/I2P)
**Timeline:** 3-4 weeks  
**Budget:** $10K-15K  
**Priority:** MEDIUM-HIGH

**Deliverables:**
- Tor proxy support for Zebra node connections
- I2P proxy support
- Privacy network configuration UI
- Network routing test results

**Why This Matters:**
Privacy network routing moves the privacy surface in the right direction. It enables users to connect to Zcash nodes through privacy networks, enhancing privacy.

---

## 🎯 Grant Application Focus

### What We're Asking For

**Phase 1: Core Zcash Enhancements** - $80K-120K

This phase focuses exclusively on the **load-bearing elements** that anchor core Zcash functionality:
- Deterministic Orchard note detection (NU6.1)
- Indexing layer with correctness proof
- Security hardening
- Privacy network integration

### What We're NOT Asking For (Yet)

- Optional cross-chain features (Monero, Secret Network) - These are discretionary
- Mobile/Desktop apps - These come after core is solid
- Advanced features - These build on the foundation

**Why This Approach:**
We're focusing on the **foundation first**. Once deterministic behavior and indexing are proven, everything else builds on this solid base.

---

## 📊 Current Status

### ✅ Production-Ready (Completed)
- **Zcash Core**: 100% production ready
  - HD wallet, address generation, note scanning
  - Transaction building, signing, broadcasting
  - NU 6.1 support, transaction history
  - Complete CLI interface
  - Orchard-only flows (shielded-default)

### ⏳ Grant-Funded Enhancements (Requested)
- Deterministic Orchard note detection (NU6.1)
- Indexing layer with correctness proof
- Security hardening (remove unsafe patterns)
- Privacy network integration (Tor/I2P)

---

## 🔬 Technical Approach

### Deterministic Note Detection

**Algorithm:**
1. Scan blocks in deterministic order (by height)
2. Process Orchard actions in deterministic order
3. Track note commitments consistently
4. Ensure identical results across restores

**Verification:**
- Test vectors with known block ranges
- Multiple restore tests showing identical results
- NU6.1 compliance validation

### Indexing Layer

**Architecture:**
1. Local SQLite database for indexed data
2. Block-by-block indexing with progress tracking
3. Orchard action indexing
4. Transaction history caching
5. Correctness verification against direct queries

**Verification:**
- Correctness proof for fixed block range
- Performance benchmarks
- Index rebuild tests (deterministic behavior)

---

## 📈 Success Metrics

### Phase 1 Success Criteria

- ✅ **Deterministic Note Detection:**
  - Test vectors validate correctness for blocks 3,146,400-3,146,500
  - Identical results across wallet restores
  - NU6.1 compliance verified

- ✅ **Indexing Layer:**
  - Correctness proof completed for blocks 3,000,000-3,100,000
  - Performance benchmarks show >10x speedup
  - Index rebuild produces identical results

- ✅ **Security:**
  - Zero unsafe patterns in production code
  - All error paths properly handled

- ✅ **Privacy Networks:**
  - Tor routing working for Zebra node connections
  - I2P routing working for Zebra node connections

---

## 💰 Budget Breakdown

### Phase 1: Core Zcash Enhancements
**Total: $80K-120K**

| Component | Budget | Priority |
|-----------|--------|----------|
| Deterministic Note Detection (NU6.1) | $25K-35K | ⭐ HIGHEST |
| Indexing Layer with Correctness Proof | $30K-45K | ⭐ HIGHEST |
| Security Hardening | $15K-25K | HIGH |
| Privacy Network Integration | $10K-15K | MEDIUM-HIGH |

**Allocation:**
- Developer salaries: $50K-70K (60-70%)
- Security audit preparation: $15K-25K (15-20%)
- Testing infrastructure: $10K-15K (10-15%)
- Documentation: $5K-10K (5-10%)

---

## 🎓 Why This Grant Matters

### For the Zcash Ecosystem

1. **Reliability:** Deterministic behavior ensures wallets can be trusted
2. **Performance:** Indexing enables fast, responsive wallet operations
3. **Security:** Removing unsafe patterns improves security posture
4. **Privacy:** Privacy network integration enhances user privacy

### For NozyWallet

1. **Foundation:** These elements enable all future features
2. **Production-Ready:** Moves wallet from functional to production-ready
3. **Trust:** Verification artifacts build community trust
4. **Scalability:** Indexing enables handling large transaction histories

---

## 📋 Verification Approach

### Early Traces and Sample Interactions

We will provide verification artifacts for each deliverable:

1. **Deterministic Note Detection:**
   - Test vector file with block range and expected results
   - Multiple restore test results showing identical outputs
   - NU6.1 compliance validation report

2. **Indexing Correctness:**
   - Correctness proof document with comparison results
   - Performance benchmarks showing speedup
   - Index rebuild test results

3. **Security:**
   - Code review report showing zero unsafe patterns
   - Error handling verification

4. **Privacy Networks:**
   - Network routing test results
   - Connection status verification

---

## 🚀 Timeline

### Phase 1: Core Zcash Enhancements (2025 Q1-Q2)

**Q1 2025:**
- Weeks 1-6: Deterministic Orchard note detection (NU6.1)
- Weeks 1-8: Indexing layer development (parallel)
- Weeks 9-12: Security hardening

**Q2 2025:**
- Weeks 1-4: Privacy network integration
- Weeks 5-8: Verification artifacts and documentation
- Weeks 9-12: Testing and refinement

**Total:** 6 months for Phase 1 core enhancements

---

## ✅ Grant Eligibility

### All ZCG Criteria Met

1. **Impact** ✅
   - Load-bearing elements anchor core Zcash invariants
   - Deterministic behavior and indexing are essential for production wallets

2. **Clarity** ✅
   - Clear boundaries between core Zcash and optional features
   - Narrow, verifiable milestones with specific deliverables

3. **Alignment** ✅
   - Focused on Zcash-native components
   - Privacy-first design (Orchard-only, shielded-default)

4. **Deliverability** ✅
   - Narrow, verifiable milestones
   - Specific deliverables with verification artifacts
   - Realistic timeline and budget

5. **Verification** ✅
   - Test vectors for deterministic behavior
   - Correctness proof for indexing
   - Compliance reports for NU6.1
   - Network routing test results

---

## 🎯 Conclusion

**NozyWallet's grant application focuses on the load-bearing elements that anchor core Zcash functionality.**

Deterministic Orchard note detection and the indexing layer are not optional features—they are the **foundation** that enables everything else. By funding these core enhancements, ZCG invests in:

1. **Wallet Reliability** - Deterministic behavior ensures wallets can be trusted
2. **User Experience** - Indexing enables fast, responsive operations
3. **Production Readiness** - Security hardening and privacy networks complete the foundation

Once these load-bearing elements are proven, NozyWallet will have a solid foundation for all future enhancements.

---

**Project:** NozyWallet  
**Grant Request:** Phase 1 Core Zcash Enhancements  
**Budget:** $80K-120K  
**Timeline:** 6 months (2025 Q1-Q2)  
**Status:** Ready for ZCG consideration  
**Focus:** Load-bearing elements (deterministic behavior, indexing)

---

**Last Updated:** Current  
**Next Steps:** Awaiting ZCG evaluation and feedback
