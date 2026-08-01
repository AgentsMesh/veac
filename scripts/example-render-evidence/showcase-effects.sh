#!/usr/bin/env bash

check_effects_evidence() {
  local dir="$PREVIEW_ROOT/video-effects"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4" early_edge clear_edge clear_sat style_sat
  local clear_halo sharp_halo grain_delta spill_excess
  local style_center style_corner clear_green keyed_green keyed_rose luma_rose luma_green
  video_contract "$video" 3.6
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
  style_center=$(region_yavg "$video" 1.5 'iw/3:ih/3:iw/3:ih/3')
  style_corner=$(region_yavg "$video" 1.5 'iw/8:ih/8:0:0')
  awk -v center="$style_center" -v corner="$style_corner" \
    'BEGIN { exit !(center > corner + 8) }' ||
    fail "video-effects vignette does not darken the corners"

  clear_halo=$(frame_sharpen_halo_contrast "$video" 0.9)
  sharp_halo=$(frame_sharpen_halo_contrast "$video" 1.5)
  awk -v clear="$clear_halo" -v sharp="$sharp_halo" \
    'BEGIN { exit !(sharp > clear + 2.5) }' ||
    fail "video-effects sharpen does not produce an edge halo: $clear_halo->$sharp_halo"
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
}
