#!/usr/bin/env bash

check_delivery_image_contracts() {
  local canonical=$1
  jq -e '
    first(.project.render_configs[] | select(.id == "out_master")) as $config |
    first($config.deliverables[] | select(.id == "dlv_loop-preview")) as $gif |
    first($config.deliverables[] | select(.id == "dlv_cover")) as $cover |
    $gif.target == {"type":"file","name":"loop-preview.gif"} and
    $gif.kind == {"type":"animated_image","settings":{"type":"gif","settings":{
      "playback":{"mode":"forever"},"dither":"sierra2"}}} and
    $cover.target == {"type":"file","name":"cover.png"} and
    $cover.kind == {"type":"still_image","settings":{
      "frame":{"mode":"containing","at":{"timescale":1000,"value":2000}},
      "encoding":"png"}}
  ' "$canonical" >/dev/null || fail "delivery GIF/still canonical contract failed"
}

assert_gif_forever_loop() {
  local gif=$1 hex
  hex=$(od -An -tx1 -v "$gif" | tr -d '[:space:]')
  [[ $hex == *21ff0b4e45545343415045322e300301000000* ]] \
    || fail "delivery GIF lacks the declared infinite-loop extension"
}

check_delivery_gif() {
  local dir=$1 gif="$1/rendered/loop-preview.gif" master="$1/rendered/master.mp4"
  local width height rate duration expected start cover_time cover frames opening closing first last
  local before after before_frame after_frame before_png after_png
  read -r width height rate duration expected start cover_time cover < <(delivery_metadata "$dir")
  read -r opening closing < <(delivery_stage_times "$duration")
  read -r before after < <(delivery_boundary_times "$expected" "$rate")
  first=$(delivery_frame_path "$dir/rendered" "$start")
  last=$(delivery_frame_path "$dir/rendered" "$((start + expected - 1))")
  read -r before_frame after_frame < <(delivery_boundary_frame_numbers "$start" "$expected")
  before_png=$(delivery_frame_path "$dir/rendered" "$before_frame")
  after_png=$(delivery_frame_path "$dir/rendered" "$after_frame")
  assert_delivery_regular_file "$gif" "delivery GIF"
  assert_stream_count "$gif" v 1 "delivery GIF"
  assert_stream_count "$gif" a 0 "delivery GIF"
  assert_stream_field "$gif" v:0 codec_name gif "delivery GIF"
  assert_stream_field "$gif" v:0 width "$width" "delivery GIF"
  assert_stream_field "$gif" v:0 height "$height" "delivery GIF"
  assert_duration_close "$gif" "$duration" 0.12 "delivery GIF"
  frames=$(ffprobe -v error -count_frames -select_streams v:0 \
    -show_entries stream=nb_read_frames -of default=nw=1:nk=1 "$gif")
  [[ $frames =~ ^[0-9]+$ ]] || fail "delivery GIF frame count is unavailable"
  [[ $frames == "$expected" ]] ||
    fail "delivery GIF: expected $expected decoded frames, got $frames"
  assert_unique_frames "$gif" 2 "$opening" "$closing"
  assert_delivery_color "$gif" "$opening" 13 27 42 10 "delivery GIF opening stage"
  assert_delivery_color "$gif" "$closing" 196 58 105 12 "delivery GIF closing stage"
  assert_delivery_frames_match "$gif" "$opening" "$master" "$opening" \
    "delivery GIF opening does not match master"
  assert_delivery_frames_match "$gif" "$closing" "$master" "$closing" \
    "delivery GIF closing does not match master"
  assert_delivery_frames_match "$gif" "$before" "$master" "$before" \
    "delivery GIF before boundary does not match master"
  assert_delivery_frames_match "$gif" "$after" "$master" "$after" \
    "delivery GIF after boundary does not match master"
  assert_delivery_frames_match "$gif" "$opening" "$first" 0 \
    "delivery GIF opening does not match PNG sequence"
  assert_delivery_frames_match "$gif" "$closing" "$last" 0 \
    "delivery GIF closing does not match PNG sequence"
  assert_delivery_frames_match "$gif" "$before" "$before_png" 0 \
    "delivery GIF before boundary does not match PNG sequence"
  assert_delivery_frames_match "$gif" "$after" "$after_png" 0 \
    "delivery GIF after boundary does not match PNG sequence"
  assert_gif_forever_loop "$gif"
  assert_delivery_decodes "$gif" "delivery GIF"
}

check_delivery_cover() {
  local dir=$1 rendered="$1/rendered" cover="$1/rendered/cover.png"
  local width height rate duration _count start cover_time cover_frame reference
  read -r width height rate duration _count start cover_time cover_frame < <(delivery_metadata "$dir")
  reference=$(delivery_frame_path "$rendered" "$cover_frame")
  local bright bbox_width bbox_height minimum_width minimum_height
  assert_delivery_regular_file "$cover" "delivery cover"
  assert_stream_count "$cover" v 1 "delivery cover"
  assert_stream_count "$cover" a 0 "delivery cover"
  assert_stream_field "$cover" v:0 codec_name png "delivery cover"
  assert_stream_field "$cover" v:0 width "$width" "delivery cover"
  assert_stream_field "$cover" v:0 height "$height" "delivery cover"
  assert_delivery_color "$cover" 0 196 58 105 10 "delivery cover at ${cover_time}s"
  minimum_width=$((width / 6))
  minimum_height=$((height / 24))
  read -r bright bbox_width bbox_height < <(frame_bright_bbox "$cover" 0 "$width")
  ((bright >= 40 && bbox_width >= minimum_width && bbox_height >= minimum_height)) ||
    fail "delivery cover does not retain the burned caption at 2s"
  assert_frames_visually_equal "$cover" "$reference" \
    "delivery cover does not visually match frame $cover_frame at ${cover_time}s"
  assert_delivery_decodes "$cover" "delivery cover"
}

check_delivery_images() {
  local dir=$1
  check_delivery_image_contracts "$dir/project/project.veac.json"
  check_delivery_gif "$dir"
  check_delivery_cover "$dir"
}
