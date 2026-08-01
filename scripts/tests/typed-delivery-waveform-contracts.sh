#!/usr/bin/env bash

run_delivery_waveform_contracts() {
  local root rendered

  root="$TMP_DIR/arbitrary-nonuniform-waveform"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -f lavfi \
    -i 'testsrc2=size=1280x720:rate=1:duration=1' -frames:v 1 \
    "$rendered/video-waveform.png"
  expect_typed_failure arbitrary_nonuniform_waveform "$root" check_delivery_waveform \
    "is not a sparse column waveform"

  root="$TMP_DIR/uniform-waveform"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -f lavfi \
    -i 'color=black:size=1280x720:rate=1:duration=1' -frames:v 1 \
    "$rendered/video-waveform.png"
  expect_typed_failure uniform_waveform "$root" check_delivery_waveform \
    "is not a sparse column waveform"

  root="$TMP_DIR/wrong-source-waveform"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -ss 2 -i "$rendered/master.mp4" \
    -vf 'format=yuv444p,waveform=mode=column:components=7:display=overlay,scale=1280:720' \
    -frames:v 1 "$rendered/video-waveform.png"
  expect_typed_failure wrong_source_waveform "$root" check_delivery_waveform \
    "does not match master at 1s"

  root="$TMP_DIR/wrong-waveform-scope-declaration"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'walk(if type == "object" and .id? == "dlv_video-waveform"
      then .kind.settings.scope = "vectorscope" else . end)'
  expect_typed_failure wrong_waveform_scope_declaration "$root" check_delivery_waveform \
    "video waveform canonical contract failed"
}
