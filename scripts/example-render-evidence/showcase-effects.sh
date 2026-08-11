#!/usr/bin/env bash

assert_sharpen_evidence() {
  local video=$1 clear_halo sharp_halo halo_delta
  clear_halo=$(frame_sharpen_halo_contrast "$video" 0.9)
  sharp_halo=$(frame_sharpen_halo_contrast "$video" 1.5)
  halo_delta=$(awk -v clear="$clear_halo" -v sharp="$sharp_halo" \
    'BEGIN { delta=sharp-clear; print (delta < 0 ? -delta : delta) }')
  awk -v delta="$halo_delta" 'BEGIN { exit !(delta > .2) }' ||
    fail "video-effects sharpen does not produce an edge halo: $clear_halo->$sharp_halo"
}

assert_directional_blur_evidence() {
  local video=$1 horizontal vertical early peak
  local horizontal_x horizontal_y vertical_x vertical_y early_x early_y peak_x peak_y
  horizontal=$(frame_axis_difference_avg "$video" 10.5) ||
    fail "video-effects horizontal directional blur pixels are unavailable"
  vertical=$(frame_axis_difference_avg "$video" 11.5) ||
    fail "video-effects vertical directional blur pixels are unavailable"
  early=$(frame_axis_difference_avg "$video" 12.15) ||
    fail "video-effects early animated directional blur pixels are unavailable"
  peak=$(frame_axis_difference_avg "$video" 13.15) ||
    fail "video-effects peak animated directional blur pixels are unavailable"
  read -r horizontal_x horizontal_y <<<"$horizontal"
  read -r vertical_x vertical_y <<<"$vertical"
  read -r early_x early_y <<<"$early"
  read -r peak_x peak_y <<<"$peak"

  awk -v hx="$horizontal_x" -v hy="$horizontal_y" \
    -v vx="$vertical_x" -v vy="$vertical_y" '
    BEGIN { exit !(hy > hx*2.5 && vx > vy*2.5 && hy > vy*2.5 && vx > hx*2.5) }
  ' || fail "video-effects directional blur axes are not distinct: horizontal=$horizontal vertical=$vertical"
  awk -v ex="$early_x" -v ey="$early_y" -v px="$peak_x" -v py="$peak_y" '
    BEGIN { exit !(ex+ey > 1 && px+py > 1 && ex > px*2 && ey > py*2) }
  ' || fail "video-effects directional blur radius does not animate: early=$early peak=$peak"
}

check_effects_evidence() {
  local dir="$PREVIEW_ROOT/video-effects"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4" early_edge clear_edge clear_sat style_sat
  local grain_delta spill_excess plugin_color_sat plugin_mono_sat
  local style_center style_corner clear_green keyed_green keyed_rose luma_rose luma_green
  video_contract "$video" 13.6
  early_edge=$(frame_edge_avg "$video" 0.1)
  clear_edge=$(frame_edge_avg "$video" 0.9)
  awk -v early="$early_edge" -v clear="$clear_edge" \
    'BEGIN { exit !(clear > early * 1.18) }' ||
    fail "video-effects blur does not recover edge detail: early=$early_edge clear=$clear_edge"

  clear_sat=$(frame_saturation_avg "$video" 0.9)
  style_sat=$(frame_saturation_avg "$video" 1.5)
  awk -v clear="$clear_sat" -v style="$style_sat" \
    'BEGIN { exit !(style < clear * .82) }' ||
    fail "video-effects style stage does not reduce saturation: clear=$clear_sat style=$style_sat"
  style_center=$(region_yavg "$video" 1.5 'iw/8:ih/8:7*iw/16:3*ih/4')
  style_corner=$(region_yavg "$video" 1.5 'iw/8:ih/8:0:0')
  awk -v center="$style_center" -v corner="$style_corner" \
    'BEGIN { exit !(center > corner + 8) }' ||
    fail "video-effects vignette does not darken the corners"

  assert_sharpen_evidence "$video"
  grain_delta=$(frame_difference_avg "$video" 1.25 1.75)
  awk -v delta="$grain_delta" 'BEGIN { exit !(delta > 0.04) }' ||
    fail "video-effects grain has no temporal texture: delta=$grain_delta"

  clear_green=$(region_color_count "$video" 0.9 'iw:ih:0:0' green)
  keyed_green=$(region_color_count "$video" 2.5 'iw:ih:0:0' green)
  keyed_rose=$(region_color_count "$video" 2.5 'iw:ih:0:0' rose)
  ((keyed_green * 3 < clear_green && keyed_rose > 1000)) ||
    fail "video-effects chroma key evidence failed: green=$clear_green->$keyed_green rose=$keyed_rose"
  spill_excess=$(frame_green_excess_avg "$video" 2.5)
  awk -v excess="$spill_excess" 'BEGIN { exit !(excess < 1.5) }' ||
    fail "video-effects spill suppression leaves a green fringe: excess=$spill_excess"

  luma_rose=$(region_color_count "$video" 3.5 'iw:ih:0:0' rose)
  luma_green=$(region_color_count "$video" 3.5 'iw:ih:0:0' green)
  ((luma_rose > 1000 && luma_green > 200)) ||
    fail "video-effects luma key does not reveal dark areas while retaining highlights"

  plugin_color_sat=$(frame_saturation_avg "$video" 8.5)
  plugin_mono_sat=$(frame_saturation_avg "$video" 9.5)
  awk -v color="$plugin_color_sat" -v mono="$plugin_mono_sat" \
    'BEGIN { exit !(color > 20 && mono < color * .2) }' ||
    fail "video-effects versioned plugin does not produce monochrome output: $plugin_color_sat->$plugin_mono_sat"
  assert_directional_blur_evidence "$video"
}
