#!/usr/bin/env python3
"""Generate NozyWallet white paper (~12–15 pages) as Word document."""

from datetime import date
from pathlib import Path

from docx import Document
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.shared import Inches, Pt, RGBColor

REPO_ROOT = Path(__file__).resolve().parents[1]
OUTPUT_DOCX = REPO_ROOT / "docs" / "NozyWallet_Whitepaper.docx"
OUTPUT_MD = REPO_ROOT / "docs" / "reference" / "NozyWallet_Whitepaper.md"


def set_margins(section, top=1.0, bottom=1.0, left=1.0, right=1.0):
    section.top_margin = Inches(top)
    section.bottom_margin = Inches(bottom)
    section.left_margin = Inches(left)
    section.right_margin = Inches(right)


def add_heading(doc, text, level=1):
    h = doc.add_heading(text, level=level)
    for run in h.runs:
        run.font.color.rgb = RGBColor(0x1A, 0x1A, 0x2E)
    return h


def add_body(doc, text, space_after=6):
    p = doc.add_paragraph()
    run = p.add_run(text)
    run.font.size = Pt(11)
    run.font.name = "Calibri"
    p.paragraph_format.space_after = Pt(space_after)
    p.paragraph_format.line_spacing = 1.15
    return p


def add_bullet(doc, text):
    p = doc.add_paragraph(text, style="List Bullet")
    for run in p.runs:
        run.font.size = Pt(11)
        run.font.name = "Calibri"
    p.paragraph_format.space_after = Pt(3)


def add_adr(doc, title, context, decision, consequences):
    add_heading(doc, title, 3)
    add_body(doc, f"Context: {context}")
    add_body(doc, f"Decision: {decision}")
    add_body(doc, f"Consequences: {consequences}")


def add_table(doc, headers, rows):
    table = doc.add_table(rows=1 + len(rows), cols=len(headers))
    table.style = "Table Grid"
    for i, h in enumerate(headers):
        table.rows[0].cells[i].text = h
    for ri, row in enumerate(rows, 1):
        for ci, cell in enumerate(row):
            table.rows[ri].cells[ci].text = str(cell)
    doc.add_paragraph()


def cover_page(doc):
    for _ in range(5):
        doc.add_paragraph()
    title = doc.add_paragraph()
    title.alignment = WD_ALIGN_PARAGRAPH.CENTER
    r = title.add_run("NozyWallet")
    r.bold = True
    r.font.size = Pt(36)
    r.font.name = "Calibri"
    r.font.color.rgb = RGBColor(0x1A, 0x1A, 0x2E)

    sub = doc.add_paragraph()
    sub.alignment = WD_ALIGN_PARAGRAPH.CENTER
    r2 = sub.add_run(
        "White Paper\n"
        "Architecture, Phased Development, and Lessons from Mainnet Operation"
    )
    r2.font.size = Pt(14)
    r2.font.name = "Calibri"
    r2.italic = True

    doc.add_paragraph()
    meta = doc.add_paragraph()
    meta.alignment = WD_ALIGN_PARAGRAPH.CENTER
    for line in [
        "LEONINE DAO",
        f"Version 2.3.6.5 (Teriyaki Hot)",
        date.today().strftime("%B %Y"),
        "",
        "github.com/LEONINE-DAO/Nozy-wallet",
    ]:
        run = meta.add_run(line + "\n")
        run.font.size = Pt(11)
        run.font.name = "Calibri"
    doc.add_page_break()


