#!/usr/bin/env bash

waveform_structure_metrics() {
  ffmpeg -nostdin -v error -i "$1" -vf 'scale=160:90:flags=area,format=rgb24' \
    -frames:v 1 -f rawvideo - | od -An -v -tu1 | awk -v width=160 -v height=90 '
      { for (i=1; i<=NF; i++) {
          rgb[channel++]=$i
          if (channel == 3) {
            sum=rgb[0]+rgb[1]+rgb[2]
            if (sum <= 30) dark++
            if (sum >= 90) { active++; columns[pixel%width]=1; rows[int(pixel/width)]=1 }
            pixels++; pixel++; channel=0
          }
        }
      }
      END {
        for (column in columns) column_count++
        for (row in rows) row_count++
        if (pixels != width*height) exit 1
        print dark/pixels, active/pixels, column_count/width, row_count/height
      }'
}

assert_waveform_structure() {
  local image=$1 label=$2 dark active columns rows
  read -r dark active columns rows < <(waveform_structure_metrics "$image") ||
    fail "$label structure metrics are unavailable"
  awk -v dark="$dark" -v active="$active" -v columns="$columns" -v rows="$rows" '
    BEGIN { exit !(dark >= 0.85 && active >= 0.005 && active <= 0.20 &&
                    columns >= 0.75 && rows >= 0.02 && rows <= 0.50) }
  ' || fail "$label is not a sparse column waveform: dark=$dark active=$active columns=$columns rows=$rows"
}

waveform_plan_spec() {
  jq -er '
    .output as $output |
    first($output.deliverables[] | select(.id == "dlv_video-waveform")) as $scope |
    $output.raster.frame_rate as $rate | $scope.kind.settings.at as $at |
    (($at.value * $rate.numerator / ($at.timescale * $rate.denominator)) | floor) as $frame |
    if $scope.kind.settings.width <= 0 or $scope.kind.settings.height <= 0 or
       $rate.numerator <= 0 or $rate.denominator <= 0 then error("invalid waveform plan")
    else [$scope.kind.settings.width,$scope.kind.settings.height,
      ($frame * $rate.denominator / $rate.numerator)] | @tsv end
  ' "$1"
}

waveform_provenance_metrics() {
  local image=$1 master=$2 time=$3 width=$4 height=$5
  ffmpeg -nostdin -v error -i "$image" -ss "$time" -i "$master" -filter_complex \
    "[0:v]scale=320:180:flags=area,format=rgb24[a];[1:v]format=yuv444p,waveform=mode=column:components=7:display=overlay,scale=${width}:${height},scale=320:180:flags=area,format=rgb24[b];[a][b]blend=all_mode=difference,format=rgb24" \
    -frames:v 1 -f rawvideo - | od -An -v -tu1 | awk '
      { for (i=1; i<=NF; i++) { total+=$i; if ($i>8) changed++; count++ } }
      END { if (count) print total/count, changed/count; else exit 1 }'
}

assert_waveform_provenance() {
  local image=$1 master=$2 time=$3 width=$4 height=$5 label=$6 mean changed
  read -r mean changed < <(waveform_provenance_metrics \
    "$image" "$master" "$time" "$width" "$height") ||
    fail "$label provenance metrics are unavailable"
  awk -v mean="$mean" -v changed="$changed" \
    'BEGIN { exit !(mean <= 2.5 && changed <= 0.025) }' ||
    fail "$label does not match master at ${time}s: mean=$mean changed=$changed"
}

check_delivery_waveform() {
  local dir=$1 canonical="$1/project/project.veac.json" plan
  local waveform="$1/rendered/video-waveform.png"
  plan=$(example_preview_plan "$dir" out_master)
  jq -e '
    first(.project.render_configs[] | select(.id == "out_master")) as $config |
    first($config.deliverables[] | select(.id == "dlv_video-waveform")) as $scope |
    $scope.target == {"type":"file","name":"video-waveform.png"} and
    $scope.kind.type == "scope" and $scope.kind.settings.scope == "waveform" and
    $scope.kind.settings.at == {"timescale":1000,"value":1000} and
    $scope.kind.settings.format == "png" and $scope.kind.settings.width > 0 and
    $scope.kind.settings.height > 0
  ' "$canonical" >/dev/null || fail "delivery video waveform canonical contract failed"
  local width height time
  read -r width height time < <(waveform_plan_spec "$plan")
  assert_delivery_regular_file "$waveform" "delivery video waveform"
  assert_stream_count "$waveform" v 1 "delivery video waveform"
  assert_stream_field "$waveform" v:0 codec_name png "delivery video waveform"
  assert_stream_field "$waveform" v:0 width "$width" "delivery video waveform"
  assert_stream_field "$waveform" v:0 height "$height" "delivery video waveform"
  assert_waveform_structure "$waveform" "delivery video waveform"
  assert_waveform_provenance "$waveform" "$dir/rendered/master.mp4" "$time" \
    "$width" "$height" "delivery video waveform"
}
