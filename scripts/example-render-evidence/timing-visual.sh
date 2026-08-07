#!/usr/bin/env bash
# shellcheck disable=SC2016

assert_transition_gallery_plan_evidence() {
  assert_plan_query "$1" '
    . as $plan |
    def as_seconds: .value / .timescale;
    def near($value; $target):
      (($value | as_seconds) - $target | if . < 0 then -. else . end) < 0.000001;
    def transition($kind):
      [$plan.sequences[].tracks[].transitions[] | select(.kind.type == $kind)] |
      if length == 1 then .[0] else error("transition kind is not unique") end;
    transition("pixelize").kind.amount == 0.65 and
    ([
      ["dissolve",0.65,0.825], ["fade",2.65,2.825],
      ["wipe",4.65,4.825], ["slide",6.65,6.825],
      ["zoom",8.65,8.825], ["circle",10.65,10.825],
      ["pixelize",12.65,12.825]
    ] | all(.[]; . as $expected |
      transition($expected[0]) as $transition |
      $transition.alignment == "centered" and
      near($transition.record_window.start; $expected[1]) and
      near($transition.record_window.duration; 0.35) and
      near($transition.cut_time; $expected[2])))
  ' 'transition gallery plan contract failed'
}

check_transition_gallery_timing_evidence() {
  local dir=$1 plan video canonical
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview.mp4)
  canonical="$dir/project/project.veac.json"
  assert_transition_gallery_plan_evidence "$plan"

  video_contract "$video" 13.9
  local stage clip_id
  while read -r sample clip red green blue; do
    stage="transition stage at ${sample}s"
    local ir_color
    clip_id=$(canonical_clip_id "$canonical" "$clip")
    ir_color=$(jq -er --arg clip "$clip_id" '
      [
        .project.sequences[].tracks[].clips[]
        | select(.id == $clip)
        | .source
        | select(.type == "generated" and .generator.type == "solid")
        | .generator.color
        | [.red, .green, .blue]
      ] as $matches
      | if ($matches | length) == 1 then $matches[0] | @tsv
        else error("expected exactly one generated solid clip \($clip)")
        end
    ' "$canonical") || fail "transition-gallery $clip canonical clip/color is missing or ambiguous"
    [[ "$ir_color" == "$red"$'\t'"$green"$'\t'"$blue" ]] ||
      fail "transition-gallery $clip canonical color drifted: $ir_color"
    assert_rgb_near_at "$stage" "$video" "$sample" 'iw:ih:0:0' "$red $green $blue" 18
  done <<'STAGES'
0.5 dissolve-a 255 190 11
1.5 dissolve-b 251 86 7
2.5 fade-a 255 0 110
3.5 fade-b 131 56 236
4.5 wipe-a 58 134 255
5.5 wipe-b 0 180 216
6.5 slide-a 6 214 160
7.5 slide-b 138 201 38
8.5 zoom-a 255 202 58
9.5 zoom-b 255 146 76
10.5 circle-a 247 37 133
11.5 circle-b 114 9 183
STAGES
  assert_channel_gap_at 'dissolve midpoint mixes red and orange' "$video" 0.825 \
    'iw:ih:0:0' g 'iw:ih:0:0' b 20
  local fade_before fade_midpoint fade_after
  fade_before=$(frame_yavg "$video" 2.5)
  fade_midpoint=$(frame_yavg "$video" 2.825)
  fade_after=$(frame_yavg "$video" 3.5)
  awk -v before="$fade_before" -v midpoint="$fade_midpoint" -v after="$fade_after" \
    'BEGIN { exit !(midpoint + 20 < before && midpoint + 20 < after) }' \
    || fail 'fade midpoint is not visibly darker than both endpoints'
  assert_channel_gap_at 'leftward wipe midpoint advances from the right side' "$video" 4.825 \
    'iw/3:ih:0:0' r 'iw/3:ih:iw*2/3:0' r 45
  assert_channel_gap_at 'upward slide separates the bottom and top regions' "$video" 6.825 \
    'iw:ih/3:0:ih*2/3' r 'iw:ih/3:0:0' r 45
  assert_channel_gap_at 'zoom midpoint reveals the incoming center first' "$video" 8.825 \
    'iw/5:ih/5:0:0' g 'iw/5:ih/5:2*iw/5:2*ih/5' g 30
  assert_channel_gap_at 'circle midpoint reveals the incoming center' "$video" 10.825 \
    'iw/5:ih/5:0:0' r 'iw/5:ih/5:2*iw/5:2*ih/5' r 70
  assert_rgb_distance_at_least 'pixelize outgoing endpoint remains a horizontal gradient' \
    "$video" 12.5 'iw/20:ih/20:iw*.05:ih*.48' 'iw/20:ih/20:iw*.90:ih*.48' 60
  assert_rgb_distance_at_least 'pixelize incoming endpoint remains a vertical gradient' \
    "$video" 13.5 'iw/20:ih/20:iw*.48:ih*.05' 'iw/20:ih/20:iw*.48:ih*.90' 60
  assert_rgb_distance_at_most 'pixelize midpoint forms a flat coarse block' "$video" 12.825 \
    'iw/50:ih/25:iw*.05:ih*.08' 'iw/50:ih/25:iw*.18:ih*.08' 18
  assert_rgb_distance_at_least 'pixelize midpoint separates adjacent coarse blocks' "$video" 12.825 \
    'iw/50:ih/25:iw*.05:ih*.08' 'iw/50:ih/25:iw*.55:ih*.08' 35
}