def section_executive_summary(doc):
    add_heading(doc, "Executive Summary", 1)
    add_body(
        doc,
        "NozyWallet is a self-custodial, shielded-first Zcash wallet built on Orchard. It is a "
        "wallet and companion services—not a consensus node. Users run Zebrad for JSON-RPC "
        "(broadcast, chain tip, treestate) and lightwalletd for compact block sync; the wallet "
        "derives Orchard witnesses locally, computes ZIP-317 fees client-side, and builds ZIP-225 "
        "v5 transactions with Halo 2 proofs entirely on the user's device.",
    )
    add_body(
        doc,
        "The product ships today as a mainnet-validated CLI (v2.3.6.x), localhost api-server "
        "companion, and in-development extension, desktop, and mobile surfaces—all sharing one "
        "Rust core (nozy) and the Zeaking compact-sync crate. NozyWallet participates in the "
        "Shielded Labs dynamic-fee pilot: client-side standard fees, optional four-fold priority "
        "multiplier, five-block transaction expiry after the mempool build height (~six minutes "
        "at mainnet block times), and speed-up rebuilds after expired unmined transactions.",
    )
    add_body(
        doc,
        "Mainnet operator testing in June 2026 on a WSL Zebrad stack demonstrated two critical "
        "lessons. First, pre-broadcast consensus expiry (-25) on slow VPS hardware is a build-clock "
        "problem solved by late chain-tip refresh and automatic rebuild—not by stretching pilot expiry "
        "to fifteen blocks. Second, send latency on operator hardware is dominated by witness "
        "freshness and Orchard proving; syncing to tip before send reduced end-to-end time to "
        "approximately three and a half minutes with successful broadcast (TXIDs 5a03fbd1… and "
        "902cf006…). A witness lag guard rejects sends more than fifty blocks behind tip in under "
        "one tenth of a second, preventing multi-minute catch-up mid-send.",
    )
    add_body(
        doc,
        "This white paper documents architecture decisions, phased delivery, integration challenges "
        "with Zebrad and Orchard, trade-offs, security posture, and lessons learned—intended for "
        "operators, Shielded Labs pilot reviewers, grant readers, and contributors.",
    )
    doc.add_page_break()


def section_architecture_decisions(doc):
    add_heading(doc, "1. Architecture Decisions", 1)
    add_body(
        doc,
        "Each decision below follows an architecture decision record (ADR) pattern: context, "
        "decision, and consequences. Together they explain why NozyWallet is Zebrad-native, "
        "Orchard-only, and pilot-aligned without requiring zcashd or node-side witness APIs.",
    )

    add_adr(
        doc,
        "1.1 Zebrad-only stack (no zcashd)",
        "Zcash infrastructure is consolidating on Zebra for full nodes; operators and Shielded "
        "Labs pilots target Zebrad + lightwalletd. The wallet must not embed consensus logic.",
        "Use Zebrad JSON-RPC for broadcast, block fetch, and treestate; use lightwalletd gRPC "
        "for compact sync via zeaking::lwd. No zcashd dependency in this repository.",
        "Zebrad does not implement estimatefee—fees must be client-side (ZIP-317). Zebra does not "
        "serve spend-ready Orchard witnesses—the wallet must derive and persist them locally.",
    )

    add_adr(
        doc,
        "1.2 One Rust core, multiple surfaces",
        "Fee policy, expiry encoding, witness catch-up, and broadcast retry must behave identically "
        "across CLI, api-server, desktop, and extension companion paths.",
        "Centralize wallet logic in the nozy crate; expose thin surfaces. Share "
        "build_and_broadcast_send_transaction across send entry points.",
        "WASM extension builds from a separate Cargo.toml excluded from the root workspace. "
        "Surface parity requires discipline—drift caused field bugs (BUG-2026-001–003).",
    )

    add_adr(
        doc,
        "1.3 Local Orchard witness derivation",
        "Shielded spends require Merkle paths and anchors verified against chain state. Zebra's "
        "documented JSON-RPC surface does not provide wallet-grade witness lookup for spends.",
        "Persist incremental Orchard witnesses in notes.json (v2 NoteIndex); catch up via Zebra "
        "blocks and verify roots with z_gettreestate. Parallel getblock batches (10 per round) "
        "when lag is bounded.",
        "Stale witnesses cause multi-minute sends on operator stacks. Send-readiness policy: "
        "reject if witness lag exceeds fifty blocks; require sync-to-tip first.",
    )

    add_adr(
        doc,
        "1.4 Client-side ZIP-317 fees (dynamic-fee pilot)",
        "Shielded Labs pilot requires standard fee plus optional priority multiplier. Zebrad "
        "returns errors for estimatefee and z_getstandardfee.",
        "Implement fee_policy.rs with ZIP-317 conventional fee for transaction shape; priority = "
        "standard × 4. No node fee RPC on the hot path.",
        "Ecosystem may later standardize a shared base-fee source; function boundary allows swap. "
        "Fee action counting must match Orchard bundle shape (max(spends, outputs), not sum).",
    )

    add_adr(
        doc,
        "1.5 Five-block pilot expiry (not fifteen)",
        "Pilot semantics require short mempool expiry so unmined transactions become Expired "
        "quickly and users can speed up with a priority rebuild.",
        "PILOT_EXPIRY_DELTA_BLOCKS = 5; nExpiryHeight = chain_tip + 1 + 5. Slow-host "
        "reliability via late tip refresh, prove rebuild (up to 3 attempts), and broadcast "
        "retry on expiry -25—not via a fifteen-block delta (~19 minutes to Expired).",
        "Fifteen blocks was implemented briefly and reverted (commit a72bc6e8) because it "
        "degraded speed-up UX for all users while only masking slow proves on some hosts.",
    )

    add_adr(
        doc,
        "1.6 Note index v2 (NoteIndex)",
        "Wallet must load notes quickly, merge sent and received history, and mark notes spent "
        "after broadcast without full rescans.",
        "Serialize notes.json as version-2 NoteIndex with nullifier, height, and address maps; "
        "atomic write via temp file rename. Legacy array format migrates on load.",
        "All code paths must use NoteIndex load/save. Legacy Vec-only parsers caused post-send "
        "mark-spent failures until fixed in June 2026.",
    )

    add_heading(doc, "1.7 System stack diagram", 2)
    add_body(
        doc,
        "Surfaces (CLI, api-server, extension, desktop) → nozy core + zeaking::lwd → "
        "lightwalletd :9067 (compact blocks) and Zebrad :8232 (JSON-RPC). The wallet never "
        "replaces either server.",
    )
    doc.add_page_break()


