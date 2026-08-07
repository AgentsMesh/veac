#!/usr/bin/env bash
# shellcheck disable=SC2016

check_timeline_source_time_evidence() {
  local dir=$1 plan video source canonical linear ramp freeze fast_forward
  local hold_first hold_last hold_both
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview.mp4)
  source="$dir/project/assets/reel.mp4"
  canonical=$(example_authoring_canonical "$dir")
  require_file "$source"
  require_file "$canonical"
  assert_resolution_chain "$dir" "$plan" "$video" timeline-source-time
  linear=$(canonical_clip_id "$canonical" linear-trim)
  ramp=$(canonical_clip_id "$canonical" speed-ramp)
  freeze=$(canonical_clip_id "$canonical" freeze)
  fast_forward=$(canonical_clip_id "$canonical" fast-forward)
  hold_first=$(canonical_clip_id "$canonical" hold-first)
  hold_last=$(canonical_clip_id "$canonical" hold-last)
  hold_both=$(canonical_clip_id "$canonical" hold-both)

  jq -e --arg linear "$linear" --arg ramp "$ramp" --arg freeze "$freeze" \
    --arg fast "$fast_forward" --arg first "$hold_first" --arg last "$hold_last" \
    --arg both "$hold_both" '
    def as_seconds: .value / .timescale;
    def near($value; $target):
      (($value | as_seconds) - $target | if . < 0 then -. else . end) < 0.000001;
    def range_at($value; $start; $duration):
      near($value.start; $start) and near($value.duration; $duration);
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    range_at(clip($linear).record_range; 0; 1.5) and
    range_at(clip($linear).source_mapping.time_map.source_range_per_repeat; 0.5; 1.5) and
    range_at(clip($ramp).record_range; 1.5; 1.5) and
    (clip($ramp).source_mapping.time_map.segments | length) == 4 and
    all(clip($ramp).source_mapping.time_map.segments[]; near(.record_duration; 0.375)) and
    near(clip($ramp).source_mapping.time_map.segments[0].source_start; 2) and
    ([clip($ramp).source_mapping.time_map.segments[] |
      (.source_end | as_seconds) - (.source_start | as_seconds)] == [0.25,0.5,0.75,1]) and
    ([clip($ramp).source_mapping.time_map.segments[] | .source_end | as_seconds] == [2.25,2.75,3.5,4.5]) and
    near(clip($freeze).source.source_time; 5) and
    clip($fast).source_mapping.time_map.rate == {"numerator":2,"denominator":1} and
    range_at(clip($fast).source_mapping.time_map.source_range_per_repeat; 5.5; 2) and
    clip($first).source_mapping.out_of_range == "hold-first" and
    clip($last).source_mapping.out_of_range == "hold-last" and
    clip($both).source_mapping.out_of_range == "hold-both" and
    clip($both).source_mapping.time_map.rate == {"numerator":2,"denominator":1}
  ' "$plan" >/dev/null || fail 'timeline source-time plan contract failed'

  video_contract "$video" 8.9
  assert_psnr_preferred 'linear trim maps record 0.25s to source 0.75s' "$video" 0.25 "$source" 0.75 1.75 1.5
  assert_psnr_preferred 'ramp starts gently at record 1.75s' "$video" 1.75 "$source" 2.167 3 1.5
  assert_psnr_preferred 'ramp reaches the middle at record 2.25s' "$video" 2.25 "$source" 2.75 2.25 1.5
  assert_psnr_preferred 'ramp accelerates at record 2.75s' "$video" 2.75 "$source" 3.833 3 1.5
  assert_frame_psnr_at_least 'freeze remains visually constant' "$video" 3.125 "$video" 3.875 60
  assert_psnr_preferred 'freeze selects source 5s over its earlier neighbor' \
    "$video" 3.5 "$source" 5 4.5 1.5
  assert_psnr_preferred 'freeze selects source 5s over its later neighbor' \
    "$video" 3.5 "$source" 5 5.5 1.5
  assert_psnr_preferred '2x maps record 4.25s to source 6s' "$video" 4.25 "$source" 6 7 1.5
  assert_psnr_preferred '2x maps record 4.75s to source 7s' "$video" 4.75 "$source" 7 6 1.5
  assert_rgb_near_at 'hold:first freezes the first boundary frame' "$video" 5.125 'iw:ih:0:0' '209 73 91' 20
  assert_rgb_near_at 'hold:last freezes the last boundary frame' "$video" 6.875 'iw:ih:0:0' '0 121 140' 20
  assert_rgb_near_at 'hold:both freezes its first boundary' "$video" 7.125 'iw:ih:0:0' '209 73 91' 20
  assert_rgb_near_at 'hold:both freezes its last boundary' "$video" 8.75 'iw:ih:0:0' '0 121 140' 20
}

