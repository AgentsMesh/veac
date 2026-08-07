#!/usr/bin/env bash

check_audio_evidence() {
  check_audio_processing_evidence
  check_generated_silence_evidence
  check_all_features_audio_evidence
  check_executable_silence_evidence
  check_nested_multicam_audio_evidence
}

assert_fully_silent_audio() {
  local video=$1 duration=$2 label=$3 mean peak
  read -r mean peak <<< "$(audio_volume_metrics "$video" 0 "$duration" "")"
  awk -v mean="$mean" -v peak="$peak" 'BEGIN { exit !(mean <= -80 && peak <= -80) }' \
    || fail "$label is not fully silent: mean=$mean dB, peak=$peak dB"
}

check_audio_processing_evidence() {
  local dir="$PREVIEW_ROOT/audio-processing"
  [[ -d $dir ]] || return 0
  local video canonical preview plan white
  video=$(delivery_video_path "$dir" preview preview.mp4)
  canonical=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" preview)
  video_contract "$video" 3.9
  audio_stream_contract "$video" 3.9
  assert_stream_field "$video" a:0 sample_rate 48000 "audio-processing"
  assert_stream_field "$video" a:0 channels 2 "audio-processing"
  assert_audio_stream_duration "$video" 4 0.08 "audio-processing"

  assert_non_silent_window "$video" 0.2 1.6 "audio-processing 0-2s"
  assert_non_silent_window "$video" 2.2 1.6 "audio-processing 2-4s"
  assert_channel_level_bias "$video" 0.2 0.35 left 0.7 "audio-processing pan start"
  assert_channel_level_bias "$video" 1.2 0.5 right 3 "audio-processing pan finish"
  assert_audio_window_louder "$video" 1.2 0.5 0.2 0.35 2 "audio-processing gain rise"
  assert_audio_window_louder "$video" 0.4 0.4 0 0.25 5 "audio-processing fade in"
  assert_audio_window_louder "$video" 2.4 0.4 3.75 0.25 2 "audio-processing fade out"
  jq -e --slurpfile canonical "$preview" -f "$SCRIPT_DIR/check-audio-plan.jq" "$plan" >/dev/null \
    || fail "audio-processing plan omits a declared processing mechanism"
  jq -e '
    def seconds: .value/.timescale;
    def clip($key): first(.project.sequences[].tracks[].clips[] |
      select(.authorship.logical_path[-1] == $key));
    clip("voice-label") as $voice | clip("route-label") as $route |
    ($voice.record_range.start|seconds) == 0 and ($voice.record_range.duration|seconds) == 2 and
    ($route.record_range.start|seconds) == 2 and ($route.record_range.duration|seconds) == 2 and
    all($voice,$route; .visual.placement ==
      {"anchor":"bottom","inset":{"x":0,"y":48},"type":"anchor"})
  ' "$canonical" >/dev/null ||
    fail "audio-processing phase labels are missing"
  assert_unique_frames "$video" 2 1 3
  for time in 1 3; do
    white=$(region_color_count "$video" "$time" '440:90:20:150' white)
    ((white > 100)) || fail "audio-processing explanation is missing at ${time}s: white=$white"
  done
}

check_generated_silence_evidence() {
  local dir="$PREVIEW_ROOT/generated-graphics"
  [[ -d $dir ]] || return 0
  local video
  video=$(delivery_video_path "$dir" preview preview.mp4)
  audio_stream_contract "$video" 8.9
  assert_stream_field "$video" a:0 sample_rate 48000 "generated-graphics silence"
  assert_stream_field "$video" a:0 channels 2 "generated-graphics silence"
  assert_audio_stream_duration "$video" 9 0.08 "generated-graphics silence"
  assert_fully_silent_audio "$video" 9 generated-graphics
}

check_all_features_audio_evidence() {
  local dir="$PREVIEW_ROOT/all-features"
  [[ -d $dir ]] || return 0
  local video duration
  video=$(delivery_video_path "$dir" master all-features.mp4)
  duration=$(probe_duration "$video")
  audio_stream_contract "$video" 3.5
  assert_stream_field "$video" a:0 sample_rate 48000 "all-features audio"
  assert_stream_field "$video" a:0 channels 2 "all-features audio"
  assert_audio_stream_duration "$video" "$duration" 0.08 "all-features audio"
  assert_non_silent_window "$video" 0.4 3.0 "all-features processed music"
}

check_executable_silence_evidence() {
  local dir="$PREVIEW_ROOT/executable-mechanisms"
  [[ -d $dir ]] || return 0
  local video
  video=$(delivery_video_path "$dir" preview preview.mp4)
  assert_stream_count "$video" a 0 "executable-mechanisms audio boundary"
}

check_nested_multicam_audio_evidence() {
  local dir="$PREVIEW_ROOT/nested-and-multicam"
  [[ -d $dir ]] || return 0
  local video plan canonical clip group host guest
  canonical=$(example_authoring_canonical "$dir")
  plan=$(delivery_plan_path "$dir" preview)
  video=$(delivery_video_path "$dir" preview preview.mp4)
  require_file "$video"
  require_file "$plan"
  assert_stream_count "$video" a 0 "nested-and-multicam preview"
  clip=$(canonical_clip_id "$canonical" interview-cut)
  read -r group host guest < <(jq -er --arg clip "$clip" '
    first(.project.sequences[].tracks[].clips[] | select(.id==$clip)).source.group_id as $group |
    first(.project.multicam_groups[] | select(.id==$group)) as $value |
    [$group,$value.sync.reference_angle_id,
      first($value.angles[] | select(.id!=$value.sync.reference_angle_id)).id] | @tsv
  ' "$canonical")
  jq -e --arg clip "$clip" --arg group "$group" --arg host "$host" --arg guest "$guest" '
    first(.sequences[].tracks[].clips[] | select(.id == $clip)).source as $tagged |
    $tagged.type == "multicam" and
    ($tagged.source as $source |
      $source.group_id == $group and $source.sync.basis == "audio" and
      $source.sync.reference_angle_id == $host and
      (first($source.angles[] | select(.id == $guest))).source_offset as $offset |
      ($offset.value/$offset.timescale) == 0.12 and
      all($source.angles[]; .audio_stream == null))
  ' "$plan" >/dev/null || fail "nested-and-multicam audio sync plan contract failed"
}
