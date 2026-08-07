#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
SCRIPT_DIR="$ROOT/scripts"
source "$ROOT/scripts/tests/example-preview-layout-fixtures.sh"
source "$SCRIPT_DIR/example-render-evidence/common.sh"
source "$SCRIPT_DIR/example-render-evidence/timing.sh"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/veac-transform-contracts.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

make_project() {
  local dir=$1
  prepare_preview_fixture_dirs "$dir"
  cat >"$dir/project/project.veac.json" <<'JSON'
{"project":{"authorship":{"entity":{"logical_path":["transforms-and-animation"],"events":[]},"multicam_groups":[],"annotations":[],"deliveries":[{"render_config_id":"generated-preview","entity":{"logical_path":["transforms-and-animation","preview"],"events":[]}}]},"render_configs":[{"id":"generated-preview","deliverables":[{"id":"generated-video","target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video"}}]}],"sequences":[{"tracks":[{"clips":[{"id":"generated-badge","authorship":{"logical_path":["transforms-and-animation","main","graphics","badge"],"events":[]},"visual":{"opacity":{"type":"binding","binding_id":"generated-opacity"}}}]}]}]},"temporal":{"bindings":[{"id":"generated-opacity","program_id":"generated-program","result_type":"scalar","clocks":[{"input_id":0,"clock":"progress","owner":{"type":"item","item_id":"generated-badge"}}],"parameters":[],"provenance_id":"generated-provenance"}],"programs":[{"id":"generated-program"}],"provenance":[{"id":"generated-provenance","definition":{"name":"visual-opacity"},"origin":{"function":"animate"}}]}}
JSON
  mirror_fixture_preview_canonical "$dir"
  cat >"$dir/plans/preview/generated-preview.json" <<'JSON'
{"sequences":[{"tracks":[{"clips":[{"id":"generated-badge","visual":{"opacity":{"type":"binding","binding_id":"generated-opacity"},"placement":{"anchor":"bottom_right","inset":{"x":80,"y":80},"type":"anchor"},"transform":{"anchor":{"x":1,"y":1},"crop":{"value":{"x":0.14,"y":0.04,"width":0.84,"height":0.92}},"flip_horizontal":true,"flip_vertical":false,"position":{"type":"keyframes","keyframes":[{"time":{"timescale":600,"value":0},"interpolation":{"type":"hold"}},{"time":{"timescale":600,"value":270},"interpolation":{"type":"linear"}},{"time":{"timescale":600,"value":540},"interpolation":{"type":"ease_in"}},{"time":{"timescale":600,"value":810},"interpolation":{"type":"ease_out"}},{"time":{"timescale":600,"value":1080},"interpolation":{"type":"ease_in_out"}},{"time":{"timescale":600,"value":1350},"interpolation":{"type":"cubic_bezier"}},{"time":{"timescale":600,"value":1620},"interpolation":{"type":"spring"}},{"time":{"timescale":600,"value":2400},"interpolation":{"type":"linear"}}]},"rotation_degrees":{"value":5},"scale":{"value":{"x":1.12,"y":0.92}},"shear":{"x":0.22,"y":-0.08}}}}]}]}],"temporal":{"bindings":[{"id":"generated-opacity","clocks":[{"clock":"progress"}]}]}}
JSON
}

segmented_x() {
  printf '%s' "if(lt(t\,.45)\,420\,if(lt(t\,.9)\,420-100*(t-.45)\,if(lt(t\,1.35)\,375-100*(t-.9)\,if(lt(t\,1.8)\,330-100*(t-1.35)\,if(lt(t\,2.25)\,285-100*(t-1.8)\,240-100*(t-2.25))))))"
}

spring_x() {
  printf '%s' "if(lt(t\,3)\,150+166.667*(t-2.7)\,if(lt(t\,3.3)\,200+3.333*(t-3)\,if(lt(t\,3.6)\,201-3.333*(t-3.3)\,200+2*(t-3.6))))"
}

unsettled_x() {
  printf '%s' "if(lt(t\,3)\,150+166.667*(t-2.7)\,if(lt(t\,3.3)\,200+333.333*(t-3)\,if(lt(t\,3.6)\,300+266.667*(t-3.3)\,380-111.111*(t-3.6))))"
}

make_video() {
  local dir=$1 pre=${2:-segmented} spring=${3:-spring} x y
  if [[ $pre == segmented ]]; then x=$(segmented_x); y='160-12*sin(t*5)'; else x=420; y=160; fi
  if [[ $spring == spring ]]; then x="if(lt(t\,2.7)\,$x\,$(spring_x))"
  elif [[ $spring == unsettled ]]; then x="if(lt(t\,2.7)\,$x\,$(unsettled_x))"
  else x="if(lt(t\,2.7)\,$x\,150)"; fi
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0x123a34:s=480x270:r=30:d=4' \
    -f lavfi -i 'color=c=0xf2483d:s=50x36:r=30:d=4' \
    -filter_complex "[0:v][1:v]overlay=x='$x':y='$y':enable='between(t,.2,3.8)':eval=frame:shortest=1" \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
}

make_fixture() {
  make_project "$1"
  make_video "$1" "${2:-segmented}" "${3:-spring}"
}

expect_failure() {
  local label=$1 dir=$2 expected=$3
  local log="$tmp/$label.log"
  if (check_transform_animation_evidence "$dir" >"$log" 2>&1); then
    fail "$label negative transform contract unexpectedly passed"
  fi
  rg -q --fixed-strings "$expected" "$log" || { cat "$log" >&2; fail "$label failed incorrectly"; }
}

valid="$tmp/valid"; make_fixture "$valid"; check_transform_animation_evidence "$valid"
missing="$tmp/missing"; make_fixture "$missing" frozen spring
expect_failure missing_segments "$missing" 'hold/segment trajectory is wrong'
frozen="$tmp/frozen"; make_fixture "$frozen" segmented frozen
expect_failure frozen_spring "$frozen" 'Spring trajectory does not advance and settle'
unsettled="$tmp/unsettled"; make_fixture "$unsettled" segmented unsettled
expect_failure unsettled_spring "$unsettled" 'Spring trajectory does not advance and settle'
printf 'transform animation render evidence contract tests passed\n'
