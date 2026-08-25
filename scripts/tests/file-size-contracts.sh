#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
CHECKER="$ROOT/scripts/check-file-size.sh"
FIXTURE=$(mktemp -d "${TMPDIR:-/tmp}/veac-file-size.XXXXXX")
trap 'rm -rf "$FIXTURE"' EXIT

mkdir -p "$FIXTURE"/{crates,docs,examples,scripts,spec,stdlib,.github}
touch "$FIXTURE/README.md" "$FIXTURE/CONTRIBUTING.md" \
  "$FIXTURE/Makefile" "$FIXTURE/install.sh"

write_lines() {
  local count=$1
  local path=$2
  mkdir -p "$(dirname "$path")"
  awk -v count="$count" 'BEGIN { for (line = 0; line < count; line++) print "x" }' >"$path"
}

assert_rejected() {
  local relative=$1
  local output="$FIXTURE/error.txt"
  write_lines 200 "$FIXTURE/$relative"
  if VEAC_FILE_SIZE_ROOT="$FIXTURE" "$CHECKER" >"$output" 2>&1; then
    echo "error: 200-line controlled file was accepted: $relative" >&2
    exit 1
  fi
  rg -F --quiet "$relative has 200 lines" "$output"
  rm "$FIXTURE/$relative" "$output"
}

assert_rejected "stdlib/example/main.veac"
assert_rejected "stdlib/example/veac.package.lock"

write_lines 199 "$FIXTURE/stdlib/example/veac.package.lock"
VEAC_FILE_SIZE_ROOT="$FIXTURE" "$CHECKER"

printf 'file-size contracts passed\n'
