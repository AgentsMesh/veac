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
  jq -e '
    .schema == "https://veac.dev/schemas/source-index" and
    .schema_version == 8 and
    (.build_inputs | type == "array") and
    (.modules | type == "array" and length > 0) and
    (.nodes | type == "array" and length > 0)
  ' "$index" >/dev/null ||
    source_edit_evidence_error "invalid source index v8 contract" || return 1
  jq -e --slurpfile revision "$revision" '.revision == $revision[0]' \
    "$index" >/dev/null ||
    source_edit_evidence_error "source index does not match the revision" || return 1
  jq -e --slurpfile revision "$revision" --slurpfile index "$index" '
    def module_exists($name): any($index[0].modules[]; .module == $name);
    def node_site($operation; $field): any($index[0].nodes[];
      .target == $operation.target and
      any((if $field == "expressions" then .expressions
        elif $field == "statements" then .statements
        elif $field == "bodies" then .bodies else .declarations end)[];
        .site == $operation.site));
    def declaration_exists($target): any($index[0].modules[].declarations[];
      .target == $target);
    def import_exists($target): any($index[0].modules[].imports[];
      .target == $target);
    def anchor_exists($module; $anchor):
      module_exists($module) and
      if $anchor.type == "module_start" or $anchor.type == "module_end" then true
      elif $anchor.type == "before_declaration" or $anchor.type == "after_declaration"
        then $anchor.target.module == $module and declaration_exists($anchor.target)
      elif $anchor.type == "before_import" or $anchor.type == "after_import"
        then $anchor.target.module == $module and import_exists($anchor.target)
      else false end;
    def addressable($operation):
      if $operation.type == "set_expression" then
        node_site($operation; "expressions")
      elif $operation.type == "set_statement" then node_site($operation; "statements")
      elif $operation.type == "set_body" then node_site($operation; "bodies")
      elif $operation.type == "set_declaration" then node_site($operation; "declarations")
      elif $operation.type == "set_top_level_declaration" or
        $operation.type == "remove_declaration" then declaration_exists($operation.target)
      elif $operation.type == "insert_declaration" or $operation.type == "insert_import"
        then anchor_exists($operation.module; $operation.anchor)
      elif $operation.type == "remove_import" then import_exists($operation.target)
      else false end;
    .schema == "https://veac.dev/schemas/source-edit" and
    .schema_version == 6 and .atomic == true and
    .base_revision == $revision[0] and
    (.operations | type == "array" and length > 0) and
    all(.operations[]; addressable(.))
  ' "$batch" >/dev/null ||
    source_edit_evidence_error "source edit batch is not addressable in the index" || return 1
  jq -e --slurpfile revision "$revision" --slurpfile batch "$batch" '
    ([$batch[0].operations[] | (.module // .target.module)] | unique | sort) as $modules |
    (.modules | type == "array" and sort == $modules) and
    .previous_revision == $revision[0] and .new_revision != .previous_revision and
    .destinations == [] and .dry_run == true
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
  local target=$1 source_root=$2 entry=$3 source=$4 veac=$5 inputs=${6:-}
  local declared="$source_root/source-edit.json" batch lock
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
  if [[ -n $inputs ]]; then
    "$veac" source-edit --inputs "$inputs" "$source" "$batch" --dry-run \
      >"$(example_source_edit_outcome "$entry")" || return 1
  else
    "$veac" source-edit "$source" "$batch" --dry-run \
      >"$(example_source_edit_outcome "$entry")" || return 1
  fi
  lock="$(dirname "$source")/.veac-source.lock"
  [[ -f $lock && ! -L $lock ]] ||
    source_edit_evidence_error "source edit dry-run did not retain a regular source lock" || return 1
  verify_declared_source_edit_evidence "$source_root" "$entry"
}
