#!/usr/bin/env bash

WORKFLOW_SHOWCASE_SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
# shellcheck source=/dev/null
source "$WORKFLOW_SHOWCASE_SCRIPT_DIR/example-edit-evidence.sh"
# shellcheck source=/dev/null
source "$WORKFLOW_SHOWCASE_SCRIPT_DIR/example-probe-evidence.sh"

check_keying_stabilization_showcase() {
  local dir=${1:-"$PREVIEW_ROOT/keying-stabilization"}
  [[ -d $dir ]] || return 0
  local author preview plan video
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" preview); video=$(delivery_video_path "$dir" preview preview.mp4)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  workflow_keying_contract "$author" project "$author" authoring
  workflow_keying_contract "$preview" project "$author" preview
  workflow_keying_contract "$plan" plan "$author" plan
  workflow_keying_labels_contract "$author" project "$author" authoring
  workflow_keying_labels_contract "$preview" project "$author" preview
  workflow_keying_labels_contract "$plan" plan "$author" plan
  workflow_keying_input_contract "$plan" "$author"
  workflow_plan_base_contract "$plan" "$author" preview preview.mp4 4 false keying-stabilization
  workflow_assert_keying_media "$video"
}

check_audio_routing_showcase() {
  local dir=${1:-"$PREVIEW_ROOT/audio-routing-ducking"}
  [[ -d $dir ]] || return 0
  local author preview plan video
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" main); video=$(delivery_video_path "$dir" main main.mp4)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  workflow_routing_contract "$author" project "$author" authoring
  workflow_routing_contract "$preview" project "$author" preview
  workflow_routing_contract "$plan" plan "$author" plan
  workflow_routing_label_contract "$author" project "$author" authoring
  workflow_routing_label_contract "$preview" project "$author" preview
  workflow_routing_label_contract "$plan" plan "$author" plan
  workflow_routing_input_contract "$plan" "$author"
  workflow_plan_base_contract "$plan" "$author" main main.mp4 2 true audio-routing-ducking
  workflow_assert_routing_media "$video"
}

check_edit_operations_showcase() {
  local dir=${1:-"$PREVIEW_ROOT/edit-operations"}
  [[ -d $dir ]] || return 0
  local author preview plan video
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" preview); video=$(delivery_video_path "$dir" preview preview.mp4)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  workflow_edit_contract "$author" project "$author" authoring
  workflow_edit_contract "$preview" project "$author" preview
  workflow_edit_contract "$plan" plan "$author" plan
  verify_example_edit_evidence "$dir" "$author" || fail "edit-operations typed edit evidence failed"
  workflow_plan_base_contract "$plan" "$author" preview preview.mp4 4 false edit-operations none
  workflow_assert_edit_media "$video"
}

check_probe_stream_showcase() {
  local dir=${1:-"$PREVIEW_ROOT/probe-stream-selection"}
  [[ -d $dir ]] || return 0
  local author preview plan video
  author=$(example_authoring_canonical "$dir"); preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" preview); video=$(delivery_video_path "$dir" preview preview.mp4)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  workflow_probe_contract "$author" project "$author" authoring
  workflow_probe_contract "$preview" project "$author" preview
  workflow_probe_contract "$plan" plan "$author" plan
  verify_example_probe_evidence "$dir" "$author" "$plan" ||
    fail "probe-stream-selection typed probe evidence failed"
  workflow_plan_base_contract "$plan" "$author" preview preview.mp4 3 false probe-stream-selection
  workflow_assert_probe_media "$video"
}

check_workflow_showcase_evidence() {
  check_keying_stabilization_showcase
  check_audio_routing_showcase
  check_edit_operations_showcase
  check_probe_stream_showcase
}
