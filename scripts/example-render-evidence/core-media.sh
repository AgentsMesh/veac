#!/usr/bin/env bash

check_standard_preview_video() {
  local video=$1
  local duration=$2
  local label=$3
  video_contract "$video" "$duration"
  assert_duration_close "$video" "$duration" 0.03 "$label"
  assert_stream_count "$video" v 1 "$label"
  assert_stream_count "$video" a 0 "$label"
  assert_stream_field "$video" v:0 codec_name h264 "$label"
  assert_stream_field "$video" v:0 width 480 "$label"
  assert_stream_field "$video" v:0 height 270 "$label"
  assert_stream_field "$video" v:0 r_frame_rate 12/1 "$label"
}

assert_core_rgb() {
  local video=$1 time=$2 crop=$3 expected=$4 tolerance=$5 label=$6
  local actual red green blue want_red want_green want_blue
  actual=$(region_rgb "$video" "$time" "$crop")
  read -r red green blue <<<"$actual"
  read -r want_red want_green want_blue <<<"$expected"
  [[ -n ${blue:-} ]] || fail "$label: RGB sample is unavailable"
  awk -v r="$red" -v g="$green" -v b="$blue" -v er="$want_red" \
    -v eg="$want_green" -v eb="$want_blue" -v tolerance="$tolerance" '
    function abs(value) { return value < 0 ? -value : value }
    BEGIN { exit !(abs(r-er)<=tolerance && abs(g-eg)<=tolerance && abs(b-eb)<=tolerance) }
  ' || fail "$label: got $actual, expected $expected +/- $tolerance"
}

check_minimal_evidence() {
  local dir="$PREVIEW_ROOT/minimal"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4" canonical background label
  canonical=$(example_authoring_canonical "$dir")
  background=$(canonical_clip_id "$canonical" background)
  label=$(canonical_clip_id "$canonical" label)
  jq -e --arg background "$background" --arg label "$label" '
    def seconds: .value/.timescale;
    def clip($id): first(.project.sequences[].tracks[].clips[] | select(.id==$id));
    (.project.sequences | length)==1 and
    (.project.sequences[0].tracks | length)==1 and
    .project.sequences[0].tracks[0].kind=="visual" and
    (.project.sequences[0].tracks[0].clips | length)==2 and
    clip($background) as $background | clip($label) as $label |
    ($background.record_range.start|seconds)==0 and
    ($background.record_range.duration|seconds)==3 and
    $background.source=={"type":"generated","generator":{"type":"solid",
      "color":{"red":15,"green":118,"blue":110,"alpha":255}}} and
    ($label.record_range.start|seconds)==0 and ($label.record_range.duration|seconds)==3 and
    $label.source.type=="text" and
    $label.source.text=="最小 VEAC：项目、时间线、轨道、条目与交付" and
    all(.project.render_configs[].deliverables[]; .kind.settings.audio==null)
  ' "$canonical" >/dev/null || fail "minimal canonical structure contract failed"
  check_standard_preview_video "$video" 3 minimal
  local time count width height
  for time in 0.1 1.5 2.9; do
    assert_core_rgb "$video" "$time" '80:60:8:8' '15 118 110' 6 \
      "minimal clean background at ${time}s"
    read -r count width height < <(frame_bright_bbox "$video" "$time" 480 600)
    ((count >= 90 && width >= 100 && height >= 8)) ||
      fail "minimal Chinese explanation is missing at ${time}s: $count/${width}x$height"
  done
}

assert_dissolve_progress() {
  local video=$1
  local early_r early_g early_b late_r late_g late_b
  read -r early_r early_g early_b <<< "$(region_rgb "$video" 1.716 '120:80:8:8')"
  read -r late_r late_g late_b <<< "$(region_rgb "$video" 1.883 '120:80:8:8')"
  [[ -n ${late_b:-} ]] || fail "transitions dissolve frames could not be sampled"
  ((early_r > late_r && early_g < late_g && early_b < late_b)) \
    || fail "transitions dissolve does not progress monotonically toward blue"
  ((early_r < 226 && early_r > 73 && early_g > 61 && early_g < 119)) \
    || fail "transitions early dissolve frame does not mix both clips"
  ((late_r < 226 && late_r > 73 && late_b > 74 && late_b < 153)) \
    || fail "transitions late dissolve frame does not mix both clips"
}

check_transitions_evidence() {
  local dir="$PREVIEW_ROOT/transitions"
  [[ -d $dir ]] || return 0
  local video="$dir/rendered/preview.mp4" canonical first second label
  canonical=$(example_authoring_canonical "$dir")
  first=$(canonical_clip_id "$canonical" first)
  second=$(canonical_clip_id "$canonical" second)
  label=$(canonical_clip_id "$canonical" explanation)
  check_standard_preview_video "$video" 4 transitions
  local plan
  plan=$(delivery_plan_path "$dir" preview)
  require_file "$plan"
  jq -e --arg first "$first" --arg second "$second" --arg label "$label" '
      def seconds: .value/.timescale;
      def clip($id): first(.project.sequences[].tracks[].clips[] | select(.id==$id));
      clip($first) as $first | clip($second) as $second | clip($label) as $label |
      $first.source.generator.color=={"red":230,"green":57,"blue":70,"alpha":255} and
      ($first.record_range.start|seconds)==0 and ($first.record_range.duration|seconds)==2 and
      $second.source.generator.color=={"red":69,"green":123,"blue":157,"alpha":255} and
      ($second.record_range.start|seconds)==1.6 and ($second.record_range.duration|seconds)==2.4 and
      $label.source.text=="居中叠化：两个真实画面在四百毫秒窗口内交叉混合" and
      any(.project.relations[]; .kind.type=="transition" and
        .kind.from.item_id==$first.id and .kind.to.item_id==$second.id and
        .kind.transition.kind.type=="dissolve" and
        (.kind.transition.duration|seconds)==0.4 and .kind.transition.alignment=="centered")
    ' "$canonical" >/dev/null || fail "transitions canonical identity contract failed"
  jq -e '
      def seconds: .value/.timescale;
      [.sequences[].tracks[].transitions[]] as $transitions
      | ($transitions | length) == 1
      and $transitions[0].kind.type == "dissolve"
      and $transitions[0].alignment == "centered"
      and ($transitions[0].cut_time|seconds) == 1.8
      and ($transitions[0].record_window.start|seconds) == 1.6
      and ($transitions[0].record_window.duration|seconds) == 0.4
      and ($transitions[0].outgoing_range.start|seconds) == 1.6
      and ($transitions[0].outgoing_range.duration|seconds) == 0.4
      and ($transitions[0].incoming_range.start|seconds) == 0
      and ($transitions[0].incoming_range.duration|seconds) == 0.4
    ' "$plan" >/dev/null || fail "transitions plan is not a true 1.6s-2.0s overlap"
  assert_core_rgb "$video" 0.5 '120:80:8:8' '230 57 70' 6 "transitions red stage"
  assert_core_rgb "$video" 1.5 '120:80:8:8' '230 57 70' 6 "transitions before dissolve"
  assert_dissolve_progress "$video"
  assert_core_rgb "$video" 2.1 '120:80:8:8' '69 123 157' 6 "transitions after dissolve"
  assert_core_rgb "$video" 3.5 '120:80:8:8' '69 123 157' 6 "transitions blue stage"
  local count width height
  read -r count width height < <(frame_bright_bbox "$video" 0.5 480 600)
  ((count >= 90 && width >= 120 && height >= 8)) ||
    fail "transitions Chinese explanation is missing: $count/${width}x$height"
}
