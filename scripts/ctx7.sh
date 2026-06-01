#!/usr/bin/env bash
# Fetch current library docs via context7, using CONTEXT7_API_KEY from .env.
# Calls the real npx directly to dodge lazy-nvm shell shims.
#   scripts/ctx7.sh library "Tauri" "sidecar externalBin"
#   scripts/ctx7.sh docs /websites/v2_tauri_app "Command.sidecar streaming"
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
set -a; [ -f "${ROOT}/.env" ] && . "${ROOT}/.env"; set +a
NPX="$(command -v npx || true)"
for c in "${HOME}"/.nvm/versions/node/*/bin/npx; do [ -x "$c" ] && NPX="$c"; done
[ -n "${NPX}" ] || { echo "npx not found" >&2; exit 1; }
exec "${NPX}" -y ctx7@latest "$@"
