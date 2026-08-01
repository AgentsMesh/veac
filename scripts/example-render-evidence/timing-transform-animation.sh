#!/usr/bin/env bash
# shellcheck disable=SC2016

timing_red_sample() {
  local video=$1 time=$2
  ffmpeg -nostdin -hide_banner -loglevel error -i "$video" -frames:v 1 \
    -vf "trim=start=$time,setpts=PTS-STARTPTS,scale=240:135:flags=area,format=rgb24" \
    -f rawvideo - 2>/dev/null | od -An -v -tu1 | awk -v width=240 '
      {
        for (i = 1; i <= NF; i++) {
          channel = bytes % 3
          if (channel == 0) r = $i
          else if (channel == 1) g = $i
          else {
            pixel = int(bytes / 3); x = pixel % width; y = int(pixel / width)
            if (r > 135 && r > g + 45 && r > $i + 35) {
              sx += x; sy += y; count++
              if (!found || x < minx) minx = x
              if (!found || x > maxx) maxx = x
              if (!found || y < miny) miny = y
              if (!found || y > maxy) maxy = y
              found = 1
            }
          }
          bytes++
        }
      }
      END {
        if (!count) exit 1
        printf "%.2f %.2f %d %d %d %d %d\n", sx/count, sy/count,
          minx, maxx, miny, maxy, count
      }'
}

transform_sample_rows() {
  local video=$1 time sample
  shift
  for time in "$@"; do
    sample=$(timing_red_sample "$video" "$time") || {
      printf 'missing red transform sample at %ss\n' "$time" >&2
      return 1
    }
    printf '%s %s\n' "$time" "$sample"
  done
}

assert_segmented_transform_motion() {
  local video=$1 samples
  samples=$(transform_sample_rows "$video" 0 0.04 0.13 0.25 0.37 0.49 0.61 0.79) ||
    fail 'transform badge is not visible throughout the segmented motion'
  awk '
    function abs(value) { return value < 0 ? -value : value }
    NR == 1 { hold_x=$2; hold_y=$3; previous_x=$2; next }
    NR == 2 {
      if (abs($2-hold_x) > 1.5 || abs($3-hold_y) > 1.5) bad=1
      previous_x=$2; next
    }
    {
      if (!($2 < previous_x - 1.5)) bad=1
      x[NR]=$2; y[NR]=$3; previous_x=$2
    }
    END {
      if (!(NR == 8 && y[5] > y[3] + 1 && y[6] < y[5] - .5 &&
        y[7] < y[6] - .5 && y[8] < y[7] - 2)) bad=1
      exit bad
    }
  ' <<<"$samples" || fail "transform hold/segment trajectory is wrong: $samples"
}

assert_spring_transform_motion() {
  local video=$1 samples
  samples=$(transform_sample_rows "$video" 0.79 1.0 1.4 1.8 2.4) ||
    fail 'transform badge is not visible throughout the Spring motion'
  awk '
    NR == 1 { x1=$2 }
    NR == 2 { x2=$2 }
    NR == 3 { x3=$2 }
    NR == 4 { x4=$2 }
    NR == 5 { x5=$2 }
    END { exit !(NR == 5 && x2 > x1 + 2 && x3 > x2 + 8 &&
      x4 > x3 + 3 && x5 < x4 - .75) }
  ' <<<"$samples" || fail "transform Spring trajectory does not advance and settle: $samples"
}

check_transform_animation_evidence() {
  local dir=$1 plan video
  plan=$(timing_plan "$dir")
  video=$(timing_video "$dir" preview preview)
  assert_plan_query "$plan" '
    def t($v): {"timescale":1000,"value":$v};
    def p($x;$y): {"x":{"unit":"pixels","value":$x},"y":{"unit":"pixels","value":$y}};
    def badge: first(.sequences[].tracks[].clips[] | select(.id == "itm_badge"));
    def key($i;$id;$time;$x;$y;$kind):
      badge.visual.transform.position.keyframes[$i] as $key |
      $key.id == $id and $key.time == t($time) and $key.value == p($x;$y) and
      $key.interpolation.type == $kind;
    badge.visual.transform.position.type == "keyframes" and
    (badge.visual.transform.position.keyframes | length) == 8 and
    key(0;"kf_hold";0;240;0;"hold") and
    key(1;"kf_linear";120;200;-20;"linear") and
    key(2;"kf_ease-in";240;160;0;"ease_in") and
    key(3;"kf_ease-out";360;120;20;"ease_out") and
    key(4;"kf_ease-in-out";480;80;0;"ease_in_out") and
    key(5;"kf_bezier";600;40;-20;"cubic_bezier") and
    key(6;"kf_settle";800;-120;-80;"spring") and
    key(7;"kf_rest";4000;0;0;"linear") and
    badge.visual.transform.position.keyframes[5].interpolation ==
      {"type":"cubic_bezier","x1":0.42,"x2":0.58,"y1":0,"y2":1} and
    badge.visual.transform.position.keyframes[6].interpolation ==
      {"type":"spring","frequency":1.5,"decay":6,"initial_velocity":0} and
    badge.visual.transform.crop.value == {"x":0.14,"y":0.04,"width":0.84,"height":0.92} and
    badge.visual.transform.scale.value == {"x":1.12,"y":0.92} and
    badge.visual.transform.shear == {"x":0.22,"y":-0.08} and
    badge.visual.transform.rotation_degrees.value == 5 and
    badge.visual.transform.anchor == {"x":1,"y":1} and
    badge.visual.placement.anchor == "bottom_right" and
    badge.visual.transform.flip_horizontal and (badge.visual.transform.flip_vertical | not)
  ' 'transform keyframe plan contract failed'
  video_contract "$video" 3.9
  assert_segmented_transform_motion "$video"
  assert_spring_transform_motion "$video"
}
