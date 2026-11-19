# Desktop v1.0.0-beta.10 — Hot Lemon Pepper Sprinkles

Copy the sections below into the GitHub Release body when tagging **`desktop-v1.0.0-beta.10`**.

---

## Summary

**Hot Lemon Pepper Sprinkles beta.10** lets Desktop holders cast the live NU7.1 coinholder poll.

The Vote screen follows the active Valar round. For the poll open now, that is **NU7.1 Poll**, snapshot height **3,578,540**, closing **1 December 2026 at 02:59 UTC**. The five proposals are on the Vote screen. Cast after delegation confirms. The seed stays in the wallet. The `nozy-vote` helper is bundled with the app.

The CLI remains the operator surface. `nozy vote-export-notes` reads the same live snapshot height from `nozy-vote`.

This build was not reviewed by a third-party wallet auditor.

## Requirements

- **OS:** Windows 10/11, or the Linux / macOS build attached to this release
- **Node:** Zebrad + lightwalletd, or the public compact sync you already use
- **Weight:** spendable Ironwood notes at the poll's snapshot, at least 0.125 ZEC

## Install

1. Download the asset for your OS from this release.
2. Check the `.sha256` file.
3. Install or unpack, then launch **NozyWallet**.
4. Sync, open **Vote**, export notes, finish delegation, then mark every proposal and cast.

## What's new since beta.9

- Vote calendar uses the active round instead of the closed August 2026 window
- Ballot choices and **Cast ballot** are on the Desktop Vote screen
- `nozy-vote` ships inside the desktop bundle
- `nozy vote-export-notes` uses the live snapshot height

## Known limits

- Mobile can export and sign only. It cannot cast this poll.
- The extension does not show this ballot.
- Delegation still has to confirm before a cast succeeds.
- Beta software. Mainnet use is your own risk.

## Downloads

| Asset | Description |
|-------|-------------|
| `nozy-desktop-windows-x86_64-installer.exe` | Windows NSIS installer |
| `nozy-desktop-linux-x86_64.tar.gz` | Linux binary |
| `nozy-desktop-macos-x86_64.tar.gz` | macOS Intel |
| `nozy-desktop-macos-aarch64.tar.gz` | macOS Apple Silicon |

Each asset has a matching `.sha256`. CI attaches them when the GitHub Release is published.
