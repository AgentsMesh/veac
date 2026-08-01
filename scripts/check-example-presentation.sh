#!/usr/bin/env bash
set -euo pipefail

ROOT=${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}
GALLERY=${2:-$ROOT/examples/catalog/gallery.json}

fail() {
  echo "example presentation check failed: $*" >&2
  exit 1
}

command -v jq >/dev/null 2>&1 || fail "jq is required"
command -v rg >/dev/null 2>&1 || fail "rg is required"
[[ -f $GALLERY ]] || fail "missing gallery catalog: $GALLERY"

while IFS= read -r source; do
  [[ $source == examples/*/main.veac ]] || fail "invalid example path: $source"
  [[ -f $ROOT/$source ]] || fail "missing example: $source"
  while IFS= read -r declaration; do
    if [[ $declaration =~ [一-龥] ]] ||
      [[ $declaration == *'content "AgentsMesh";'* ]]; then
      continue
    fi
    fail "$source contains an English-only visible explanation: $declaration"
  done < <(rg -n '(^|[[:space:]{])(content|speaker) "|^[[:space:]]*label "' \
    "$ROOT/$source" || true)
done < <(jq -r '.examples[].source' "$GALLERY")

echo "Example presentation text is valid."
