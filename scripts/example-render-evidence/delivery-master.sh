#!/usr/bin/env bash

check_delivery_master() {
  local dir=$1 master="$1/rendered/master.mp4"
  local width height rate duration count start _cover_time _cover tolerance rms
  read -r width height rate duration count start _cover_time _cover < <(delivery_metadata "$dir")
  tolerance=$(awk -F/ '{ print $2/$1/2 }' <<<"$rate")
  require_file "$master"
  check_delivery_master_recipe "$dir/project/project.veac.json"
  assert_duration_close "$master" "$duration" "$tolerance" "delivery master"
  assert_stream_count "$master" v 1 "delivery master"
  assert_stream_count "$master" a 1 "delivery master"
  assert_stream_field "$master" v:0 codec_name h264 "delivery master"
  assert_stream_field "$master" v:0 width "$width" "delivery master"
  assert_stream_field "$master" v:0 height "$height" "delivery master"
  assert_stream_field "$master" v:0 pix_fmt yuv420p "delivery master"
  assert_stream_field "$master" v:0 r_frame_rate "$rate" "delivery master"
  assert_stream_field "$master" a:0 codec_name aac "delivery master"
  assert_stream_field "$master" a:0 sample_rate 48000 "delivery master"
  assert_stream_field "$master" a:0 channels 2 "delivery master"
  rms=$(delivery_audio_rms "$master")
  assert_delivery_audio_rms "delivery master audio" "$rms"
  assert_mp4_fast_start "$master" "delivery master"
}

check_delivery_burn_in() {
  local dir=$1 canonical="$1/project/project.veac.json" master="$1/rendered/master.mp4"
  local width height rate duration count start _cover_time _cover config
  local bright bbox_width bbox_height opening closing minimum_width minimum_height sample_time
  read -r width height rate duration count start _cover_time _cover < <(delivery_metadata "$dir")
  require_file "$canonical"
  config=$(delivery_config_id "$canonical" master)
  jq -e --arg config "$config" '
    def clip($key): first(.project.sequences[].tracks[].clips[] |
      select(.authorship.logical_path[-1] == $key));
    clip("burned-label") as $first | clip("traceable-label") as $second |
    $first.source.type == "caption" and $first.source.text == "类型化交付输出" and
    $second.source.type == "caption" and $second.source.text == "可追踪的字幕伴随文件" and
    any(.project.render_configs[]; .id == $config and
      .raster.captions == "burn_in" and
      any(.deliverables[]; .kind.type == "video" and .target.name == "master.mp4"))
  ' "$canonical" >/dev/null || fail "delivery-formats burned caption contract failed"
  read -r opening closing < <(delivery_stage_times "$duration")
  minimum_width=$((width / 7)); minimum_height=$((height / 30))
  for sample_time in "$opening" "$closing"; do
    read -r bright bbox_width bbox_height < <(
      frame_bright_bbox "$master" "$sample_time" "$width" 500
    )
    ((bright >= 40 && bbox_width >= minimum_width && bbox_height >= minimum_height)) ||
      fail "delivery master burned caption is not visible at ${sample_time}s: pixels=$bright bbox=${bbox_width}x$bbox_height"
  done
}

check_delivery_frames() {
  local dir=$1 rendered="$1/rendered" width height rate duration expected start _cover_time _cover
  local count index frame first last first_hash last_hash
  read -r width height rate duration expected start _cover_time _cover < <(delivery_metadata "$dir")
  check_delivery_frame_recipe "$dir/project/project.veac.json"
  count=$(find "$rendered" -maxdepth 1 -type f -name 'frame-*.png' | wc -l | tr -d ' ')
  [[ $count == "$expected" ]] ||
    fail "delivery frame sequence: expected $expected frames, got $count"
  for ((index=start; index<start+expected; index+=1)); do
    frame=$(delivery_frame_path "$rendered" "$index")
    require_file "$frame"
    assert_stream_field "$frame" v:0 codec_name png "delivery frame $index"
    assert_stream_field "$frame" v:0 width "$width" "delivery frame $index"
    assert_stream_field "$frame" v:0 height "$height" "delivery frame $index"
    assert_delivery_decodes "$frame" "delivery frame $index"
  done
  first=$(delivery_frame_path "$rendered" "$start")
  last=$(delivery_frame_path "$rendered" "$((start + expected - 1))")
  first_hash=$(frame_hash "$first" 0); last_hash=$(frame_hash "$last" 0)
  [[ -n $first_hash && -n $last_hash && $first_hash != "$last_hash" ]] ||
    fail "delivery frame sequence does not contain distinct visual stages"
  assert_delivery_color "$first" 0 13 27 42 5 "delivery opening frame"
  assert_delivery_color "$last" 0 196 58 105 5 "delivery closing frame"
}

check_delivery_captions() {
  local captions="$1/captions.vtt" actual expected
  require_file "$captions"
  actual=$(sed 's/\r$//' "$captions")
  expected=$(printf '%s\n' \
    'WEBVTT' '' \
    '00:00:00.000 --> 00:00:01.500' '类型化交付输出' '' \
    '00:00:01.500 --> 00:00:03.000' '可追踪的字幕伴随文件')
  [[ $actual == "$expected" ]] || fail "delivery captions.vtt content or timing is incorrect"
}
