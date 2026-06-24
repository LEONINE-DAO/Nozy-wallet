# NozyWallet White Paper

_Generated from NozyWallet_Whitepaper.docx — open the .docx in Word/LibreOffice for formatted version._






NozyWallet

White Paper
Architecture, Phased Development, and Lessons from Mainnet Operation


LEONINE DAO
Version 2.3.6.5 (Teriyaki Hot)
June 2026

github.com/LEONINE-DAO/Nozy-wallet


# Executive Summary

NozyWallet is a self-custodial, shielded-first Zcash wallet built on Orchard. It is a wallet and companion services—not a consensus node. Users run Zebrad for JSON-RPC (broadcast, chain tip, treestate) and lightwalletd for compact block sync; the wallet derives Orchard witnesses locally, computes ZIP-317 fees client-side, and builds ZIP-225 v5 transactions with Halo 2 proofs entirely on the user's device.

The product ships today as a mainnet-validated CLI (v2.3.6.x), localhost api-server companion, and in-development extension, desktop, and mobile surfaces—all sharing one Rust core (nozy) and the Zeaking compact-sync crate. NozyWallet participates in the Shielded Labs dynamic-fee pilot: client-side standard fees, optional four-fold priority multiplier, five-block transaction expiry after the mempool build height (~six minutes at mainnet block times), and speed-up rebuilds after expired unmined transactions.

Mainnet operator testing in June 2026 on a WSL Zebrad stack demonstrated two critical lessons. First, pre-broadcast consensus expiry (-25) on slow VPS hardware is a build-clock problem solved by late chain-tip refresh and automatic rebuild—not by stretching pilot expiry to fifteen blocks. Second, send latency on operator hardware is dominated by witness freshness and Orchard proving; syncing to tip before send reduced end-to-end time to approximately three and a half minutes with successful broadcast (TXIDs 5a03fbd1… and 902cf006…). A witness lag guard rejects sends more than fifty blocks behind tip in under one tenth of a second, preventing multi-minute catch-up mid-send.

This white paper documents architecture decisions, phased delivery, integration challenges with Zebrad and Orchard, trade-offs, security posture, and lessons learned—intended for operators, Shielded Labs pilot reviewers, grant readers, and contributors.


# 1. Architecture Decisions

Each decision below follows an architecture decision record (ADR) pattern: context, decision, and consequences. Together they explain why NozyWallet is Zebrad-native, Orchard-only, and pilot-aligned without requiring zcashd or node-side witness APIs.

### 1.1 Zebrad-only stack (no zcashd)

Context: Zcash infrastructure is consolidating on Zebra for full nodes; operators and Shielded Labs pilots target Zebrad + lightwalletd. The wallet must not embed consensus logic.

Decision: Use Zebrad JSON-RPC for broadcast, block fetch, and treestate; use lightwalletd gRPC for compact sync via zeaking::lwd. No zcashd dependency in this repository.

Consequences: Zebrad does not implement estimatefee—fees must be client-side (ZIP-317). Zebra does not serve spend-ready Orchard witnesses—the wallet must derive and persist them locally.

### 1.2 One Rust core, multiple surfaces

Context: Fee policy, expiry encoding, witness catch-up, and broadcast retry must behave identically across CLI, api-server, desktop, and extension companion paths.

Decision: Centralize wallet logic in the nozy crate; expose thin surfaces. Share build_and_broadcast_send_transaction across send entry points.

Consequences: WASM extension builds from a separate Cargo.toml excluded from the root workspace. Surface parity requires discipline—drift caused field bugs (BUG-2026-001–003).

### 1.3 Local Orchard witness derivation

Context: Shielded spends require Merkle paths and anchors verified against chain state. Zebra's documented JSON-RPC surface does not provide wallet-grade witness lookup for spends.

Decision: Persist incremental Orchard witnesses in notes.json (v2 NoteIndex); catch up via Zebra blocks and verify roots with z_gettreestate. Parallel getblock batches (10 per round) when lag is bounded.

Consequences: Stale witnesses cause multi-minute sends on operator stacks. Send-readiness policy: reject if witness lag exceeds fifty blocks; require sync-to-tip first.

### 1.4 Client-side ZIP-317 fees (dynamic-fee pilot)

Context: Shielded Labs pilot requires standard fee plus optional priority multiplier. Zebrad returns errors for estimatefee and z_getstandardfee.

Decision: Implement fee_policy.rs with ZIP-317 conventional fee for transaction shape; priority = standard × 4. No node fee RPC on the hot path.

Consequences: Ecosystem may later standardize a shared base-fee source; function boundary allows swap. Fee action counting must match Orchard bundle shape (max(spends, outputs), not sum).

### 1.5 Five-block pilot expiry (not fifteen)

Context: Pilot semantics require short mempool expiry so unmined transactions become Expired quickly and users can speed up with a priority rebuild.

Decision: PILOT_EXPIRY_DELTA_BLOCKS = 5; nExpiryHeight = chain_tip + 1 + 5. Slow-host reliability via late tip refresh, prove rebuild (up to 3 attempts), and broadcast retry on expiry -25—not via a fifteen-block delta (~19 minutes to Expired).

Consequences: Fifteen blocks was implemented briefly and reverted (commit a72bc6e8) because it degraded speed-up UX for all users while only masking slow proves on some hosts.

### 1.6 Note index v2 (NoteIndex)

