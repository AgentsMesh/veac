#!/usr/bin/env bash

check_delivery_plan_contracts() {
  local dir=$1 canonical plan
  canonical=$(example_preview_canonical "$dir")
  plan=$(example_preview_plan "$dir" out_master)
  require_file "$canonical"
  require_file "$plan"
  jq -e --slurpfile resolved "$plan" '
    (.project.render_configs | length) == 1 and
    first(.project.render_configs[] | select(.id == "out_master")) as $config |
    ($resolved | length) == 1 and $resolved[0].output as $output |
    $output.id == "pout_master" and $output.render_config_id == $config.id and
    $output.sequence_id == $config.sequence_id and
    $output.raster == $config.raster and $output.deliverables == $config.deliverables
  ' "$canonical" >/dev/null || fail "delivery canonical and resolved plan outputs differ"
  jq -e '
    .output.deliverables | map({id, kind:.kind.type, target:.target}) | sort_by(.id) == [
      {"id":"dlv_cover","kind":"still_image","target":{"type":"file","name":"cover.png"}},
      {"id":"dlv_frames","kind":"image_sequence","target":{"type":"image_sequence","pattern":"frame-%04d.png"}},
      {"id":"dlv_loop-preview","kind":"animated_image","target":{"type":"file","name":"loop-preview.gif"}},
      {"id":"dlv_master","kind":"video","target":{"type":"file","name":"master.mp4"}},
      {"id":"dlv_master-audio","kind":"audio_stem","target":{"type":"file","name":"master.wav"}},
      {"id":"dlv_podcast","kind":"audio_file","target":{"type":"file","name":"podcast.mp3"}},
      {"id":"dlv_stream","kind":"adaptive_package","target":{"type":"package","name":"stream"}},
      {"id":"dlv_transcript","kind":"caption_sidecar","target":{"type":"file","name":"captions.vtt"}},
      {"id":"dlv_video-waveform","kind":"scope","target":{"type":"file","name":"video-waveform.png"}}
    ]
  ' "$plan" >/dev/null || fail "delivery resolved plan does not contain the exact 9-deliverable set"
}

check_delivery_preview_policy() {
  local canonical=$1
  jq -e '
    first(.project.render_configs[] | select(.id == "out_master")) as $config |
    $config.raster.width == 480 and $config.raster.height == 270 and
    $config.raster.frame_rate == {"numerator":12,"denominator":1} and
    all($config.deliverables[] | select(.kind.type == "video");
      .kind.settings.hardware == {"type":"software"})
  ' "$canonical" >/dev/null || fail "delivery preview policy contract failed"
}

check_delivery_master_recipe() {
  local canonical=$1
  jq -e '
    first(.project.render_configs[] | select(.id == "out_master")) as $config |
    first($config.deliverables[] | select(.id == "dlv_master")) as $master |
    $master.kind.type == "video" and $master.kind.settings as $settings |
    $settings.container == "mp4" and $settings.optimize_for_streaming == true and
    $settings.pass_mode == "single" and
    $settings.video.codec == "h264" and $settings.video.pixel_format == "yuv420p" and
    $settings.video.alpha == "opaque" and
    $settings.video.rate_control == {"type":"crf","value":23} and
    $settings.audio == {"codec":"aac","sample_rate":48000,"channels":2}
  ' "$canonical" >/dev/null ||
    fail "delivery MP4 H.264/AAC CRF/pass/streaming canonical contract failed"
}

check_delivery_frame_recipe() {
  local canonical=$1
  jq -e '
    first(.project.render_configs[] | select(.id == "out_master")) as $config |
    first($config.deliverables[] | select(.id == "dlv_frames")) as $frames |
    $frames.kind == {"type":"image_sequence","settings":{
      "format":"png","start_number":1}}
  ' "$canonical" >/dev/null || fail "delivery PNG sequence numbering canonical contract failed"
}

mp4_top_level_boxes() {
  od -An -v -tu1 "$1" | awk '
    { for (i=1; i<=NF; i++) bytes[++count]=$i }
    function u32(at) {
      return bytes[at]*16777216 + bytes[at+1]*65536 + bytes[at+2]*256 + bytes[at+3]
    }
    END {
      at=1
      while (at+7 <= count) {
        size=u32(at); header=8
        type=sprintf("%c%c%c%c",bytes[at+4],bytes[at+5],bytes[at+6],bytes[at+7])
        if (size == 1) { size=u32(at+8)*4294967296+u32(at+12); header=16 }
        else if (size == 0) size=count-at+1
        if (size < header || at+size-1 > count) exit 1
        print type, at-1, size
        at+=size
      }
      if (at != count+1) exit 1
    }'
}

assert_mp4_fast_start() {
  local file=$1 label=$2 boxes moov mdat
  boxes=$(mp4_top_level_boxes "$file") || fail "$label has malformed top-level MP4 boxes"
  moov=$(awk '$1 == "moov" { print $2; exit }' <<<"$boxes")
  mdat=$(awk '$1 == "mdat" { print $2; exit }' <<<"$boxes")
  [[ $moov =~ ^[0-9]+$ && $mdat =~ ^[0-9]+$ && $moov -lt $mdat ]] ||
    fail "$label is not fast-start (moov must precede mdat)"
}

check_delivery_root_inventory() {
  local dir=$1 rendered="$1/rendered" _width _height _rate _duration count start _cover_time _cover
  read -r _width _height _rate _duration count start _cover_time _cover < <(delivery_metadata "$dir")
  local expected actual index frame
  expected=$(printf '%s\n' captions.vtt cover.png loop-preview.gif master.mp4 master.wav \
    podcast.mp3 stream video-waveform.png)
  for ((index=start; index<start+count; index+=1)); do
    frame=$(delivery_frame_path "$rendered" "$index")
    expected+=$'\n'$(basename "$frame")
  done
  expected=$(LC_ALL=C sort -u <<<"$expected")
  actual=$(find "$rendered" -mindepth 1 -maxdepth 1 -exec basename {} \; | LC_ALL=C sort)
  [[ $actual == "$expected" ]] || fail "delivery rendered root inventory is not exact"
  [[ -d $rendered/stream && ! -L $rendered/stream ]] || fail "delivery stream is not a real directory"
  local member
  while IFS= read -r member; do
    [[ $member == stream ]] && continue
    assert_delivery_regular_file "$rendered/$member" "delivery root member $member"
  done <<<"$actual"
}