def section_phased_development(doc):
    add_heading(doc, "2. Phased Development Approach", 1)
    add_body(
        doc,
        "Delivery is intentionally phased: prove core + infrastructure truth on mainnet before "
        "expanding surfaces and ecosystem observability. Gates prevent marking a phase complete "
        "without observable criteria.",
    )
    add_table(
        doc,
        ["Phase", "Name", "Shipped (status)", "Gate to next phase"],
        [
            ("0", "Foundation", "HD wallet, Orchard scan, CLI", "Mainnet note scan works"),
            ("1", "Operator stack", "zeaking::lwd, compact SQLite", "Sync to tip on Zebrad+LWD"),
            ("2", "Mainnet send", "Witness pipeline, ZIP-225 v5, broadcast", "Successful mainnet tx"),
            ("3", "NU6.2 + pilot A1", "librustzcash 0.28, 5-block expiry, ZIP-317 fix", "Branch ID + fee shape correct"),
            ("4", "Surfaces + API", "api-server, extension companion, desktop WIP", "Send/sync parity across surfaces"),
            ("5", "Reliability (2026-06)", "BUG-2026-001–011, send-readiness, unified sync", "Mainnet evidence PASS"),
            ("6", "Pilot A2 / observatory", "Planned — Zeaking fee observatory", "Shielded Labs metrics schema"),
            ("7", "Business / web / mobile", "Roadmap — ZNS, Sell mode, web app", "Per ENHANCEMENT_ROADMAP.md"),
        ],
    )
    add_body(
        doc,
        "Phase 5 (June 2026) closed the operator reliability loop: cache-first send (no 50k rescan), "
        "merged transaction history, pre-broadcast expiry fix, witness lag guard, proving warm-up, "
        "and documented mainnet TXIDs. Phase 6 awaits Shielded Labs alignment on shared pilot metrics.",
    )
    doc.add_page_break()


