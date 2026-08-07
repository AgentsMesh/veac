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

text_animation_canonical_contract() {
  jq -e '
    def seconds: .value/.timescale;
    def granular_progress:
      .type=="keyframes" and (.keyframes|map(.time|seconds)==[0,0.5,1]) and
      (.keyframes|map(.value)==[0,0.55,1]);
    def clips: reduce (.project.sequences[].tracks[].clips[] |
      select(.source.type=="text")) as $clip
      ({}; .[$clip.authorship.logical_path[-1]]=$clip);
    clips as $c |
    ["whole","line","word","grapheme"] as $keys |
    ["整块动画","第一行动画\n第二行动画","逐词 动画 类型化 关键帧",
      "逐字动画：打字机揭示"] as $texts |
    ["whole","line","word","grapheme"] as $units |
    all(range(0;4); . as $index | $keys[$index] as $key | $c[$key] as $clip |
      ($clip.record_range.start|seconds)==($index*1.5) and
      ($clip.record_range.duration|seconds)==1.5 and $clip.source.text==$texts[$index] and
      $clip.source.style.animation as $animation |
      $animation.granularity==$units[$index] and ($animation.stagger|seconds)==0.045 and
      $animation.highlight.fill=={"red":250,"green":204,"blue":21,"alpha":255} and
      (if $index==0 then
        $animation.reveal=={"type":"constant","value":1} and
        $animation.highlight.progress=={"type":"constant","value":1}
       else
        ($animation.reveal|granular_progress) and
        ($animation.highlight.progress|granular_progress)
       end) and
      ($animation.opacity.keyframes|map(.time|seconds))==[0,0.4] and
      ($animation.opacity.keyframes|map(.value))==[0,1] and
      ($animation.transform.position_offset.keyframes|map(.value.y.value))==[28,0] and
      ($animation.transform.scale.keyframes|map(.value))==
        [{"x":0.75,"y":0.75},{"x":1,"y":1}])
  ' "$1" >/dev/null || fail "text-animation canonical channel contract failed"
}

text_animation_progress() {
  local video=$1 start=$2 label=$3 first middle last
  read -r first _ < <(text_stats "$video" "$(awk -v s="$start" 'BEGIN{print s+.15}')" \
    0 0 480 270 0 yellow)
  read -r middle _ < <(text_stats "$video" "$(awk -v s="$start" 'BEGIN{print s+.6}')" \
    0 0 480 270 0 yellow)
  read -r last _ < <(text_stats "$video" "$(awk -v s="$start" 'BEGIN{print s+1.15}')" \
    0 0 480 270 0 yellow)
  assert_text_progression "$label highlight/reveal" 4 "$first" "$middle" "$last"
}

check_text_animation() {
  local dir=$1 video=$2 canonical early_count early_width early_height early_x early_y
  local late_count late_width late_height late_x late_y time
  canonical=$(example_authoring_canonical "$dir")
  text_animation_canonical_contract "$canonical"
  check_text_media "$video" 6 480 270 "text-animation"
  for time in 0.15 0.6 1.15 1.65 2.1 2.65 3.15 3.6 4.15 4.65 5.1 5.65; do
    frame_hash "$video" "$time" >/dev/null || fail "text-animation frame is missing at ${time}s"
  done
  text_animation_progress "$video" 1.5 "text-animation line"
  text_animation_progress "$video" 3 "text-animation word"
  text_animation_progress "$video" 4.5 "text-animation grapheme"
  read -r early_count early_width early_height early_x early_y < <(
    text_stats "$video" 0.3 0 0 480 270 500)
  read -r late_count late_width late_height late_x late_y < <(
    text_stats "$video" 1.15 0 0 480 270 500)
  ((late_count>early_count+15 && late_width>early_width && late_y+4<early_y)) ||
    fail "text-animation whole transform/fade is not visible: early=$early_count/${early_width}x${early_height}@$early_x,$early_y late=$late_count/${late_width}x${late_height}@$late_x,$late_y"
  read -r late_count late_width late_height _ _ < <(text_stats "$video" 2.65 0 0 480 270 500)
  ((late_count>=80 && late_height>=20)) ||
    fail "text-animation two-line terminal layout is missing: $late_count/${late_width}x$late_height"
}

check_text_overlay() {
  local dir=$1 video=$2 white cyan detail_white canonical title detail
  local panel_r panel_g panel_b background_r background_g background_b
  local cyan_width cyan_height cyan_x cyan_y shadow shadow_width shadow_height shadow_x shadow_y
  local shadow_bottom
  canonical="$dir/project/project.veac.json"
  check_text_media "$video" 4 480 270 "text-overlay"
  if [[ -f $canonical ]]; then
    title=$(canonical_clip_id "$canonical" title)
    detail=$(canonical_clip_id "$canonical" detail)
    jq -e --arg title "$title" --arg detail "$detail" '
      def seconds: .value / .timescale;
      first(.project.sequences[].tracks[].clips[] | select(.id == $title)) as $title_clip |
      first(.project.sequences[].tracks[].clips[] | select(.id == $detail)) as $detail_clip |
      $title_clip.source.style as $s |
      ($title_clip.record_range.start | seconds) == 0 and
      ($title_clip.record_range.duration | seconds) == 2 and
      ($detail_clip.record_range.start | seconds) == 2 and
      ($detail_clip.record_range.duration | seconds) == 2 and
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
  read -r white _ < <(text_stats "$video" 1 45 95 390 80 650)
  read -r detail_white _ < <(text_stats "$video" 3 45 95 390 80 650)
  read -r cyan cyan_width cyan_height cyan_x cyan_y < <(
    text_stats "$video" 1 45 95 390 80 0 cyan)
  ((white > 100 && detail_white > 100 && cyan > 40)) || fail \
    "text-overlay text or cyan outline is missing: title=$white detail=$detail_white cyan=$cyan"
  read -r panel_r panel_g panel_b < <(region_rgb "$video" 1 '40:20:30:115')
  read -r background_r background_g background_b < <(region_rgb "$video" 1 '40:20:30:30')
  ((panel_r < background_r && panel_g + 10 < background_g &&
    panel_b + 25 < background_b)) || fail \
    "text-overlay padded panel is missing: panel=$panel_r,$panel_g,$panel_b background=$background_r,$background_g,$background_b"
  read -r shadow shadow_width shadow_height shadow_x shadow_y < <(
    text_stats "$video" 1 45 95 390 80 0 red)
  shadow_bottom=$((shadow_y + shadow_height - cyan_y - cyan_height))
  ((shadow >= 12 && shadow_x > cyan_x + 10 &&
    shadow_y > cyan_y + cyan_height + 4 && shadow_bottom <= 20)) || fail \
    "text-overlay shadow direction is invalid: shadow=$shadow/${shadow_width}x${shadow_height}@$shadow_x,$shadow_y text=$cyan/${cyan_width}x${cyan_height}@$cyan_x,$cyan_y"
  text_expect_box "$video" 1 40 90 400 90 500 120 110 9 390 80 "text overlay bounds"
}
