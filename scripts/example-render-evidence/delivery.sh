#!/usr/bin/env bash

delivery_audio_rms() {
  ffmpeg -nostdin -hide_banner -nostats -loglevel info -i "$1" -vn \
    -af volumedetect -f null - 2>&1 |
    awk '/mean_volume:/ { print $(NF-1); exit }'
}

assert_delivery_audio_rms() {
  local label=$1 rms=$2
  [[ -n $rms && $rms != "-inf" ]] || fail "$label is silent"
  awk -v value="$rms" 'BEGIN { exit !(value > -45 && value < -2) }' ||
    fail "$label RMS is outside (-45, -2) dB: $rms"
}

check_delivery_master() {
  local dir=$1 rendered="$1/rendered" master="$1/rendered/master.mp4"
  local width height rate duration count start _cover_time _cover tolerance
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
  local rms
  rms=$(delivery_audio_rms "$master")
  assert_delivery_audio_rms "delivery master audio" "$rms"
  assert_mp4_fast_start "$master" "delivery master"
}

check_delivery_burn_in() {
  local dir=$1 canonical="$1/project/project.veac.json" master="$1/rendered/master.mp4"
  local width height rate duration count start _cover_time _cover
  read -r width height rate duration count start _cover_time _cover < <(delivery_metadata "$dir")
  local bright bbox_width bbox_height opening closing minimum_width minimum_height
  require_file "$canonical"
  jq -e '
    first(.project.sequences[].tracks[].clips[] | select(.id == "itm_burned-label")) as $first |
    first(.project.sequences[].tracks[].clips[] | select(.id == "itm_traceable-label")) as $second |
    $first.source.type == "caption" and $first.source.text == "类型化交付输出" and
    $second.source.type == "caption" and $second.source.text == "可追踪的字幕伴随文件" and
    any(.project.render_configs[]; .id == "out_master" and
      .raster.captions == "burn_in" and
      any(.deliverables[]; .id == "dlv_master" and .kind.type == "video"))
  ' "$canonical" >/dev/null || fail "delivery-formats burned caption contract failed"
  read -r opening closing < <(delivery_stage_times "$duration")
  minimum_width=$((width / 7))
  minimum_height=$((height / 30))
  local sample_time
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
  read -r width height rate duration expected start _cover_time _cover < <(delivery_metadata "$dir")
  local count
  check_delivery_frame_recipe "$dir/project/project.veac.json"
  count=$(find "$rendered" -maxdepth 1 -type f -name 'frame-*.png' | wc -l | tr -d ' ')
  [[ $count == "$expected" ]] ||
    fail "delivery frame sequence: expected $expected frames, got $count"
  local index frame
  for ((index=start; index<start+expected; index+=1)); do
    frame=$(delivery_frame_path "$rendered" "$index")
    require_file "$frame"
    assert_stream_field "$frame" v:0 codec_name png "delivery frame $index"
    assert_stream_field "$frame" v:0 width "$width" "delivery frame $index"
    assert_stream_field "$frame" v:0 height "$height" "delivery frame $index"
  done
  local first last
  first=$(delivery_frame_path "$rendered" "$start")
  last=$(delivery_frame_path "$rendered" "$((start + expected - 1))")
  local first_hash last_hash
  first_hash=$(frame_hash "$first" 0)
  last_hash=$(frame_hash "$last" 0)
  [[ -n $first_hash && -n $last_hash && $first_hash != "$last_hash" ]] ||
    fail "delivery frame sequence does not contain distinct visual stages"
  assert_delivery_color "$first" 0 13 27 42 5 "delivery opening frame"
  assert_delivery_color "$last" 0 196 58 105 5 "delivery closing frame"
}

check_delivery_captions() {
  local captions="$1/captions.vtt"
  require_file "$captions"
  local actual expected
  actual=$(sed 's/\r$//' "$captions")
  expected=$(printf '%s\n' \
    'WEBVTT' \
    '' \
    '00:00:00.000 --> 00:00:01.500' \
    '类型化交付输出' \
    '' \
    '00:00:01.500 --> 00:00:03.000' \
    '可追踪的字幕伴随文件')
  [[ $actual == "$expected" ]] || fail "delivery captions.vtt content or timing is incorrect"
}

