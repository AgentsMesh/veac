#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/veac-workflow-showcase-contracts.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
for module in common workflow-showcase-core workflow-keying workflow-routing workflow-operations; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/workflow-keying-routing-render-fixtures.sh"
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/workflow-operations-render-fixtures.sh"

expect_failure() {
  local label=$1 expected=$2; shift 2
  local log="$TMP/$label.log"
  if ("$@") >"$log" 2>&1; then
    echo "$label unexpectedly passed" >&2; exit 1
  fi
  rg -qF "$expected" "$log" || {
    cat "$log" >&2; echo "$label failed for the wrong reason" >&2; exit 1;
  }
}

json_variant() {
  local label=$1 source=$2 filter=$3 expected=$4 checker=$5 mode=$6 identity=$7
  local output="$TMP/$label.json"
  jq "$filter" "$source" >"$output"
  expect_failure "$label" "$expected" "$checker" "$output" "$mode" "$identity" "$label"
}

make_valid_fixtures() {
  write_keying_contract_fixtures "$TMP"
  write_routing_contract_fixtures "$TMP"
  write_edit_contract_fixtures "$TMP"
  write_probe_contract_fixtures "$TMP"
  workflow_keying_contract "$TMP/keying-author.json" project "$TMP/keying-author.json" valid
  workflow_keying_contract "$TMP/keying-plan.json" plan "$TMP/keying-author.json" valid
  workflow_keying_labels_contract "$TMP/keying-plan.json" plan "$TMP/keying-author.json" valid
  workflow_keying_input_contract "$TMP/keying-plan.json" "$TMP/keying-author.json"
  workflow_routing_contract "$TMP/routing-author.json" project "$TMP/routing-author.json" valid
  workflow_routing_contract "$TMP/routing-plan.json" plan "$TMP/routing-author.json" valid
  workflow_routing_label_contract "$TMP/routing-plan.json" plan "$TMP/routing-author.json" valid
  workflow_routing_input_contract "$TMP/routing-plan.json" "$TMP/routing-author.json"
  workflow_edit_contract "$TMP/edit-author.json" project "$TMP/edit-author.json" valid
  workflow_edit_contract "$TMP/edit-plan.json" plan "$TMP/edit-author.json" valid
  workflow_plan_base_contract "$TMP/edit-plan.json" "$TMP/edit-author.json" \
    preview preview.mp4 4 false edit-fixture none
  workflow_probe_contract "$TMP/probe-author.json" project "$TMP/probe-author.json" valid
  workflow_probe_contract "$TMP/probe-plan.json" plan "$TMP/probe-author.json" valid
}

make_valid_fixtures
KEY_AUTHOR="$TMP/keying-author.json"; KEY_PLAN="$TMP/keying-plan.json"
ROUTE_AUTHOR="$TMP/routing-author.json"; ROUTE_PLAN="$TMP/routing-plan.json"
EDIT_AUTHOR="$TMP/edit-author.json"; EDIT_PLAN="$TMP/edit-plan.json"
PROBE_AUTHOR="$TMP/probe-author.json"; PROBE_PLAN="$TMP/probe-plan.json"

json_variant key_similarity "$KEY_AUTHOR" \
  '(.project.sequences[].tracks[].clips[]|select(.id=="itm_chroma").effects[0].effect.similarity.value)=0.2' \
  'typed contract failed' workflow_keying_contract project "$KEY_AUTHOR"
json_variant missing_stabilize "$KEY_PLAN" \
  '(.sequences[].tracks[].clips[]|select(.id=="itm_stable").effects)=[]' \
  'typed contract failed' workflow_keying_contract plan "$KEY_AUTHOR"
jq '(.inputs[]|select(.material_id=="med_plate").canonical_uri)="assets/wrong.png"' \
  "$KEY_PLAN" >"$TMP/key_input.json"
expect_failure key_input 'resolved input contract failed' \
  workflow_keying_input_contract "$TMP/key_input.json" "$KEY_AUTHOR"
