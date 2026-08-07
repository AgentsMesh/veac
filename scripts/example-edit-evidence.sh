#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout.sh"
EDIT_EVIDENCE_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
EDIT_BATCH_FILTER="$EDIT_EVIDENCE_DIR/example-edit-batch.jq"
EDIT_EVIDENCE_CHECKER="$EDIT_EVIDENCE_DIR/check-example-edit-evidence.jq"

edit_evidence_error() {
  echo "examples preview: $*" >&2
  return 1
}

edit_evidence_requested() {
  jq -e 'any(.expected_artifacts[];
    .kind == "edit_batch" or .kind == "edit_outcome" or
    .kind == "edit_replay_outcome")' <<<"$1" >/dev/null
}

verify_edit_evidence_files() {
  local entry=$1
  require_preview_regular_file "$(example_edit_batch "$entry")" "edit batch" || return 1
  require_preview_regular_file "$(example_edit_outcome "$entry")" "edit outcome" || return 1
  require_preview_regular_file "$(example_edit_replay_outcome "$entry")" \
    "edit replay outcome" || return 1
}

verify_example_edit_evidence() {
  local entry=$1 authoring=$2 batch outcome replay
  batch=$(example_edit_batch "$entry")
  outcome=$(example_edit_outcome "$entry")
  replay=$(example_edit_replay_outcome "$entry")
  verify_edit_evidence_files "$entry" || return 1
  jq -e --slurpfile authoring "$authoring" --slurpfile batch "$batch" \
    --slurpfile replay "$replay" -f "$EDIT_EVIDENCE_CHECKER" "$outcome" >/dev/null ||
    edit_evidence_error "typed edit evidence contract failed: $entry"
}

build_example_edit_evidence() {
  local entry=$1 authoring=$2 veac=$3 batch outcome replay applied before after result=0
  batch=$(example_edit_batch "$entry")
  outcome=$(example_edit_outcome "$entry")
  replay=$(example_edit_replay_outcome "$entry")
  applied=$(mktemp "$entry/.edit-applied.XXXXXX.json") || return 1
  before=$(git hash-object --no-filters "$authoring") || result=1
  if [[ $result -eq 0 ]]; then
    echo "[edit-operations] typed atomic dry-run -> idempotent replay"
    jq -ec -f "$EDIT_BATCH_FILTER" "$authoring" >"$batch" || result=1
  fi
  if [[ $result -eq 0 ]]; then
    "$veac" edit --dry-run "$authoring" "$batch" >"$outcome" || result=1
  fi
  if [[ $result -eq 0 ]]; then
    jq -ec '.project' "$outcome" >"$applied" || result=1
    "$veac" edit --dry-run "$applied" "$batch" >"$replay" || result=1
  fi
  after=$(git hash-object --no-filters "$authoring") || result=1
  rm -f "$applied"
  [[ $result -eq 0 && $before == "$after" ]] ||
    edit_evidence_error "edit dry-run mutated authoring canonical project" || return 1
  verify_example_edit_evidence "$entry" "$authoring"
}
