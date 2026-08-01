#!/usr/bin/env bash

# shellcheck source=example-render-evidence/timing-media.sh
source "$SCRIPT_DIR/example-render-evidence/timing-media.sh"
# shellcheck source=example-render-evidence/timing-visual.sh
source "$SCRIPT_DIR/example-render-evidence/timing-visual.sh"
# shellcheck source=example-render-evidence/timing-transform-animation.sh
source "$SCRIPT_DIR/example-render-evidence/timing-transform-animation.sh"

timing_plan() {
  local example_dir=$1 output_id=${2:-preview}
  local plan
  plan=$(example_preview_plan "$example_dir" "out_$output_id") ||
    fail "unsafe timing output ID: $output_id"
  require_file "$plan"
  printf '%s\n' "$plan"
}

timing_video() {
  local example_dir=$1 delivery_id=${2:-preview} artifact_id=${3:-preview}
  local video
  video=$(delivery_video_path "$example_dir" "$delivery_id" "$artifact_id")
  require_file "$video"
  printf '%s\n' "$video"
}

assert_plan_query() {
  local plan=$1 query=$2 message=$3
  jq -e "$query" "$plan" >/dev/null || fail "$message"
}

timing_region_rgb() {
  local file=$1 sample_time=$2 crop=$3
  ffmpeg -nostdin -hide_banner -loglevel error -i "$file" -frames:v 1 \
    -vf "trim=start=$sample_time,setpts=PTS-STARTPTS,crop=$crop,scale=1:1:flags=area,format=rgb24" \
    -f rawvideo - | od -An -tu1 -N3 | awk '{ print $1, $2, $3 }'
}

assert_rgb_near_at() {
  local message=$1 file=$2 sample_time=$3 crop=$4 expected=$5 tolerance=$6
  local actual_r actual_g actual_b expected_r expected_g expected_b
  read -r actual_r actual_g actual_b < <(timing_region_rgb "$file" "$sample_time" "$crop")
  read -r expected_r expected_g expected_b <<<"$expected"
  [[ -n ${actual_b:-} ]] || fail "$message (frame unavailable at ${sample_time}s)"
  awk -v ar="$actual_r" -v ag="$actual_g" -v ab="$actual_b" \
    -v er="$expected_r" -v eg="$expected_g" -v eb="$expected_b" -v t="$tolerance" \
    'function abs(v) { return v < 0 ? -v : v }
     BEGIN { exit !(abs(ar-er) <= t && abs(ag-eg) <= t && abs(ab-eb) <= t) }' \
    || fail "$message (got $actual_r $actual_g $actual_b, expected $expected +/- $tolerance)"
}

rgb_channel() {
  local rgb=$1 channel=$2
  local red green blue
  read -r red green blue <<<"$rgb"
  case "$channel" in
    r) printf '%s\n' "$red" ;;
    g) printf '%s\n' "$green" ;;
    b) printf '%s\n' "$blue" ;;
    *) fail "unsupported RGB channel: $channel" ;;
  esac
}

assert_channel_gap_at() {
  local message=$1 file=$2 sample_time=$3 crop_a=$4 channel_a=$5
  local crop_b=$6 channel_b=$7 gap=$8
  local value_a value_b
  value_a=$(rgb_channel "$(timing_region_rgb "$file" "$sample_time" "$crop_a")" "$channel_a")
  value_b=$(rgb_channel "$(timing_region_rgb "$file" "$sample_time" "$crop_b")" "$channel_b")
  ((value_a >= value_b + gap)) \
    || fail "$message (first=$value_a second=$value_b required_gap=$gap)"
}

timing_rgb_distance_at() {
  local file=$1 sample_time=$2 crop_a=$3 crop_b=$4
  local ar ag ab br bg bb
  read -r ar ag ab < <(timing_region_rgb "$file" "$sample_time" "$crop_a")
  read -r br bg bb < <(timing_region_rgb "$file" "$sample_time" "$crop_b")
  awk -v ar="$ar" -v ag="$ag" -v ab="$ab" -v br="$br" -v bg="$bg" -v bb="$bb" '
    function abs(v) { return v < 0 ? -v : v }
    BEGIN { print abs(ar-br) + abs(ag-bg) + abs(ab-bb) }
  '
}

assert_rgb_distance_at_least() {
  local message=$1 file=$2 sample_time=$3 crop_a=$4 crop_b=$5 minimum=$6
  local distance
  distance=$(timing_rgb_distance_at "$file" "$sample_time" "$crop_a" "$crop_b")
  ((distance >= minimum)) || fail "$message (distance=$distance required=$minimum)"
}

