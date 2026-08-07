#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-source-graph.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-source-edit-evidence.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-workflow-evidence.sh"

preview_finalization_error() {
  echo "examples preview: $*" >&2
  return 1
}

reject_preview_source_symlinks() {
  local source_root=$1 entry=$2
  if find "$source_root" -type l -print -quit | grep -q .; then
    preview_finalization_error "example source graph contains a symlink: $source_root"
    return 1
  fi
  if find "$entry/project" -type l -name '*.veac' -print -quit | grep -q .; then
    preview_finalization_error "staged source graph contains a symlink: $entry/project"
    return 1
  fi
}

reject_undeclared_source_edit_evidence() {
  local entry=$1 file
  for file in "$(example_source_revision "$entry")" "$(example_source_index "$entry")" \
      "$(example_source_edit_batch "$entry")" "$(example_source_edit_outcome "$entry")"; do
    [[ ! -e $file && ! -L $file ]] || {
      preview_finalization_error "undeclared source edit evidence: $file"
      return 1
    }
  done
}

example_source_graph_oid() {
  local source_root=$1 path relative blob manifest=""
  while IFS= read -r path; do
    relative=${path#"$source_root"/}
    blob=$(git hash-object --no-filters "$path") || return 1
    manifest+="$relative"$'\t'"$blob"$'\n'
  done < <(find "$source_root" -type f -name '*.veac' | LC_ALL=C sort)
  [[ -n $manifest ]] || return 1
  printf '%s' "$manifest" | git hash-object --stdin
}

capture_example_publication_guard() {
  local source_root=$1 entry=$2 target=$3 graph_oid evidence_oid=none workflow_oid
  reject_preview_source_symlinks "$source_root" "$entry" || return 1
  verify_example_source_graph "$source_root" "$entry/project" || return 1
  graph_oid=$(example_source_graph_oid "$source_root") || return 1
  if source_edit_evidence_requested "$target"; then
    verify_declared_source_edit_evidence "$source_root" "$entry" || return 1
    evidence_oid=$(source_edit_evidence_oid "$entry") || return 1
  else
    reject_undeclared_source_edit_evidence "$entry" || return 1
  fi
  workflow_oid=$(workflow_evidence_oid "$target" "$entry") || return 1
  printf 'source-graph:%s;source-edit:%s;workflow:%s\n' \
    "$graph_oid" "$evidence_oid" "$workflow_oid"
}

verify_example_publication_guard() {
  local source_root=$1 entry=$2 target=$3 expected=$4 actual
  actual=$(capture_example_publication_guard "$source_root" "$entry" "$target") || return 1
  [[ $actual == "$expected" ]] ||
    preview_finalization_error "example publication evidence changed: $entry"
}
