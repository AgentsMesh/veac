#!/usr/bin/env bash

agentsmesh_authoring_contract() {
  local canonical=$1
  jq -e '
    def t($v): {"timescale":1000,"value":$v};
    def r($s;$d): {"start":t($s),"duration":t($d)};
    def clip($id): first(.project.sequences[].tracks[].clips[] | select(.id == $id));
    clip("itm_backdrop") as $backdrop | clip("itm_title") as $title |
    clip("itm_promise") as $promise | clip("itm_caption") as $caption |
    $backdrop.record_range == r(0;15000) and
    $backdrop.source.generator.gradient == {"end":{"x":1,"y":1},"start":{"x":0,"y":0},
      "stops":[
        {"color":{"alpha":255,"blue":67,"green":42,"red":16},"offset":0},
        {"color":{"alpha":255,"blue":110,"green":118,"red":15},"offset":0.55},
        {"color":{"alpha":255,"blue":94,"green":197,"red":34},"offset":1}],"type":"linear"} and
    $title.record_range == r(0;5000) and $title.source.text == "AgentsMesh" and
    $title.source.style.animation as $a |
    $a.granularity == "grapheme" and $a.stagger == t(45) and
    $a.reveal.keyframes[0].value == 0.1 and
    $a.reveal.keyframes[0].interpolation == {"type":"ease_out"} and
    $a.reveal.keyframes[1].time == t(900) and $a.reveal.keyframes[1].value == 1 and
    $a.reveal.keyframes[1].interpolation == {"type":"linear"} and
    $a.opacity.keyframes[0].value == 0.75 and
    $a.opacity.keyframes[0].interpolation == {"type":"ease_out"} and
    $a.opacity.keyframes[1].time == t(700) and $a.opacity.keyframes[1].value == 1 and
    $a.opacity.keyframes[1].interpolation == {"type":"linear"} and
    $a.transform.position_offset.keyframes[0].value.y == {"unit":"pixels","value":48} and
    $a.transform.position_offset.keyframes[0].interpolation == {"type":"ease_out"} and
    $a.transform.position_offset.keyframes[1].time == t(900) and
    $a.transform.position_offset.keyframes[1].value.y == {"unit":"pixels","value":0} and
    $a.transform.position_offset.keyframes[1].interpolation == {"type":"linear"} and
    $a.transform.scale.keyframes[0].value == {"x":0.5,"y":0.5} and
    $a.transform.scale.keyframes[0].interpolation == {"type":"ease_out"} and
    $a.transform.scale.keyframes[1].time == t(900) and
    $a.transform.scale.keyframes[1].value == {"x":1,"y":1} and
    $a.transform.scale.keyframes[1].interpolation == {"type":"linear"} and
    $promise.record_range == r(5000;10000) and
    $promise.source.text == "一次构建系统，持续交付类型化产品。" and
    $caption.record_range == r(2000;10000) and
    $caption.source.text == "视频编辑成为类型化、可审查的代码。" and
    $caption.source.style.background == {"color":{"alpha":204,"blue":45,"green":31,"red":0},
      "padding_pixels":18}
  ' "$canonical" >/dev/null || fail "agentsmesh authoring contract failed"
}

agentsmesh_preview_contract() {
  local canonical=$1
  jq -e '
    def t($v): {"timescale":1000,"value":$v};
    def clip($id): first(.project.sequences[].tracks[].clips[] | select(.id == $id));
    first(.project.render_configs[] | select(.id == "out_preview")) as $out |
    clip("itm_backdrop").record_range.duration == t(15000) and
    clip("itm_title").record_range == {"start":t(0),"duration":t(5000)} and
    clip("itm_promise").record_range == {"start":t(5000),"duration":t(10000)} and
    clip("itm_caption").record_range == {"start":t(2000),"duration":t(10000)} and
    $out.raster == {"captions":"burn_in","frame_rate":{"denominator":1,"numerator":12},
      "height":270,"width":480} and
    first($out.deliverables[] | select(.id == "dlv_preview")) as $video |
    $video.target == {"name":"preview.mp4","type":"file"} and
    $video.kind.settings.video.codec == "h264" and
    $video.kind.settings.audio == null and $video.kind.settings.hardware == {"type":"software"}
  ' "$canonical" >/dev/null || fail "agentsmesh preview contract failed"
}

agentsmesh_plan_contract() {
  local plan=$1
  jq -e '
    def t($v): {"timescale":1000,"value":$v};
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    .output.id == "pout_preview" and .output.render_config_id == "out_preview" and
    .output.sequence_id == "seq_main" and
    first(.sequences[] | select(.id == "seq_main")).duration == t(15000) and
    clip("itm_title").source.content.presentation.style.animation.granularity == "grapheme" and
    clip("itm_title").source.content.presentation.style.animation.stagger == t(45) and
    clip("itm_title").record_range.duration == t(5000) and
    clip("itm_promise").record_range.start == t(5000) and
    clip("itm_caption").record_range == {"start":t(2000),"duration":t(10000)}
  ' "$plan" >/dev/null || fail "agentsmesh preview plan contract failed"
}

agentsmesh_text_sample() {
  text_stats "$1" "$2" 40 65 400 100 500
}

