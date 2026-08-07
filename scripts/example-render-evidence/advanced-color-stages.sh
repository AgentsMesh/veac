#!/usr/bin/env bash

advanced_color_canonical_contract() {
  local canonical=$1 label=$2 root=${3:-.project} identity=${4:-$1}
  local reference hsl curves wheels full tone background
  reference=$(canonical_clip_id "$identity" reference)
  hsl=$(canonical_clip_id "$identity" hsl)
  curves=$(canonical_clip_id "$identity" curves)
  wheels=$(canonical_clip_id "$identity" wheels)
  full=$(canonical_clip_id "$identity" full)
  tone=$(canonical_clip_id "$identity" tone)
  background=$(canonical_clip_id "$identity" alpha-check)
  jq -e --arg root "$root" --arg reference "$reference" --arg hsl "$hsl" \
    --arg curves "$curves" --arg wheels "$wheels" --arg full "$full" \
    --arg tone "$tone" --arg background "$background" '
    def seconds: .value/.timescale;
    def clips:
      if $root == ".project" then .project.sequences[].tracks[].clips[]
      else .sequences[].tracks[].clips[] end;
    def clip($id): first(clips | select(.id == $id));
    def material($id):
      [.project.materials[] | select(.id == $id)] |
      if length == 1 then .[0] else null end;
    def input($id):
      [.inputs[] | select(.id == $id)] |
      if length == 1 then .[0] else null end;
    clip($reference) as $reference |
    clip($hsl) as $hsl |
    clip($curves) as $curves |
    clip($wheels) as $wheels |
    clip($full) as $full |
    clip($tone) as $tone |
    clip($background) as $background |
    {type:"anchor",anchor:"center",inset:{x:0,y:0}} as $centered |
    [[$reference,$hsl,$curves,$wheels,$full,$tone][] |
      [(.record_range.start|seconds),(.record_range.duration|seconds)]] ==
      [[0,1],[1,1],[2,1],[3,1],[4,2],[6,2]] and
    ($reference.source.generator.gradient.stops | length == 4 and all(.color.alpha == 179)) and
    all([$hsl,$curves,$wheels,$full,$tone][];
      .visual.placement == $centered and .visual.transform.anchor == {x:0.5,y:0.5}) and
    ($background.source.generator.gradient.stops | map(.color)) == [
      {"alpha":255,"blue":5,"green":5,"red":5},{"alpha":255,"blue":5,"green":5,"red":5},
      {"alpha":255,"blue":250,"green":250,"red":250},{"alpha":255,"blue":250,"green":250,"red":250}] and
    $hsl.visual.color_pipeline.stages == [{"type":"hsl","adjustment":
      {"hue_degrees":18,"lightness":0.06,"range":"red","saturation":0.3}}] and
    $curves.visual.color_pipeline.stages == [{"type":"curves","curves":{"blue":null,"green":null,
      "luma":{"interpolation":"monotonic","points":[{"input":0,"output":0.03},
      {"input":0.45,"output":0.52},{"input":1,"output":0.96}]},"red":{"interpolation":"monotonic",
      "points":[{"input":0,"output":0},{"input":0.5,"output":0.56},{"input":1,"output":1}]}}}] and
    $wheels.visual.color_pipeline.stages == [{"type":"wheels","wheels":
      {"gain":{"blue":-0.01,"green":0.02,"red":0.05},"gamma":{"blue":0,"green":0,"red":0.03},
       "lift":{"blue":0.04,"green":0,"red":0}}}] and
    ($full.visual.color_pipeline.stages | map(.type)) ==
      ["basic","matrix","hsl","curves","wheels","lut"] and
    $full.visual.color_pipeline.stages[0].adjustment ==
      {"exposure_stops":0.35,"fade":0.04,"highlights":-0.2,"shadows":0.18,
       "temperature_kelvin":4800,"tint":0.08} and
    $full.visual.color_pipeline.stages[1].adjustment ==
      {"matrix":[1.08,0.02,0,0.01,0.96,0.03,0,0.04,0.9],"offset":[0.01,0,0.02]} and
    $full.visual.color_pipeline.stages[2].adjustment ==
      {"hue_degrees":12,"lightness":0.04,"range":"red","saturation":0.18} and
    $full.visual.color_pipeline.stages[3].curves ==
      {"blue":null,"green":null,"luma":{"interpolation":"monotonic","points":[
       {"input":0,"output":0.03},{"input":0.45,"output":0.52},{"input":1,"output":0.96}]},
       "red":null} and
    $full.visual.color_pipeline.stages[4].wheels ==
      {"gain":{"blue":-0.01,"green":0.02,"red":0.05},
       "gamma":{"blue":0,"green":0,"red":0.03},"lift":{"blue":0.04,"green":0,"red":0}} and
    (if $root == ".project" then
      $full.visual.color_pipeline.stages[5].application.interpolation == "tetrahedral" and
      (material($full.visual.color_pipeline.stages[5].application.material_id) |
        .kind == "lut3d" and .source == {"type":"file","uri":"assets/cinematic.cube"})
     else
      $full.visual.color_pipeline.stages[5].application as $lut |
      $lut.interpolation == "tetrahedral" and $lut.kind == "three_dimensional" and
      (input($lut.input_id) | .canonical_uri == "assets/cinematic.cube")
     end) and
    ($tone.visual.color_pipeline.stages | length) == 1 and
    (if $root == ".project" then
      $tone.visual.color_pipeline.stages[0] as $lut |
      $lut.type == "lut" and $lut.application.interpolation == "linear" and
      (material($lut.application.material_id) |
        .kind == "lut1d" and .source == {"type":"file","uri":"assets/tone-curve.cube"})
     else
      $tone.visual.color_pipeline.stages[0] as $lut |
      $lut.type == "lut" and $lut.application.interpolation == "linear" and
      $lut.application.kind == "one_dimensional" and
      (input($lut.application.input_id) |
        .canonical_uri == "assets/tone-curve.cube" and
        .kind == {"material_kind":"lut1d","type":"resource"})
     end)
  ' "$canonical" >/dev/null || fail "advanced-color $label stage contract failed"
}

