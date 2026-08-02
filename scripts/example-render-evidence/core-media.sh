#!/usr/bin/env bash

check_standard_preview_video() {
  local video=$1
  local duration=$2
  local label=$3
  video_contract "$video" "$duration"
  assert_duration_close "$video" "$duration" 0.03 "$label"
  assert_stream_count "$video" v 1 "$label"
  assert_stream_count "$video" a 0 "$label"
  assert_stream_field "$video" v:0 codec_name h264 "$label"
  assert_stream_field "$video" v:0 width 480 "$label"
  assert_stream_field "$video" v:0 height 270 "$label"
  assert_stream_field "$video" v:0 r_frame_rate 12/1 "$label"
}

check_minimal_evidence() {
  local dir="$PREVIEW_ROOT/minimal"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4"
  check_standard_preview_video "$video" 3 minimal
  local time
  for time in 0.1 1.5 2.9; do
    assert_rgb_near "$video" "$time" 27 58 87 4 "minimal at ${time}s"
    assert_uniform_frame "$video" "$time" 2 "minimal at ${time}s"
  done
}

assert_dissolve_progress() {
  local video=$1
  local early_r early_g early_b late_r late_g late_b
  read -r early_r early_g early_b <<< "$(frame_rgb "$video" 1.916)"
  read -r late_r late_g late_b <<< "$(frame_rgb "$video" 2.083)"
  [[ -n ${late_b:-} ]] || fail "transitions dissolve frames could not be sampled"
  ((early_r > late_r && early_g < late_g && early_b < late_b)) \
    || fail "transitions dissolve does not progress monotonically toward blue"
  ((early_r < 226 && early_r > 73 && early_g > 61 && early_g < 119)) \
    || fail "transitions early dissolve frame does not mix both clips"
  ((late_r < 226 && late_r > 73 && late_b > 74 && late_b < 153)) \
    || fail "transitions late dissolve frame does not mix both clips"
}

check_transitions_evidence() {
  local dir="$PREVIEW_ROOT/transitions"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4"
  check_standard_preview_video "$video" 4 transitions
  local plan
  plan=$(example_preview_plan "$dir" out_preview)
  require_file "$plan"
  jq -e '
      [.sequences[].tracks[].transitions[]] as $transitions
      | ($transitions | length) == 1
      and $transitions[0].kind.type == "dissolve"
      and $transitions[0].record_window.start == { value: 1800, timescale: 1000 }
      and $transitions[0].record_window.duration == { value: 400, timescale: 1000 }
    ' "$plan" >/dev/null || fail "transitions plan is not a 400ms dissolve at 1.8s"
  assert_rgb_near "$video" 0.5 230 57 70 4 "transitions red stage"
  assert_rgb_near "$video" 1.7 230 57 70 4 "transitions before dissolve"
  assert_dissolve_progress "$video"
  assert_rgb_near "$video" 2.3 69 123 157 4 "transitions after dissolve"
  assert_rgb_near "$video" 3.5 69 123 157 4 "transitions blue stage"
}
