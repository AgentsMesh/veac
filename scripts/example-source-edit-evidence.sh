#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout.sh"

source_edit_evidence_error() {
  echo "examples preview: $*" >&2
  return 1
}

source_edit_evidence_requested() {
  jq -e 'any(.expected_artifacts[];
    .kind == "source_revision" or .kind == "source_index" or
    .kind == "source_edit_batch" or .kind == "source_edit_outcome")' \
    <<<"$1" >/dev/null
}

verify_source_edit_json() {
  local file=$1 label=$2
  require_preview_regular_file "$file" "$label" || return 1
  jq -e 'type == "object"' "$file" >/dev/null ||
    source_edit_evidence_error "invalid $label JSON: $file"
}

verify_example_source_edit_evidence() {
  local entry=$1 revision index batch outcome
  revision=$(example_source_revision "$entry")
  index=$(example_source_index "$entry")
  batch=$(example_source_edit_batch "$entry")
  outcome=$(example_source_edit_outcome "$entry")
  verify_source_edit_json "$revision" "source revision" || return 1
  verify_source_edit_json "$index" "source index" || return 1
  verify_source_edit_json "$batch" "source edit batch" || return 1
  verify_source_edit_json "$outcome" "source edit outcome" || return 1
  jq -e '
    keys == ["source_graph_sha256"] and
    (.source_graph_sha256 | test("^[0-9a-f]{64}$"))
  ' "$revision" >/dev/null ||
    source_edit_evidence_error "invalid source revision contract" || return 1
  jq -e --slurpfile revision "$revision" '
    .schema == "https://veac.dev/schemas/source-index" and
    .schema_version == 1 and .revision == $revision[0] and
    (.nodes | type == "array" and length > 0)
  ' "$index" >/dev/null ||
    source_edit_evidence_error "source index does not match the revision" || return 1
  jq -e --slurpfile revision "$revision" --slurpfile index "$index" '
    .schema == "https://veac.dev/schemas/source-edit" and
    .schema_version == 1 and .atomic == true and
    .base_revision == $revision[0] and
    (.operations | type == "array" and length > 0) and
    all(.operations[]; . as $operation |
      any($index[0].nodes[];
        .target == $operation.target and
        any(.expressions[]; .site == $operation.site)))
  ' "$batch" >/dev/null ||
    source_edit_evidence_error "source edit batch is not addressable in the index" || return 1
  jq -e --slurpfile revision "$revision" --slurpfile batch "$batch" '
    .module as $module |
    ($module | type == "string" and length > 0) and
    all($batch[0].operations[]; .target.module == $module) and
    .previous_revision == $revision[0] and .new_revision != .previous_revision and
    .destination == null and .dry_run == true
  ' "$outcome" >/dev/null ||
    source_edit_evidence_error "source edit dry-run outcome is inconsistent"
}

source_edit_evidence_oid() {
  local entry=$1 file
  for file in "$(example_source_revision "$entry")" "$(example_source_index "$entry")" \
      "$(example_source_edit_batch "$entry")" "$(example_source_edit_outcome "$entry")"; do
    git hash-object --no-filters "$file" || return 1
  done | git hash-object --stdin
}

verify_declared_source_edit_evidence() {
  local source_root=$1 entry=$2
  cmp -s "$source_root/source-edit.json" "$(example_source_edit_batch "$entry")" ||
    source_edit_evidence_error "published source edit batch drifted" || return 1
  verify_example_source_edit_evidence "$entry"
}

build_example_source_edit_evidence() {
  local target=$1 source_root=$2 entry=$3 source=$4 veac=$5
  local declared="$source_root/source-edit.json" batch
  if ! source_edit_evidence_requested "$target"; then
    [[ ! -e $declared && ! -L $declared ]] ||
      source_edit_evidence_error "undeclared source edit batch: $declared"
    return
  fi
  [[ -f $declared && ! -L $declared ]] ||
    source_edit_evidence_error "missing regular source edit batch: $declared" || return 1
  batch=$(example_source_edit_batch "$entry")
  cp "$declared" "$batch" || return 1
  "$veac" source-revision "$source" >"$(example_source_revision "$entry")" || return 1
  "$veac" source-index "$source" >"$(example_source_index "$entry")" || return 1
  "$veac" source-edit "$source" "$batch" --dry-run \
    >"$(example_source_edit_outcome "$entry")" || return 1
  [[ ! -e "$(dirname "$source")/.veac-source.lock" ]] ||
    source_edit_evidence_error "source edit dry-run left a source lock" || return 1
  verify_declared_source_edit_evidence "$source_root" "$entry"
}
