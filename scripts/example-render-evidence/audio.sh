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
  local video
  video=$(delivery_video_path "$dir" preview preview)
  video_contract "$video" 7.5
  audio_stream_contract "$video" 7.5
  assert_stream_field "$video" a:0 sample_rate 48000 "audio-processing"
  assert_stream_field "$video" a:0 channels 2 "audio-processing"
  assert_audio_stream_duration "$video" 8 0.08 "audio-processing"

  assert_non_silent_window "$video" 0.2 1.6 "audio-processing 0-2s"
  assert_non_silent_window "$video" 2.2 1.6 "audio-processing 2-4s"
  assert_non_silent_window "$video" 4.2 1.6 "audio-processing 4-6s"
  assert_non_silent_window "$video" 6.2 1.6 "audio-processing 6-8s"
  assert_channel_tone_bias "$video" 0.2 1.6 440 left 4 \
    "audio-processing dialogue pan"
  assert_channel_tone_bias "$video" 0.2 1.6 1200 right 4 \
    "audio-processing key pan"
  assert_tone_dominates "$video" 2.2 1.6 880 440 6 \
    "audio-processing follow-speed pitch"
  assert_tone_dominates "$video" 4.2 1.6 660 440 7 \
    "audio-processing loudness phase frequency"
  assert_tone_dominates "$video" 6.2 1.6 440 660 7 \
    "audio-processing normalize effect frequency"
  assert_loudness_target "$video" 4 2 -16 2.5 -25 -0.5 \
    "audio-processing loudness processor"
  assert_loudness_target "$video" 6 2 -18 2.5 -28 -0.5 \
    "audio-processing normalize effect"
  jq -e -f "$SCRIPT_DIR/check-audio-plan.jq" \
    "$(example_preview_plan "$dir" out_preview)" >/dev/null \
    || fail "audio-processing plan omits a declared processing mechanism"
  assert_unique_frames "$video" 4 0.5 2.5 4.5 6.5
  local visual
  visual=$(frame_yavg "$video" 1.0)
  awk -v value="$visual" 'BEGIN { exit !(value > 18) }' \
    || fail "audio-processing monitor is not visible"
}

check_generated_silence_evidence() {
  local dir="$PREVIEW_ROOT/generated-graphics"
  [[ -d $dir ]] || return 0
  local video
  video=$(delivery_video_path "$dir" preview preview)
  audio_stream_contract "$video" 7.9
  assert_stream_field "$video" a:0 sample_rate 48000 "generated-graphics silence"
  assert_stream_field "$video" a:0 channels 2 "generated-graphics silence"
  assert_audio_stream_duration "$video" 8 0.08 "generated-graphics silence"
  assert_fully_silent_audio "$video" 8 generated-graphics
}

check_all_features_audio_evidence() {
  local dir="$PREVIEW_ROOT/all-features"
  [[ -d $dir ]] || return 0
  local video duration
  video=$(delivery_video_path "$dir" master master)
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
  video=$(delivery_video_path "$dir" main main)
  audio_stream_contract "$video" 1.9
  assert_stream_field "$video" a:0 sample_rate 48000 "executable-mechanisms silence"
  assert_stream_field "$video" a:0 channels 1 "executable-mechanisms silence"
  assert_audio_stream_duration "$video" 2 0.08 "executable-mechanisms silence"
  assert_fully_silent_audio "$video" 2 executable-mechanisms
}

check_nested_multicam_audio_evidence() {
  local dir="$PREVIEW_ROOT/nested-and-multicam"
  [[ -d $dir ]] || return 0
  local video plan
  plan=$(example_preview_plan "$dir" out_preview)
  video=$(delivery_video_path "$dir" preview preview)
  require_file "$video"
  require_file "$plan"
  assert_stream_count "$video" a 0 "nested-and-multicam preview"
  jq -e '
    def t($v; $s): {"timescale":$s,"value":$v};
    first(.sequences[].tracks[].clips[] | select(.id == "itm_interview-cut")).source as $tagged |
    $tagged.type == "multicam" and
    ($tagged.source as $source |
      $source.sync.basis == "audio" and
      $source.sync.reference_angle_id == "ang_host-angle" and
      (first($source.angles[] | select(.id == "ang_guest-angle"))).source_offset == t(120;1000) and
      all($source.angles[]; .audio_stream == null))
  ' "$plan" >/dev/null || fail "nested-and-multicam audio sync plan contract failed"
}
