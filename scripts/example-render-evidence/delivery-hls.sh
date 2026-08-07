#!/usr/bin/env bash

check_hls_media() {
  local playlist=$1 width=$2 height=$3 rate=$4 duration=$5 count=$6
  local reference=$7 master=$8 label=$9 rms
  ffprobe -v error -show_entries stream=codec_type -of json "$playlist" \
    | jq -e '([.streams[] | select(.codec_type == "video")] | length) == 1 and
      ([.streams[] | select(.codec_type == "audio")] | length) == 1' >/dev/null \
    || fail "$label does not contain exactly one video and one audio stream"
  assert_stream_field "$playlist" v:0 codec_name h264 "$label"
  assert_stream_field "$playlist" v:0 width "$width" "$label"
  assert_stream_field "$playlist" v:0 height "$height" "$label"
  assert_stream_field "$playlist" v:0 pix_fmt yuv420p "$label"
  assert_stream_field "$playlist" v:0 r_frame_rate "$rate" "$label"
  assert_stream_field "$playlist" a:0 codec_name aac "$label"
  assert_stream_field "$playlist" a:0 sample_rate 48000 "$label"
  assert_stream_field "$playlist" a:0 channels 2 "$label"
  assert_duration_close "$playlist" "$duration" 0.2 "$label"
  rms=$(delivery_audio_rms "$playlist")
  assert_delivery_audio_rms "$label audio" "$rms"
  assert_delivery_audio_matches "$reference" "$playlist" "$duration" "$label audio"
  local opening closing before after
  read -r opening closing < <(delivery_stage_times "$duration")
  read -r before after < <(delivery_boundary_times "$count" "$rate")
  assert_delivery_frames_match "$playlist" "$opening" "$master" "$opening" \
    "$label opening does not match master"
  assert_delivery_frames_match "$playlist" "$closing" "$master" "$closing" \
    "$label closing does not match master"
  assert_delivery_frames_match "$playlist" "$before" "$master" "$before" \
    "$label before boundary does not match master"
  assert_delivery_frames_match "$playlist" "$after" "$master" "$after" \
    "$label after boundary does not match master"
  assert_delivery_decodes "$playlist" "$label"
}

check_delivery_hls_contract() {
  local canonical=$1 config
  config=$(delivery_config_id "$canonical" master)
  jq -e --arg config "$config" '
    first(.project.render_configs[] | select(.id == $config)) as $config |
    first($config.deliverables[] | select(.kind.type == "adaptive_package" and
      .target.name == "stream")) as $stream |
    $stream.target == {"type":"package","name":"stream"} and
    $stream.kind.type == "adaptive_package" and $stream.kind.settings.type == "hls" and
    $stream.kind.settings.settings as $hls |
    $hls.segment_duration.value == $hls.segment_duration.timescale and
    $hls.audio.source == {"type":"master"} and
    $hls.audio.encoding == {"type":"aac","settings":{
      "bitrate_bps":128000,"sample_rate_hz":48000,"channel_layout":"stereo"}} and
    ($hls.renditions | map({raster,encoding}) | sort_by(.raster.width)) == [
      {"raster":{"width":640,"height":360},"encoding":{
        "type":"h264","settings":{"rate_control":{
          "target_bps":1000000,"max_bps":1100000,"buffer_size_bits":2000000},
          "b_frames":null,"profile":"main","level":null,"color_space":null}}},
      {"raster":{"width":1280,"height":720},"encoding":{
        "type":"h264","settings":{"rate_control":{
          "target_bps":3000000,"max_bps":3210000,"buffer_size_bits":6000000},
          "b_frames":null,"profile":"high","level":null,"color_space":null}}}]
  ' "$canonical" >/dev/null || fail "delivery HLS canonical contract failed"
}

check_delivery_hls() {
  local dir=$1 root="$1/rendered/stream"
  local _width _height rate duration count _start _cover_time _cover seconds
  read -r _width _height rate duration count _start _cover_time _cover < <(delivery_metadata "$dir")
  seconds=$(jq -er '
    first(.output.deliverables[] | select(.kind.type == "adaptive_package" and
      .target.name == "stream")) |
    .kind.settings.settings.segment_duration | .value/.timescale
  ' "$(delivery_plan_path "$dir" master)")
  check_delivery_hls_contract "$dir/project/project.veac.json"
  check_hls_closed_tree "$root" "$seconds"
  assert_delivery_decodes "$root/master.m3u8" "HLS master playlist"
  local id rendition_width rendition_height target max _buffer
  while IFS=$'\t' read -r id rendition_width rendition_height target max _buffer; do
    assert_hls_master_variant "$root/master.m3u8" "rendition-${id}.m3u8" \
      "$rendition_width" "$rendition_height" "$target" "$max" 128000 "HLS $id rendition"
    check_hls_media "$root/rendition-${id}.m3u8" "$rendition_width" "$rendition_height" \
      "$rate" "$duration" "$count" "$dir/rendered/master.wav" "$dir/rendered/master.mp4" \
      "HLS $id rendition"
  done < <(jq -er '
    first(.output.deliverables[] | select(.kind.type == "adaptive_package" and
      .target.name == "stream")) |
    .kind.settings.settings.renditions[] |
    [.id,.raster.width,.raster.height,.encoding.settings.rate_control.target_bps,
     .encoding.settings.rate_control.max_bps,.encoding.settings.rate_control.buffer_size_bits] | @tsv
  ' "$(delivery_plan_path "$dir" master)")
}
