#!/usr/bin/env bash

WORKFLOW_EVIDENCE_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$WORKFLOW_EVIDENCE_DIR/example-edit-evidence.sh"
source "$WORKFLOW_EVIDENCE_DIR/example-probe-evidence.sh"

workflow_evidence_requested() {
  edit_evidence_requested "$1" || probe_evidence_requested "$1"
}

build_example_workflow_evidence() {
  local target=$1 entry=$2 authoring=$3 veac=$4
  if edit_evidence_requested "$target"; then
    build_example_edit_evidence "$entry" "$authoring" "$veac" || return 1
  fi
  if probe_evidence_requested "$target"; then
    build_example_probe_evidence "$entry" "$authoring" "$veac" || return 1
  fi
}

verify_example_workflow_evidence() {
  local target=$1 entry=$2 authoring=$3 plan=$4
  if edit_evidence_requested "$target"; then
    verify_example_edit_evidence "$entry" "$authoring" || return 1
  fi
  if probe_evidence_requested "$target"; then
    verify_example_probe_evidence "$entry" "$authoring" "$plan" || return 1
  fi
}

workflow_evidence_path() {
  local entry=$1 kind=$2
  case "$kind" in
    edit_batch) example_edit_batch "$entry" ;;
    edit_outcome) example_edit_outcome "$entry" ;;
    edit_replay_outcome) example_edit_replay_outcome "$entry" ;;
    probe_snapshot) example_probe_snapshot "$entry" ;;
    *) return 1 ;;
  esac
}

workflow_evidence_oid() {
  local target=$1 entry=$2 kind path blob manifest="" declared
  declared=$(jq -r '.expected_artifacts[].kind |
    select(. == "edit_batch" or . == "edit_outcome" or
      . == "edit_replay_outcome" or . == "probe_snapshot")' <<<"$target") || return 1
  for kind in edit_batch edit_outcome edit_replay_outcome probe_snapshot; do
    path=$(workflow_evidence_path "$entry" "$kind") || return 1
    if grep -qx "$kind" <<<"$declared"; then
      require_preview_regular_file "$path" "$kind" || return 1
      jq -e 'type == "object"' "$path" >/dev/null || return 1
      blob=$(git hash-object --no-filters "$path") || return 1
      manifest+="$kind"$'\t'"$blob"$'\n'
    elif [[ -e $path || -L $path ]]; then
      echo "examples preview: undeclared workflow evidence: $path" >&2
      return 1
    fi
  done
  if [[ -n $manifest ]]; then printf '%s' "$manifest" | git hash-object --stdin; else echo none; fi
}
