#!/usr/bin/env bash

all_features_audio_contract() {
  local canonical=$1 label=$2 prefix=${3:-.project} identity=${4:-$1} music
  music=$(canonical_clip_id "$identity" music-item)
  jq -e --arg label "$label" --arg prefix "$prefix" --arg music "$music" '
    def seconds: .value/.timescale;
    def clip: first(if $prefix == ".project" then .project.sequences[] else .sequences[] end
      | .tracks[].clips[] | select(.id == $music));
    clip as $music |
    ($music.record_range.start|seconds) == 0 and
    ($music.record_range.duration|seconds) == 8 and
    $music.audio.gain.type == "constant" and
    (($music.audio.gain.value - 0.501187) | fabs) < 0.000001 and
    $music.audio.pan == {"type":"constant","value":0} and
    $music.audio.muted == false and $music.audio.normalize == true and
    $music.audio.pitch_policy == "preserve" and
    $music.audio.crossfade.curve == "equal_power" and
    ($music.audio.crossfade.fade_in|seconds) == 0.25 and
    ($music.audio.crossfade.fade_out|seconds) == 0.5 and
    ($music.audio.processors | map(.kind)) == [{
      "attack_ms":1,"ceiling_db":-1,"release_ms":80,"type":"limiter"}]
  ' "$canonical" >/dev/null || fail "all-features $label audio-chain contract failed"
}

all_features_assert_fade_window() {
  local video=$1 quiet_start=$2 quiet_duration=$3 loud_start=$4 loud_duration=$5 label=$6
  local quiet loud
  quiet=$(audio_mean_db "$video" "$quiet_start" "$quiet_duration" "")
  loud=$(audio_mean_db "$video" "$loud_start" "$loud_duration" "")
  awk -v quiet="$quiet" -v loud="$loud" 'BEGIN { exit !(loud >= quiet + 5) }' ||
    fail "$label is missing: quiet=$quiet dB loud=$loud dB"
}

check_all_features_audio_evidence() {
  local dir=${1:-"$PREVIEW_ROOT/all-features"}
  [[ -d $dir ]] || return 0
  local author preview plan video
  author=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" master)
  video=$(delivery_video_path "$dir" master all-features.mp4)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  all_features_audio_contract "$author" authoring
  all_features_audio_contract "$preview" preview
  all_features_audio_contract "$plan" plan .plan "$author"
  audio_stream_contract "$video" 7.9
  assert_stream_field "$video" a:0 sample_rate 48000 "all-features audio"
  assert_stream_field "$video" a:0 channels 2 "all-features audio"
  assert_audio_stream_duration "$video" 8 0.08 "all-features audio"
  assert_non_silent_window "$video" 1 6 "all-features normalized music"
  all_features_assert_fade_window "$video" 0 0.06 0.3 0.2 "all-features fade-in"
  all_features_assert_fade_window "$video" 7.92 0.06 7.3 0.2 "all-features fade-out"
  assert_loudness_target "$video" 0.5 7 -16 3.5 -30 -0.1 \
    "all-features normalization and limiter"
}
