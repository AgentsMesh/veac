#!/usr/bin/env bash

screen_over_rgb() {
  local base=$1 source=$2 opacity=$3
  local br bg bb sr sg sb
  read -r br bg bb <<<"$base"
  read -r sr sg sb <<<"$source"
  awk -v br="$br" -v bg="$bg" -v bb="$bb" \
    -v sr="$sr" -v sg="$sg" -v sb="$sb" -v opacity="$opacity" '
    function channel(base, source) {
      screen = 255 - (255 - base) * (255 - source) / 255
      return base * (1 - opacity) + screen * opacity
    }
    BEGIN { printf "%.0f %.0f %.0f\n", channel(br, sr), channel(bg, sg), channel(bb, sb) }
  '
}

assert_rgb_triplet_near() {
  local label=$1 actual=$2 expected=$3 tolerance=$4
  local ar ag ab er eg eb
  read -r ar ag ab <<<"$actual"
  read -r er eg eb <<<"$expected"
  [[ -n ${ab:-} && -n ${eb:-} ]] || fail "$label: RGB sample is unavailable"
  awk -v ar="$ar" -v ag="$ag" -v ab="$ab" \
    -v er="$er" -v eg="$eg" -v eb="$eb" -v tolerance="$tolerance" '
    function abs(value) { return value < 0 ? -value : value }
    BEGIN { exit !(abs(ar-er) <= tolerance && abs(ag-eg) <= tolerance &&
      abs(ab-eb) <= tolerance) }
  ' || fail "$label: got $actual, expected $expected +/- $tolerance"
}

check_image_overlay_evidence() {
  local dir="$PREVIEW_ROOT/image-overlay"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4" logo="$dir/project/assets/logo.png"
  video_contract "$video" 2.5
  require_file "$logo"

  local base source actual expected
  base=$(region_rgb "$video" 1.5 'iw/12:ih/12:iw/20:ih/20')
  source=$(region_rgb "$logo" 0 '8:8:2*iw/5:9*ih/20')
  actual=$(region_rgb "$video" 1.5 '4:4:17*iw/20:3*ih/4')
  expected=$(screen_over_rgb "$base" "$source" 0.9)
  assert_rgb_triplet_near 'image overlay must use 90% opacity with screen blending' \
    "$actual" "$expected" 12

  local subject right_edge bottom_edge margin
  subject=$(region_yavg "$video" 1.5 'iw/5:ih/4:7*iw/10:3*ih/5')
  right_edge=$(region_yavg "$video" 1.5 'iw/50:ih/12:47*iw/50:3*ih/4')
  bottom_edge=$(region_yavg "$video" 1.5 'iw/12:ih/30:4*iw/5:23*ih/25')
  margin=$(region_yavg "$video" 1.5 'iw/50:ih/30:97*iw/100:19*ih/20')
  local background
  background=$(region_yavg "$video" 1.5 'iw/5:ih/4:iw/20:ih/20')
  awk -v content="$subject" -v edge="$background" 'BEGIN { exit !(content > edge + 12) }' \
    || fail 'image overlay is not visible over its background'
  awk -v edge="$right_edge" -v base="$background" 'BEGIN { exit !(edge > base + 8) }' \
    || fail 'image overlay right edge is inset too far'
  awk -v edge="$bottom_edge" -v base="$background" 'BEGIN { exit !(edge > base + 8) }' \
    || fail 'image overlay bottom edge is inset too far'
  awk -v area="$margin" -v base="$background" \
    'BEGIN { exit !((area-base < 8) && (base-area < 8)) }' \
    || fail 'image overlay does not preserve its bottom-right margin'
}
