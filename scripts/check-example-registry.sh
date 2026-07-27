#!/usr/bin/env bash
set -euo pipefail

ROOT="${VEAC_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
MECHANISMS="${1:?combined mechanism JSON is required}"
COMPOSITION="$ROOT/examples/catalog/mechanisms/transitions-composition.json"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

fail() {
  echo "example registry check failed: $*" >&2
  exit 1
}

compare() {
  local name="$1"
  local actual="$2"
  local catalog="$3"
  if ! diff -u "$actual" "$catalog" > "$tmp/$name.diff"; then
    cat "$tmp/$name.diff" >&2
    fail "$name catalog differs from its Rust registry"
  fi
}

extract_enum() {
  local file="$1"
  local enum_name="$2"
  awk -v enum_name="$enum_name" '
    $0 ~ "pub enum " enum_name { inside = 1; next }
    inside && /^}/ { exit }
    inside {
      value = $0
      sub(/^[[:space:]]+/, "", value)
      sub(/[({,].*$/, "", value)
      sub(/[[:space:]]+$/, "", value)
      if (value !~ /^[A-Z][A-Za-z0-9]*$/) next
      snake = ""
      for (i = 1; i <= length(value); i++) {
        char = substr(value, i, 1)
        if (char ~ /[A-Z]/) {
          if (i > 1) snake = snake "_"
          snake = snake tolower(char)
        } else snake = snake char
      }
      print snake
    }
  ' "$file" | sort
}

sed -n '/const EFFECTS:/,/];/p' "$ROOT/crates/veac-ir/src/registry.rs" |
  sed -n 's/.*"\([^"]*\)".*/\1/p' | sort > "$tmp/effects-rust"
jq -r '.[] | select(.registry_key != null) | .registry_key' "$MECHANISMS" |
  sort > "$tmp/effects-catalog"
compare effect-registry "$tmp/effects-rust" "$tmp/effects-catalog"

extract_enum "$ROOT/crates/veac-ir/src/model/transition.rs" TransitionKind \
  > "$tmp/transitions-rust"
jq -r '.transition_enum_keys[]' "$COMPOSITION" | sort > "$tmp/transitions-catalog"
compare transition-enum "$tmp/transitions-rust" "$tmp/transitions-catalog"

extract_enum "$ROOT/crates/veac-ir/src/model/properties/composition.rs" BlendMode \
  > "$tmp/blends-rust"
jq -r '.blend_enum_keys[]' "$COMPOSITION" | sort > "$tmp/blends-catalog"
compare blend-enum "$tmp/blends-rust" "$tmp/blends-catalog"

echo "Example registry mappings are exact."
