#!/usr/bin/env bash

executable_text_family_project_contract() {
  local file=$1 word=$2 grapheme=$3 line=$4 whole=$5 font=$6
  jq -e --arg word "$word" --arg grapheme "$grapheme" --arg line "$line" \
    --arg whole "$whole" --arg font "$font" '
    def seconds: .value/.timescale;
    def clip($id): first(.project.sequences[].tracks[].clips[]|select(.id==$id));
    [clip($word),clip($grapheme),clip($line),clip($whole)] as $clips |
    first(.project.materials[]|select(.id==$font)) as $font_asset |
    [$clips[].record_range.start|seconds]==[0,2,4,6] and
    all($clips[];(.record_range.duration|seconds)==2) and
    [$clips[].source.text]==["逐词动画：代码化视频","逐字动画：中文可观察",
      "逐行动画：布局与装饰","整体动画：路径文字"] and
    [$clips[].source.style.animation.granularity]==["word","grapheme","line","whole"] and
    [$clips[].source.style.layout.writing_mode]==
      ["horizontal-tb","vertical-rl","vertical-lr","horizontal-tb"] and
    [$clips[].source.style.layout.orientation]==["mixed","upright","mixed","sideways"] and
    all($clips[];.source.style.font.material_id==$font) and
    $font_asset.kind=="font" and $font_asset.source.uri=="assets/veac-example-zh.ttf" and
    $font_asset.identity.digest=="64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774" and
    $clips[0].source.style.spans[0].start==0 and $clips[0].source.style.spans[0].end==4 and
    ($clips[3].source.style.path.points|length)==3 and
    $clips[3].source.style.path.start_offset=={value:286,unit:"pixels"} and
    ([ $clips[] | (.source.style.animation.transform.position_offset.keyframes[].id),
       (.source.style.animation.reveal.keyframes[].id) ] as $ids |
      ($ids|length)==40 and ($ids|unique|length)==40)
  ' "$file" >/dev/null || fail "executable-text-family project contract failed: $file"
}

check_executable_text_family_evidence() {
  local dir="$PREVIEW_ROOT/executable-text-family"
  [[ -d $dir ]] || return 0
  local canonical preview plan video word grapheme line whole font
  IFS=$'\t' read -r canonical preview plan video < <(executable_preview_artifacts "$dir")
  word=$(executable_clip_id "$canonical" text word)
  whole=$(executable_clip_id "$canonical" text whole)
  grapheme=$(executable_clip_id "$canonical" captions grapheme)
  line=$(executable_clip_id "$canonical" captions line)
  font=$(executable_material_id "$canonical" example-font)
  executable_text_family_project_contract "$canonical" "$word" "$grapheme" "$line" \
    "$whole" "$font"
  executable_text_family_project_contract "$preview" "$word" "$grapheme" "$line" \
    "$whole" "$font"
  jq -e --arg word "$word" '
    def clip($id): first(.project.sequences[].tracks[].clips[]|select(.id==$id));
    all(.project.sequences[].tracks[].clips[]|select(.source.type=="text" or .source.type=="caption");
      .source.style.fallback_fonts==
        [{type:"family",family:"PingFang SC"},{type:"family",family:"Noto Sans CJK SC"}]) and
    clip($word).source.style.spans[0].font=={type:"family",family:"sans-serif"}
  ' "$canonical" >/dev/null || fail "executable-text-family authoring font stack failed"
  jq -e --arg word "$word" '
    def clip($id): first(.project.sequences[].tracks[].clips[]|select(.id==$id));
    [ .project.sequences[].tracks[].clips[] |
      select(.source.type=="text" or .source.type=="caption") |
      .source.style.fallback_fonts[] | select(.type=="material") | .material_id ] as $fallbacks |
    .project.materials as $materials | clip($word).source.style.spans[0].font as $span_font |
    ($fallbacks|length)==8 and
    all($fallbacks[]; . as $id | any($materials[];.id==$id and .kind=="font")) and
    $span_font.type=="material" and
    any($materials[];.id==$span_font.material_id and .kind=="font")
  ' "$preview" >/dev/null || fail "executable-text-family preview font binding failed"
  jq -e --arg word "$word" --arg grapheme "$grapheme" --arg line "$line" \
    --arg whole "$whole" --arg font "$font" '
    def clip($id): first(.sequences[].tracks[].clips[]|select(.id==$id));
    [clip($word),clip($grapheme),clip($line),clip($whole)] as $clips |
    (.sequences[0].duration.value/600)==8 and
    .output.raster=={width:480,height:270,frame_rate:{numerator:12,denominator:1},captions:"burn_in"} and
    [$clips[].source.content.text]==["逐词动画：代码化视频","逐字动画：中文可观察",
      "逐行动画：布局与装饰","整体动画：路径文字"] and
    [$clips[].source.content.presentation.style.animation.granularity]==
      ["word","grapheme","line","whole"] and
    [$clips[].source.content.presentation.style.layout.writing_mode]==
      ["horizontal-tb","vertical-rl","vertical-lr","horizontal-tb"] and
    all($clips[];.source.content.presentation.style.font.requested.material_id==$font) and
    any(.inputs[];.material_id==$font and .canonical_uri=="assets/veac-example-zh.ttf" and
      .observed_identity.digest=="64afada094cf447d71fad4ee726799c4fa74917f7d9a1113ed7ea3c4234ff774") and
    all($clips[];(.source.content.presentation.style.fallback_fonts|length)==2) and
    ($clips[0].source.content.presentation.style.spans|length)==1 and
    ($clips[3].source.content.presentation.style.path.points|length)==3
  ' "$plan" >/dev/null || fail "executable-text-family plan contract failed"
  assert_executable_video "$video" 8 480 270 0 executable-text-family
  assert_unique_frames "$video" 4 1.5 3.5 5.5 7.5
  local c0 w0 h0 c1 w1 h1 c2 w2 h2 c3 w3 h3 early late
  read -r c0 w0 h0 < <(frame_bright_bbox "$video" 1.5 480 450)
  read -r c1 w1 h1 < <(frame_bright_bbox "$video" 3.5 480 450)
  read -r c2 w2 h2 < <(frame_bright_bbox "$video" 5.5 480 450)
  read -r c3 w3 h3 < <(frame_bright_bbox "$video" 7.5 480 450)
  ((c0>1000 && w0>250 && h0<70)) || fail "text-family word phase is not horizontal"
  ((c1>700 && w1<70 && h1>130)) || fail "text-family grapheme phase is not vertical"
  ((c2>700 && w2<70 && h2>130)) || fail "text-family line phase is not vertical"
  ((c3>1200 && w3>220 && h3>30)) || fail "text-family path phase is not visible"
  read -r early _ _ < <(frame_bright_bbox "$video" 0.2 480 450)
  read -r late _ _ < <(frame_bright_bbox "$video" 0.9 480 450)
  ((late>early+800)) || fail "text-family word reveal does not progress"
}
