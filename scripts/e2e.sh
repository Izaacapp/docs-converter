#!/usr/bin/env bash
# Real edge-case harness: convert the sample corpus off the LIVE converter
# server (the sidecar forwards; the server does the work — no local CPU).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORPUS="${CORPUS:-$HOME/Theory/THEINTERNETINANUTSHELL/Development_Practices/Technical_Writing}"
TRIPLE="$(rustc -Vv | sed -n 's/^host: //p')"

BIN="${DOC_CONVERT_BIN:-$ROOT/src-tauri/binaries/doc-convert-$TRIPLE}"
[ -x "$BIN" ] || BIN="$ROOT/.artifacts/rust/release/doc-convert"
[ -x "$BIN" ] || { echo "sidecar not built — run scripts/build-sidecar.sh"; exit 1; }

set -a; [ -f "$ROOT/.env" ] && . "$ROOT/.env"; set +a
: "${CONVERTER_API_URL:?set CONVERTER_API_URL in .env (run: just deploy)}"
export CONVERTER_API_URL
if ! curl -fsS "${CONVERTER_API_URL%/}/health" >/dev/null 2>&1; then
  echo "server $CONVERTER_API_URL not reachable — run: just deploy"; exit 1
fi
echo "server: $CONVERTER_API_URL"; echo "corpus: $CORPUS"; echo

OUT="$(mktemp -d)"
# Each row exercises a distinct real-world edge case: file|format|label
ROWS=(
  "The_Pragmatic_Programmer.pdf|md|digital 324pp, structure/headings"
  "Writing_For_Developers.pdf|md|726pp performance (CID TrueType)"
  "Dreyers_English.pdf|tex|Type-3 fonts + ligature normalization"
  "On_writing_well_fourth_edition.pdf|md|glyphless Internet-Archive scan"
  "Communication for Engineer.pdf|md|spaces in filename + CID fonts"
  "Style Lessons in Clarity and Grace.pdf|html|spaces in filename + Type-1C"
  "writing_without_bullshit.pdf|docx|binary target (CID Tinos)"
  "Think_Like_A_Software_Engineering_Manager.pdf|pdf|xelatex render"
)

sniff() { # path format -> ok|bad
  local f="$1" fmt="$2"
  [ -s "$f" ] || { echo bad; return; }
  case "$fmt" in
    pdf)  head -c4 "$f" | grep -q '%PDF' && echo ok || echo bad ;;
    docx) head -c2 "$f" | grep -q 'PK'   && echo ok || echo bad ;;
    html) grep -qi '<' "$f" && echo ok || echo bad ;;
    tex)  [ "$(wc -c <"$f")" -gt 50 ]  && echo ok || echo bad ;;
    md)   [ "$(wc -c <"$f")" -gt 200 ] && echo ok || echo bad ;;
  esac
}

printf '%-46s %-5s %-8s %-6s %s\n' FILE FMT RESULT SECS EDGE-CASE
pass=0; fail=0
for row in "${ROWS[@]}"; do
  IFS='|' read -r file fmt label <<<"$row"
  src="$CORPUS/$file"
  if [ ! -f "$src" ]; then
    printf '%-46.46s %-5s %-8s\n' "$file" "$fmt" "MISSING"; continue
  fi
  dst="$OUT/${file%.pdf}.$fmt"
  t0=$(date +%s)
  "$BIN" -i "$src" -t "$fmt" -o "$dst" >/dev/null 2>"$OUT/err"
  code=$?; t1=$(date +%s)
  if [ $code -eq 0 ]; then res="$(sniff "$dst" "$fmt")"; else res="exit$code"; fi
  if [ "$res" = ok ]; then pass=$((pass+1)); else fail=$((fail+1)); fi
  printf '%-46.46s %-5s %-8s %-6s %s\n' "$file" "$fmt" "$res" "$((t1-t0))" "$label"
  [ "$res" = ok ] || tail -1 "$OUT/err" | sed 's/^/      ! /'
done
echo "---"; echo "pass=$pass fail=$fail   outputs: $OUT"
[ $fail -eq 0 ]
