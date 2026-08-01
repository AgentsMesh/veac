#!/usr/bin/env bash

assert_text_progression() {
  local label=$1 minimum=$2 values previous current
  shift 2
  values="$*"
  previous=$1
  shift
  for current in "$@"; do
    ((current > previous + minimum)) || fail "$label does not progress: $values"
    previous=$current
  done
}

check_text_animation() {
  local video=$1 fade_start fade_mid fade_end whole_visible
  local highlight_first highlight_middle highlight_full
  local slide_early slide_early_x slide_late slide_late_x
  local scale_early scale_early_width scale_early_height
  local scale_late scale_late_width scale_late_height
  local _ignored_width _ignored_height _ignored_y _ignored_x
  local line_top line_bottom line_late_bottom
  local word_one word_two word_three word_four
  local grapheme_one grapheme_two grapheme_three grapheme_four grapheme_five grapheme_six
  check_text_media "$video" 12 480 270 "text-animation"
  fade_start=$(region_yavg "$video" 0.05 '300:120:60:75')
  fade_mid=$(region_yavg "$video" 0.45 '300:120:60:75')
  fade_end=$(region_yavg "$video" 0.8 '300:120:60:75')
  read -r whole_visible _ < <(text_stats "$video" 0.8 0 0 480 270 100)
  awk -v start="$fade_start" -v mid="$fade_mid" -v end="$fade_end" \
    'BEGIN { exit !(mid > start + .5 && end > mid + .5) }' ||
    fail "text-animation whole fade does not progress: $fade_start,$fade_mid,$fade_end"
  ((whole_visible > 40)) || fail "text-animation whole fade has no visible terminal text"
  read -r slide_early _ignored_width _ignored_height slide_early_x _ignored_y < <(
    text_stats "$video" 0.8 0 0 480 270 100
  )
  read -r slide_late _ignored_width _ignored_height slide_late_x _ignored_y < <(
    text_stats "$video" 1.6 0 0 480 270 100
  )
  ((slide_early > 40 && slide_late > 40 && slide_late_x > slide_early_x + 20)) ||
    fail "text-animation whole slide is not visible: early=${slide_early}/${slide_early_x} late=${slide_late}/${slide_late_x}"
  read -r scale_early scale_early_width scale_early_height _ignored_x _ignored_y < <(
    text_stats "$video" 1.5 0 0 480 270 100
  )
  read -r scale_late scale_late_width scale_late_height _ignored_x _ignored_y < <(
    text_stats "$video" 2.5 0 0 480 270 100
  )
  ((scale_late > scale_early + 100 && scale_late_width > scale_early_width + 30 &&
    scale_late_height > scale_early_height + 5)) ||
    fail "text-animation whole scale is not visible: early=${scale_early}/${scale_early_width}x${scale_early_height} late=${scale_late}/${scale_late_width}x${scale_late_height}"
  read -r highlight_first _ < <(text_stats "$video" 3.55 0 0 480 270 0 yellow)
  read -r highlight_middle _ < <(text_stats "$video" 4.2 0 0 480 270 0 yellow)
  read -r highlight_full _ < <(text_stats "$video" 4.75 0 0 480 270 0 yellow)
  assert_text_progression "text-animation word highlight" 40 \
    "$highlight_first" "$highlight_middle" "$highlight_full"

  read -r line_top _ < <(text_stats "$video" 5.3 0 70 480 65 0 cyan)
  read -r line_bottom _ < <(text_stats "$video" 5.3 0 135 480 65 0 cyan)
  ((line_top > line_bottom + 20)) || fail "text-animation line stagger is not visible"
  read -r line_late_bottom _ < <(text_stats "$video" 5.9 0 135 480 65 0 cyan)
  ((line_late_bottom > line_bottom + 20)) ||
    fail "text-animation staggered second line does not appear"
  read -r word_one _ < <(text_stats "$video" 7.2 0 0 480 270 0 orange)
  read -r word_two _ < <(text_stats "$video" 7.5 0 0 480 270 0 orange)
  read -r word_three _ < <(text_stats "$video" 7.8 0 0 480 270 0 orange)
  read -r word_four _ < <(text_stats "$video" 8.1 0 0 480 270 0 orange)
  assert_text_progression "text-animation four-word stagger" 20 \
    "$word_one" "$word_two" "$word_three" "$word_four"
  read -r grapheme_one _ < <(text_stats "$video" 9.3 0 0 480 270 500)
  read -r grapheme_two _ < <(text_stats "$video" 9.55 0 0 480 270 500)
  read -r grapheme_three _ < <(text_stats "$video" 9.8 0 0 480 270 500)
  read -r grapheme_four _ < <(text_stats "$video" 10.05 0 0 480 270 500)
  read -r grapheme_five _ < <(text_stats "$video" 10.3 0 0 480 270 500)
  read -r grapheme_six _ < <(text_stats "$video" 10.55 0 0 480 270 500)
  assert_text_progression "text-animation six-grapheme reveal" 20 \
    "$grapheme_one" "$grapheme_two" "$grapheme_three" \
    "$grapheme_four" "$grapheme_five" "$grapheme_six"
}

