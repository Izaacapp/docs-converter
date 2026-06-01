#!/usr/bin/env bash
# Build the doc-convert sidecar and install it where Tauri expects it
# (binaries/doc-convert-<target-triple>).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TRIPLE="$(rustc -Vv | sed -n 's/^host: //p')"
echo "Building doc-convert for ${TRIPLE}…"
cargo build --release --manifest-path "${ROOT}/server/sidecars/doc-convert/Cargo.toml"
mkdir -p "${ROOT}/src-tauri/binaries"
install -m 0755 \
  "${ROOT}/server/sidecars/doc-convert/target/release/doc-convert" \
  "${ROOT}/src-tauri/binaries/doc-convert-${TRIPLE}"
echo "Installed: src-tauri/binaries/doc-convert-${TRIPLE}"
