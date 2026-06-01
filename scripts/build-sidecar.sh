#!/usr/bin/env bash
# Build the doc-convert sidecar and install it where Tauri expects it
# (src-tauri/binaries/doc-convert-<target-triple>). Build output lives in
# .artifacts/rust (see .cargo/config.toml).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
TRIPLE="$(rustc -Vv | sed -n 's/^host: //p')"
echo "Building doc-convert for ${TRIPLE}…"
cargo build --release --manifest-path server/sidecars/doc-convert/Cargo.toml
mkdir -p src-tauri/binaries
install -m 0755 \
  ".artifacts/rust/release/doc-convert" \
  "src-tauri/binaries/doc-convert-${TRIPLE}"
echo "Installed: src-tauri/binaries/doc-convert-${TRIPLE}"