check_text_overlay() {
  local dir=$1 video=$2 white cyan panel background canonical
  local shadow_right shadow_left shadow_bottom shadow_top
  canonical="$dir/project/project.veac.json"
  check_text_media "$video" 4 480 270 "text-overlay"
  if [[ -f $canonical ]]; then
    jq -e '
      first(.project.sequences[].tracks[].clips[] | select(.id == "itm_title")).source.style as $s |
      $s.color == {"alpha":255,"blue":255,"green":255,"red":255} and
      $s.outline.width_pixels == 2 and $s.outline.color ==
        {"alpha":255,"blue":216,"green":180,"red":0} and
      $s.background.color == {"alpha":153,"blue":0,"green":0,"red":0} and
      $s.background.padding_pixels == 14 and
      $s.shadow.color == {"alpha":255,"blue":126,"green":71,"red":255} and
      $s.shadow.opacity == 0.8 and $s.shadow.offset == {"x":24,"y":28} and
      $s.shadow.blur_pixels == 8
    ' "$canonical" >/dev/null || fail "text-overlay canonical style contract failed"
  fi
  read -r white _ < <(text_stats "$video" 2 45 95 390 80 650)
  cyan=$(region_color_count "$video" 2 '390:80:45:95' cyan)
  ((white > 100 && cyan > 40)) ||
    fail "text-overlay white fill or cyan outline is missing: white=$white cyan=$cyan"
  panel=$(region_color_count "$video" 2 '390:80:45:95' black)
  background=$(region_color_count "$video" 2 '390:50:45:30' black)
  ((panel > background + 300)) || fail \
    "text-overlay black padded panel is missing: panel=$panel background=$background"
  shadow_right=$(region_red_excess_avg "$video" 2 '6:26:326:125')
  shadow_left=$(region_red_excess_avg "$video" 2 '6:26:148:125')
  shadow_bottom=$(region_red_excess_avg "$video" 2 '176:6:154:142')
  shadow_top=$(region_red_excess_avg "$video" 2 '176:6:154:126')
  awk -v right="$shadow_right" -v left="$shadow_left" \
    'BEGIN { exit !(right > left + 3) }' ||
    fail "text-overlay rightward shadow is missing: right=$shadow_right left=$shadow_left"
  awk -v bottom="$shadow_bottom" -v top="$shadow_top" \
    'BEGIN { exit !(bottom > top + 3) }' ||
    fail "text-overlay downward shadow is missing: bottom=$shadow_bottom top=$shadow_top"
  text_expect_box "$video" 2 40 90 400 90 500 120 120 10 390 80 "text overlay bounds"
}
