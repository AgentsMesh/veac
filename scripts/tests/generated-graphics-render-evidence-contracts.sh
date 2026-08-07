#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
for module in common pixels showcase-pixels mechanism-pixels visual-mechanisms generated-graphics; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
tmp=$(mktemp -d "${TMPDIR:-/tmp}/veac-generated-contracts.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

delivery_video_path() { printf '%s/rendered/preview.mp4\n' "$1"; }

write_canonical() {
  local file=$1
  cat >"$file" <<'JSON'
{"project":{"sequences":[{"tracks":[{"clips":[{"authorship":{"logical_path":["generated-graphics","plate"]},"record_range":{"start":{"timescale":1,"value":0},"duration":{"timescale":1,"value":9}},"source":{"type":"generated","generator":{"type":"solid","color":{"red":17,"green":24,"blue":39,"alpha":255}}}},{"authorship":{"logical_path":["generated-graphics","solid"]},"record_range":{"start":{"timescale":1,"value":0},"duration":{"timescale":1,"value":1}},"source":{"generator":{"type":"solid","color":{"red":220,"green":38,"blue":38,"alpha":255}}}},{"authorship":{"logical_path":["generated-graphics","transparent"]},"record_range":{"start":{"timescale":1,"value":1},"duration":{"timescale":1,"value":1}},"source":{"generator":{"type":"transparent"}}},{"authorship":{"logical_path":["generated-graphics","linear"]},"record_range":{"start":{"timescale":1,"value":2},"duration":{"timescale":1,"value":1}},"source":{"generator":{"type":"gradient","gradient":{"type":"linear"}}}},{"authorship":{"logical_path":["generated-graphics","radial"]},"record_range":{"start":{"timescale":1,"value":3},"duration":{"timescale":1,"value":1}},"source":{"generator":{"type":"gradient","gradient":{"type":"radial"}}}},{"authorship":{"logical_path":["generated-graphics","rectangle"]},"record_range":{"start":{"timescale":1,"value":4},"duration":{"timescale":1,"value":1}},"source":{"generator":{"type":"shape","shape":{"geometry":{"type":"rectangle"}}}}},{"authorship":{"logical_path":["generated-graphics","rounded"]},"record_range":{"start":{"timescale":1,"value":5},"duration":{"timescale":1,"value":1}},"source":{"generator":{"type":"shape","shape":{"geometry":{"type":"rounded_rectangle"}}}}},{"authorship":{"logical_path":["generated-graphics","ellipse"]},"record_range":{"start":{"timescale":1,"value":6},"duration":{"timescale":1,"value":1}},"source":{"generator":{"type":"shape","shape":{"geometry":{"type":"ellipse"}}}}},{"authorship":{"logical_path":["generated-graphics","polygon"]},"record_range":{"start":{"timescale":1,"value":7},"duration":{"timescale":1,"value":1}},"source":{"generator":{"type":"shape","shape":{"geometry":{"type":"polygon","points":[{},{},{},{}]}}}}},{"authorship":{"logical_path":["generated-graphics","path"]},"record_range":{"start":{"timescale":1,"value":8},"duration":{"timescale":1,"value":1}},"source":{"generator":{"type":"shape","shape":{"geometry":{"type":"path","commands":[{"type":"move_to"},{"type":"line_to"},{"type":"line_to"},{"type":"line_to"},{"type":"close"}]},"fill":null,"stroke":{"width_pixels":8}}}}}]}]}]}}
JSON
  jq '
    (.. | objects | select(.authorship?.logical_path[-1]? == "path") |
      .source.generator.shape.geometry.commands) |=
    [.[0]+{point:{x:0.1,y:0.8}},.[1]+{point:{x:0.35,y:0.2}},
     .[2]+{point:{x:0.65,y:0.8}},.[3]+{point:{x:0.9,y:0.2}},.[4]]
  ' "$file" >"$file.tmp"
  mv "$file.tmp" "$file"
}

make_video() {
  local file=$1
  ffmpeg -v error -y -f lavfi -i 'color=c=0x111827:s=480x270:r=12:d=9' -vf \
    "drawbox=x=0:y=0:w=iw:h=ih:color=0xdc2626:t=fill:enable='lt(t,1)',drawbox=x=0:y=0:w=160:h=210:color=0xef4444:t=fill:enable='between(t,2,2.999)',drawbox=x=320:y=0:w=160:h=210:color=0x22d3ee:t=fill:enable='between(t,2,2.999)',drawbox=x=0:y=0:w=iw:h=210:color=0x22d3ee:t=fill:enable='between(t,3,3.999)',drawbox=x=140:y=105:w=200:h=105:color=0xfacc15:t=fill:enable='between(t,3,3.999)',drawbox=x=72:y=54:w=336:h=162:color=0xfacc15:t=fill:enable='between(t,4,4.999)',drawbox=x=95:y=64:w=290:h=142:color=0x10b981:t=fill:enable='between(t,5,5.999)',drawbox=x=130:y=65:w=220:h=140:color=0xef4444:t=fill:enable='between(t,6,6.999)',drawbox=x=150:y=75:w=180:h=130:color=0xfacc15:t=fill:enable='between(t,7,7.999)',drawbox=x=100:y=124:w=24:h=24:color=0xfacc15:t=fill:enable='gte(t,8)',drawbox=x=80:y=232:w=320:h=22:color=white:t=fill" \
    -c:v libx264 -pix_fmt yuv420p "$file"
}

expect_fail() {
  local label=$1 root=$2
  if (PREVIEW_ROOT=$root check_generated_graphics_evidence >/dev/null 2>&1); then
    fail "$label negative generated-graphics contract passed"
  fi
}

valid="$tmp/valid/generated-graphics"
mkdir -p "$valid/project" "$valid/rendered"
write_canonical "$valid/project/project.veac.json"
make_video "$valid/rendered/preview.mp4"
PREVIEW_ROOT="$tmp/valid" check_generated_graphics_evidence

bad_order="$tmp/bad-order"
mkdir -p "$bad_order"
cp -R "$valid" "$bad_order/generated-graphics"
jq '(.project.sequences[0].tracks[0].clips[] |
  select(.authorship.logical_path[-1]=="linear").record_range.start.value)=3' \
  "$bad_order/generated-graphics/project/project.veac.json" >"$bad_order/value.tmp"
mv "$bad_order/value.tmp" "$bad_order/generated-graphics/project/project.veac.json"
expect_fail wrong_generator_order "$bad_order"

bad_path="$tmp/bad-path"
mkdir -p "$bad_path"
cp -R "$valid" "$bad_path/generated-graphics"
jq '(.. | objects | select(.authorship?.logical_path[-1]? == "path") |
  .source.generator.shape.geometry.commands[1].point.x)=0.4' \
  "$bad_path/generated-graphics/project/project.veac.json" >"$bad_path/value.tmp"
mv "$bad_path/value.tmp" "$bad_path/generated-graphics/project/project.veac.json"
expect_fail wrong_path_geometry "$bad_path"

old_duration="$tmp/old-duration"
mkdir -p "$old_duration"
cp -R "$valid" "$old_duration/generated-graphics"
ffmpeg -v error -y -i "$valid/rendered/preview.mp4" -t 8 -c copy \
  "$old_duration/generated-graphics/rendered/preview.mp4"
expect_fail old_eight_second_media "$old_duration"

printf 'generated graphics render evidence contract tests passed\n'
