#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
SCRIPT_DIR="$ROOT/scripts"
source "$ROOT/scripts/tests/example-preview-layout-fixtures.sh"
# shellcheck source=../example-render-evidence/common.sh
source "$SCRIPT_DIR/example-render-evidence/common.sh"
# shellcheck source=../example-render-evidence/timing.sh
source "$SCRIPT_DIR/example-render-evidence/timing.sh"

tmp=$(mktemp -d "${TMPDIR:-/tmp}/veac-transform-contracts.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

make_project() {
  local dir=$1
  prepare_preview_fixture_dirs "$dir"
  cat >"$dir/project/project.veac.json" <<'JSON'
{"project":{"render_configs":[{"id":"out_preview","deliverables":[{"id":"dlv_preview","target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video"}}]}]}}
JSON
  mirror_fixture_preview_canonical "$dir"
  cat >"$dir/plans/preview/out_preview.json" <<'JSON'
{"sequences":[{"tracks":[{"clips":[{"id":"itm_badge","visual":{"placement":{"anchor":"bottom_right"},"transform":{"anchor":{"x":1,"y":1},"crop":{"value":{"x":0.14,"y":0.04,"width":0.84,"height":0.92}},"flip_horizontal":true,"flip_vertical":false,"position":{"type":"keyframes","keyframes":[{"id":"kf_hold","time":{"timescale":1000,"value":0},"value":{"x":{"unit":"pixels","value":240},"y":{"unit":"pixels","value":0}},"interpolation":{"type":"hold"}},{"id":"kf_linear","time":{"timescale":1000,"value":120},"value":{"x":{"unit":"pixels","value":200},"y":{"unit":"pixels","value":-20}},"interpolation":{"type":"linear"}},{"id":"kf_ease-in","time":{"timescale":1000,"value":240},"value":{"x":{"unit":"pixels","value":160},"y":{"unit":"pixels","value":0}},"interpolation":{"type":"ease_in"}},{"id":"kf_ease-out","time":{"timescale":1000,"value":360},"value":{"x":{"unit":"pixels","value":120},"y":{"unit":"pixels","value":20}},"interpolation":{"type":"ease_out"}},{"id":"kf_ease-in-out","time":{"timescale":1000,"value":480},"value":{"x":{"unit":"pixels","value":80},"y":{"unit":"pixels","value":0}},"interpolation":{"type":"ease_in_out"}},{"id":"kf_bezier","time":{"timescale":1000,"value":600},"value":{"x":{"unit":"pixels","value":40},"y":{"unit":"pixels","value":-20}},"interpolation":{"type":"cubic_bezier","x1":0.42,"x2":0.58,"y1":0,"y2":1}},{"id":"kf_settle","time":{"timescale":1000,"value":800},"value":{"x":{"unit":"pixels","value":-120},"y":{"unit":"pixels","value":-80}},"interpolation":{"type":"spring","frequency":1.5,"decay":6,"initial_velocity":0}},{"id":"kf_rest","time":{"timescale":1000,"value":4000},"value":{"x":{"unit":"pixels","value":0},"y":{"unit":"pixels","value":0}},"interpolation":{"type":"linear"}}]},"rotation_degrees":{"value":5},"scale":{"value":{"x":1.12,"y":0.92}},"shear":{"x":0.22,"y":-0.08}}}}]}]}]}
JSON
}

segmented_x() {
  printf "%s" "if(lt(t\,.12)\,420\,if(lt(t\,.24)\,400-250*(t-.12)\,if(lt(t\,.36)\,370-250*(t-.24)\,if(lt(t\,.48)\,340-250*(t-.36)\,if(lt(t\,.6)\,310-250*(t-.48)\,280-500*(t-.6))))))"
}

segmented_y() {
  printf "%s" "if(lt(t\,.12)\,160\,if(lt(t\,.24)\,150+83.333*(t-.12)\,if(lt(t\,.36)\,160+83.333*(t-.24)\,if(lt(t\,.48)\,170-83.333*(t-.36)\,if(lt(t\,.6)\,160-83.333*(t-.48)\,150-200*(t-.6))))))"
}

spring_x() {
  printf "%s" "if(lt(t\,1)\,180+200*(t-.8)\,if(lt(t\,1.4)\,220+300*(t-1)\,if(lt(t\,1.8)\,340+100*(t-1.4)\,if(lt(t\,2.4)\,380-33.333*(t-1.8)\,360))))"
}

make_video() {
  local dir=$1 pre=${2:-segmented} spring=${3:-spring} x y
  if [[ $pre == segmented ]]; then x=$(segmented_x); y=$(segmented_y); else x=420; y=160; fi
  if [[ $spring == spring ]]; then
    x="if(lt(t\,.8)\,$x\,$(spring_x))"
    y="if(lt(t\,.8)\,$y\,110+30*(t-.8))"
  else
    x="if(lt(t\,.8)\,$x\,180)"
    y="if(lt(t\,.8)\,$y\,110)"
  fi
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0x123a34:s=480x270:r=30:d=4' \
    -f lavfi -i 'color=c=0xf2483d:s=50x36:r=30:d=4' \
    -filter_complex "[0:v][1:v]overlay=x='$x':y='$y':eval=frame:shortest=1" \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
}

make_fixture() {
  local dir=$1 pre=${2:-segmented} spring=${3:-spring}
  make_project "$dir"
  make_video "$dir" "$pre" "$spring"
}

expect_failure() {
  local label=$1 dir=$2 expected=$3
  local log="$tmp/$label.log"
  if (check_transform_animation_evidence "$dir" >"$log" 2>&1); then
    fail "$label negative transform contract unexpectedly passed"
  fi
  rg -q --fixed-strings "$expected" "$log" || {
    cat "$log" >&2
    fail "$label failed for the wrong transform mechanism"
  }
}

valid="$tmp/valid"
make_fixture "$valid"
check_transform_animation_evidence "$valid"

missing_segments="$tmp/missing-segments"
make_fixture "$missing_segments" frozen spring
expect_failure missing_segments "$missing_segments" 'hold/segment trajectory is wrong'

frozen_spring="$tmp/frozen-spring"
make_fixture "$frozen_spring" segmented frozen
expect_failure frozen_spring "$frozen_spring" 'Spring trajectory does not advance and settle'

printf 'transform animation render evidence contract tests passed\n'
