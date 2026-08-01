#!/usr/bin/env bash

advanced_color_canonical_contract() {
  local canonical=$1 label=$2 root=${3:-.project}
  jq -e --arg root "$root" '
    def t($v): {"timescale":1000,"value":$v};
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
    clip("itm_reference") as $reference |
    clip("itm_hsl-only") as $hsl |
    clip("itm_curves-only") as $curves |
    clip("itm_wheels-only") as $wheels |
    clip("itm_full-grade") as $full |
    clip("itm_tone-curve") as $tone |
    clip("itm_chart") as $chart |
    clip("itm_alpha-check") as $background |
    $reference.record_range == {"start":t(0),"duration":t(1000)} and
    $hsl.record_range == {"start":t(1000),"duration":t(1000)} and
    $curves.record_range == {"start":t(2000),"duration":t(1000)} and
    $wheels.record_range == {"start":t(3000),"duration":t(1000)} and
    $full.record_range == {"start":t(4000),"duration":t(2000)} and
    $tone.record_range == {"start":t(6000),"duration":t(2000)} and
    ($chart.source.generator.gradient.stops | length == 4 and all(.color.alpha == 179)) and
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
      $full.visual.color_pipeline.stages[5].application ==
        {"interpolation":"tetrahedral","material_id":"med_cinematic"}
     else
      $full.visual.color_pipeline.stages[5].application ==
        {"input_id":"pin_cinematic","interpolation":"tetrahedral","kind":"three_dimensional"}
     end) and
    ($tone.visual.color_pipeline.stages | length) == 1 and
    (if $root == ".project" then
      $tone.visual.color_pipeline.stages[0] ==
        {"application":{"interpolation":"linear","material_id":"med_tone-curve"},"type":"lut"}
     else
      $tone.visual.color_pipeline.stages[0] ==
        {"application":{"input_id":"pin_tone-curve","interpolation":"linear",
         "kind":"one_dimensional"},"type":"lut"}
     end) and
    (if $root == ".project" then
      (material("med_cinematic") |
        .kind == "lut3d" and .source == {"type":"file","uri":"assets/cinematic.cube"}) and
      (material("med_tone-curve") |
        .kind == "lut1d" and .source == {"type":"file","uri":"assets/tone-curve.cube"})
     else
      (input("pin_cinematic") |
        .material_id == "med_cinematic" and .canonical_uri == "assets/cinematic.cube" and
        .kind == {"material_kind":"lut3d","type":"resource"} and
        .observed_identity.algorithm == "sha256" and
        ((.observed_identity.digest // "") | test("^[0-9a-f]{64}$"))) and
      (input("pin_tone-curve") |
        .material_id == "med_tone-curve" and .canonical_uri == "assets/tone-curve.cube" and
        .kind == {"material_kind":"lut1d","type":"resource"} and
        .observed_identity.algorithm == "sha256" and
        ((.observed_identity.digest // "") | test("^[0-9a-f]{64}$")))
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
  for first in "${times[@]}"; do colors+=("$(visual_region_rgb "$video" "$first" 'iw:ih:0:0')"); done
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
  bottom=$(visual_region_rgb "$video" 0.5 '24:45:0:225')
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
  plan=$(example_preview_plan "$dir" out_preview)
  video=$(delivery_video_path "$dir" preview preview)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  advanced_color_canonical_contract "$author" authoring
  advanced_color_canonical_contract "$preview" preview
  advanced_color_canonical_contract "$plan" plan .plan
  video_contract "$video" 7.9
  assert_unique_frames "$video" 6 0.5 1.5 2.5 3.5 5 7
  advanced_color_assert_rendered_stages "$video"
  advanced_color_assert_alpha_composite "$video"
}
