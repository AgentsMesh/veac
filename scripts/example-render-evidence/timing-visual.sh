#!/usr/bin/env bash
# shellcheck disable=SC2016

check_transition_gallery_timing_evidence() {
  local dir=$1 plan video canonical
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview)
  canonical="$dir/project/project.veac.json"
  assert_plan_query "$plan" '
    def t($v; $s): {"timescale":$s,"value":$v};
    def r($sv; $ss; $dv; $ds): {"start":t($sv;$ss),"duration":t($dv;$ds)};
    def transition($id): first(.sequences[].tracks[].transitions[] | select(.relation_id == $id));
    transition("rel_dissolve-cut").kind.type == "dissolve" and
    transition("rel_fade-cut").kind.type == "fade" and
    transition("rel_wipe-cut").kind.type == "wipe" and
    transition("rel_slide-cut").kind.type == "slide" and
    transition("rel_zoom-cut").kind.type == "zoom" and
    transition("rel_circle-cut").kind.type == "circle" and
    transition("rel_pixelize-cut").kind.amount == 0.65 and
    transition("rel_dissolve-cut").cut_time == t(1000;1000) and
    transition("rel_dissolve-cut").record_window == r(825;1000;350;1000) and
    transition("rel_pixelize-cut").cut_time == t(13000;1000)
  ' 'transition gallery plan contract failed'

  video_contract "$video" 13.9
  local stage
  while read -r sample clip red green blue; do
    stage="transition stage at ${sample}s"
    local ir_color
    ir_color=$(jq -er --arg clip "itm_$clip" '
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
  assert_channel_gap_at 'dissolve midpoint mixes red and orange' "$video" 1.0 \
    'iw:ih:0:0' g 'iw:ih:0:0' b 20
  local fade_before fade_midpoint fade_after
  fade_before=$(frame_yavg "$video" 2.5)
  fade_midpoint=$(frame_yavg "$video" 3.0)
  fade_after=$(frame_yavg "$video" 3.5)
  awk -v before="$fade_before" -v midpoint="$fade_midpoint" -v after="$fade_after" \
    'BEGIN { exit !(midpoint + 20 < before && midpoint + 20 < after) }' \
    || fail 'fade midpoint is not visibly darker than both endpoints'
  assert_channel_gap_at 'leftward wipe midpoint advances from the right side' "$video" 5.0 \
    'iw/3:ih:0:0' r 'iw/3:ih:iw*2/3:0' r 45
  assert_channel_gap_at 'upward slide separates the bottom and top regions' "$video" 7.0 \
    'iw:ih/3:0:ih*2/3' r 'iw:ih/3:0:0' r 45
  assert_channel_gap_at 'zoom midpoint reveals the incoming center first' "$video" 9.0 \
    'iw/5:ih/5:0:0' g 'iw/5:ih/5:2*iw/5:2*ih/5' g 30
  assert_channel_gap_at 'circle midpoint reveals the incoming center' "$video" 11.0 \
    'iw/5:ih/5:0:0' r 'iw/5:ih/5:2*iw/5:2*ih/5' r 70
  assert_rgb_distance_at_least 'pixelize outgoing endpoint remains a horizontal gradient' \
    "$video" 12.5 'iw/20:ih/20:iw*.05:ih*.48' 'iw/20:ih/20:iw*.90:ih*.48' 60
  assert_rgb_distance_at_least 'pixelize incoming endpoint remains a vertical gradient' \
    "$video" 13.5 'iw/20:ih/20:iw*.48:ih*.05' 'iw/20:ih/20:iw*.48:ih*.90' 60
  assert_rgb_distance_at_most 'pixelize midpoint forms a flat coarse block' "$video" 13.0 \
    'iw/50:ih/25:iw*.05:ih*.08' 'iw/50:ih/25:iw*.18:ih*.08' 18
  assert_rgb_distance_at_least 'pixelize midpoint separates adjacent coarse blocks' "$video" 13.0 \
    'iw/50:ih/25:iw*.05:ih*.08' 'iw/50:ih/25:iw*.55:ih*.08' 35
}

check_executable_mechanisms_evidence() {
  local dir=$1 plan video
  plan=$(timing_plan "$dir" main)
  video=$(timing_video "$dir" main main)
  assert_plan_query "$plan" '
    def t($v; $s): {"timescale":$s,"value":$v};
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    def transition($id): first(.sequences[].tracks[].transitions[] | select(.relation_id == $id));
    .output.id == "pout_main" and .output.render_config_id == "out_main" and
    clip("itm_warm").source.generator.gradient.type == "linear" and
    clip("itm_warm").source.generator.gradient.start == {"x":0,"y":0.5} and
    clip("itm_warm").source.generator.gradient.end == {"x":1,"y":0.5} and
    (clip("itm_warm").source.generator.gradient.stops | map(.color)) ==
      [{"red":255,"green":0,"blue":0,"alpha":255},{"red":255,"green":255,"blue":0,"alpha":255}] and
    (clip("itm_cool").source.generator.gradient.stops | map(.color)) ==
      [{"red":0,"green":0,"blue":255,"alpha":255},{"red":0,"green":255,"blue":255,"alpha":255}] and
    transition("rel_dissolve").kind.type == "dissolve" and
    transition("rel_dissolve").cut_time == t(1000;1000) and
    clip("itm_badge").visual.transform.position.type == "keyframes" and
    (clip("itm_badge").visual.transform.position.keyframes | length) == 2 and
    clip("itm_badge").visual.transform.scale.value == {"x":0.35,"y":0.35} and
    clip("itm_badge").visual.masks[0].shape.type == "circle" and
    clip("itm_badge").visual.opacity.value == 0.85 and
    clip("itm_badge").visual.compositing.blend_mode == "screen"
  ' 'executable mechanism plan contract failed'

  video_contract "$video" 1.9
  assert_rgb_near_at 'warm gradient is visible before the transition' "$video" 0.25 \
    'iw*.20:ih*.70:iw*.08:ih*.15' '255 46 0' 28
  assert_rgb_near_at 'cool gradient is visible after the transition' "$video" 1.75 \
    'iw*.20:ih*.70:iw*.40:ih*.15' '0 128 255' 30
  assert_channel_gap_at 'animated circular badge is visible over the base' "$video" 0.75 \
    'iw*.18:ih*.32:iw*.41:ih*.34' g 'iw*.18:ih*.32:iw*.05:ih*.34' g 25
}
