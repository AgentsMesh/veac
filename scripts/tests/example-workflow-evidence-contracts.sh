#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
source "$ROOT/scripts/example-workflow-evidence.sh"
source "$ROOT/scripts/tests/example-workflow-evidence-fixtures.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
ENTRY="$TMP/entry"
PROJECT="$ENTRY/project"
AUTHORING="$PROJECT/project.veac.json"
PLAN="$ENTRY/plan.json"
mkdir -p "$PROJECT"

fail() {
  echo "workflow evidence contract failed: $*" >&2
  exit 1
}

expect_edit_failure() {
  local label=$1 authoring=$2 batch=$3 outcome=$4 replay=$5
  if jq -e --slurpfile authoring "$authoring" --slurpfile batch "$batch" \
      --slurpfile replay "$replay" -f "$EDIT_EVIDENCE_CHECKER" "$outcome" \
      >/dev/null 2>&1; then
    fail "$label edit evidence was accepted"
  fi
}

expect_probe_failure() {
  local label=$1 authoring=$2 plan=$3 snapshot=$4
  if jq -e --slurpfile authoring "$authoring" --slurpfile plan "$plan" \
      -f "$PROBE_EVIDENCE_CHECKER" "$snapshot" >/dev/null 2>&1; then
    fail "$label probe evidence was accepted"
  fi
}

write_workflow_authoring_fixture "$AUTHORING"
jq -ec -f "$EDIT_BATCH_FILTER" "$AUTHORING" >"$(example_edit_batch "$ENTRY")"
write_edit_outcome_fixtures "$AUTHORING" "$(example_edit_batch "$ENTRY")" \
  "$(example_edit_outcome "$ENTRY")" "$(example_edit_replay_outcome "$ENTRY")"
write_probe_evidence_fixtures "$(example_probe_snapshot "$ENTRY")" "$PLAN"
verify_example_edit_evidence "$ENTRY" "$AUTHORING" || fail "valid edit evidence"
verify_example_probe_evidence "$ENTRY" "$AUTHORING" "$PLAN" || fail "valid probe evidence"

jq '(.project.project.sequences[].tracks[].clips[] |
  select(.id == "itm_second").record_range.start.value) = 1400' \
  "$(example_edit_outcome "$ENTRY")" >"$TMP/bad-move.json"
expect_edit_failure bad-group-move "$AUTHORING" "$(example_edit_batch "$ENTRY")" \
  "$TMP/bad-move.json" "$(example_edit_replay_outcome "$ENTRY")"
jq 'del(.project.project.relations[] |
  select(.id == "rel_workflow_linked_av_right"))' \
  "$(example_edit_outcome "$ENTRY")" >"$TMP/bad-split.json"
expect_edit_failure missing-linked-split "$AUTHORING" "$(example_edit_batch "$ENTRY")" \
  "$TMP/bad-split.json" "$(example_edit_replay_outcome "$ENTRY")"
jq '.atomic = false' "$(example_edit_batch "$ENTRY")" >"$TMP/bad-batch.json"
expect_edit_failure non-atomic "$AUTHORING" "$TMP/bad-batch.json" \
  "$(example_edit_outcome "$ENTRY")" "$(example_edit_replay_outcome "$ENTRY")"
jq 'del(.preconditions[] |
  select(.type == "clip_exists" and .clip_id == "itm_second"))' \
  "$(example_edit_batch "$ENTRY")" >"$TMP/missing-second.json"
expect_edit_failure missing-group-member-precondition "$AUTHORING" "$TMP/missing-second.json" \
  "$(example_edit_outcome "$ENTRY")" "$(example_edit_replay_outcome "$ENTRY")"
jq '.operation_recorded = false' "$(example_edit_replay_outcome "$ENTRY")" \
  >"$TMP/bad-replay.json"
expect_edit_failure unrecorded-replay "$AUTHORING" "$(example_edit_batch "$ENTRY")" \
  "$(example_edit_outcome "$ENTRY")" "$TMP/bad-replay.json"

jq '.observed_identity.digest = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"' \
  "$(example_probe_snapshot "$ENTRY")" >"$TMP/bad-identity.json"
expect_probe_failure wrong-identity "$AUTHORING" "$PLAN" "$TMP/bad-identity.json"
jq '.streams |= map(select(.media_type != "audio"))' \
  "$(example_probe_snapshot "$ENTRY")" >"$TMP/no-audio.json"
expect_probe_failure missing-source-audio "$AUTHORING" "$PLAN" "$TMP/no-audio.json"
jq '.selected_audio_stream = {global_index:1,type_index:0}' \
  "$(example_probe_snapshot "$ENTRY")" >"$TMP/selected-audio.json"
expect_probe_failure selected-disabled-audio "$AUTHORING" "$PLAN" "$TMP/selected-audio.json"
jq '.streams[0].video.cadence = "invalid"' \
  "$(example_probe_snapshot "$ENTRY")" >"$TMP/invalid-cadence.json"
expect_probe_failure invalid-cadence "$AUTHORING" "$PLAN" "$TMP/invalid-cadence.json"
jq '.inputs[0].audio = {selection:{global_index:1}}' "$PLAN" >"$TMP/bad-plan.json"
expect_probe_failure enabled-plan-audio "$AUTHORING" "$TMP/bad-plan.json" \
  "$(example_probe_snapshot "$ENTRY")"

TARGET='{"expected_artifacts":[{"kind":"edit_batch"},{"kind":"edit_outcome"},{"kind":"edit_replay_outcome"},{"kind":"probe_snapshot"}]}'
[[ $(workflow_evidence_oid "$TARGET" "$ENTRY") != none ]] || fail "missing evidence digest"
PLAIN='{"expected_artifacts":[{"kind":"canonical_project"}]}'
workflow_evidence_oid "$PLAIN" "$ENTRY" >/dev/null 2>&1 &&
  fail "undeclared workflow evidence was accepted"

echo "example workflow evidence contracts passed"
