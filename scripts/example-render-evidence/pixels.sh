#!/usr/bin/env bash

frame_hash() {
  local video=$1
  local time=$2
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 -vf format=rgb24 -f framemd5 - \
    | awk -F, '/^[^#]/ && !found { gsub(/[[:space:]]/, "", $NF); hash=$NF; found=1 }
        END { if (found) print hash; else exit 1 }'
}

assert_unique_frames() {
  local video=$1
  local expected=$2
  shift 2
  local hashes=()
  local time
  for time in "$@"; do
    hashes+=("$(frame_hash "$video" "$time")")
  done
  local unique
  unique=$(printf '%s\n' "${hashes[@]}" | sort -u | wc -l | tr -d ' ')
  [[ $unique == "$expected" ]] || fail "expected $expected distinct frames in $video, got $unique"
}

frame_bright_bbox() {
  local file=$1 sample_time=$2 width=$3 threshold=${4:-680}
  ffmpeg -v error -ss "$sample_time" -i "$file" -frames:v 1 -vf format=rgb24 -f rawvideo - |
    od -An -v -tu1 | awk -v width="$width" -v threshold="$threshold" '
      { for (i=1; i<=NF; i++) { rgb[channel++]=$i
          if (channel==3) { if (rgb[0]+rgb[1]+rgb[2]>=threshold) {
              x=pixel%width; y=int(pixel/width); count++
              if (!seen || x<minx) minx=x; if (!seen || x>maxx) maxx=x
              if (!seen || y<miny) miny=y; if (!seen || y>maxy) maxy=y; seen=1 }
            channel=0; pixel++ } } }
      END { print count+0, seen ? maxx-minx+1 : 0, seen ? maxy-miny+1 : 0 }'
}

assert_frames_visually_equal() {
  local first=$1 second=$2 label=$3 ratio average maximum
  read -r ratio average maximum < <(
    ffmpeg -v error -i "$first" -i "$second" -filter_complex \
      '[0:v]format=rgb24[a];[1:v]format=rgb24[b];[a][b]blend=all_mode=difference,format=rgb24' \
      -frames:v 1 -f rawvideo - |
      od -An -v -tu1 | awk '
        { for (i=1; i<=NF; i++) { count++; total+=$i
            if ($i>0) changed++; if ($i>maximum) maximum=$i } }
        END { if (count) printf "%.8f %.8f %d\n", changed/count, total/count, maximum }'
  )
  [[ -n ${maximum:-} ]] || fail "$label: RGB difference is unavailable"
  awk -v ratio="$ratio" -v average="$average" -v maximum="$maximum" \
    'BEGIN { exit !(ratio <= .001 && average <= .01 && maximum <= 2) }' ||
    fail "$label: RGB difference ratio=$ratio average=$average maximum=$maximum"
}

frame_yavg() {
  local video=$1
  local time=$2
  ffmpeg -v error -ss "$time" -i "$video" \
    -vf "signalstats,metadata=print:file=-" -frames:v 1 -f null - 2>/dev/null \
    | awk -F= '/lavfi.signalstats.YAVG=/ { print $2; exit }'
}

region_yavg() {
  local video=$1
  local time=$2
  local crop=$3
  ffmpeg -v error -ss "$time" -i "$video" \
    -vf "crop=$crop,signalstats,metadata=print:file=-" -frames:v 1 -f null - 2>/dev/null \
    | awk -F= '/lavfi.signalstats.YAVG=/ { print $2; exit }'
}

frame_rgb() {
  local video=$1
  local time=$2
  ffmpeg -v error -ss "$time" -i "$video" -frames:v 1 \
    -vf "scale=1:1:flags=area,format=rgb24" -f rawvideo - \
    | od -An -tu1 -N3 | awk '{ print $1, $2, $3 }'
}

assert_rgb_near() {
  local video=$1
  local time=$2
  local tolerance=$6
  local label=$7
  local expected=("$3" "$4" "$5")
  local sampled
  sampled=$(frame_rgb "$video" "$time")
  local actual=()
  read -r -a actual <<< "$sampled"
  ((${#actual[@]} == 3)) || fail "$label: could not sample RGB at ${time}s"
  local index delta
  for index in 0 1 2; do
    delta=$((actual[index] - expected[index]))
    ((delta < 0)) && delta=$((-delta))
    ((delta <= tolerance)) || fail "$label: RGB $sampled is outside tolerance"
  done
}

frame_yuv_spread() {
  local media=$1
  local time=$2
  ffmpeg -v error -ss "$time" -i "$media" -frames:v 1 \
    -vf "format=yuv444p,signalstats,metadata=print:file=-" -f null - 2>/dev/null \
    | awk -F= '
        /YMIN=/ && !have_ymin { ymin=$2; have_ymin=1 }
        /YMAX=/ && !have_ymax { ymax=$2; have_ymax=1 }
        /UMIN=/ && !have_umin { umin=$2; have_umin=1 }
        /UMAX=/ && !have_umax { umax=$2; have_umax=1 }
        /VMIN=/ && !have_vmin { vmin=$2; have_vmin=1 }
        /VMAX=/ && !have_vmax { vmax=$2; have_vmax=1 }
        END { if (have_ymax && have_umax && have_vmax) print ymax-ymin, umax-umin, vmax-vmin }
      '
}

assert_uniform_frame() {
  local video=$1
  local time=$2
  local maximum_spread=$3
  local label=$4
  local yspread uspread vspread
  read -r yspread uspread vspread <<< "$(frame_yuv_spread "$video" "$time")"
  [[ -n ${vspread:-} ]] || fail "$label: could not measure channel spread"
  ((yspread <= maximum_spread && uspread <= maximum_spread && vspread <= maximum_spread)) \
    || fail "$label: frame is not uniform (spread $yspread/$uspread/$vspread)"
}

assert_non_uniform_image() {
  local image=$1
  local label=$2
  local yspread uspread vspread
  read -r yspread uspread vspread <<< "$(frame_yuv_spread "$image" 0)"
  [[ -n ${vspread:-} ]] || fail "$label: could not measure image pixels"
  ((yspread > 0 || uspread > 0 || vspread > 0)) || fail "$label: image has no visible variation"
}
