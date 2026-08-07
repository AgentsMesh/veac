#!/usr/bin/env bash

executable_clip_id() {
  local canonical=$1 track=$2 key=$3
  jq -er --arg track "$track" --arg key "$key" '
    [.project.sequences[].tracks[].clips[] |
      select(.authorship.logical_path[-3] == $track and
        .authorship.logical_path[-1] == $key) | .id] |
    if length == 1 then .[0] else error("executable clip is not unique") end
  ' "$canonical" || fail "cannot resolve executable clip: $track/$key"
}

executable_material_id() {
  local canonical=$1 key=$2
  jq -er --arg key "$key" '
    [.project.materials[] |
      select(.authorship.logical_path[-1] == $key) | .id] |
    if length == 1 then .[0] else error("executable material is not unique") end
  ' "$canonical" || fail "cannot resolve executable material: $key"
}

executable_relation_id() {
  local canonical=$1 key=$2
  jq -er --arg key "$key" '
    [.project.sequences[].authorship.relations[] |
      select(.entity.logical_path[-1] == $key) | .relation_id] |
    if length == 1 then .[0] else error("executable relation is not unique") end
  ' "$canonical" || fail "cannot resolve executable relation: $key"
}

executable_preview_artifacts() {
  local dir=$1 canonical preview plan video
  canonical=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" preview)
  video=$(delivery_video_path "$dir" preview preview.mp4)
  require_file "$canonical"; require_file "$preview"; require_file "$plan"; require_file "$video"
  printf '%s\t%s\t%s\t%s\n' "$canonical" "$preview" "$plan" "$video"
}

assert_executable_rgb() {
  local video=$1 time=$2 crop=$3 expected=$4 tolerance=$5 label=$6 actual
  actual=$(region_rgb "$video" "$time" "$crop")
  local red green blue want_red want_green want_blue
  read -r red green blue <<<"$actual"
  read -r want_red want_green want_blue <<<"$expected"
  [[ -n ${blue:-} ]] || fail "$label: RGB sample is unavailable"
  awk -v r="$red" -v g="$green" -v b="$blue" -v er="$want_red" \
    -v eg="$want_green" -v eb="$want_blue" -v t="$tolerance" '
    function abs(value) { return value < 0 ? -value : value }
    BEGIN { exit !(abs(r-er)<=t && abs(g-eg)<=t && abs(b-eb)<=t) }
  ' || fail "$label: got $actual, expected $expected +/- $tolerance"
}

assert_executable_silence() {
  local video=$1 start=$2 duration=$3 label=$4 level
  level=$(audio_mean_db "$video" "$start" "$duration" '')
  awk -v level="$level" 'BEGIN { exit !(level <= -75) }' ||
    fail "$label: expected silence, measured ${level}dB"
}

assert_executable_gt() {
  local value=$1 baseline=$2 gap=$3 label=$4
  awk -v value="$value" -v baseline="$baseline" -v gap="$gap" \
    'BEGIN { exit !(value > baseline + gap) }' ||
    fail "$label: $value is not greater than $baseline + $gap"
}

assert_executable_video() {
  local video=$1 duration=$2 width=$3 height=$4 audio=$5 label=$6
  video_contract "$video" "$duration"
  assert_duration_close "$video" "$duration" 0.03 "$label"
  assert_stream_count "$video" v 1 "$label"
  assert_stream_count "$video" a "$audio" "$label"
  assert_stream_field "$video" v:0 codec_name h264 "$label"
  assert_stream_field "$video" v:0 width "$width" "$label"
  assert_stream_field "$video" v:0 height "$height" "$label"
  assert_stream_field "$video" v:0 r_frame_rate 12/1 "$label"
}
