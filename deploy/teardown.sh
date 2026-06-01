#!/usr/bin/env bash
# Stop the doc-convert service and remove its source on the converter server.
# Touches nothing else (the podman stack is untouched).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
set -a; . "$ROOT/.env"; set +a
: "${CONVERTER_SERVER_HOST:?}"; : "${CONVERTER_SERVER_USER:?}"; : "${CONVERTER_SERVER_PASSWORD:?}"
export SSHPASS="$CONVERTER_SERVER_PASSWORD"
sshpass -e ssh -o StrictHostKeyChecking=accept-new \
  "${CONVERTER_SERVER_USER}@${CONVERTER_SERVER_HOST}" 'bash -s' <<'REMOTE'
pkill -f "doc-convert serve" 2>/dev/null || true
rm -rf ~/doc-convert ~/doc-convert.log
echo "stopped doc-convert and removed ~/doc-convert"
REMOTE
