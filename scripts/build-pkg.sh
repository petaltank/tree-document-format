#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG="$ROOT/pkg"

echo "==> Building WASM (release)..."
cargo build -p tree-doc-wasm --target wasm32-unknown-unknown --release

echo "==> Running wasm-bindgen (bundler target)..."
rm -rf "$PKG"
wasm-bindgen \
  --target bundler \
  --out-dir "$PKG" \
  "$ROOT/target/wasm32-unknown-unknown/release/tree_doc_wasm.wasm"

# Optional: shrink the .wasm binary if wasm-opt is available
if command -v wasm-opt &>/dev/null; then
  echo "==> Optimizing with wasm-opt..."
  wasm-opt -Oz -o "$PKG/tree_doc_wasm_bg.wasm" "$PKG/tree_doc_wasm_bg.wasm"
else
  echo "    (wasm-opt not found, skipping optimization)"
fi

echo "==> Copying npm source files into pkg/..."
cp "$ROOT/npm/package.json" "$PKG/package.json"
cp "$ROOT/npm/index.js"     "$PKG/index.js"
cp "$ROOT/npm/index.d.ts"   "$PKG/index.d.ts"

echo "==> Done. Contents of pkg/:"
ls -lh "$PKG"
