#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
CHECK="$ROOT/scripts/check-architecture-boundaries.py"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

fail() {
  printf 'architecture boundary test failed: %s\n' "$1" >&2
  exit 1
}

python3 "$CHECK" --root "$ROOT" >/dev/null || fail 'current repository contract failed'

fixture() {
  local root="$TMP/$1"
  mkdir -p "$root/docs/rfcs" "$root/docs/language-design" "$root/scripts"
  cp "$ROOT/docs/architecture.md" "$root/docs/architecture.md"
  cp "$ROOT/docs/architecture-dependencies.json" "$root/docs/architecture-dependencies.json"
  cp "$ROOT/docs/rfcs/compiler-query-database.md" "$root/docs/rfcs/compiler-query-database.md"
  cp "$ROOT/docs/language-design/compiler-boundaries.md" "$root/docs/language-design/compiler-boundaries.md"
  cp "$ROOT/Makefile" "$root/Makefile"
  cp "$ROOT/scripts/check-rust-structure.sh" "$root/scripts/check-rust-structure.sh"
  printf '%s\n' "$root"
}

expect_failure() {
  local mode=$1
  local expected=$2
  local root
  root=$(fixture "$mode")
  python3 - "$root" "$mode" <<'PY'
import sys
from pathlib import Path

root = Path(sys.argv[1])
mode = sys.argv[2]
architecture = root / "docs/architecture.md"
query = root / "docs/rfcs/compiler-query-database.md"
structure = root / "scripts/check-rust-structure.sh"
if mode == "planned-ownership":
    text = architecture.read_text()
    anchor = "| `veac-lang-model` |"
    architecture.write_text(text.replace(anchor, "| `veac-syntax` | planned | planned |\n" + anchor, 1))
elif mode == "missing-active":
    architecture.write_text("\n".join(
        line for line in architecture.read_text().splitlines()
        if "| `veac-ir` |" not in line
    ) + "\n")
elif mode == "query-claim":
    text = query.read_text()
    query.write_text(text.replace("当前实现提供的是有界、可丢弃的性能缓存", "当前实现提供的是缓存", 1))
elif mode == "wiring":
    structure.write_text(structure.read_text().replace(
        'python3 "$SCRIPT_DIR/check-architecture-boundaries.py"\n', "", 1
    ))
else:
    raise AssertionError(mode)
PY
  if python3 "$CHECK" --root "$root" >"$TMP/$mode.log" 2>&1; then
    fail "$mode fixture unexpectedly passed"
  fi
  rg -Fq "$expected" "$TMP/$mode.log" || fail "$mode fixture did not report: $expected"
}

expect_failure planned-ownership 'planned crates are presented as physically owned: veac-syntax'
expect_failure missing-active 'active crates missing from architecture ownership table: veac-ir'
expect_failure query-claim 'query RFC is missing boundary marker:'
expect_failure wiring 'Rust structure gate does not run architecture boundary checks'

printf 'architecture boundary contracts passed\n'