def section_challenges(doc):
    add_heading(doc, "3. Challenges and Responses", 1)

    add_heading(doc, "3.1 Zebrad integration", 2)
    add_table(
        doc,
        ["Challenge", "Response"],
        [
            ("No fee RPC on Zebrad", "Client ZIP-317 in fee_policy.rs"),
            ("No spend witness RPC", "Local incremental witness + block catch-up"),
            ("Tor policy vs VPS direct RPC", "trusted_zebra_urls; structured connect error codes"),
            ("Node behind tip → mempool disabled", "Status UX; sync-to-tip before send"),
            ("NU6.2 branch ID mismatch (-25)", "librustzcash 0.28 bump (PR #58)"),
        ],
    )

    add_heading(doc, "3.2 Orchard shielded send pipeline", 2)
    add_body(
        doc,
        "Shielded sends involve two clocks that must not be conflated. The build clock covers "
        "witness fetch, bundle construction, Halo 2 proving, signing, and broadcast. The mempool "
        "expiry clock starts only after successful broadcast and governs pilot speed-up.",
    )
    add_table(
        doc,
        ["Challenge", "Response"],
        [
            ("Prove latency >> block time", "Late expiry encode + rebuild loop (≤3 attempts)"),
            ("Pre-broadcast -25 (Gilmore)", "BUG-2026-011: refresh tip before nExpiryHeight"),
            ("Anchor / Merkle path errors", "ZebraJsonRpcOrchardWitnessProvider + treestate RPC"),
            ("Halo2 cold start on first send", "warm_orchard_proving_key() on unlock / API start"),
            ("Wrong history expiry metadata", "Persist on-chain expiry_height from signed tx"),
        ],
    )

    add_heading(doc, "3.3 Sync protocols", 2)
    add_table(
        doc,
        ["Path", "Role", "Challenge", "Mitigation"],
        [
            ("JSON-RPC scan", "nozy sync, note discovery", "Slow on large ranges", "Incremental scan; sync --to-tip"),
            ("Compact LWD", "zeaking::lwd, extension resume", "gRPC reachability, DB size", "Resume-safe SQLite; prune"),
            ("Unified orchestrator", "wallet_sync (v2.3.6.2+)", "API vs CLI drift", "Single orchestrator in nozy crate"),
        ],
    )

    add_heading(doc, "3.4 Surface parity", 2)
    add_table(
        doc,
        ["Challenge", "Response"],
        [
            ("api-server stale binary vs CLI", "Document rebuild; scripts/run-nozy-api.ps1"),
            ("WASM extension separate workspace", "Companion API on localhost :3000"),
            ("50k-block rescan on send", "BUG-2026-001: cache-first notes.json"),
            ("Empty history despite balance", "BUG-2026-002: merge received notes into history"),
        ],
    )
    doc.add_page_break()


def section_tradeoffs(doc):
    add_heading(doc, "4. Trade-offs", 1)
    add_body(
        doc,
        "The following matrix summarizes deliberate product and engineering choices. Each row "
        "reflects a decision we rejected an alternative for documented reasons.",
    )
    add_table(
        doc,
        ["Topic", "Option A", "Option B", "NozyWallet choice", "Rationale"],
        [
            ("Expiry delta", "5 blocks (~6 min)", "15 blocks (~19 min)", "5", "Fast expire/speed-up UX"),
            ("Slow prove fix", "Longer expiry", "Rebuild/retry", "Rebuild", "Preserves pilot semantics"),
            ("Stale witness", "Catch-up at send", "Reject if lag >50", "Reject + sync", "Predictable latency"),
            ("Fee source", "Node estimator", "ZIP-317 client", "ZIP-317", "Zebrad has no estimatefee"),
            ("Sync default", "Full rescan always", "Incremental + cache", "Incremental", "Operator bandwidth"),
            ("Address policy", "Allow transparent t1", "Orchard-only u1", "Orchard-only", "Privacy product stance"),
            ("Multichain", "Monolith wallet", "ZEC-first modules", "ZEC-first", "Focus before sidecars"),
        ],
    )
    doc.add_page_break()


def section_security(doc):
    add_heading(doc, "5. Security and Privacy Considerations", 1)
    add_body(
        doc,
        "NozyWallet treats all wallet code as high impact. The following practices are implemented "
        "today. This document does not claim a completed third-party security audit.",
    )
    add_table(
        doc,
        ["Area", "Practice"],
        [
            ("Keys", "AES-GCM encrypted wallet.dat; zeroize for sensitive buffers; mnemonics never returned from API"),
            ("Network", "Optional Tor; trusted_zebra_urls for operator direct RPC when privacy policy allows"),
            ("Transactions", "Orchard-only sends; transparent t1 rejected at validation"),
            ("API companion", "Localhost-first; optional NOZY_API_KEY in production deployments"),
            ("Viewing keys", "UFVK export for Keystone hardware path; selective disclosure planned for business"),
            ("Supply chain", "Pinned librustzcash / orchard versions; NU upgrade discipline (NU6.2)"),
            ("Incidents", "BUG registry, RCA docs, mainnet evidence with TXIDs; responsible disclosure policy"),
        ],
    )
    add_body(
        doc,
        "Privacy properties follow Orchard: sender, receiver, and amount are shielded in Orchard "
        "transactions. The wallet enforces shielded-first policy at the product layer so users "
        "cannot accidentally send via transparent addresses.",
    )
    doc.add_page_break()


