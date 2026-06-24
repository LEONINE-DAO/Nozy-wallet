# Issues and bugs — documentation hub

**Purpose:** Track bugs and feature issues in-repo with repro steps, root cause, and fix status — alongside (not instead of) [GitHub Issues](https://github.com/LEONINE-DAO/Nozy-wallet/issues).

---

## Two-track system

| Track | Where | Use for |
|-------|--------|---------|
| **GitHub Issue** | github.com/LEONINE-DAO/Nozy-wallet/issues | Active work, PR links, community reports |
| **In-repo docs** | This folder | Detailed repro, root-cause writeups, registry, paste-ready issue bodies |

**Rule of thumb:** Open a **GitHub issue** when work starts. Add or update **in-repo docs** when the bug is non-trivial (needs repro, RCA, or will be cited in CHANGELOG / forum / grants).

---

## Files in this folder

| File | Purpose |
|------|---------|
| [`BUG_REGISTRY.md`](BUG_REGISTRY.md) | Master list — ID, status, severity, reporter, fix version, links |
| [`FEATURE_REGISTRY.md`](FEATURE_REGISTRY.md) | Feature / enhancement requests (non-bug) |
| [`TEMPLATE_BUG_REPORT.md`](TEMPLATE_BUG_REPORT.md) | Copy for new bug writeups |
| [`TEMPLATE_FEATURE_ISSUE.md`](TEMPLATE_FEATURE_ISSUE.md) | Copy for new feature requests |
| [`bugs/`](bugs/) | One file per significant bug (detailed RCA) |
| [`features/`](features/) | One file per major feature issue / RFC companion |
| [`../MAINNET_SEND_EXPIRY_TEST.md`](../MAINNET_SEND_EXPIRY_TEST.md) | Mainnet verification for BUG-2026-011 |
| [`../reference/PILOT_EXPIRY_PROVING_LATENCY.md`](../reference/PILOT_EXPIRY_PROVING_LATENCY.md) | Paper / design notes for pilot expiry |
| [`../reference/MAINNET_SEND_READINESS_EVIDENCE.md`](../reference/MAINNET_SEND_READINESS_EVIDENCE.md) | Recorded mainnet sync+send evidence (paper / lecture) |
| [`../reference/CLI_BALANCE_NOTEINDEX.md`](../reference/CLI_BALANCE_NOTEINDEX.md) | CLI balance / NoteIndex v2 (paper / lecture) |

---

## Workflow

### Someone reports a bug

1. **GitHub:** Create issue (or find existing). Label: `bug`, surface (`cli`, `api-server`, `desktop`, etc.).
2. **Registry:** Add row to [`BUG_REGISTRY.md`](BUG_REGISTRY.md) with status `Open`.
3. **If complex:** Copy [`TEMPLATE_BUG_REPORT.md`](TEMPLATE_BUG_REPORT.md) → `bugs/YYYY-MM-DD-short-title.md`.
4. **Session log:** Note in [`../journal/log/`](../journal/log/) when investigating.

### Fixing a bug

1. PR references GitHub issue `#N`.
2. Update **bug doc** with root cause + fix commit.
3. Update **BUG_REGISTRY** → `Fixed in vX.Y.Z` or `Fixed on master`.
4. Add entry to **`CHANGELOG.md`** `[Unreleased]` or release section.
5. Optional: [`../journal/log/`](../journal/log/) session entry.

### Feature / enhancement (not a bug)

1. GitHub issue with label `enhancement`.
2. Row in [`FEATURE_REGISTRY.md`](FEATURE_REGISTRY.md).
3. Large features: also `docs/rfcs/` or `docs/*_TODO.md` + GitHub issue #85-style alignment for behavior changes.

---

## Bug ID convention

`BUG-YYYY-NNN` — e.g. `BUG-2026-001`. Assign next number in [`BUG_REGISTRY.md`](BUG_REGISTRY.md).

Feature IDs: `FEAT-YYYY-NNN` in [`FEATURE_REGISTRY.md`](FEATURE_REGISTRY.md).

---

## Severity (for registry)

| Level | Meaning |
|-------|---------|
| **P0** | Loss of funds risk, seed exposure, wrong balance/spend |
| **P1** | Send/receive/sync broken on mainnet path |
| **P2** | Degraded UX, wrong error message, perf |
| **P3** | Docs, cosmetic, dev-only |

---

## Surfaces (tag bugs)

`cli` · `api-server` · `desktop` · `extension` · `mobile` · `zeaking` · `docs` · `operator-vps`

---

## Related

- [`../journal/README.md`](../journal/README.md) — session logs and research
- [`../grant/LOCAL_WORK_LEDGER.md`](../grant/LOCAL_WORK_LEDGER.md) — local ship evidence (gitignored)
- [`../../CHANGELOG.md`](../../CHANGELOG.md) — public fix history
