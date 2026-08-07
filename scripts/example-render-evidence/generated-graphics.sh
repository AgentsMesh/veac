#!/usr/bin/env bash

generated_graphics_canonical_contract() {
  local canonical=$1
  jq -e '
    def seconds: .value/.timescale;
    def clips: reduce (.project.sequences[].tracks[].clips[]) as $clip
      ({}; .[$clip.authorship.logical_path[-1]]=$clip);
    clips as $c |
    ["solid","transparent","linear","radial","rectangle","rounded","ellipse","polygon","path"] as $keys |
    all(range(0;9); . as $index | $keys[$index] as $key |
      ($c[$key].record_range.start|seconds)==$index and
      ($c[$key].record_range.duration|seconds)==1) and
    [$c.solid.source.generator.type,$c.transparent.source.generator.type,
      $c.linear.source.generator.type+":"+$c.linear.source.generator.gradient.type,
      $c.radial.source.generator.type+":"+$c.radial.source.generator.gradient.type,
      $c.rectangle.source.generator.shape.geometry.type,
      $c.rounded.source.generator.shape.geometry.type,
      $c.ellipse.source.generator.shape.geometry.type,
      $c.polygon.source.generator.shape.geometry.type,
      $c.path.source.generator.shape.geometry.type] ==
      ["solid","transparent","gradient:linear","gradient:radial","rectangle",
       "rounded_rectangle","ellipse","polygon","path"] and
    $c.solid.source.generator.color=={"red":220,"green":38,"blue":38,"alpha":255} and
    ($c.polygon.source.generator.shape.geometry.points|length)==4 and
    ($c.path.source.generator.shape.geometry.commands|map(.type))==
      ["move_to","line_to","line_to","line_to","close"] and
    ($c.path.source.generator.shape.geometry.commands|map(.point // null))==
      [{"x":0.1,"y":0.8},{"x":0.35,"y":0.2},{"x":0.65,"y":0.8},
       {"x":0.9,"y":0.2},null] and
    $c.path.source.generator.shape.fill==null and
    $c.path.source.generator.shape.stroke.width_pixels==8
  ' "$canonical" >/dev/null || fail "generated-graphics canonical generator order contract failed"
}

generated_graphics_assert_rgb() {
  local video=$1 time=$2 expected=$3 tolerance=$4 label=$5 actual
  actual=$(visual_region_rgb "$video" "$time" '140:70:8:8')
  local delta
  delta=$(rgb_delta "$actual" "$expected")
  awk -v delta="$delta" -v tolerance="$tolerance" 'BEGIN { exit !(delta<=tolerance) }' ||
    fail "$label: RGB $actual differs from $expected by $delta"
}

check_generated_graphics_evidence() {
  local dir="$PREVIEW_ROOT/generated-graphics"
  [[ -d $dir ]] || return 0
  local canonical video time corner center line rectangle_corner rounded_corner ellipse_corner
  canonical=$(example_authoring_canonical "$dir")
  video=$(delivery_video_path "$dir" preview preview.mp4)
  generated_graphics_canonical_contract "$canonical"
  video_contract "$video" 8.9
  assert_unique_frames "$video" 9 0.5 1.5 2.5 3.5 4.5 5.5 6.5 7.5 8.5
  generated_graphics_assert_rgb "$video" 0.5 '220 38 38' 18 "纯色生成器"
  generated_graphics_assert_rgb "$video" 1.5 '17 24 39' 15 "透明生成器显露背景"
  assert_rgb_delta_above "$video" 2.5 '120:80:20:20' 2.5 '120:80:340:170' 60 \
    "线性渐变具有方向层次"
  assert_rgb_delta_above "$video" 3.5 '100:70:142:127' 3.5 '100:70:360:12' 45 \
    "径向渐变具有中心到边缘层次"
  for time in 4.5 5.5 6.5 7.5 8.5; do
    corner=$(region_yavg "$video" "$time" '28:20:4:4')
    awk -v value="$corner" 'BEGIN { exit !(value<=45) }' ||
      fail "形状 ${time}s 的画布角落未显露背景：$corner"
  done
  center=$(region_yavg "$video" 4.5 '80:50:200:110')
  assert_gt "$center" 20 30 "矩形内部有渐变填充"
  center=$(region_yavg "$video" 5.5 '80:50:200:110')
  assert_gt "$center" 20 25 "圆角矩形内部有填充"
  center=$(region_yavg "$video" 7.5 '60:40:210:115')
  assert_gt "$center" 20 25 "多边形内部有填充"
  rectangle_corner=$(region_yavg "$video" 4.5 '12:8:76:58')
  rounded_corner=$(region_yavg "$video" 5.5 '12:8:76:58')
  ellipse_corner=$(region_yavg "$video" 6.5 '12:8:100:58')
  assert_gt "$rectangle_corner" "$rounded_corner" 12 "矩形与圆角矩形角部可辨"
  assert_gt "$rectangle_corner" "$ellipse_corner" 12 "矩形与椭圆角部可辨"
  center=$(region_yavg "$video" 8.5 '60:30:210:10')
  line=$(region_yavg "$video" 8.5 '24:24:100:124')
  assert_gt "$line" "$center" 8 "路径仅在折线位置出现渐变描边"
  local count width height
  read -r count width height < <(frame_bright_bbox "$video" 1.5 480 600)
  ((count>=80 && width>=120 && height>=8)) ||
    fail "generated-graphics 中文说明缺失：$count/${width}x$height"
}
