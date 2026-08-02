#!/usr/bin/env bash

make_custom_hls_rendition() {
  local source=$1 root=$2 id=$3 width=$4 height=$5 fps=$6 pixel=$7 seconds=$8
  local target=$9 max=${10} buffer=${11} gop
  gop=$(awk -v fps="$fps" -v seconds="$seconds" 'BEGIN { print int(fps*seconds+0.5) }')
  ffmpeg -nostdin -v error -y -i "$source" -map 0:v:0 -map 0:a:0 -t 3 \
    -vf "scale=${width}:${height}" -r "$fps" -c:v libx264 -pix_fmt "$pixel" \
    -b:v "$target" -maxrate "$max" -bufsize "$buffer" -g "$gop" -keyint_min "$gop" \
    -sc_threshold 0 -force_key_frames "expr:gte(t,n_forced*${seconds})" \
    -c:a aac -b:a 128k -ar 48000 -ac 2 -f hls -hls_segment_type mpegts \
    -hls_time "$seconds" -hls_list_size 0 -hls_playlist_type vod \
    -hls_flags independent_segments -start_number 0 \
    -hls_segment_filename "$root/segment-${id}-%06d.ts" "$root/rendition-${id}.m3u8"
}

run_delivery_hls_contracts() {
  local root dir stream segment source

  root="$TMP_DIR/missing-hls-segment"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  segment=$(find "$stream" -maxdepth 1 -type f -name 'segment-rnd_hd-*.ts' | head -n 1)
  mv "$segment" "$segment.missing"
  expect_typed_failure missing_hls_segment "$root" check_delivery_hls "missing or empty artifact"

  root="$TMP_DIR/unsafe-hls-uri"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed 's#rendition-rnd_mobile.m3u8#https://example.invalid/mobile.m3u8#' \
    "$stream/master.m3u8" >"$stream/master.tmp"
  mv "$stream/master.tmp" "$stream/master.m3u8"
  expect_typed_failure unsafe_hls_uri "$root" check_delivery_hls "unsafe or non-local URI"

  root="$TMP_DIR/fake-hls-endlist"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed 's/^#EXT-X-ENDLIST$/#EXT-X-ENDLIST-FAKE/' "$stream/rendition-rnd_mobile.m3u8" \
    >"$stream/mobile.tmp"
  mv "$stream/mobile.tmp" "$stream/rendition-rnd_mobile.m3u8"
  expect_typed_failure fake_hls_endlist "$root" check_delivery_hls \
    "exactly one #EXT-X-ENDLIST tag"

  root="$TMP_DIR/fake-hls-vod"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed 's/^#EXT-X-PLAYLIST-TYPE:VOD$/#EXT-X-PLAYLIST-TYPE:VODGARBAGE/' \
    "$stream/rendition-rnd_mobile.m3u8" >"$stream/mobile.tmp"
  mv "$stream/mobile.tmp" "$stream/rendition-rnd_mobile.m3u8"
  expect_typed_failure fake_hls_vod "$root" check_delivery_hls \
    "exactly one #EXT-X-PLAYLIST-TYPE:VOD tag"

  root="$TMP_DIR/malformed-hls-extinf"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed -E 's/^#EXTINF:[0-9.]+,/#EXTINF:1oops,/' "$stream/rendition-rnd_mobile.m3u8" \
    >"$stream/mobile.tmp"
  mv "$stream/mobile.tmp" "$stream/rendition-rnd_mobile.m3u8"
  expect_typed_failure malformed_hls_extinf "$root" check_delivery_hls \
    "exactly three 1s segments"

  root="$TMP_DIR/inexact-hls-resolution"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed 's/RESOLUTION=640x360/RESOLUTION=640x3600/' "$stream/master.m3u8" >"$stream/master.tmp"
  mv "$stream/master.tmp" "$stream/master.m3u8"
  expect_typed_failure inexact_hls_resolution "$root" check_delivery_hls \
    "attributes do not match its codecs and raster"

  root="$TMP_DIR/wrong-hls-raster"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats/rendered"; stream="$dir/stream"
  make_hls_rendition "$dir/master.mp4" "$stream" rnd_mobile 320 180 1000k 1100k 2000k
  expect_typed_failure wrong_hls_mobile_raster "$root" check_delivery_hls "expected width=640"

  root="$TMP_DIR/wrong-hls-duration"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  sed -E 's/^#EXTINF:[0-9.]+,/#EXTINF:0.500000,/' \
    "$stream/rendition-rnd_mobile.m3u8" >"$stream/mobile.tmp"
  mv "$stream/mobile.tmp" "$stream/rendition-rnd_mobile.m3u8"
  expect_typed_failure wrong_hls_duration "$root" check_delivery_hls \
    "exactly three 1s segments"

  root="$TMP_DIR/four-hls-segments"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats/rendered"; stream="$dir/stream"
  make_custom_hls_rendition "$dir/master.mp4" "$stream" rnd_mobile 640 360 12 \
    yuv420p 0.75 1000k 1100k 2000k
  expect_typed_failure four_hls_segments "$root" check_delivery_hls \
    "exactly three 1s segments"

  root="$TMP_DIR/wrong-hls-fps"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats/rendered"; stream="$dir/stream"
  make_custom_hls_rendition "$dir/master.mp4" "$stream" rnd_mobile 640 360 10 \
    yuv420p 1 1000k 1100k 2000k
  expect_typed_failure wrong_hls_fps "$root" check_delivery_hls "expected r_frame_rate=12/1"

  root="$TMP_DIR/wrong-hls-pixel-format"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats/rendered"; stream="$dir/stream"
  make_custom_hls_rendition "$dir/master.mp4" "$stream" rnd_mobile 640 360 12 \
    yuv422p 1 1000k 1100k 2000k
  expect_typed_failure wrong_hls_pixel "$root" check_delivery_hls "expected pix_fmt=yuv420p"

  root="$TMP_DIR/wrong-hls-visual"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats/rendered"; stream="$dir/stream"; source="$root/wrong-visual.mp4"
  ffmpeg -nostdin -v error -y -f lavfi -i 'color=green:s=480x270:r=12:d=3' \
    -f lavfi -i 'sine=frequency=220:sample_rate=48000:duration=3' \
    -c:v libx264 -pix_fmt yuv420p -c:a aac -ac 2 -shortest "$source"
  make_hls_rendition "$source" "$stream" rnd_mobile 640 360 1000k 1100k 2000k
  expect_typed_failure wrong_hls_visual "$root" check_delivery_hls "opening does not match master"

  root="$TMP_DIR/wrong-middle-hls-visual"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats/rendered"; stream="$dir/stream"; source="$root/wrong-middle.mp4"
  ffmpeg -nostdin -v error -y -i "$dir/master.mp4" \
    -vf "drawbox=x=0:y=0:w=iw:h=ih:color=green:t=fill:enable='between(t,1.25,1.75)'" \
    -c:v libx264 -pix_fmt yuv420p -c:a copy "$source"
  make_hls_rendition "$source" "$stream" rnd_mobile 640 360 1000k 1100k 2000k
  expect_typed_failure wrong_middle_hls_visual "$root" check_delivery_hls \
    "before boundary does not match master"

  root="$TMP_DIR/wrong-hls-audio"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats/rendered"; stream="$dir/stream"; source="$root/wrong-audio.mp4"
  ffmpeg -nostdin -v error -y -i "$dir/master.mp4" -f lavfi \
    -i 'sine=frequency=440:sample_rate=48000:duration=3' -map 0:v:0 -map 1:a:0 \
    -c:v copy -c:a aac -ac 2 -t 3 "$source"
  make_hls_rendition "$source" "$stream" rnd_mobile 640 360 1000k 1100k 2000k
  expect_typed_failure wrong_hls_audio "$root" check_delivery_hls "same 220 Hz master mix"

  root="$TMP_DIR/corrupt-hls-segment"
  clone_delivery_fixture "$root"
  stream="$root/delivery-formats/rendered/stream"
  segment=$(find "$stream" -maxdepth 1 -type f -name 'segment-rnd_mobile-*.ts' | tail -n 1)
  printf '%s\n' 'not a transport stream' >"$segment"
  expect_typed_failure corrupt_hls_segment "$root" check_delivery_hls "does not fully decode"

  root="$TMP_DIR/wrong-hls-rate-control"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'walk(if type == "object" and .id? == "rnd_mobile"
      then .encoding.settings.rate_control.target_bps = 900000 else . end)'
  expect_typed_failure wrong_hls_rate_control "$root" check_delivery_hls \
    "HLS canonical contract"
}
