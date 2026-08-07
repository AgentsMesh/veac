#!/usr/bin/env bash

visual_region_rgb() {
  local media=$1 time=$2 crop=$3
  local sample rgb
  sample=$(mktemp "${TMPDIR:-/tmp}/veac-region-rgb.XXXXXX")
  if ! ffmpeg -v error -y -ss "$time" -i "$media" -frames:v 1 \
    -vf "crop=$crop,scale=1:1:flags=area,format=rgb24" -f rawvideo "$sample"; then
    rm -f "$sample"
    fail "could not sample RGB at ${time}s"
  fi
  rgb=$(od -An -tu1 -N3 "$sample" | awk 'NF >= 3 { print $1, $2, $3 }')
  rm -f "$sample"
  printf '%s\n' "$rgb"
}

rgb_delta() {
  awk -v first="$1" -v second="$2" '
    BEGIN {
      split(first, a); split(second, b); total = 0
      for (i = 1; i <= 3; i++) { delta = a[i] - b[i]; total += delta < 0 ? -delta : delta }
      print total
    }'
}

assert_gt() {
  awk -v actual="$1" -v reference="$2" -v gap="$3" \
    'BEGIN { exit !(actual >= reference + gap) }' || fail "$4: $1 is not >= $2 + $3"
}

assert_lt() {
  awk -v actual="$1" -v reference="$2" -v gap="$3" \
    'BEGIN { exit !(actual + gap <= reference) }' || fail "$4: $1 + $3 is not <= $2"
}

assert_rgb_delta_above() {
  local first second delta
  first=$(visual_region_rgb "$1" "$2" "$3")
  second=$(visual_region_rgb "$1" "$4" "$5")
  delta=$(rgb_delta "$first" "$second")
  assert_gt "$delta" 0 "$6" "$7 (RGB $first versus $second)"
}

assert_rgb_delta_below() {
  local first second delta
  first=$(visual_region_rgb "$1" "$2" "$3")
  second=$(visual_region_rgb "$1" "$4" "$5")
  delta=$(rgb_delta "$first" "$second")
  awk -v delta="$delta" -v maximum="$6" 'BEGIN { exit !(delta <= maximum) }' \
    || fail "$7: RGB distance $delta exceeds $6 ($first versus $second)"
}

assert_spread_above() {
  local y u v total
  read -r y u v <<< "$(frame_yuv_spread "$1" "$2")"
  [[ -n ${v:-} ]] || fail "$4: could not measure channel spread"
  total=$(awk -v y="$y" -v u="$u" -v v="$v" 'BEGIN { print y + u + v }')
  assert_gt "$total" 0 "$3" "$4 (spread $y/$u/$v)"
}

assert_blurred_circle_edge() {
  local video=$1 outside outer middle inner core
  outside=$(region_yavg "$video" 1 '8:24:112:123')
  outer=$(region_yavg "$video" 1 '8:24:124:123')
  middle=$(region_yavg "$video" 1 '8:24:132:123')
  inner=$(region_yavg "$video" 1 '8:24:140:123')
  core=$(region_yavg "$video" 1 '8:24:152:123')
  assert_gt "$outer" "$outside" 2 "圆形模糊边界缺少外侧过渡"
  assert_gt "$middle" "$outer" 8 "圆形模糊边界缺少第二级过渡"
  assert_gt "$inner" "$middle" 8 "圆形模糊边界缺少第三级过渡"
  assert_gt "$core" "$inner" 1.5 "圆形模糊边界没有平滑进入内部"
}

check_advanced_color_evidence() {
  local dir="$PREVIEW_ROOT/advanced-color"
  [[ -d $dir ]] || return 0
  local video first second
  video=$(delivery_video_path "$dir" preview preview.mp4)
  video_contract "$video" 7.9
  assert_mechanism_spread_above "$video" 0.5 20 "高级调色完整管线保留多色层次"
  assert_mechanism_spread_above "$video" 5.5 20 "一维查找表保留多色层次"
  assert_rgb_delta_above "$video" 0.5 "$(mechanism_crop)" 5.5 "$(mechanism_crop)" 18 \
    "两条查找表管线产生不同结果"
  first=$(mechanism_yavg "$video" 5.0); second=$(mechanism_yavg "$video" 5.1)
  assert_gt "$first" 16 4 "五秒剪切点不是空白帧"
  assert_gt "$second" 16 4 "剪切点后不是空白帧"
}

