#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "$ROOT/scripts/example-render-evidence/common.sh"
source "$ROOT/scripts/tests/example-preview-layout-fixtures.sh"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/veac-resolution-chain.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

write_project() {
  local file=$1 width=$2 height=$3 raster_width=$4 raster_height=$5 rate=$6
  jq -n --argjson width "$width" --argjson height "$height" \
    --argjson raster_width "$raster_width" --argjson raster_height "$raster_height" \
    --argjson rate "$rate" '
    {project:{entry_sequence_id:"seq_main",sequences:[{id:"seq_main",settings:{
      width:$width,height:$height,frame_rate:{numerator:$rate,denominator:1}}}],
      render_configs:[{id:"out_preview",raster:{width:$raster_width,height:$raster_height,
        frame_rate:{numerator:$rate,denominator:1}}}]}}' >"$file"
}

write_fixture() {
  local dir=$1
  prepare_preview_fixture_dirs "$dir"
  write_project "$dir/project/project.veac.json" 640 360 640 360 30
  write_project "$dir/project/project.preview.veac.json" 640 360 480 270 12
  jq -n '{output:{sequence_id:"seq_main",raster:{width:480,height:270}},
    sequences:[{id:"seq_main",settings:{width:640,height:360}}]}' \
    >"$dir/plans/preview/out_preview.json"
  ffmpeg -nostdin -v error -y -f lavfi -i 'color=c=0x2563eb:s=480x270:r=12:d=1' \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
}

expect_failure() {
  local label=$1 expected=$2 dir="$tmp/$1" log="$tmp/$1.log"
  cp -R "$tmp/valid" "$dir"
  "$3" "$dir"
  if (assert_resolution_chain "$dir" "$dir/plans/preview/out_preview.json" \
    "$dir/rendered/preview.mp4" fixture) >"$log" 2>&1; then
    fail "$label negative resolution contract unexpectedly passed"
  fi
  rg -qF "$expected" "$log" || fail "$label failed for the wrong reason"
}

valid="$tmp/valid"
write_fixture "$valid"
assert_resolution_chain "$valid" "$valid/plans/preview/out_preview.json" \
  "$valid/rendered/preview.mp4" fixture
bad_author() { jq '.project.sequences[0].settings.width=320' "$1/project/project.veac.json" >"$1/x"; mv "$1/x" "$1/project/project.veac.json"; }
bad_preview() { jq '.project.render_configs[0].raster.height=180' "$1/project/project.preview.veac.json" >"$1/x"; mv "$1/x" "$1/project/project.preview.veac.json"; }
bad_plan() { jq '.output.raster.width=320' "$1/plans/preview/out_preview.json" >"$1/x"; mv "$1/x" "$1/plans/preview/out_preview.json"; }
bad_media() { ffmpeg -nostdin -v error -y -f lavfi -i 'color=c=red:s=320x180:d=1' -c:v libx264 -pix_fmt yuv420p "$1/rendered/x.mp4"; mv "$1/rendered/x.mp4" "$1/rendered/preview.mp4"; }
expect_failure author 'authoring resolution contract failed' bad_author
expect_failure preview 'preview resolution contract failed' bad_preview
expect_failure plan 'plan resolution contract failed' bad_plan
expect_failure media 'rendered resolution: expected width=480' bad_media
printf 'resolution chain render evidence contracts passed\n'
