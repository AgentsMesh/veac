#!/usr/bin/env bash

check_fixture_coverage() {
  local name delivery_id artifact_id time
  while IFS=: read -r name delivery_id artifact_id time; do
    local dir="$PREVIEW_ROOT/$name"
    [[ -d $dir ]] || continue
    local video
    video=$(delivery_video_path "$dir" "$delivery_id" "$artifact_id")
    video_contract "$video" "$time"
    local luma
    luma=$(frame_yavg "$video" "$time")
    awk -v value="$luma" 'BEGIN { exit !(value > 15) }' \
      || fail "$name does not visibly contain its staged fixture"
  done <<'CASES'
all-features:master:master:0.5
speed-demo:preview:preview:1.0
template-fill:preview:preview:1.0
timeline-source-time:preview:preview:1.0
nested-and-multicam:preview:preview:3.0
CASES
}
