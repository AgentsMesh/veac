#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
source "$ROOT/scripts/example-preview-finalization.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
SOURCE="$TMP/source"
ENTRY="$TMP/entry"
BASELINE="$TMP/baseline"
BUILD="$ROOT/scripts/build-examples.sh"
TARGET='{"expected_artifacts":[{"kind":"source_revision"},{"kind":"source_index"},{"kind":"source_edit_batch"},{"kind":"source_edit_outcome"}]}'
PLAIN_TARGET='{"expected_artifacts":[{"kind":"canonical_project"}]}'

fail() {
  echo "preview finalization contract failed: $*" >&2
  exit 1
}

expect_guard_failure() {
  local label=$1
  if verify_example_publication_guard "$SOURCE" "$ENTRY" "$TARGET" "$GUARD" \
      >/dev/null 2>&1; then
    fail "$label was accepted at the publication boundary"
  fi
}

restore_entry() {
  rm -rf "$ENTRY"
  cp -R "$BASELINE" "$ENTRY"
}

mkdir -p "$SOURCE" "$ENTRY/project"
cat >"$SOURCE/main.veac" <<'VEAC'
import "./brand.veac" as brand;
const time duration = brand.duration;
project demo { entry sequence main; sequence main {} }
VEAC
cat >"$SOURCE/brand.veac" <<'VEAC'
module { export const time duration = 1s; }
VEAC
cat >"$SOURCE/source-edit.json" <<'JSON'
{"schema":"https://veac.dev/schemas/source-edit","schema_version":1,
"operation_id":"op_finalization","base_revision":{"source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
"atomic":true,"preconditions":[],"operations":[{"type":"set_expression",
"target":{"module":"main.veac","path":{"kind":"constant","constant":"duration"}},
"site":{"type":"constant_value"},"expression":{"source":"2s"}}]}
JSON
cp "$SOURCE/main.veac" "$SOURCE/brand.veac" "$ENTRY/project/"
cp "$SOURCE/source-edit.json" "$ENTRY/project/source-edit.json"
cat >"$ENTRY/project/source.revision.json" <<'JSON'
{"source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
JSON
cat >"$ENTRY/project/source.index.json" <<'JSON'
{"schema":"https://veac.dev/schemas/source-index","schema_version":1,
"revision":{"source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
"nodes":[{"target":{"module":"main.veac","path":{"kind":"constant","constant":"duration"}},
"range":{"start":0,"end":1},"expressions":[{"site":{"type":"constant_value"},
"source":"brand.duration","range":{"start":0,"end":1}}]}]}
JSON
cat >"$ENTRY/project/source-edit.outcome.json" <<'JSON'
{"module":"main.veac",
"previous_revision":{"source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
"new_revision":{"source_graph_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},
"destination":null,"dry_run":true}
JSON

GUARD=$(capture_example_publication_guard "$SOURCE" "$ENTRY" "$TARGET") ||
  fail "valid publication evidence was rejected"
verify_example_publication_guard "$SOURCE" "$ENTRY" "$TARGET" "$GUARD" ||
  fail "unchanged publication evidence was rejected"
cp -R "$ENTRY" "$BASELINE"

rm "$ENTRY/project/source.index.json"
expect_guard_failure "missing source index"
restore_entry
printf ' \n' >>"$ENTRY/project/source-edit.outcome.json"
capture_example_publication_guard "$SOURCE" "$ENTRY" "$TARGET" >/dev/null ||
  fail "semantically valid evidence mutation was rejected before digest comparison"
expect_guard_failure "changed evidence digest"
restore_entry
printf '// changed after render\n' >>"$ENTRY/project/brand.veac"
expect_guard_failure "changed staged source module"
restore_entry
printf '// coordinated change\n' >>"$SOURCE/brand.veac"
printf '// coordinated change\n' >>"$ENTRY/project/brand.veac"
expect_guard_failure "changed source graph digest"
cp "$BASELINE/project/brand.veac" "$SOURCE/brand.veac"
restore_entry

PLAIN_ENTRY="$TMP/plain"
mkdir -p "$PLAIN_ENTRY/project"
cp "$SOURCE/main.veac" "$SOURCE/brand.veac" "$PLAIN_ENTRY/project/"
PLAIN_GUARD=$(capture_example_publication_guard "$SOURCE" "$PLAIN_ENTRY" "$PLAIN_TARGET") ||
  fail "example without source edit evidence was rejected"
printf '{}\n' >"$PLAIN_ENTRY/project/source.revision.json"
if verify_example_publication_guard "$SOURCE" "$PLAIN_ENTRY" "$PLAIN_TARGET" \
    "$PLAIN_GUARD" >/dev/null 2>&1; then
  fail "undeclared source edit evidence was accepted"
fi

index_line=$(rg -n '^bash .*write-examples-index[.]sh' "$BUILD" | cut -d: -f1)
guard_line=$(rg -n '^  verify_example_publication_guard ' "$BUILD" | cut -d: -f1)
publish_line=$(rg -n '^publish_preview_staging ' "$BUILD" | cut -d: -f1)
[[ -n $index_line && -n $guard_line && -n $publish_line ]] ||
  fail "publication guard wiring is missing"
((index_line < guard_line && guard_line < publish_line)) ||
  fail "publication guard must run after index generation and before publication"

echo "example preview finalization contracts passed"
