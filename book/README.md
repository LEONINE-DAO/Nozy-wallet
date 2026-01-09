# NozyWallet Documentation Book

This directory contains the source files for the NozyWallet documentation book, built with [mdBook](https://rust-lang.github.io/mdBook/).

## Building Locally

### Install mdBook

```bash
# Using cargo
cargo install mdbook

# Or download from releases
# https://github.com/rust-lang/mdBook/releases
```

### Build the Book

From the repository root:

```bash
mdbook build
```

The built book will be in `book/book/` directory.

### Serve Locally

To preview the book while editing:

```bash
mdbook serve
```

Then open http://localhost:3000 in your browser.

## Structure

- `book/src/` - Source markdown files
- `book/book/` - Built HTML output (gitignored)
- `book.toml` - mdBook configuration

## Adding Content

1. Add markdown files to `book/src/`
2. Update `book/src/SUMMARY.md` to include new pages
3. Build and test locally with `mdbook serve`
4. Commit and push - GitHub Actions will deploy automatically

## Deployment

The book is automatically deployed to GitHub Pages via GitHub Actions when changes are pushed to the `master` branch.

See `.github/workflows/pages.yml` for the deployment configuration.