check_executable_mechanisms_evidence() {
  local dir=$1 plan video canonical warm cool badge config badge_delta
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview.mp4)
  canonical=$(example_authoring_canonical "$dir")
  assert_resolution_chain "$dir" "$plan" "$video" executable-mechanisms
  warm=$(canonical_clip_id "$canonical" warm)
  cool=$(canonical_clip_id "$canonical" cool)
  badge=$(canonical_clip_id "$canonical" badge)
  config=$(delivery_config_id "$canonical" preview)
  jq -e --arg warm "$warm" --arg cool "$cool" --arg badge "$badge" \
    --arg config "$config" '
    def as_seconds: .value / .timescale;
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    def transition:
      [.sequences[].tracks[].transitions[] | select(.kind.type == "dissolve")] |
      if length == 1 then .[0] else error("dissolve transition is not unique") end;
    .output.render_config_id == $config and
    clip($warm).source.generator.gradient.type == "linear" and
    clip($warm).source.generator.gradient.start == {"x":0,"y":0.5} and
    clip($warm).source.generator.gradient.end == {"x":1,"y":0.5} and
    (clip($warm).source.generator.gradient.stops | map(.color)) ==
      [{"red":239,"green":68,"blue":68,"alpha":255},{"red":250,"green":204,"blue":21,"alpha":255}] and
    (clip($cool).source.generator.gradient.stops | map(.color)) ==
      [{"red":37,"green":99,"blue":235,"alpha":255},{"red":34,"green":211,"blue":238,"alpha":255}] and
    (transition.record_window.start | as_seconds) == 0.8 and
    (transition.record_window.duration | as_seconds) == 0.4 and
    (transition.cut_time | as_seconds) == 1.0 and
    clip($badge).visual.transform.position.type == "keyframes" and
    (clip($badge).visual.transform.position.keyframes | length) == 2 and
    clip($badge).visual.transform.scale.value == {"x":0.65,"y":0.65} and
    clip($badge).visual.masks[0].shape.type == "circle" and
    clip($badge).visual.opacity.value == 0.82 and
    clip($badge).visual.compositing.blend_mode == "screen"
  ' "$plan" >/dev/null || fail 'executable mechanism plan contract failed'

  video_contract "$video" 1.9
  assert_rgb_near_at 'warm gradient is visible before the transition' "$video" 0.25 \
    'iw*.20:ih*.70:iw*.08:ih*.15' '241 95 58' 28
  assert_rgb_near_at 'cool gradient is visible after the transition' "$video" 1.75 \
    'iw*.20:ih*.70:iw*.40:ih*.15' '35 155 237' 30
  badge_delta=$(frame_difference_avg "$video" 0.25 0.75)
  awk -v delta="$badge_delta" 'BEGIN { exit !(delta > 1.0) }' ||
    fail "animated circular badge is not visible over the stable warm base: delta=$badge_delta"
}
