#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
BUILD="$ROOT/crates/veac-cli/build.rs"

fail() {
  echo "project backend identity contract failed: $1" >&2
  exit 1
}

inventory() {
  local domain=$1
  awk -v domain="$domain" '
    index($0, "domain: \"" domain "\"") { active = 1 }
    active { print }
    active && /^[[:space:]]*},$/ { exit }
  ' "$BUILD"
}

require() {
  local block=$1 value=$2
  [[ $block == *"$value"* ]] || fail "missing $value"
}

reject() {
  local block=$1 value=$2
  [[ $block != *"$value"* ]] || fail "unexpected $value"
}

backend=$(inventory veac.project.backend)
project=$(inventory veac.project.build)
evidence=$(inventory veac.evidence.backend)

require "$backend" 'embedded_sources: &[]'
require "$project" '"crates/veac-project/src"'
require "$project" '"crates/veac-project/src/project.veac"'
require "$project" '"crates/veac-project/Cargo.toml"'
reject "$project" 'evidence.veac'
require "$evidence" '"crates/veac-evidence/src/evidence.veac"'
reject "$evidence" 'project.veac'
rg -q '^[[:space:]]*\.embedded_sources$' "$BUILD" || fail "embedded sources are not fingerprinted"
rg -Fq 'cargo:rerun-if-changed={}' "$BUILD" || fail "inventory paths do not trigger rebuilds"

rg -Fq 'include_str!("../project.veac")' \
  "$ROOT/crates/veac-project/src/authored/mod.rs" || fail "project prelude is not embedded"
rg -Fq 'include_str!("../evidence.veac")' \
  "$ROOT/crates/veac-evidence/src/authored/mod.rs" || fail "evidence prelude is not embedded"

echo "project backend identity contracts passed"
