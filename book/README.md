# Book source layout

This directory is the **mdBook** source for [NozyWallet documentation](https://leonine-dao.github.io/Nozy-wallet/book/).

## Navigation

Only files linked from [`SUMMARY.md`](src/SUMMARY.md) appear in the published book table of contents.

## Orphan files

Many `*.md` files under `src/` (e.g. duplicate paths in `faq/`, `examples/`, `cli/`) are **legacy scaffolds** and are **not** in `SUMMARY.md`. They can be ignored or removed in a future cleanup PR. When adding docs, **edit the SUMMARY-linked file** or add a new entry to `SUMMARY.md`.

## Build locally

```bash
cd book
mdbook build
# output in book/book/
mdbook serve   # live preview
```

Install mdBook: `cargo install mdbook`

## Deeper reference docs

Paper-grade evidence and RCAs live in repo [`docs/`](../../docs/README.md), especially [`docs/reference/`](../../docs/reference/).
