#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
MATRIX="$ROOT/docs/capability-matrix-roadmap.md"
EVIDENCE=(
  "$ROOT/docs/capabilities/p1-evidence-a.md"
  "$ROOT/docs/capabilities/p1-evidence-b.md"
  "$ROOT/docs/capabilities/p2-evidence.md"
)
status=0

fail() {
  echo "error: $*" >&2
  status=1
}

for file in "$MATRIX" "${EVIDENCE[@]}"; do
  lines=$(awk 'END { print NR }' "$file")
  if (( lines >= 200 )); then
    fail "${file#"$ROOT/"} has $lines lines; capability indexes must stay below 200"
  fi
done

ids=()
while IFS= read -r id; do
  ids+=("$id")
done < <(sed -n 's/^| `\(P[12]-[0-9][0-9]\)` |.*/\1/p' "$MATRIX")

if (( ${#ids[@]} != 52 )); then
  fail "capability matrix must declare exactly 52 acceptance rows, found ${#ids[@]}"
fi
duplicates=$(printf '%s\n' "${ids[@]}" | sort | uniq -d)
if [[ -n "$duplicates" ]]; then
  fail "duplicate capability IDs: $duplicates"
fi

for id in "${ids[@]}"; do
  heading_count=$(rg --no-filename -x "### $id" "${EVIDENCE[@]}" | wc -l | tr -d ' ')
  if (( heading_count != 1 )); then
    fail "$id must have exactly one evidence section, found $heading_count"
  fi
  for kind in implementation verification; do
    if ! rg -q "^- $id $kind: \[[^]]+\]\([^)]+\)$" "${EVIDENCE[@]}"; then
      fail "$id is missing its $kind evidence link"
    fi
  done
  if [[ "$id" == P1-* ]]; then
    if ! rg -q "^\| \`$id\` \| P1 \|.*\| Delivered \|" "$MATRIX"; then
      fail "$id must be a Delivered P1 row"
    fi
    if ! rg -q "^- $id observable: \[[^]]+\]\([^)]+\)$" "${EVIDENCE[@]}"; then
      fail "$id is missing observable or public integration evidence"
    fi
  else
    if ! rg -q "^\| \`$id\` \| P2 \|.*\| Delivered(/Contract)? \|" "$MATRIX"; then
      fail "$id must be a Delivered or Delivered/Contract P2 row"
    fi
    for kind in application contract; do
      if ! rg -q "^- $id $kind: \[[^]]+\]\([^)]+\)$" "${EVIDENCE[@]}"; then
        fail "$id is missing its provider $kind evidence link"
      fi
    done
  fi
  anchor=$(printf '%s' "$id" | tr '[:upper:]' '[:lower:]')
  if ! rg -q "\[$id\]\(capabilities/[^)]*#$anchor\)" "$MATRIX"; then
    fail "$id matrix row does not link to its evidence anchor"
  fi
done

while IFS= read -r link; do
  path=${link%%#*}
  if [[ ! -e "$ROOT/docs/capabilities/$path" ]]; then
    fail "broken capability evidence link: $link"
  fi
done < <(sed -n 's/^- .*: \[[^]]*\](\([^)]*\))$/\1/p' "${EVIDENCE[@]}")

"$ROOT/scripts/check-example-capabilities.sh" >/dev/null

exit "$status"