check_nested_multicam_evidence() {
  local dir=$1 plan video canonical nested interview intro group host guest
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview.mp4)
  canonical=$(example_authoring_canonical "$dir")
  nested=$(canonical_clip_id "$canonical" nested-intro)
  interview=$(canonical_clip_id "$canonical" interview-cut)
  intro=$(canonical_sequence_id "$canonical" intro)
  group=$(jq -er --arg clip "$interview" '
    first(.project.sequences[].tracks[].clips[] | select(.id==$clip)).source.group_id
  ' "$canonical")
  host=$(jq -er --arg group "$group" '
    first(.project.multicam_groups[] | select(.id==$group)).sync.reference_angle_id
  ' "$canonical")
  guest=$(jq -er --arg group "$group" --arg host "$host" '
    first(.project.multicam_groups[] | select(.id==$group)).angles |
    first(.[] | select(.id != $host)).id
  ' "$canonical")
  jq -e --arg nested "$nested" --arg interview "$interview" --arg intro "$intro" \
    --arg group "$group" --arg host "$host" --arg guest "$guest" '
    def as_seconds: .value / .timescale;
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    clip($nested).source == {"type":"sequence","sequence_id":$intro} and
    (clip($interview).source | .type == "multicam" and .source as $m |
      $m.group_id == $group and $m.sync.basis == "audio" and
      $m.sync.reference_angle_id == $host and
      ($m.angles | map(.id) | sort) == ([$guest,$host] | sort) and
      ((first($m.angles[] | select(.id == $guest))).source_offset | as_seconds) == 0.12 and
      ($m.switches[0].range.start | as_seconds) == 0 and
      ($m.switches[0].range.duration | as_seconds) == 2 and $m.switches[0].angle_id == $host and
      ($m.switches[1].range.start | as_seconds) == 2 and
      ($m.switches[1].range.duration | as_seconds) == 2 and $m.switches[1].angle_id == $guest)
  ' "$plan" >/dev/null || fail 'nested sequence and multicam plan contract failed'

  video_contract "$video" 5.9
  assert_rgb_near_at 'nested sequence renders the intro slate' "$video" 0.5 'iw:ih:0:0' '17 138 178' 20
  assert_rgb_near_at 'multicam first interval uses the host angle' "$video" 2.5 'iw:ih:0:0' '197 69 46' 20
  assert_rgb_near_at 'multicam switch uses the guest angle' "$video" 4.5 'iw:ih:0:0' '20 136 177' 20
}

check_speed_demo_evidence() {
  local dir=$1 plan video source canonical fast slow
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview.mp4)
  source="$dir/project/assets/source.mp4"
  canonical=$(example_authoring_canonical "$dir")
  require_file "$source"
  fast=$(canonical_clip_id "$canonical" fast)
  slow=$(canonical_clip_id "$canonical" slow)
  jq -e --arg fast "$fast" --arg slow "$slow" '
    def as_seconds: .value / .timescale;
    def range_at($value; $start; $duration):
      ($value.start | as_seconds) == $start and ($value.duration | as_seconds) == $duration;
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    range_at(clip($fast).record_range; 0; 2) and
    range_at(clip($fast).source_mapping.time_map.source_range_per_repeat; 0; 4) and
    clip($fast).source_mapping.time_map.rate == {"numerator":2,"denominator":1} and
    range_at(clip($slow).record_range; 2; 4) and
    range_at(clip($slow).source_mapping.time_map.source_range_per_repeat; 4; 2) and
    clip($slow).source_mapping.time_map.rate == {"numerator":1,"denominator":2}
  ' "$plan" >/dev/null || fail 'speed plan rates and ranges are incorrect'

  video_contract "$video" 5.9
  assert_psnr_preferred '2x maps record 0.5s to source 1s' "$video" 0.5 "$source" 1 0.5 1.5
  assert_psnr_preferred '2x maps record 1.5s to source 3s' "$video" 1.5 "$source" 3 1.5 1.5
  assert_psnr_preferred '0.5x maps record 2.5s to source 4.25s' "$video" 2.5 "$source" 4.25 5 1.5
  assert_psnr_preferred '0.5x maps record 4.5s to source 5.25s' "$video" 4.5 "$source" 5.25 6.5 1.5
}