advanced_color_rgb_distance() {
  awk -v first="$1" -v second="$2" '
    BEGIN { split(first,a); split(second,b); for (i=1;i<=3;i++) {
      d=a[i]-b[i]; total += d < 0 ? -d : d } print total }'
}

advanced_color_assert_rendered_stages() {
  local video=$1 first second label delta index colors=() times=(0.5 1.5 2.5 3.5 5 7)
  for first in "${times[@]}"; do
    colors+=("$(visual_region_rgb "$video" "$first" "$(mechanism_crop)")")
  done
  for index in 1 2 3 4 5; do
    first=${colors[index-1]}; second=${colors[index]}; label="${times[index-1]}/${times[index]}"
    delta=$(advanced_color_rgb_distance "$first" "$second")
    awk -v value="$delta" 'BEGIN { exit !(value >= 6) }' ||
      fail "advanced-color rendered $label stages are not distinct: RGB distance $delta"
  done
}

advanced_color_assert_alpha_composite() {
  local video=$1 top bottom top_delta bottom_delta separation
  top=$(visual_region_rgb "$video" 0.5 '24:90:0:45')
  bottom=$(visual_region_rgb "$video" 0.5 '24:45:0:170')
  top_delta=$(advanced_color_rgb_distance "$top" '16 25 44')
  bottom_delta=$(advanced_color_rgb_distance "$bottom" '88 98 117')
  separation=$(advanced_color_rgb_distance "$top" "$bottom")
  awk -v top="$top_delta" -v bottom="$bottom_delta" -v separation="$separation" \
    'BEGIN { exit !(top <= 40 && bottom <= 40 && separation >= 150) }' ||
    fail "advanced-color alpha over dark/light backgrounds is wrong: top=$top bottom=$bottom"
}

check_advanced_color_stage_evidence() {
  local dir=${1:-"$PREVIEW_ROOT/advanced-color"}
  [[ -d $dir ]] || return 0
  local author preview plan video
  author=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  plan=$(delivery_plan_path "$dir" preview)
  video=$(delivery_video_path "$dir" preview preview.mp4)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  advanced_color_canonical_contract "$author" authoring
  advanced_color_canonical_contract "$preview" preview
  advanced_color_canonical_contract "$plan" plan .plan "$author"
  video_contract "$video" 7.9
  assert_unique_mechanism_regions "$video" 6 0.5 1.5 2.5 3.5 5 7
  assert_visual_crop_variety "$video" 5 "70:150:400:10" \
    "高级调色覆盖右侧无标签 ROI" 0.5 1.5 2.5 3.5 5 7
  assert_visual_crop_variety "$video" 5 "100:45:10:190" \
    "高级调色覆盖下方无标签 ROI" 0.5 1.5 2.5 3.5 5 7
  advanced_color_assert_rendered_stages "$video"
  advanced_color_assert_alpha_composite "$video"
}