json_variant key_label "$KEY_PLAN" \
  '(.sequences[].tracks[].clips[]|select(.id=="itm_key_label").source.content.text)="wrong"' \
  'Chinese labels contract failed' workflow_keying_labels_contract plan "$KEY_AUTHOR"

json_variant wrong_bus "$ROUTE_AUTHOR" \
  '(.project.sequences[].tracks[]|select(.id=="trk_music").routing.bus_id)="bus_wrong"' \
  'typed contract failed' workflow_routing_contract project "$ROUTE_AUTHOR"
json_variant wrong_sidechain_ratio "$ROUTE_PLAN" \
  '(.sequences[].tracks[].clips[]|select(.id=="itm_music").audio.sidechain.ratio)=8' \
  'typed contract failed' workflow_routing_contract plan "$ROUTE_AUTHOR"
json_variant wrong_audio_fade "$ROUTE_PLAN" \
  '(.sequences[].tracks[].clips[]|select(.id=="itm_key").audio.crossfade.fade_out.value)=60' \
  'typed contract failed' workflow_routing_contract plan "$ROUTE_AUTHOR"
jq '(.inputs[]|select(.material_id=="med_tone").audio.selection.global_index)=1' \
  "$ROUTE_PLAN" >"$TMP/route_input.json"
expect_failure route_input 'resolved input contract failed' \
  workflow_routing_input_contract "$TMP/route_input.json" "$ROUTE_AUTHOR"

json_variant missing_group_member "$EDIT_AUTHOR" \
  '(.project.relations[]|select(.kind.type=="group").kind.members)|=.[0:1]' \
  'typed contract failed' workflow_edit_contract project "$EDIT_AUTHOR"
jq --slurpfile author "$EDIT_AUTHOR" '
  (.sequences[].tracks[]|select(.id=="trk_remove").clips) +=
    [$author[0].project.sequences[].tracks[].clips[]|select(.id=="itm_remove")]' \
  "$EDIT_PLAN" >"$TMP/removed_reappears.json"
expect_failure removed_reappears 'typed contract failed' \
  workflow_edit_contract "$TMP/removed_reappears.json" plan "$EDIT_AUTHOR" removed_reappears
json_variant wrong_scene_time "$EDIT_PLAN" \
  '(.sequences[].tracks[].clips[]|select(.id=="itm_second").record_range.start.value)=900' \
  'typed contract failed' workflow_edit_contract plan "$EDIT_AUTHOR"
jq '.output.deliverables[0].kind.settings.video.rate_control.value=21' \
  "$EDIT_PLAN" >"$TMP/wrong_output.json"
expect_failure wrong_output 'render-plan envelope contract failed' \
  workflow_plan_base_contract "$TMP/wrong_output.json" "$EDIT_AUTHOR" \
  preview preview.mp4 4 false wrong-output none
jq '.sequences[0].settings.frame_rate.numerator=30' "$EDIT_PLAN" >"$TMP/wrong_plan_fps.json"
expect_failure wrong_plan_fps 'render-plan envelope contract failed' \
  workflow_plan_base_contract "$TMP/wrong_plan_fps.json" "$EDIT_AUTHOR" \
  preview preview.mp4 4 false wrong-fps none

json_variant enabled_probe_audio "$PROBE_AUTHOR" \
  '(.project.materials[]|select(.id=="med_source").stream_intent.audio)={type:"auto"}' \
  'typed contract failed' workflow_probe_contract project "$PROBE_AUTHOR"
json_variant wrong_probe_stream "$PROBE_PLAN" \
  '(.sequences[].tracks[].clips[]|select(.id=="itm_stream").source.video_stream.global_index)=1' \
  'typed contract failed' workflow_probe_contract plan "$PROBE_AUTHOR"
json_variant probe_label "$PROBE_PLAN" \
  '(.sequences[].tracks[].clips[]|select(.id=="itm_text").source.content.text)="wrong"' \
  'typed contract failed' workflow_probe_contract plan "$PROBE_AUTHOR"

echo 'workflow showcase render evidence contracts passed'
