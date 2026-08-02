#!/usr/bin/env bash

run_delivery_image_contracts() {
  local root rendered

  root="$TMP_DIR/static-gif"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -f lavfi -i 'color=#c43a69:s=480x270:r=12:d=3' \
    -loop 0 "$rendered/loop-preview.gif"
  expect_typed_failure static_delivery_gif "$root" check_delivery_images \
    "expected 2 distinct frames"

  root="$TMP_DIR/moving-wrong-color-gif"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -f lavfi \
    -i 'testsrc2=size=480x270:rate=12:duration=3' -loop 0 "$rendered/loop-preview.gif"
  expect_typed_failure moving_wrong_color_gif "$root" check_delivery_images \
    "delivery GIF opening stage"

  root="$TMP_DIR/wrong-middle-gif"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -i "$rendered/master.mp4" \
    -vf "drawbox=x=0:y=0:w=iw:h=ih:color=green:t=fill:enable='between(t,1.25,1.75)'" \
    -an -loop 0 "$rendered/loop-preview.gif"
  expect_typed_failure wrong_middle_gif "$root" check_delivery_images \
    "before boundary does not match master"

  root="$TMP_DIR/wrong-boundary-frame-gif"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -i "$rendered/master.mp4" \
    -vf "drawbox=x=0:y=0:w=iw:h=ih:color=green:t=fill:enable='eq(n,18)'" \
    -an -loop 0 "$rendered/loop-preview.gif"
  expect_typed_failure wrong_boundary_frame_gif "$root" check_delivery_images \
    "after boundary does not match master"

  root="$TMP_DIR/missing-gif-loop"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -framerate 12 -start_number 1 \
    -i "$rendered/frame-%04d.png" -loop -1 "$rendered/loop-preview.gif"
  expect_typed_failure missing_delivery_gif_loop "$root" check_delivery_images \
    "infinite-loop extension"

  root="$TMP_DIR/wrong-cover-time"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  cp "$rendered/frame-0001.png" "$rendered/cover.png"
  expect_typed_failure wrong_delivery_cover_time "$root" check_delivery_images \
    "delivery cover at 2s"

  root="$TMP_DIR/tampered-cover"
  clone_delivery_fixture "$root"
  rendered="$root/delivery-formats/rendered"
  ffmpeg -nostdin -v error -y -i "$rendered/cover.png" \
    -vf 'drawbox=x=20:y=20:w=24:h=24:color=0xd45a79:t=fill' "$rendered/cover.tmp.png"
  mv "$rendered/cover.tmp.png" "$rendered/cover.png"
  expect_typed_failure tampered_delivery_cover "$root" check_delivery_images \
    "does not visually match frame 25"
}
