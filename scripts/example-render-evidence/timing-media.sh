#!/usr/bin/env bash
# shellcheck disable=SC2016

check_timeline_source_time_evidence() {
  local dir=$1 plan video source
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview)
  source="$dir/project/assets/reel.mp4"
  require_file "$source"

  assert_plan_query "$plan" '
    def t($v; $s): {"timescale":$s,"value":$v};
    def r($sv; $ss; $dv; $ds): {"start":t($sv;$ss),"duration":t($dv;$ds)};
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    clip("itm_linear-trim").record_range == r(0;1000;3000;1000) and
    clip("itm_linear-trim").source_mapping.time_map.source_range_per_repeat == r(5000;1000;3000;1000) and
    clip("itm_speed-ramp").record_range == r(3000;1000;3000;1000) and
    (clip("itm_speed-ramp").source_mapping.time_map.segments | length) == 4 and
    all(clip("itm_speed-ramp").source_mapping.time_map.segments[];
      .record_duration == t(750;1000)) and
    clip("itm_speed-ramp").source_mapping.time_map.segments[0].source_start == t(10000;1000) and
    ([clip("itm_speed-ramp").source_mapping.time_map.segments[] |
      .source_end.value - .source_start.value] == [500,1000,1500,2000]) and
    ([clip("itm_speed-ramp").source_mapping.time_map.segments[] |
      .source_end.value] == [10500,11500,13000,15000]) and
    clip("itm_freeze").source.source_time == t(18000;1000) and
    clip("itm_fast-forward").source_mapping.time_map.rate == {"numerator":2,"denominator":1} and
    clip("itm_fast-forward").source_mapping.time_map.source_range_per_repeat == r(20000;1000;4000;1000) and
    clip("itm_hold-first").source_mapping.out_of_range == "hold-first" and
    clip("itm_hold-last").source_mapping.out_of_range == "hold-last" and
    clip("itm_hold-both").source_mapping.out_of_range == "hold-both"
  ' 'timeline source-time plan contract failed'

  video_contract "$video" 17.9
  assert_psnr_preferred 'linear trim maps record 0.5s to source 5.5s' "$video" 0.5 "$source" 5.5 6.5 1.5
  assert_psnr_preferred 'ramp starts gently at record 3.5s' "$video" 3.5 "$source" 10.333 11.5 1.5
  assert_psnr_preferred 'ramp reaches the middle at record 4.5s' "$video" 4.5 "$source" 11.5 10.5 1.5
  assert_psnr_preferred 'ramp accelerates at record 5.5s' "$video" 5.5 "$source" 13.667 12.5 1.5
  assert_frame_psnr_at_least 'freeze remains visually constant' "$video" 6.25 "$video" 7.75 60
  assert_psnr_preferred 'freeze selects source 18s over its earlier neighbor' \
    "$video" 7.0 "$source" 18 17.5 1.5
  assert_psnr_preferred 'freeze selects source 18s over its later neighbor' \
    "$video" 7.0 "$source" 18 18.5 1.5
  assert_psnr_preferred '2x maps record 8.5s to source 21s' "$video" 8.5 "$source" 21 20.5 1.5
  assert_psnr_preferred '2x maps record 9.5s to source 23s' "$video" 9.5 "$source" 23 22 1.5
  assert_rgb_near_at 'hold:first freezes the first boundary frame' "$video" 10.25 'iw:ih:0:0' '209 73 91' 20
  assert_rgb_near_at 'hold:last freezes the last boundary frame' "$video" 13.75 'iw:ih:0:0' '0 121 140' 20
  assert_rgb_near_at 'hold:both freezes its first boundary' "$video" 14.25 'iw:ih:0:0' '209 73 91' 20
  assert_rgb_near_at 'hold:both freezes its last boundary' "$video" 17.75 'iw:ih:0:0' '0 121 140' 20
}

check_nested_multicam_evidence() {
  local dir=$1 plan video
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview)
  assert_plan_query "$plan" '
    def t($v; $s): {"timescale":$s,"value":$v};
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    clip("itm_nested-intro").source == {"type":"sequence","sequence_id":"seq_intro"} and
    (clip("itm_interview-cut").source | .type == "multicam" and .source as $m |
      $m.group_id == "mcg_interview" and $m.sync.basis == "audio" and
      $m.sync.reference_angle_id == "ang_host-angle" and
      ($m.angles | map(.id) | sort) == ["ang_guest-angle","ang_host-angle"] and
      (first($m.angles[] | select(.id == "ang_guest-angle"))).source_offset == t(120;1000) and
      $m.switches[0] == {"range":{"start":t(0;1000),"duration":t(2000;1000)},"angle_id":"ang_host-angle"} and
      $m.switches[1] == {"range":{"start":t(2000;1000),"duration":t(2000;1000)},"angle_id":"ang_guest-angle"})
  ' 'nested sequence and multicam plan contract failed'

  video_contract "$video" 5.9
  assert_rgb_near_at 'nested sequence renders the intro slate' "$video" 0.5 'iw:ih:0:0' '17 138 178' 20
  assert_rgb_near_at 'multicam first interval uses the host angle' "$video" 2.5 'iw:ih:0:0' '197 69 46' 20
  assert_rgb_near_at 'multicam switch uses the guest angle' "$video" 4.5 'iw:ih:0:0' '20 136 177' 20
}

check_speed_demo_evidence() {
  local dir=$1 plan video source
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview)
  source="$dir/project/assets/source.mp4"
  require_file "$source"
  assert_plan_query "$plan" '
    def t($v; $s): {"timescale":$s,"value":$v};
    def r($sv; $ss; $dv; $ds): {"start":t($sv;$ss),"duration":t($dv;$ds)};
    def clip($id): first(.sequences[].tracks[].clips[] | select(.id == $id));
    clip("itm_fast").record_range == r(0;1000;2000;1000) and
    clip("itm_fast").source_mapping.time_map.source_range_per_repeat == r(0;1000;4000;1000) and
    clip("itm_fast").source_mapping.time_map.rate == {"numerator":2,"denominator":1} and
    clip("itm_slow").record_range == r(2000;1000;4000;1000) and
    clip("itm_slow").source_mapping.time_map.source_range_per_repeat == r(4000;1000;2000;1000) and
    clip("itm_slow").source_mapping.time_map.rate == {"numerator":1,"denominator":2}
  ' 'speed plan rates and ranges are incorrect'

  video_contract "$video" 5.9
  assert_psnr_preferred '2x maps record 0.5s to source 1s' "$video" 0.5 "$source" 1 0.5 1.5
  assert_psnr_preferred '2x maps record 1.5s to source 3s' "$video" 1.5 "$source" 3 1.5 1.5
  assert_psnr_preferred '0.5x maps record 2.5s to source 4.25s' "$video" 2.5 "$source" 4.25 5 1.5
  assert_psnr_preferred '0.5x maps record 4.5s to source 5.25s' "$video" 4.5 "$source" 5.25 6.5 1.5
}
