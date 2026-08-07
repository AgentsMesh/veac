#!/usr/bin/env bash

workflow_assert_keying_media() {
  local video=$1 rose green white motion unique content
  workflow_video_contract "$video" 4 keying-stabilization
  assert_stream_count "$video" a 0 "keying-stabilization"
  rose=$(region_color_count "$video" 0.5 'iw:ih-50:0:0' rose)
  green=$(region_color_count "$video" 0.5 'iw:ih-50:0:0' green)
  ((rose > 50000 && green < 1000)) ||
    fail "keying-stabilization chroma-key pixels are wrong: rose=$rose green=$green"
  green=$(region_color_count "$video" 1.5 'iw:ih-50:0:0' green)
  ((green > 50000)) || fail "keying-stabilization luma-key pixels are missing: green=$green"
  assert_unique_frames "$video" 3 0.5 1.5 2.5
  for time in 0.5 1.5 3; do
    white=$(region_color_count "$video" "$time" 'iw:60:0:ih-60' white)
    ((white > 500)) || fail "keying-stabilization Chinese label is missing at ${time}s"
  done
  motion=$(stabilization_motion_score "$video" 2.1 2.3 2.5 2.7 2.9 3.1 3.3 3.5 3.7 3.9)
  awk -v value="$motion" 'BEGIN { exit !(value <= 4) }' ||
    fail "keying-stabilization output still moves: score=$motion"
  unique=$(stabilization_unique_count "$video" 2.1 2.3 2.5 2.7 2.9 3.1 3.3 3.5 3.7 3.9)
  content=$(stabilization_content_unique_count "$video" 2.1 2.3 2.5 2.7 2.9 3.1 3.3 3.5 3.7 3.9)
  ((unique >= 4 && content >= 4)) ||
    fail "keying-stabilization output is frozen: phase=$unique content=$content"
}

workflow_assert_routing_media() {
  local video=$1 white left right
  workflow_video_contract "$video" 2 audio-routing-ducking
  audio_stream_contract "$video" 1.9
  assert_stream_field "$video" a:0 sample_rate 48000 "audio-routing-ducking"
  assert_stream_field "$video" a:0 channels 2 "audio-routing-ducking"
  assert_non_silent_window "$video" 0.2 1.6 "audio-routing-ducking routed mix"
  assert_audio_window_louder "$video" 0.3 0.5 0 0.08 3 "audio-routing-ducking fade-in"
  assert_audio_window_louder "$video" 1.2 0.5 1.92 0.08 3 "audio-routing-ducking fade-out"
  left=$(audio_mean_db "$video" 0.3 1.4 'pan=mono|c0=c0')
  right=$(audio_mean_db "$video" 0.3 1.4 'pan=mono|c0=c1')
  awk -v left="$left" -v right="$right" \
    'BEGIN { gap=left-right; exit !(gap >= .3 && gap <= .7) }' ||
    fail "audio-routing-ducking sidechain/pan balance is wrong: left=$left right=$right"
  white=$(region_color_count "$video" 1 'iw:70:0:ih-70' white)
  ((white > 500)) || fail "audio-routing-ducking Chinese explanation is missing"
}

workflow_assert_edit_media() {
  local video=$1 white
  workflow_video_contract "$video" 4 edit-operations
  assert_stream_count "$video" a 0 edit-operations
  assert_unique_frames "$video" 2 1 3
  workflow_assert_region_rgb "$video" 1 '80:60:8:8' 37 99 235 8 \
    "edit-operations first scene"
  workflow_assert_region_rgb "$video" 3 '80:60:8:8' 219 39 119 8 \
    "edit-operations second scene"
  white=$(region_color_count "$video" 1 'iw:80:0:ih-80' white)
  ((white > 500)) || fail "edit-operations Chinese explanation is missing"
}

workflow_assert_probe_media() {
  local video=$1 white scene_white
  workflow_video_contract "$video" 3 probe-stream-selection
  assert_stream_count "$video" a 0 probe-stream-selection
  workflow_assert_region_rgb "$video" 1.5 '80:60:8:8' 76 176 81 8 \
    "probe-stream-selection selected stream"
  scene_white=$(region_color_count "$video" 1.5 '300:100:90:60' white)
  ((scene_white > 400)) || fail "probe-stream-selection SCENE 1 stream marker is missing"
  white=$(region_color_count "$video" 1.5 'iw:80:0:ih-80' white)
  ((white > 500)) || fail "probe-stream-selection Chinese explanation is missing"
}
