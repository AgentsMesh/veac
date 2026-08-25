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
TARGET='{"expected_artifacts":[{"kind":"source_revision"},{"kind":"source_index"},{"kind":"source_edit_batch"},{"kind":"source_edit_outcome"},{"kind":"edit_batch"},{"kind":"edit_outcome"},{"kind":"edit_replay_outcome"},{"kind":"probe_snapshot"}]}'
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
fn main(context: Context) -> Project {
  project(identifier("demo"), project_settings(600))
}
VEAC
cat >"$SOURCE/brand.veac" <<'VEAC'
module { export const time duration = 1s; }
VEAC
cat >"$SOURCE/source-edit.json" <<'JSON'
{"schema":"https://veac.dev/schemas/source-edit","schema_version":9,
"operation_id":"op_finalization","base_revision":{"authored_source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","complete_source_graph_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"},
"atomic":true,"preconditions":[],"operations":[{"type":"set_expression",
"target":{"module":"main.veac","path":{"kind":"constant","constant":"duration"}},
"site":{"type":"constant_value"},"expression":{"source":"2s"}}]}
JSON
cp "$SOURCE/main.veac" "$SOURCE/brand.veac" "$ENTRY/project/"
cp "$SOURCE/source-edit.json" "$ENTRY/project/source-edit.json"
cat >"$ENTRY/project/source.revision.json" <<'JSON'
{"authored_source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","complete_source_graph_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}
JSON
cat >"$ENTRY/project/source.index.json" <<'JSON'
{"schema":"https://veac.dev/schemas/source-index","schema_version":11,"build_inputs":[],
"revision":{"authored_source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","complete_source_graph_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"},
"modules":[{"module":"main.veac","range":{"start":0,"end":1},"imports":[],
"declarations":[{"target":{"module":"main.veac","path":{"kind":"constant","constant":"duration"}},
"source":"const time duration = brand.duration;","range":{"start":0,"end":1}}]}],
"nodes":[{"target":{"module":"main.veac","path":{"kind":"constant","constant":"duration"}},
"range":{"start":0,"end":1},"expressions":[{"site":{"type":"constant_value"},
"source":"brand.duration","range":{"start":0,"end":1}}],"statements":[],"bodies":[],"declarations":[]}]}
JSON
cat >"$ENTRY/project/source-edit.outcome.json" <<'JSON'
{"modules":["main.veac"],
"previous_revision":{"authored_source_graph_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","complete_source_graph_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"},
"new_revision":{"authored_source_graph_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","complete_source_graph_sha256":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"},
"destinations":[],"dry_run":true}
JSON
for evidence in edit.batch.json edit.outcome.json edit.replay.outcome.json \
    probe.snapshot.json; do
  printf '{}\n' >"$ENTRY/project/$evidence"
done

GUARD=$(capture_example_publication_guard "$SOURCE" "$ENTRY" "$TARGET") ||
  fail "valid publication evidence was rejected"
verify_example_publication_guard "$SOURCE" "$ENTRY" "$TARGET" "$GUARD" ||
  fail "unchanged publication evidence was rejected"
cp -R "$ENTRY" "$BASELINE"

rm "$ENTRY/project/source.index.json"
expect_guard_failure "missing source index"
restore_entry
jq '.schema_version = 10' "$ENTRY/project/source.index.json" >"$TMP/source.index.json"
mv "$TMP/source.index.json" "$ENTRY/project/source.index.json"
expect_guard_failure "legacy source index"
restore_entry
jq 'del(.build_inputs)' "$ENTRY/project/source.index.json" >"$TMP/source.index.json"
mv "$TMP/source.index.json" "$ENTRY/project/source.index.json"
expect_guard_failure "source index without Build input inventory"
restore_entry
rm "$ENTRY/project/probe.snapshot.json"
expect_guard_failure "missing probe snapshot"
restore_entry
printf ' \n' >>"$ENTRY/project/edit.outcome.json"
capture_example_publication_guard "$SOURCE" "$ENTRY" "$TARGET" >/dev/null ||
  fail "valid workflow evidence mutation was rejected before digest comparison"
expect_guard_failure "changed workflow evidence digest"
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
rm "$PLAIN_ENTRY/project/source.revision.json"
printf '{}\n' >"$PLAIN_ENTRY/project/edit.batch.json"
if verify_example_publication_guard "$SOURCE" "$PLAIN_ENTRY" "$PLAIN_TARGET" \
    "$PLAIN_GUARD" >/dev/null 2>&1; then
  fail "undeclared workflow evidence was accepted"
fi

index_line=$(rg -n '^bash .*write-examples-index[.]sh' "$BUILD" | cut -d: -f1)
guard_line=$(rg -n '^  verify_example_publication_guard ' "$BUILD" | cut -d: -f1)
publish_line=$(rg -n '^publish_preview_staging ' "$BUILD" | cut -d: -f1)
[[ -n $index_line && -n $guard_line && -n $publish_line ]] ||
  fail "publication guard wiring is missing"
((index_line < guard_line && guard_line < publish_line)) ||
  fail "publication guard must run after index generation and before publication"

echo "example preview finalization contracts passed"