def section_mainnet_evidence(doc):
    add_heading(doc, "6. Mainnet Field Evidence (June 2026)", 1)
    add_body(
        doc,
        "Recorded on operator hardware: Windows host, Zebrad in WSL (JSON-RPC 172.20.199.206:8232), "
        "nozy CLI release build. Amounts were dust (0.0001 ZEC) for regression testing.",
    )
    add_table(
        doc,
        ["Run", "Witness lag", "Sync", "Send time", "Broadcast", "TXID (prefix)"],
        [
            ("Gilmore pre-fix", "~5000+ blk", "—", "~12+ min", "FAIL -25", "daed46a0…"),
            ("Post-fix stale", "~5000 blk", "partial", "~419 s", "PASS", "e4f0f504…"),
            ("Sync 5132 + send", "1 blk after", "32 s", "198.7 s", "PASS", "5a03fbd1…"),
            ("Stale guard", ">50 blk", "—", "0.09 s", "Rejected", "—"),
            ("Sync 54 + send", "≤1 blk", "0.3 s", "205.8 s", "PASS", "902cf006…"),
        ],
    )
    add_body(
        doc,
        "Full TXIDs for successful June 2026 sends:",
    )
    add_bullet(
        doc,
        "5a03fbd19547f9499182d78c88791eeb4eaab32e5d158b69ec8ffdc6068d2612",
    )
    add_bullet(
        doc,
        "902cf006efdeef3f15fed4312f8a15fcb1162f52495098c3bffb4acbe3cde4e5",
    )
    add_body(
        doc,
        "Timing model: total send time ≈ witness_catchup + proving_setup + halo2_prove + sign + "
        "broadcast. Synced wallet (lag ≤50 blocks) observed ~200 s end-to-end on this stack. "
        "Proving warm-up: ~2.1 s cold, negligible when cached.",
    )
    doc.add_page_break()


def section_dynamic_fee_pilot(doc):
    add_heading(doc, "7. Dynamic-Fee Pilot Alignment", 1)
    add_table(
        doc,
        ["Pilot feature", "NozyWallet implementation", "Mainnet lesson (2026)"],
        [
            ("Standard fee", "ZIP-317 client-side (fee_policy.rs)", "Zebrad has no fee RPC"),
            ("Priority ×4", "CLI flag, API, extension", "Speed-up after Expired status"),
            ("Short expiry", "nExpiryHeight = tip + 1 + 5", "Keep 5 blocks; rebuild for slow proves"),
            ("Speed-up", "Rebuild new tx at priority fee", "Not rebroadcast of expired bytes"),
        ],
    )
    add_body(
        doc,
        "Paper-ready summary: NozyWallet participates in the Shielded Labs dynamic-fee pilot with "
        "client-side ZIP-317 fees, an optional four-fold priority multiplier, and a five-block "
        "transaction expiry after the mempool build height. VPS testing showed Orchard proving can "
        "span multiple blocks between construction and broadcast; we address this through late "
        "chain-tip refresh, automatic rebuild, and broadcast retry while deliberately preserving "
        "the five-block delta rather than extending it to fifteen blocks.",
    )
    doc.add_page_break()


def section_lessons(doc):
    add_heading(doc, "8. Lessons Learned", 1)
    lessons = [
        "Two clocks on shielded sends: build-time expiry vs mempool expiry. The pilot measures the second; operator VPS bugs often hit the first.",
        "Wallet ≠ node: witnesses and fee policy are wallet responsibilities on Zebrad—not node bugs when RPC is missing.",
        "Sync-before-send is product policy: witness lag guard at fifty blocks prevented seven-plus minute send hangs.",
        "Keep pilot knobs stable: fix runtime (rebuild, warm prove) before changing nExpiryHeight policy.",
        "Operator stacks need first-class testing: WSL Zebrad + Windows CLI matches real users, not laptop-only CI.",
        "Cache format migrations matter: v2 NoteIndex vs legacy array caused subtle post-send bugs.",
        "Surface parity is expensive: api-server, CLI, and extension must share one send pipeline.",
        "Evidence wins trust: TXIDs and timings in public docs outperform ad-hoc claims.",
    ]
    for i, lesson in enumerate(lessons, 1):
        add_body(doc, f"{i}. {lesson}")
    doc.add_page_break()


