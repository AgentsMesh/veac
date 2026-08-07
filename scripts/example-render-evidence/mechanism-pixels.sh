#!/usr/bin/env bash

mechanism_crop() {
  printf '%s\n' 'iw:2*ih/3:0:0'
}

mechanism_region_hash() {
  ffmpeg -v error -ss "$2" -i "$1" -frames:v 1 \
    -vf "crop=$(mechanism_crop),format=rgb24" -f framemd5 - |
    awk -F, '/^[^#]/ && !found { gsub(/[[:space:]]/, "", $NF); print $NF; found=1 }'
}

visual_crop_hash() {
  ffmpeg -v error -ss "$2" -i "$1" -frames:v 1 \
    -vf "crop=$3,format=rgb24" -f framemd5 - |
    awk -F, '/^[^#]/ && !found { gsub(/[[:space:]]/, "", $NF); print $NF; found=1 }'
}

assert_visual_crop_variety() {
  local video=$1 minimum=$2 crop=$3 label=$4 time unique hashes=()
  shift 4
  for time in "$@"; do hashes+=("$(visual_crop_hash "$video" "$time" "$crop")"); done
  unique=$(printf '%s\n' "${hashes[@]}" | sort -u | wc -l | tr -d ' ')
  ((unique >= minimum)) || fail "$label: expected at least $minimum distinct crops, got $unique"
}

assert_unique_mechanism_regions() {
  local video=$1 expected=$2 time unique hashes=()
  shift 2
  for time in "$@"; do
    hashes+=("$(mechanism_region_hash "$video" "$time")")
  done
  unique=$(printf '%s\n' "${hashes[@]}" | sort -u | wc -l | tr -d ' ')
  [[ $unique == "$expected" ]] ||
    fail "expected $expected distinct label-free mechanism regions in $video, got $unique"
}

mechanism_region_spread() {
  ffmpeg -v error -ss "$2" -i "$1" -frames:v 1 \
    -vf "crop=$(mechanism_crop),format=yuv444p,signalstats,metadata=print:file=-" \
    -f null - 2>/dev/null | awk -F= '
      /YMIN=/ { ymin=$2 } /YMAX=/ { ymax=$2 }
      /UMIN=/ { umin=$2 } /UMAX=/ { umax=$2 }
      /VMIN=/ { vmin=$2 } /VMAX=/ { vmax=$2 }
      END { if (ymax != "") print ymax-ymin, umax-umin, vmax-vmin }'
}

assert_mechanism_spread_above() {
  local y u v total
  read -r y u v <<<"$(mechanism_region_spread "$1" "$2")"
  [[ -n ${v:-} ]] || fail "$4: could not measure label-free channel spread"
  total=$(awk -v y="$y" -v u="$u" -v v="$v" 'BEGIN { print y+u+v }')
  assert_gt "$total" 0 "$3" "$4 (label-free spread $y/$u/$v)"
}

mechanism_yavg() {
  region_yavg "$1" "$2" "$(mechanism_crop)"
}
