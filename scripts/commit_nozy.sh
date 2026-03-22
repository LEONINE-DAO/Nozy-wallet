#!/usr/bin/env bash
set -euo pipefail

# One-off dev helper (adjust paths / author before use). Prefer running from your Nozy-wallet clone.

cd ~/projects/Nozy-wallet

git add Cargo.toml Cargo.lock desktop-client/src/lib/api.ts src/main.rs
git add -u desktop-client/src-tauri/Cargo.toml \
  desktop-client/src-tauri/Cargo.lock \
  desktop-client/src-tauri/src/commands/sync.rs \
  desktop-client/src-tauri/src/lib.rs

GIT_AUTHOR_NAME="lowo" \
GIT_AUTHOR_EMAIL="leoninedao@outlook.com" \
GIT_COMMITTER_NAME="lowo" \
GIT_COMMITTER_EMAIL="leoninedao@outlook.com" \
git commit --trailer "Made-with: Cursor" -m "$(cat <<'EOF'
fix(sync): prevent skipped wallet scans and guard browser invoke

Use actual scanned block bounds when updating last_scan_height in CLI and desktop sync paths, and bound default sync ranges to avoid jumping checkpoints. Also add a Tauri runtime guard for frontend invoke calls and enable indexmap std feature to resolve desktop/CLI build issues.
EOF
)"

git status -sb
