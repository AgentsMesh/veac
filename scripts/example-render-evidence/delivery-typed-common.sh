#!/usr/bin/env bash

assert_delivery_decodes() {
  local media=$1 label=$2
  ffmpeg -nostdin -hide_banner -v error -xerror -i "$media" \
    -map '0:v?' -map '0:a?' -f null - >/dev/null 2>&1 \
    || fail "$label does not fully decode"
}

assert_delivery_regular_file() {
  local file=$1 label=$2
  require_file "$file"
  [[ -f $file && ! -L $file ]] || fail "$label is not a regular non-symlink file"
}

assert_numeric_between() {
  local value=$1 minimum=$2 maximum=$3 label=$4
  [[ $value =~ ^[0-9]+([.][0-9]+)?$ ]] || fail "$label is not numeric: ${value:-<empty>}"
  awk -v value="$value" -v minimum="$minimum" -v maximum="$maximum" \
    'BEGIN { exit !(value >= minimum && value <= maximum) }' \
    || fail "$label is outside [$minimum, $maximum]: $value"
}

delivery_metadata() {
  local plan
  plan=$(example_preview_plan "$1" out_master)
  require_file "$plan"
  jq -er '
    .output as $output |
    first(.sequences[] | select(.id == $output.sequence_id)) as $sequence |
    first($output.deliverables[] | select(.id == "dlv_frames")) as $frames |
    first($output.deliverables[] | select(.id == "dlv_cover")) as $cover |
    ($output.raster.frame_rate.numerator) as $numerator |
    ($output.raster.frame_rate.denominator) as $denominator |
    ($sequence.duration.value * $numerator) as $frame_ticks |
    ($sequence.duration.timescale * $denominator) as $frame_scale |
    if ($output.raster.width|type) != "number" or $output.raster.width <= 0 or
       ($output.raster.height|type) != "number" or $output.raster.height <= 0 or
       ($numerator|type) != "number" or $numerator <= 0 or
       ($denominator|type) != "number" or $denominator <= 0 or
       ($frame_ticks % $frame_scale) != 0 or
       $frames.kind.settings.start_number < 0 or
       $cover.kind.settings.frame.mode != "containing"
    then error("invalid delivery preview metadata") else
      ($sequence.duration.value / $sequence.duration.timescale) as $duration |
      ($cover.kind.settings.frame.at) as $at |
      (($at.value * $numerator / ($at.timescale * $denominator)) | floor) as $offset |
      [$output.raster.width, $output.raster.height,
       (($numerator|tostring) + "/" + ($denominator|tostring)), $duration,
       ($frame_ticks / $frame_scale), $frames.kind.settings.start_number,
       ($at.value / $at.timescale), ($frames.kind.settings.start_number + $offset)] | @tsv
    end
  ' "$plan"
}

delivery_stage_times() {
  local duration=$1
  awk -v duration="$duration" 'BEGIN { print duration/6, duration*5/6 }'
}

delivery_boundary_times() {
  local count=$1 rate=$2
  awk -v count="$count" -v rate="$rate" '
    BEGIN { split(rate, parts, "/"); before=int((count-1)/2); after=before+1;
      printf "%.17g %.17g\n", before*parts[2]/parts[1], after*parts[2]/parts[1] }
  '
}

delivery_boundary_frame_numbers() {
  local start=$1 count=$2
  ((count >= 4)) || fail "delivery boundary requires at least four frames"
  local before=$(((count - 1) / 2))
  printf '%d %d\n' "$((start + before))" "$((start + before + 1))"
}

delivery_frame_path() {
  local rendered=$1 number=$2 pattern entry plan
  entry=$(dirname "$rendered")
  plan=$(example_preview_plan "$entry" out_master)
  pattern=$(jq -er '
    first(.output.deliverables[] | select(.id == "dlv_frames")).target.pattern |
    if test("^frame-%0[1-9][0-9]*d[.]png$") then . else error("unsafe frame pattern") end
  ' "$plan")
  [[ $pattern == 'frame-%04d.png' ]] || fail "unsupported delivery frame pattern"
  printf '%s/frame-%04d.png\n' "$rendered" "$number"
}

frame_difference_mean() {
  local first=$1 first_time=$2 second=$3 second_time=$4
  ffmpeg -nostdin -v error -ss "$first_time" -i "$first" \
    -ss "$second_time" -i "$second" -filter_complex \
    '[0:v]scale=96:54:flags=area,format=rgb24[a];[1:v]scale=96:54:flags=area,format=rgb24[b];[a][b]blend=all_mode=difference,format=rgb24' \
    -frames:v 1 -f rawvideo - | od -An -v -tu1 | awk '
      { for (i=1; i<=NF; i++) { total+=$i; count++ } }
      END { if (count) print total/count; else exit 1 }'
}

assert_delivery_frames_match() {
  local first=$1 first_time=$2 second=$3 second_time=$4 label=$5 mean
  mean=$(frame_difference_mean "$first" "$first_time" "$second" "$second_time") ||
    fail "$label: RGB difference is unavailable"
  awk -v mean="$mean" 'BEGIN { exit !(mean <= 8) }' ||
    fail "$label: mean RGB difference is $mean"
}

delivery_corner_rgb() {
  ffmpeg -nostdin -v error -ss "$2" -i "$1" -frames:v 1 \
    -vf 'crop=iw/5:ih/5:0:0,scale=1:1:flags=area,format=rgb24' -f rawvideo - |
    od -An -tu1 -N3 | awk '{ print $1, $2, $3 }'
}

assert_delivery_color() {
  local media=$1 time=$2 expected_r=$3 expected_g=$4 expected_b=$5 tolerance=$6 label=$7
  local actual_r actual_g actual_b delta
  read -r actual_r actual_g actual_b < <(delivery_corner_rgb "$media" "$time")
  [[ $actual_b =~ ^[0-9]+$ ]] || fail "$label: RGB sample is unavailable"
  local expected=("$expected_r" "$expected_g" "$expected_b")
  local actual=("$actual_r" "$actual_g" "$actual_b") index
  for index in 0 1 2; do
    delta=$((actual[index] - expected[index])); ((delta < 0)) && delta=$((-delta))
    ((delta <= tolerance)) ||
      fail "$label: RGB $actual_r $actual_g $actual_b is outside tolerance"
  done
}
