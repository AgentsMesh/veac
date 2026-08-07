#!/usr/bin/env bash

programming_language_project_contract() {
  local file=$1 fps=$2 first=$3 second=$4 first_title=$5 second_title=$6 font=$7
  jq -e --argjson fps "$fps" --arg first "$first" --arg second "$second" \
    --arg first_title "$first_title" --arg second_title "$second_title" --arg font "$font" '
    def seconds: .value/.timescale;
    def clip($id): first(.project.sequences[].tracks[].clips[] | select(.id==$id));
    clip($first) as $a | clip($second) as $b |
    clip($first_title) as $at | clip($second_title) as $bt |
    first(.project.materials[]|select(.id==$font)) as $font_asset |
    .project.sequences[0].name ==
      "可编程语言能力：结构与方法：静态类型组件 / 枚举与控制流：穷尽匹配生成" and
    [.project.sequences[0].settings.width,.project.sequences[0].settings.height,
      .project.sequences[0].settings.frame_rate.numerator] == [1280,720,$fps] and
    [($a.record_range.start|seconds),($a.record_range.duration|seconds),
      ($b.record_range.start|seconds),($b.record_range.duration|seconds)] == [0,3,3,3] and
    [$a.source.generator.color,$b.source.generator.color] ==
      [{red:43,green:101,blue:116,alpha:255},{red:123,green:49,blue:93,alpha:255}] and
    $a.visual.opacity.type == "binding" and $b.visual.opacity.type == "binding" and
    [$at.source.text,$bt.source.text] ==
      ["组件化：结构与方法：静态类型组件","枚举与控制流：穷尽匹配生成"] and
    all([$at,$bt][]; .source.style.font.material_id==$font) and
    $font_asset.kind=="font" and $font_asset.source.uri=="assets/veac-example-zh.ttf" and
    $font_asset.identity.digest=="64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774" and
    [.project.authorship.entity.events[].operation] == [8222,8234,8235,8231,8238] and
    [.project.sequences[0].authorship.entity.events[].operation] == [8223,8239] and
    ([.project.sequences[0].authorship.tracks[].entity.events|map(.operation)]|sort) ==
      ([[8226,8242],[8226,8242]]|sort) and
    all([$a,$b,$at,$bt][]; any(.authorship.events[].call_stack[]?; .function=="<closure>"))
  ' "$file" >/dev/null || fail "programming-language project contract failed: $file"
}

check_programming_language_evidence() {
  local dir="$PREVIEW_ROOT/programming-language"
  [[ -d $dir ]] || return 0
  local canonical preview plan video first second first_title second_title font
  IFS=$'\t' read -r canonical preview plan video < <(executable_preview_artifacts "$dir")
  first=$(executable_clip_id "$canonical" backdrops first)
  second=$(executable_clip_id "$canonical" backdrops second)
  first_title=$(executable_clip_id "$canonical" titles first)
  second_title=$(executable_clip_id "$canonical" titles second)
  font=$(executable_material_id "$canonical" example-font)
  programming_language_project_contract "$canonical" 30 "$first" "$second" \
    "$first_title" "$second_title" "$font"
  programming_language_project_contract "$preview" 12 "$first" "$second" \
    "$first_title" "$second_title" "$font"
  jq -e --arg first "$first" --arg second "$second" --arg font "$font" \
    --arg first_title "$first_title" --arg second_title "$second_title" '
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id==$id));
    clip($first) as $a | clip($second) as $b |
    .header.source.timebase==600 and (.sequences[0].duration.value/600)==6 and
    .output.raster=={width:480,height:270,frame_rate:{numerator:12,denominator:1},captions:"discard"} and
    [clip($first_title).source.content.text,clip($second_title).source.content.text] ==
      ["组件化：结构与方法：静态类型组件","枚举与控制流：穷尽匹配生成"] and
    [$a.visual.opacity.type,$b.visual.opacity.type]==["binding","binding"] and
    ([.temporal.bindings[].clocks[0].owner.item_id]|sort)==([$first,$second]|sort) and
    (.temporal.bindings|map(.program_id)|unique|length)==1 and
    any(.inputs[];.material_id==$font and .canonical_uri=="assets/veac-example-zh.ttf" and
      .observed_identity.digest=="64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774") and
    (.temporal.programs|length)==1 and
    [.temporal.programs[0].nodes[].kind.type] ==
      ["input","literal","binary","literal","compare","literal","select",
       "literal","compare","literal","select","curve_sample"]
  ' "$plan" >/dev/null || fail "programming-language resolved program contract failed"
  assert_executable_video "$video" 6 480 270 0 programming-language
  assert_executable_rgb "$video" 1.5 '80:60:20:20' '43 101 116' 20 \
    "programming-language first component color"
  assert_executable_rgb "$video" 4.5 '80:60:20:20' '123 49 93' 22 \
    "programming-language enum color"
  local early late
  early=$(region_yavg "$video" 0.2 '80:60:20:20')
  late=$(region_yavg "$video" 1.5 '80:60:20:20')
  assert_executable_gt "$late" "$early" 35 "programming-language temporal fade"
  assert_unique_frames "$video" 2 1.5 4.5
}