def section_conclusion(doc):
    add_heading(doc, "9. Conclusion", 1)
    add_body(
        doc,
        "NozyWallet demonstrates that a shielded-first Orchard wallet can operate on modern Zebrad "
        "and lightwalletd infrastructure with client-side fees, short pilot expiry, and local "
        "witness derivation—without zcashd and without lengthening mempool expiry to absorb slow "
        "proving. Mainnet evidence in June 2026 validates broadcast success and operator-send "
        "latency when witnesses are fresh.",
    )
    add_body(
        doc,
        "Continued work focuses on surface parity, Shielded Labs pilot metrics, Zeaking "
        "observatory indexing, and business features (ZNS, Sell mode) on a pure Zcash foundation. "
        "Contributors, operator feedback, and formal review before disclosure features ship remain "
        "essential.",
    )

    add_heading(doc, "References", 1)
    refs = [
        "LEONINE-DAO/Nozy-wallet — github.com/LEONINE-DAO/Nozy-wallet",
        "docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md",
        "docs/reference/PILOT_EXPIRY_PROVING_LATENCY.md",
        "docs/reference/WHITEPAPER_OUTLINE.md",
        "ZEBRAD_SHIELDED_SEND_LIMIT.md",
        "Zebra — github.com/ZcashFoundation/zebra",
        "lightwalletd — github.com/zcash/lightwalletd",
        "ZIP-316 Unified addresses, ZIP-317 Conventional fees, ZIP-225 Orchard transactions",
    ]
    for r in refs:
        add_bullet(doc, r)

    add_heading(doc, "Appendix A: Bug registry summary (2026-06)", 1)
    add_table(
        doc,
        ["ID", "Summary", "Status"],
        [
            ("BUG-2026-001", "Send rescanned ~50k blocks", "Fixed"),
            ("BUG-2026-002", "History empty despite balance", "Fixed"),
            ("BUG-2026-011", "Pre-broadcast expiry -25 on slow VPS", "Fixed"),
            ("—", "Send-readiness: witness lag guard, warm prove", "Fixed (unreleased)"),
            ("—", "NoteIndex v2 mark-spent after broadcast", "Fixed (unreleased)"),
        ],
    )

    add_heading(doc, "Appendix B: Glossary", 1)
    add_table(
        doc,
        ["Term", "Definition"],
        [
            ("Anchor", "Orchard commitment tree root at a block height"),
            ("Compact block", "lightwalletd compressed block for efficient sync"),
            ("nExpiryHeight", "ZIP-225 field: last block tx may be mined"),
            ("Pilot expiry", "Five blocks after mempool build height in NozyWallet"),
            ("Witness", "Merkle inclusion proof for an Orchard note commitment"),
            ("UFVK", "Unified full viewing key (ZIP-316)"),
        ],
    )

    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.CENTER
    run = p.add_run("— End of White Paper —")
    run.italic = True
    run.font.size = Pt(10)


def export_markdown(doc):
    """Plain-text markdown mirror for editors that cannot open .docx (e.g. Cursor)."""
    lines = [
        "# NozyWallet White Paper",
        "",
        "_Open `docs/NozyWallet_Whitepaper.docx` in Word or LibreOffice for the formatted version._",
        "",
    ]
    for p in doc.paragraphs:
        text = p.text.strip()
        if not text:
            lines.append("")
            continue
        style = (p.style.name or "") if p.style else ""
        if style.startswith("Heading 1"):
            lines.append(f"# {text}")
        elif style.startswith("Heading 2"):
            lines.append(f"## {text}")
        elif style.startswith("Heading 3"):
            lines.append(f"### {text}")
        elif "List Bullet" in style:
            lines.append(f"- {text}")
        else:
            lines.append(text)
        lines.append("")
    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines), encoding="utf-8")


def main():
    doc = Document()
    set_margins(doc.sections[0])
    cover_page(doc)
    section_executive_summary(doc)
    section_architecture_decisions(doc)
    section_phased_development(doc)
    section_challenges(doc)
    section_tradeoffs(doc)
    section_security(doc)
    section_mainnet_evidence(doc)
    section_dynamic_fee_pilot(doc)
    section_lessons(doc)
    section_conclusion(doc)

    OUTPUT_DOCX.parent.mkdir(parents=True, exist_ok=True)
    doc.save(str(OUTPUT_DOCX))
    export_markdown(doc)
    print(f"Saved: {OUTPUT_DOCX}")
    print(f"Saved: {OUTPUT_MD}")
    print(f"Paragraphs: {len(doc.paragraphs)} (approx. 12–16 pages in Word at 11pt)")


if __name__ == "__main__":
    main()