agentsmesh_assert_intro_motion() {
  local video=$1 times=(0.05 0.35 0.7 1.1 1.4) counts=() widths=() heights=() ys=()
  local time count width height x y index
  for time in "${times[@]}"; do
    read -r count width height x y < <(agentsmesh_text_sample "$video" "$time")
    counts+=("$count"); widths+=("$width"); heights+=("$height"); ys+=("$y")
  done
  for index in 1 2 3; do
    ((counts[index] > counts[index-1] + 8)) ||
      fail "agentsmesh grapheme reveal does not progress at ${times[index]}s"
  done
  # The 1.4s sample is the completed state compared with 4.8s below.
  ((widths[4] > widths[0] + 80 && heights[4] > heights[0] + 5)) ||
    fail "agentsmesh title scale is not visible"
  ((ys[4] + 5 < ys[0])) || fail "agentsmesh title does not rise into position"
  read -r count width height x y < <(agentsmesh_text_sample "$video" 4.8)
  ((counts[4] > 100 && count > 100 && width == widths[4] && height == heights[4] && y == ys[4])) ||
    fail "agentsmesh title is not complete and stable after 1.305s"
}

agentsmesh_assert_switch() {
  local video=$1 before_count before_width before_height before_x before_y
  local after_count after_width after_height after_x after_y before_title after_title center
  read -r before_count before_width before_height before_x before_y < <(
    text_stats "$video" 4.9 40 65 400 110 500)
  read -r after_count after_width after_height after_x after_y < <(
    text_stats "$video" 5.1 40 65 400 110 500)
  read -r before_title _ < <(text_stats "$video" 4.9 40 65 400 110 710)
  read -r after_title _ < <(text_stats "$video" 5.1 40 65 400 110 710)
  ((before_count > 100 && after_count > 100 && before_title > 40 && after_title <= 5 &&
    after_height <= before_height + 8)) ||
    fail "agentsmesh title/promise switch is not exclusive at 5s: before=${before_count}/${before_width}x${before_height}@${before_x},${before_y} after=${after_count}/${after_width}x${after_height}@${after_x},${after_y} title=$before_title/$after_title"
  center=$((40 + after_x + after_width / 2))
  ((center >= 228 && center <= 252)) ||
    fail "agentsmesh promise is not centered at 5s: center=$center"
}

agentsmesh_assert_caption_panel() {
  local video=$1 count width height x y abs_y center left right top bottom before
  read -r count width height x y < <(text_stats "$video" 6 0 190 480 75 500)
  ((count > 40 && width > 100 && height > 5 && x > 5 && y > 5)) ||
    fail "agentsmesh bottom caption is missing"
  center=$((x + width / 2))
  ((center >= 228 && center <= 252)) ||
    fail "agentsmesh bottom caption is not centered: center=$center"
  abs_y=$((190 + y))
  left=$(region_yavg "$video" 6 "3:${height}:$((x-4)):${abs_y}")
  right=$(region_yavg "$video" 6 "3:${height}:$((x+width+1)):${abs_y}")
  top=$(region_yavg "$video" 6 "${width}:3:${x}:$((abs_y-4))")
  bottom=$(region_yavg "$video" 6 "${width}:3:${x}:$((abs_y+height+1))")
  before=$(region_yavg "$video" 1.9 "${width}:$((height+8)):$((x-4)):$((abs_y-4))")
  awk -v l="$left" -v r="$right" -v t="$top" -v b="$bottom" -v bg="$before" '
    BEGIN { exit !(l<bg-6 && r<bg-6 && t<bg-6 && b<bg-6 && l>3 && r>3) }' ||
    fail "agentsmesh caption panel opacity or padding is not visible"
  local early late
  read -r early _ < <(text_stats "$video" 1.9 0 190 480 75 500)
  read -r late _ < <(text_stats "$video" 12.1 0 190 480 75 500)
  ((early < 20 && late < 20)) || fail "agentsmesh caption does not honor the 2-12s range"
}

agentsmesh_assert_gradient() {
  local video=$1 tl mid br
  read -r _ tl _ <<<"$(region_rgb "$video" 7 '50:40:5:5')"
  read -r _ mid _ <<<"$(region_rgb "$video" 7 '50:40:215:35')"
  read -r _ br _ <<<"$(region_rgb "$video" 7 '50:40:425:225')"
  ((mid > tl + 25 && br > mid + 25)) ||
    fail "agentsmesh diagonal blue-green gradient is missing: $tl/$mid/$br"
}

check_agentsmesh_intro_evidence() {
  local dir=${1:-"$PREVIEW_ROOT/agentsmesh-intro-15s"}
  [[ -d $dir ]] || return 0
  local author preview plan video
  author=$(example_authoring_canonical "$dir")
  preview=$(example_preview_canonical "$dir")
  plan=$(example_preview_plan "$dir" out_preview)
  video=$(delivery_video_path "$dir" preview preview)
  for file in "$author" "$preview" "$plan" "$video"; do require_file "$file"; done
  agentsmesh_authoring_contract "$author"
  agentsmesh_preview_contract "$preview"
  agentsmesh_plan_contract "$plan"
  video_contract "$video" 14.9
  assert_duration_close "$video" 15 0.08 "agentsmesh intro"
  assert_stream_count "$video" a 0 "agentsmesh intro"
  agentsmesh_assert_intro_motion "$video"
  agentsmesh_assert_switch "$video"
  agentsmesh_assert_caption_panel "$video"
  agentsmesh_assert_gradient "$video"
}
