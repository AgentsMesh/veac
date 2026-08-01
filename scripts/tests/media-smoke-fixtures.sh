#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout-fixtures.sh"

write_smoke_plan() {
  cat >"$1" <<JSON
{"output":{"render_config_id":"$2","sequence_id":"seq_main"},"sequences":[{"id":"seq_main","duration":{"timescale":1000,"value":$3}}]}
JSON
}

complete_video_settings() {
  jq '(.project.render_configs[].deliverables[]|select(.kind.type=="video")|.kind.settings)|={container:"mp4",video:{codec:"h264",pixel_format:"yuv420p",alpha:"opaque"},audio:.audio}' "$1" >"$1.tmp"
  mv "$1.tmp" "$1"
}

write_smoke_project() {
  local dir=$1 audio=$2 width=${3:-96} rate=${4:-12} duration=${5:-1000}
  prepare_preview_fixture_dirs "$dir"
  cat > "$dir/project/project.veac.json" <<JSON
{"project":{"render_configs":[{"id":"out_preview","raster":{"width":$width,
"height":54,"frame_rate":{"numerator":$rate,"denominator":1}},"deliverables":[
{"target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video",
"settings":{"container":"mp4","video":{"codec":"h264","pixel_format":"yuv420p",
"alpha":"opaque"},"audio":$audio}}}]}]}}
JSON
  mirror_fixture_preview_canonical "$dir"
  cat > "$dir/plans/preview/out_preview.json" <<JSON
{"output":{"render_config_id":"out_preview","sequence_id":"seq_main"},"sequences":[
{"id":"seq_main","duration":{"timescale":1000,"value":$duration}}]}
JSON
}

make_smoke_video() {
  local file=$1 color=$2 audio=${3:-none}
  case $audio in
    none)
      ffmpeg -v error -y -f lavfi -i "color=c=$color:s=96x54:r=12:d=1" \
        -c:v libx264 -pix_fmt yuv420p "$file" ;;
    aac)
      ffmpeg -v error -y -f lavfi -i "color=c=$color:s=96x54:r=12:d=1" \
        -f lavfi -i 'sine=frequency=440:sample_rate=48000:duration=1' \
        -c:v libx264 -pix_fmt yuv420p -c:a aac -ac 2 -shortest "$file" ;;
    short-aac)
      ffmpeg -v error -y -f lavfi -i "color=c=$color:s=96x54:r=12:d=1" \
        -f lavfi -i 'sine=frequency=440:sample_rate=48000:duration=0.2' -t 1 \
        -c:v libx264 -pix_fmt yuv420p -c:a aac -ac 2 "$file" ;;
    short-video)
      ffmpeg -v error -y -f lavfi -i "color=c=$color:s=96x54:r=12:d=0.75" \
        -f lavfi -i 'sine=frequency=440:sample_rate=48000:duration=1' \
        -c:v libx264 -pix_fmt yuv420p -c:a aac -ac 2 "$file" ;;
  esac
}

make_smoke_visibility_video() {
  local file=$1 filter=$2
  ffmpeg -v error -y -f lavfi -i 'color=c=black:s=96x54:r=12:d=1' \
    -vf "$filter" -c:v libx264 -pix_fmt yuv420p "$file"
}
