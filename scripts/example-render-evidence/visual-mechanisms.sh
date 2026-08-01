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

edge_yavg() {
  ffmpeg -v error -ss "$2" -i "$1" -frames:v 1 \
    -vf "crop=$3,format=gray,edgedetect=low=0.02:high=0.1,signalstats,metadata=print:file=-" \
    -f null - 2>/dev/null | awk -F= '/YAVG=/ { print $2; exit }'
}

check_advanced_color_evidence() {
  local dir="$PREVIEW_ROOT/advanced-color"
  [[ -d $dir ]] || return 0
  local video first second
  video=$(delivery_video_path "$dir" preview preview)
  video_contract "$video" 7.9
  assert_spread_above "$video" 0.5 20 "高级调色完整管线保留多色层次"
  assert_spread_above "$video" 5.5 20 "一维查找表保留多色层次"
  assert_rgb_delta_above "$video" 0.5 "iw:ih:0:0" 5.5 "iw:ih:0:0" 18 \
    "两条查找表管线产生不同结果"
  first=$(frame_yavg "$video" 5.0); second=$(frame_yavg "$video" 5.1)
  assert_gt "$first" 16 4 "五秒剪切点不是空白帧"
  assert_gt "$second" 16 4 "剪切点后不是空白帧"
}

check_color_grade_evidence() {
  local dir="$PREVIEW_ROOT/color-grade"
  [[ -d $dir ]] || return 0
  local video ref_left grade_left ref_rgb grade_rgb ref_warm grade_warm
  video=$(delivery_video_path "$dir" preview preview)
  video_contract "$video" 3.9
  assert_rgb_delta_below "$video" 0.3 "iw:ih:0:0" 1.7 "iw:ih:0:0" 3 \
    "未调色参照片段保持稳定"
  assert_rgb_delta_below "$video" 2.3 "iw:ih:0:0" 3.7 "iw:ih:0:0" 3 \
    "基础调色片段保持稳定"
  assert_spread_above "$video" 0.5 18 "未调色参照是连续渐变"
  assert_spread_above "$video" 2.5 18 "调色后仍保留渐变结构"
  ref_left=$(region_yavg "$video" 0.5 "96:180:24:45")
  grade_left=$(region_yavg "$video" 2.5 "96:180:24:45")
  assert_gt "$grade_left" "$ref_left" 4 "基础调色抬升暗部"
  ref_rgb=$(visual_region_rgb "$video" 0.5 "iw:ih:0:0")
  grade_rgb=$(visual_region_rgb "$video" 2.5 "iw:ih:0:0")
  read -r ref_r _ ref_b <<< "$ref_rgb"; read -r grade_r _ grade_b <<< "$grade_rgb"
  ref_warm=$((ref_r - ref_b)); grade_warm=$((grade_r - grade_b))
  assert_gt "$grade_warm" "$ref_warm" 4 "基础调色提升红蓝暖色差"
  assert_rgb_delta_above "$video" 0.5 "iw:ih:0:0" 2.5 "iw:ih:0:0" 12 \
    "同源原始画面与调色结果可辨"
}

check_blend_modes_evidence() {
  local dir="$PREVIEW_ROOT/blend-modes"
  [[ -d $dir ]] || return 0
  local video screen multiply darken lighten dodge burn normal
  video=$(delivery_video_path "$dir" preview preview)
  video_contract "$video" 11.9
  assert_unique_frames "$video" 12 0.5 1.5 2.5 3.5 4.5 5.5 6.5 7.5 8.5 9.5 10.5 11.5
  for time in 0.5 1.5 2.5 3.5 4.5 5.5 6.5 7.5 8.5 9.5 10.5 11.5; do
    assert_spread_above "$video" "$time" 24 "混合模式 ${time}s 保留同源空间纹理"
  done
  screen=$(frame_yavg "$video" 0.5); multiply=$(frame_yavg "$video" 1.5)
  darken=$(frame_yavg "$video" 3.5); lighten=$(frame_yavg "$video" 4.5)
  dodge=$(frame_yavg "$video" 5.5); burn=$(frame_yavg "$video" 6.5)
  normal=$(frame_yavg "$video" 11.5)
  assert_gt "$screen" "$normal" 3 "滤色比普通合成更亮"
  assert_lt "$multiply" "$normal" 3 "正片叠底比普通合成更暗"
  assert_gt "$lighten" "$darken" 5 "变亮与变暗的亮度关系"
  assert_gt "$dodge" "$burn" 8 "颜色减淡与颜色加深的亮度关系"
  assert_rgb_delta_above "$video" 9.5 "iw:ih:0:0" 10.5 "iw:ih:0:0" 12 \
    "差值与排除模式可辨"
}

