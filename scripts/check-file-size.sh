#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
STATUS=0

check_file() {
  local file=$1
  local lines relative
  lines=$(awk 'END { print NR }' "$file")
  if ((lines >= 200)); then
    relative=${file#"$ROOT"/}
    echo "error: $relative has $lines lines; controlled files must stay below 200" >&2
    STATUS=1
  fi
}

while IFS= read -r -d '' file; do
  check_file "$file"
done < <(
  find "$ROOT/crates" "$ROOT/docs" "$ROOT/examples" "$ROOT/scripts" \
    -type f \( -name '*.rs' -o -name '*.sh' -o -name '*.jq' -o -name '*.md' \
      -o -name '*.veac' -o -name '*.json' -o -name '*.toml' -o -name '*.cube' \) \
    -print0
)

for file in "$ROOT/README.md" "$ROOT/CONTRIBUTING.md" "$ROOT/Makefile"; do
  [[ -f "$file" ]] || {
    echo "error: missing controlled file: ${file#"$ROOT"/}" >&2
    STATUS=1
    continue
  }
  check_file "$file"
done

exit "$STATUS"