check_color_grade_evidence() {
  local dir="$PREVIEW_ROOT/color-grade"
  [[ -d $dir ]] || return 0
  local canonical video ref_left grade_left ref_rgb grade_rgb ref_warm grade_warm
  canonical=$(example_authoring_canonical "$dir")
  color_grade_canonical_contract "$canonical"
  video=$(delivery_video_path "$dir" preview preview.mp4)
  video_contract "$video" 3.9
  assert_rgb_delta_below "$video" 0.3 "$(mechanism_crop)" 1.7 "$(mechanism_crop)" 3 \
    "未调色参照片段保持稳定"
  assert_rgb_delta_below "$video" 2.3 "$(mechanism_crop)" 3.7 "$(mechanism_crop)" 3 \
    "基础调色片段保持稳定"
  assert_mechanism_spread_above "$video" 0.5 18 "未调色参照是连续渐变"
  assert_mechanism_spread_above "$video" 2.5 18 "调色后仍保留渐变结构"
  ref_left=$(region_yavg "$video" 0.5 "96:110:24:45")
  grade_left=$(region_yavg "$video" 2.5 "96:110:24:45")
  assert_gt "$grade_left" "$ref_left" 4 "无标签 ROI 中基础调色抬升暗部"
  ref_rgb=$(visual_region_rgb "$video" 0.5 "$(mechanism_crop)")
  grade_rgb=$(visual_region_rgb "$video" 2.5 "$(mechanism_crop)")
  read -r ref_r _ ref_b <<< "$ref_rgb"; read -r grade_r _ grade_b <<< "$grade_rgb"
  ref_warm=$((ref_r - ref_b)); grade_warm=$((grade_r - grade_b))
  assert_gt "$grade_warm" "$ref_warm" 4 "基础调色提升红蓝暖色差"
  assert_rgb_delta_above "$video" 0.5 "$(mechanism_crop)" 2.5 "$(mechanism_crop)" 12 \
    "同源原始画面与调色结果可辨"
  assert_rgb_delta_above "$video" 0.5 "70:150:400:10" 2.5 "70:150:400:10" 12 \
    "基础调色覆盖右侧无标签 ROI"
  assert_rgb_delta_above "$video" 0.5 "100:45:10:190" 2.5 "100:45:10:190" 12 \
    "基础调色覆盖下方无标签 ROI"
}

check_blend_modes_evidence() {
  local dir="$PREVIEW_ROOT/blend-modes"
  [[ -d $dir ]] || return 0
  local canonical video screen multiply darken lighten dodge burn normal
  canonical=$(example_authoring_canonical "$dir")
  blend_modes_canonical_contract "$canonical"
  video=$(delivery_video_path "$dir" preview preview.mp4)
  video_contract "$video" 11.9
  assert_unique_mechanism_regions "$video" 12 \
    0.5 1.5 2.5 3.5 4.5 5.5 6.5 7.5 8.5 9.5 10.5 11.5
  for time in 0.5 1.5 2.5 3.5 4.5 5.5 6.5 7.5 8.5 9.5 10.5 11.5; do
    assert_mechanism_spread_above "$video" "$time" 24 \
      "混合模式 ${time}s 保留同源空间纹理"
  done
  screen=$(mechanism_yavg "$video" 0.5); multiply=$(mechanism_yavg "$video" 1.5)
  darken=$(mechanism_yavg "$video" 3.5); lighten=$(mechanism_yavg "$video" 4.5)
  dodge=$(mechanism_yavg "$video" 5.5); burn=$(mechanism_yavg "$video" 6.5)
  normal=$(mechanism_yavg "$video" 11.5)
  assert_gt "$screen" "$normal" 3 "滤色比普通合成更亮"
  assert_lt "$multiply" "$normal" 3 "正片叠底比普通合成更暗"
  assert_gt "$lighten" "$darken" 5 "变亮与变暗的亮度关系"
  assert_gt "$dodge" "$burn" 8 "颜色减淡与颜色加深的亮度关系"
  assert_rgb_delta_above "$video" 9.5 "$(mechanism_crop)" \
    10.5 "$(mechanism_crop)" 12 \
    "差值与排除模式可辨"
  assert_visual_crop_variety "$video" 8 "70:150:400:10" \
    "混合模式覆盖右侧无标签 ROI" \
    0.5 1.5 2.5 3.5 4.5 5.5 6.5 7.5 8.5 9.5 10.5 11.5
  assert_visual_crop_variety "$video" 8 "100:45:10:190" \
    "混合模式覆盖下方无标签 ROI" \
    0.5 1.5 2.5 3.5 4.5 5.5 6.5 7.5 8.5 9.5 10.5 11.5
}

check_apply_scopes_evidence() {
  local dir="$PREVIEW_ROOT/apply-scopes"
  [[ -d $dir ]] || return 0
  local canonical video
  canonical=$(example_authoring_canonical "$dir")
  apply_scopes_canonical_contract "$canonical"
  video=$(delivery_video_path "$dir" preview preview.mp4)
  video_contract "$video" 5.9
  assert_unique_mechanism_regions "$video" 3 1 3 5
  assert_blurred_circle_edge "$video"
  assert_rgb_delta_below "$video" 1 "40:40:420:170" 3 "40:40:420:170" 6 \
    "图层作用域保持无叠加的外侧背景"
  assert_rgb_delta_above "$video" 1 "60:60:210:105" 3 "60:60:210:105" 50 \
    "图层作用域只提亮叠加图层"
  assert_rgb_delta_above "$video" 3 "40:40:420:170" 5 "40:40:420:170" 60 \
    "条目集合处理覆盖背景条目"
  assert_rgb_delta_above "$video" 3 "60:60:210:105" 5 "60:60:210:105" 100 \
    "条目集合处理覆盖叠加条目"
  local center outside center_red center_green center_blue outside_red
  center=$(visual_region_rgb "$video" 3 "36:36:222:117")
  outside=$(visual_region_rgb "$video" 3 "36:36:20:20")
  read -r center_red center_green center_blue <<< "$center"
  read -r outside_red _ _ <<< "$outside"
  assert_gt "$center_red" "$outside_red" 70 "强调圆覆盖画面中心而非左上角"
  assert_gt "$center_red" "$center_green" 25 "居中强调圆保持红色主体"
}
