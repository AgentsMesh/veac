#!/usr/bin/env bash

# shellcheck source=example-render-evidence/composition-image-overlay.sh
source "$SCRIPT_DIR/example-render-evidence/composition-image-overlay.sh"

check_positioned_evidence() {
  local dir="$PREVIEW_ROOT/transforms-and-animation"
  if [[ -d $dir ]]; then
    local video="$dir/rendered/preview.mp4"
    video_contract "$video" 3.8
    assert_unique_frames "$video" 4 0.9 1.4 2.4 3.8
    local content edge
    content=$(region_yavg "$video" 1.2 "iw/6:ih/5:19*iw/25:7*ih/10")
    edge=$(region_yavg "$video" 1.2 "iw/12:ih/8:0:0")
    awk -v content="$content" -v edge="$edge" 'BEGIN { exit !(content > edge + 8) }' \
      || fail "transform subject is clipped or off-canvas"
  fi

  dir="$PREVIEW_ROOT/text-layout"
  if [[ -d $dir ]]; then
    local video="$dir/rendered/preview.mp4"
    video_contract "$video" 3.5
    local content edge
    content=$(region_yavg "$video" 1.0 "iw/3:ih/8:iw/3:ih/5")
    edge=$(region_yavg "$video" 1.0 "iw/20:ih/10:0:0")
    awk -v content="$content" -v edge="$edge" 'BEGIN { exit !(content > edge + 3) }' \
      || fail "text layout content is missing or outside its frame"
  fi
}

check_mask_evidence() {
  local dir="$PREVIEW_ROOT/masks-and-mattes"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4"
  video_contract "$video" 9.5
  local times=(0.5 1.5 2.5 3.5 4.5 5.5 7.0 9.0)
  assert_unique_frames "$video" 8 "${times[@]}"
  local time center corner
  for time in "${times[@]:0:7}"; do
    center=$(region_yavg "$video" "$time" "iw/10:ih/10:9*iw/20:9*ih/20")
    corner=$(region_yavg "$video" "$time" "iw/12:ih/12:iw/20:ih/20")
    awk -v center="$center" -v corner="$corner" 'BEGIN { exit !(center > corner + 20) }' \
      || fail "mask stage at ${time}s does not reveal its center"
  done
  center=$(region_yavg "$video" 9.0 "iw/10:ih/10:9*iw/20:9*ih/20")
  corner=$(region_yavg "$video" 9.0 "iw/12:ih/12:iw/20:ih/20")
  awk -v center="$center" -v corner="$corner" 'BEGIN { exit !(corner > center + 8) }' \
    || fail "inverted alpha matte does not reveal the outer region"
}