Context: Wallet must load notes quickly, merge sent and received history, and mark notes spent after broadcast without full rescans.

Decision: Serialize notes.json as version-2 NoteIndex with nullifier, height, and address maps; atomic write via temp file rename. Legacy array format migrates on load.

Consequences: All code paths must use NoteIndex load/save. Legacy Vec-only parsers caused post-send mark-spent failures until fixed in June 2026.

## 1.7 System stack diagram

Surfaces (CLI, api-server, extension, desktop) → nozy core + zeaking::lwd → lightwalletd :9067 (compact blocks) and Zebrad :8232 (JSON-RPC). The wallet never replaces either server.


# 2. Phased Development Approach

Delivery is intentionally phased: prove core + infrastructure truth on mainnet before expanding surfaces and ecosystem observability. Gates prevent marking a phase complete without observable criteria.


Phase 5 (June 2026) closed the operator reliability loop: cache-first send (no 50k rescan), merged transaction history, pre-broadcast expiry fix, witness lag guard, proving warm-up, and documented mainnet TXIDs. Phase 6 awaits Shielded Labs alignment on shared pilot metrics.


# 3. Challenges and Responses

## 3.1 Zebrad integration


## 3.2 Orchard shielded send pipeline

Shielded sends involve two clocks that must not be conflated. The build clock covers witness fetch, bundle construction, Halo 2 proving, signing, and broadcast. The mempool expiry clock starts only after successful broadcast and governs pilot speed-up.


## 3.3 Sync protocols


## 3.4 Surface parity



# 4. Trade-offs

The following matrix summarizes deliberate product and engineering choices. Each row reflects a decision we rejected an alternative for documented reasons.



# 5. Security and Privacy Considerations

NozyWallet treats all wallet code as high impact. The following practices are implemented today. This document does not claim a completed third-party security audit.


Privacy properties follow Orchard: sender, receiver, and amount are shielded in Orchard transactions. The wallet enforces shielded-first policy at the product layer so users cannot accidentally send via transparent addresses.


# 6. Mainnet Field Evidence (June 2026)

Recorded on operator hardware: Windows host, Zebrad in WSL (JSON-RPC 172.20.199.206:8232), nozy CLI release build. Amounts were dust (0.0001 ZEC) for regression testing.


Full TXIDs for successful June 2026 sends:

- 5a03fbd19547f9499182d78c88791eeb4eaab32e5d158b69ec8ffdc6068d2612

- 902cf006efdeef3f15fed4312f8a15fcb1162f52495098c3bffb4acbe3cde4e5

Timing model: total send time ≈ witness_catchup + proving_setup + halo2_prove + sign + broadcast. Synced wallet (lag ≤50 blocks) observed ~200 s end-to-end on this stack. Proving warm-up: ~2.1 s cold, negligible when cached.


# 7. Dynamic-Fee Pilot Alignment


Paper-ready summary: NozyWallet participates in the Shielded Labs dynamic-fee pilot with client-side ZIP-317 fees, an optional four-fold priority multiplier, and a five-block transaction expiry after the mempool build height. VPS testing showed Orchard proving can span multiple blocks between construction and broadcast; we address this through late chain-tip refresh, automatic rebuild, and broadcast retry while deliberately preserving the five-block delta rather than extending it to fifteen blocks.


# 8. Lessons Learned

1. Two clocks on shielded sends: build-time expiry vs mempool expiry. The pilot measures the second; operator VPS bugs often hit the first.

2. Wallet ≠ node: witnesses and fee policy are wallet responsibilities on Zebrad—not node bugs when RPC is missing.

3. Sync-before-send is product policy: witness lag guard at fifty blocks prevented seven-plus minute send hangs.

4. Keep pilot knobs stable: fix runtime (rebuild, warm prove) before changing nExpiryHeight policy.

5. Operator stacks need first-class testing: WSL Zebrad + Windows CLI matches real users, not laptop-only CI.

6. Cache format migrations matter: v2 NoteIndex vs legacy array caused subtle post-send bugs.

7. Surface parity is expensive: api-server, CLI, and extension must share one send pipeline.

8. Evidence wins trust: TXIDs and timings in public docs outperform ad-hoc claims.


# 9. Conclusion

NozyWallet demonstrates that a shielded-first Orchard wallet can operate on modern Zebrad and lightwalletd infrastructure with client-side fees, short pilot expiry, and local witness derivation—without zcashd and without lengthening mempool expiry to absorb slow proving. Mainnet evidence in June 2026 validates broadcast success and operator-send latency when witnesses are fresh.

Continued work focuses on surface parity, Shielded Labs pilot metrics, Zeaking observatory indexing, and business features (ZNS, Sell mode) on a pure Zcash foundation. Contributors, operator feedback, and formal review before disclosure features ship remain essential.

# References

- LEONINE-DAO/Nozy-wallet — github.com/LEONINE-DAO/Nozy-wallet

- docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md

- docs/reference/PILOT_EXPIRY_PROVING_LATENCY.md

- docs/reference/WHITEPAPER_OUTLINE.md

- ZEBRAD_SHIELDED_SEND_LIMIT.md

- Zebra — github.com/ZcashFoundation/zebra

- lightwalletd — github.com/zcash/lightwalletd

- ZIP-316 Unified addresses, ZIP-317 Conventional fees, ZIP-225 Orchard transactions

# Appendix A: Bug registry summary (2026-06)


# Appendix B: Glossary


— End of White Paper —