assert_rgb_distance_at_most() {
  local message=$1 file=$2 sample_time=$3 crop_a=$4 crop_b=$5 maximum=$6
  local distance
  distance=$(timing_rgb_distance_at "$file" "$sample_time" "$crop_a" "$crop_b")
  ((distance <= maximum)) || fail "$message (distance=$distance maximum=$maximum)"
}

timing_frame_yuv() {
  local file=$1 sample_time=$2 output=$3
  ffmpeg -nostdin -hide_banner -loglevel error -i "$file" -frames:v 1 \
    -vf "trim=start=$sample_time,setpts=PTS-STARTPTS,scale=160:90:flags=bicubic,format=yuv420p" \
    -f rawvideo -y "$output"
}

timing_frame_psnr() {
  local first=$1 first_time=$2 second=$3 second_time=$4
  local tmp_dir first_frame second_frame score
  tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/veac-timing.XXXXXX")
  first_frame="$tmp_dir/first.yuv"
  second_frame="$tmp_dir/second.yuv"
  timing_frame_yuv "$first" "$first_time" "$first_frame"
  timing_frame_yuv "$second" "$second_time" "$second_frame"
  score=$(ffmpeg -nostdin -hide_banner -loglevel error \
    -f rawvideo -pixel_format yuv420p -video_size 160x90 -i "$first_frame" \
    -f rawvideo -pixel_format yuv420p -video_size 160x90 -i "$second_frame" \
    -filter_complex '[0:v][1:v]psnr=stats_file=-' -frames:v 1 -f null - 2>&1 \
    | awk -F'psnr_avg:' '/psnr_avg:/ { split($2, a, " "); print (a[1] == "inf" ? 100 : a[1]); exit }')
  rm -rf "$tmp_dir"
  printf '%s\n' "$score"
}

assert_psnr_preferred() {
  local message=$1 render=$2 record_time=$3 source=$4 expected_time=$5 wrong_time=$6 margin=$7
  local expected_score wrong_score
  expected_score=$(timing_frame_psnr "$render" "$record_time" "$source" "$expected_time")
  wrong_score=$(timing_frame_psnr "$render" "$record_time" "$source" "$wrong_time")
  [[ -n $expected_score && -n $wrong_score ]] || fail "$message (PSNR could not be measured)"
  awk -v expected="$expected_score" -v wrong="$wrong_score" -v margin="$margin" \
    'BEGIN { exit !(expected >= wrong + margin) }' \
    || fail "$message (expected=$expected_score wrong=$wrong_score margin=$margin)"
}

assert_frame_psnr_at_least() {
  local message=$1 first=$2 first_time=$3 second=$4 second_time=$5 minimum=$6
  local score
  score=$(timing_frame_psnr "$first" "$first_time" "$second" "$second_time")
  [[ -n $score ]] || fail "$message (PSNR could not be measured)"
  awk -v score="$score" -v minimum="$minimum" 'BEGIN { exit !(score >= minimum) }' \
    || fail "$message (PSNR=$score required=$minimum)"
}

timing_selected() {
  local id=$1 selector=${VEAC_EXAMPLES:-}
  [[ $selector == all ]] && return 0
  [[ -n $selector ]] || return 1
  selector=${selector//,/ }
  selector=${selector//$'\t'/ }
  selector=${selector//$'\n'/ }
  [[ " $selector " == *" $id "* ]]
}

assert_selected_timing_directories() {
  local preview_root=$1 target
  for target in timeline-source-time nested-and-multicam speed-demo \
    transition-gallery executable-mechanisms transforms-and-animation; do
    timing_selected "$target" || continue
    [[ -d $preview_root/$target ]] || fail "selected timing example was not built: $target"
  done
}

check_timing_evidence() {
  local preview_root=$1
  assert_selected_timing_directories "$preview_root"
  [[ ! -d $preview_root/timeline-source-time ]] || check_timeline_source_time_evidence "$preview_root/timeline-source-time"
  [[ ! -d $preview_root/nested-and-multicam ]] || check_nested_multicam_evidence "$preview_root/nested-and-multicam"
  [[ ! -d $preview_root/speed-demo ]] || check_speed_demo_evidence "$preview_root/speed-demo"
  [[ ! -d $preview_root/transition-gallery ]] || check_transition_gallery_timing_evidence "$preview_root/transition-gallery"
  [[ ! -d $preview_root/executable-mechanisms ]] || check_executable_mechanisms_evidence "$preview_root/executable-mechanisms"
  [[ ! -d $preview_root/transforms-and-animation ]] || check_transform_animation_evidence "$preview_root/transforms-and-animation"
}
