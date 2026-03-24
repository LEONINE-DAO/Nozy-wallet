#!/usr/bin/env bash
# Build WASM + popup and copy a loadable extension tree to dist/nozy-extension
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "$ROOT/../.." && pwd)"

export PATH="${HOME}/.cargo/bin:${PATH}"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "Install wasm-pack: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh" >&2
  exit 1
fi

cd "$ROOT/wasm-core"
rustup target add wasm32-unknown-unknown 2>/dev/null || true
wasm-pack build --target web --out-dir ../wasm/pkg --release

cd "$ROOT/wasm-core/popup"
npm ci
npm run build

OUT="${1:-$REPO_ROOT/dist/nozy-extension}"
rm -rf "$OUT"
mkdir -p "$OUT/wasm-core/popup"
cp "$ROOT/manifest.json" "$OUT/manifest.json"
cp -r "$ROOT/background" "$OUT/background"
rm -f "$OUT/background"/*.test.mjs 2>/dev/null || true
cp -r "$ROOT/content" "$OUT/content"
cp -r "$ROOT/wasm" "$OUT/wasm"
cp -r "$ROOT/wasm-core/popup/dist" "$OUT/wasm-core/popup/dist"

echo "Extension tree ready: $OUT"
echo "Load this folder as unpacked in chrome://extensions"
