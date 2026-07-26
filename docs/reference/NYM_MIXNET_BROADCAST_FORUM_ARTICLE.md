# Mixnet broadcast for Ironwood: proving IP↔tx is broken (on purpose)

**Draft for Zcash Community Forum / long-form**  
**NozyWallet · LEONINE-DAO · July 2026**  
**Status:** Ready to paste / edit — update evidence table when D2b/D2c live PASS lands  
**Case breakdown:** [NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md](NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md)  
**Companion:** [NYM_ZCASH_HYBRID_FORUM_ARTICLE.md](NYM_ZCASH_HYBRID_FORUM_ARTICLE.md) (hygiene + hybrid overview) · [zcash-sdk.nym.com](https://zcash-sdk.nym.com/)

---

## Suggested title

Breaking IP↔tx on purpose: NozyWallet’s Nym smolmix broadcast path for Ironwood

## Suggested thread

Reply under Ironwood / safer-migration / Nym×Zcash threads, linking Harry & Mark’s wallet guidance.

---

## Body (paste below)

Hi all —

Follow-up to our hygiene note: Nym’s [recommended architecture](https://zcash-sdk.nym.com/recommended/) for native wallets is **hybrid** — bulk sync on dVPN, **broadcasts on the mixnet**. This post is NozyWallet’s engineering status on the **broadcast** half (issue #147).

### The linkage we are trying to break

Shielded notes hide contents. They do **not** hide:

> “This IP just submitted this raw transaction to my lightwalletd / Zebrad.”

For Ironwood turnstiles the stakes are higher: the **amount is public**, so IP↔submit becomes IP↔amount. Local Zebrad (wallet → `127.0.0.1`) already avoids that. Remote stacks need mixnet (or Tor) on **submit**, not only on sync.

### What we shipped

1. **Isolated smolmix helper** (`tools/nym-smolmix-broadcast-spike`)  
   - `--ip-relocate` — clearnet vs mixnet exit IP  
   - `--rpc-probe` — `getblockcount` over mixnet  
   - `--sendraw-stdin` — wallet subprocess path for `sendrawtransaction`  
   - `--dry-reachability` — classify LAN vs exit-reachable **without** wasting a tunnel  
   - `--evidence-json` — structured rows for case-breakdown logs  

2. **Wallet gate** (`nym_mixnet_broadcast`)  
   Opt-in via config/env. Remote URL → spawn helper. **Local/LAN always stays direct** (Case A1).  
   Subprocess on purpose: linking smolmix into the root crate hits a sqlite `links` clash with our compact-sync stack.

3. **Ironwood migrate-broadcast** uses the same egress policy as normal send (Priority 1 mode `NymMixnetBroadcast`).

4. **Operator CLI:** `nozy privacy-network nym-mixnet` prints helper readiness without opening a tunnel.

5. **Baseline hygiene** (separate post / track A) remains on by default — mixnet does not erase destination wall-clock timing.

### Evidence (honest)

| Step | Status |
|------|--------|
| IP relocate (exit ≠ host) | **PASS** (2026-07-11); harness ready to re-run |
| Refuse RFC1918 / loopback as mixnet targets | **PASS** (by design — not a bug) |
| Live `getblockcount` to exit-reachable Zebrad | **Blocked on reachability** — we need an operator-supplied public/testnet RPC |
| Live remote `sendrawtransaction` over mixnet | **Open** — same URL dependency + test tx |
| Product marketing “Nym supported” | **Not yet** |

Important: probing our WSL LAN Zebrad through the mixnet and watching it time out is **not** evidence that smolmix is broken. Public exits cannot route `172.20/16`. We refuse those targets before tunnel build so the failure mode is explicit.

### Ask

If someone operates an **exit-reachable testnet JSON-RPC** suitable for wallet probes (even read-only `getblockcount` first), we can close D2b quickly and then D2c with a disposable testnet send. Prefer coordinating out-of-band rather than pasting long-lived open `sendraw` endpoints in-forum.

We are also happy to take pushback on the subprocess helper vs waiting for an upstream sqlite/`links` resolution to go in-process.

### Links

- Case breakdown: `docs/reference/NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md`  
- Hygiene + hybrid article: `docs/reference/NYM_ZCASH_HYBRID_FORUM_ARTICLE.md`  
- Nym wallet guidance: https://zcash-sdk.nym.com/guidance/  
- Issue: https://github.com/LEONINE-DAO/Nozy-wallet/issues/147  

— NozyWallet / LEONINE-DAO

---

## AI disclosure

Draft assisted by Cursor Agent; human author owns accuracy before posting.