check_delivery_audio() {
  local dir=$1 wav="$1/rendered/master.wav"
  local width height rate duration count start _cover_time _cover rms tolerance
  read -r width height rate duration count start _cover_time _cover < <(delivery_metadata "$dir")
  tolerance=$(awk -F/ '{ print $2/$1/2 }' <<<"$rate")
  jq -e '
    first(.project.render_configs[] | select(.id == "out_master")) as $config |
    first($config.deliverables[] | select(.id == "dlv_master-audio")) as $stem |
    $stem.target == {"type":"file","name":"master.wav"} and
    $stem.kind == {"type":"audio_stem","settings":{
      "source":{"type":"master"},"format":"wav",
      "audio":{"codec":"pcm_s24_le","sample_rate":48000,"channels":2}}}
  ' "$dir/project/project.veac.json" >/dev/null ||
    fail "delivery WAV master-source/PCM canonical contract failed"
  require_file "$wav"
  assert_duration_close "$wav" "$duration" "$tolerance" "delivery WAV"
  assert_stream_count "$wav" a 1 "delivery WAV"
  assert_stream_count "$wav" v 0 "delivery WAV"
  assert_stream_field "$wav" a:0 codec_name pcm_s24le "delivery WAV"
  assert_stream_field "$wav" a:0 bits_per_raw_sample 24 "delivery WAV"
  assert_stream_field "$wav" a:0 sample_rate 48000 "delivery WAV"
  assert_stream_field "$wav" a:0 channels 2 "delivery WAV"
  rms=$(delivery_audio_rms "$wav")
  assert_delivery_audio_rms "delivery PCM audio" "$rms"
}

delivery_audio_window() {
  awk -v duration="$1" 'BEGIN { print duration/10, duration*8/10 }'
}

delivery_audio_signature() {
  local media=$1 start=$2 duration=$3
  printf '%s ' "$(delivery_audio_rms "$media")"
  printf '%s ' "$(audio_band_db "$media" "$start" "$duration" 220)"
  printf '%s ' "$(audio_band_db "$media" "$start" "$duration" 440)"
  printf '%s\n' "$(audio_band_db "$media" "$start" "$duration" 660)"
}

assert_delivery_audio_matches() {
  local reference=$1 candidate=$2 duration=$3 label=$4 start window
  local rr rt r440 r660 cr ct c440 c660
  read -r start window < <(delivery_audio_window "$duration")
  read -r rr rt r440 r660 < <(delivery_audio_signature "$reference" "$start" "$window")
  read -r cr ct c440 c660 < <(delivery_audio_signature "$candidate" "$start" "$window")
  awk -v rr="$rr" -v rt="$rt" -v r440="$r440" -v r660="$r660" \
    -v cr="$cr" -v ct="$ct" -v c440="$c440" -v c660="$c660" '
      function abs(v) { return v < 0 ? -v : v }
      BEGIN { exit !(rt >= r440+12 && rt >= r660+12 &&
                      ct >= c440+12 && ct >= c660+12 &&
                      abs(rr-cr) <= 1.5 && abs(rt-ct) <= 2.5) }
    ' || fail "$label does not carry the same 220 Hz master mix"
}

assert_delivery_audio_match() {
  local dir=$1 _width _height _rate duration _count _start _cover_time _cover
  read -r _width _height _rate duration _count _start _cover_time _cover < <(delivery_metadata "$dir")
  assert_delivery_audio_matches "$dir/rendered/master.wav" "$dir/rendered/master.mp4" \
    "$duration" "delivery MP4 audio"
}

check_delivery_evidence() {
  local dir="$PREVIEW_ROOT/delivery-formats"
  [[ -d $dir ]] || return 0
  local rendered="$dir/rendered"
  [[ -d $rendered ]] || fail "delivery-formats rendered directory is missing"
  check_delivery_plan_contracts "$dir"
  check_delivery_preview_policy "$(example_preview_canonical "$dir")"
  check_delivery_root_inventory "$dir"
  check_delivery_master "$dir"
  check_delivery_burn_in "$dir"
  check_delivery_frames "$dir"
  check_delivery_captions "$rendered"
  check_delivery_audio "$dir"
  assert_delivery_audio_match "$dir"
  check_delivery_waveform "$dir"
  check_delivery_mp3 "$dir"
  check_delivery_images "$dir"
  check_delivery_hls "$dir"
}