check_generated_graphics_evidence() {
  local dir="$PREVIEW_ROOT/generated-graphics"
  [[ -d $dir ]] || return 0
  local video corner center line ellipse_corner rectangle_corner
  video=$(delivery_video_path "$dir" preview preview)
  video_contract "$video" 7.9
  assert_unique_frames "$video" 8 0.5 1.5 2.5 3.5 4.5 5.5 6.5 7.5
  assert_rgb_delta_above "$video" 0.5 "180:100:20:20" 0.5 "180:100:280:20" 45 \
    "透明生成器显露棋盘明暗格"
  assert_rgb_delta_below "$video" 0.5 "180:100:20:20" 0.5 "180:100:280:150" 8 \
    "棋盘同色对角格一致"
  assert_uniform_frame "$video" 1.5 3 "纯色生成器铺满画布"
  assert_spread_above "$video" 2.5 70 "径向渐变具有中心到边缘层次"
  for time in 3.5 4.5 5.5 6.5 7.5; do
    corner=$(region_yavg "$video" "$time" "32:24:4:4")
    awk -v value="$corner" 'BEGIN { exit !(value <= 22) }' \
      || fail "形状 ${time}s 的画布角落应保持黑色，实测 $corner"
  done
  center=$(region_yavg "$video" 3.5 "80:60:200:105")
  assert_gt "$center" 16 25 "圆角矩形中心有渐变填充"
  center=$(region_yavg "$video" 4.5 "80:60:200:120")
  assert_gt "$center" 16 40 "多边形内部有填充"
  center=$(region_yavg "$video" 5.5 "60:40:210:115")
  line=$(region_yavg "$video" 5.5 "200:14:140:21")
  assert_gt "$line" "$center" 8 "路径仅在轮廓位置出现渐变描边"
  ellipse_corner=$(region_yavg "$video" 6.5 "12:8:68:52")
  rectangle_corner=$(region_yavg "$video" 7.5 "12:8:68:52")
  assert_gt "$rectangle_corner" "$ellipse_corner" 20 "矩形角部与椭圆几何可辨"
}

check_apply_scopes_evidence() {
  local dir="$PREVIEW_ROOT/apply-scopes"
  [[ -d $dir ]] || return 0
  local video blurred sharp
  video=$(delivery_video_path "$dir" preview preview)
  video_contract "$video" 5.9
  assert_unique_frames "$video" 3 1 3 5
  blurred=$(edge_yavg "$video" 1 "300:220:90:25")
  sharp=$(edge_yavg "$video" 3 "300:220:90:25")
  assert_gt "$sharp" "$blurred" 0.25 "合成时间带模糊弱化圆形边缘"
  assert_rgb_delta_below "$video" 1 "64:48:8:8" 3 "64:48:8:8" 9 \
    "图层作用域不改变外侧背景"
  assert_rgb_delta_above "$video" 1 "100:80:190:95" 3 "100:80:190:95" 14 \
    "合成时间带与叠加图层结果可辨"
  assert_rgb_delta_above "$video" 3 "64:48:8:8" 5 "64:48:8:8" 12 \
    "条目集合处理覆盖背景条目"
  assert_rgb_delta_above "$video" 3 "100:80:190:95" 5 "100:80:190:95" 14 \
    "条目集合处理覆盖叠加条目"
}
