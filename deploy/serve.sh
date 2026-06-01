#!/usr/bin/env bash
# Deploy the doc-convert HTTP service to the converter server: rsync the Rust
# source, build it natively there, and run `doc-convert serve`. A tiny CPU
# service (pdf_oxide) — it never touches the GPU (free for the LLM) and never
# the local machine's CPU. Idempotent. Teardown: deploy/teardown.sh.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
set -a; . "$ROOT/.env"; set +a
: "${CONVERTER_SERVER_HOST:?set in .env}"
: "${CONVERTER_SERVER_USER:?}"
: "${CONVERTER_SERVER_PASSWORD:?}"
PORT="${CONVERTER_API_PORT:-8088}"
DEST="${CONVERTER_SERVER_USER}@${CONVERTER_SERVER_HOST}"
export SSHPASS="$CONVERTER_SERVER_PASSWORD"

echo ">> syncing source to ${DEST}:~/doc-convert/"
sshpass -e rsync -az --delete -e "ssh -o StrictHostKeyChecking=accept-new" \
  --exclude target --exclude .artifacts \
  "$ROOT/server/sidecars/doc-convert/" "$DEST:doc-convert/"

echo ">> build + run on server (first build installs rust + pandoc)…"
sshpass -e ssh -o StrictHostKeyChecking=accept-new "$DEST" \
  "bash -s -- '$PORT' '$CONVERTER_SERVER_PASSWORD'" <<'REMOTE'
set -euo pipefail
PORT="${1:-8088}"; SUDO_PW="${2:-}"
if ! command -v cargo >/dev/null 2>&1; then
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal
fi
source "$HOME/.cargo/env"
# pandoc (html/tex/docx) via apt
if ! command -v pandoc >/dev/null 2>&1; then
  echo "$SUDO_PW" | sudo -S apt-get update -qq \
    && echo "$SUDO_PW" | sudo -S apt-get install -y pandoc \
    || echo "WARN: pandoc not installed (html/tex/docx need it)"
fi
# tectonic (pdf) — lightweight Rust LaTeX, single binary (not in apt). Drop it in
# ~/.cargo/bin so the service finds it on PATH.
if ! command -v tectonic >/dev/null 2>&1; then
  ( cd "$HOME" && curl -fsSL https://drop-sh.fullyjustified.net | sh ) \
    && install -m 0755 "$HOME/tectonic" "$HOME/.cargo/bin/tectonic" && rm -f "$HOME/tectonic" \
    || echo "WARN: tectonic install failed (pdf needs a LaTeX engine)"
fi
cd ~/doc-convert
cargo build --release
pkill -f "doc-convert serve" 2>/dev/null || true
sleep 1
nohup ./target/release/doc-convert serve --port "$PORT" > ~/doc-convert.log 2>&1 &
sleep 2
curl -fsS "http://localhost:$PORT/health" && echo " <- healthy on :$PORT" || { echo "NOT healthy:"; tail -5 ~/doc-convert.log; }
REMOTE
echo ">> API: http://${CONVERTER_SERVER_HOST}:${PORT}  (POST /convert?to=md, body=PDF)"
