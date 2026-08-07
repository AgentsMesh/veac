#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/example-render-evidence/common.sh"
# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/example-render-evidence/pixels.sh"
# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/example-render-evidence/showcase-pixels.sh"
# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/example-render-evidence/text.sh"
# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/example-render-evidence/text-showcase.sh"
# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/example-preview-fixtures.sh"
# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/tests/text-render-layout-fixtures.sh"

make_video() {
  local path=$1 duration=$2 size=$3 filter=$4
  ffmpeg -v error -y -f lavfi -i "color=c=black:s=${size}:d=${duration}:r=12" \
    -vf "$filter,fps=12" -c:v libx264 -pix_fmt yuv420p "$path"
}

expect_overlay_failure() {
  local video=$1 label=$2 log="$TMP_DIR/$2.log"
  if (check_text_render_contract text-overlay "$TMP_DIR/text-overlay" "$video") >"$log" 2>&1; then
    fail "$label should fail"
  fi
  rg -qF "shadow direction is invalid" "$log" || {
    cat "$log" >&2; fail "$label failed for the wrong reason"
  }
}

make_example() {
  local name=$1
  mkdir -p "$TMP_DIR/$name/rendered"
}

make_example text-layout
make_text_layout_project "$TMP_DIR/text-layout"
make_video "$TMP_DIR/text-layout/rendered/preview.mp4" 6 480x270 \
  "drawbox=x=25:y=25:w=120:h=55:color=cyan:t=fill:enable='lt(t,2)',drawbox=x=170:y=25:w=100:h=55:color=0xffd166:t=fill:enable='lt(t,2)',drawbox=x=300:y=25:w=100:h=55:color=white:t=fill:enable='lt(t,2)',drawbox=x=40:y=35:w=35:h=170:color=orange:t=fill:enable='between(t,2,3.999)',drawbox=x=405:y=35:w=35:h=170:color=cyan:t=fill:enable='between(t,2,3.999)',drawbox=x=50:y=215:w=380:h=28:color=0xffd166:t=fill:enable='gte(t,4)',drawbox=x=210:y=130:w=40:h=110:color=0xffd166:t=fill:enable='gte(t,4)'"

make_example text-overlay
make_video "$TMP_DIR/text-overlay/rendered/preview.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x1b263b:t=fill,drawbox=x=20:y=110:w=380:h=52:color=black@0.6:t=fill,drawbox=x=95:y=144:w=260:h=12:color=0xff477e@0.8:t=fill,drawbox=x=80:y=125:w=260:h=12:color=cyan:t=fill,drawbox=x=82:y=127:w=256:h=8:color=white:t=fill"

make_example captions-and-sidecars
mkdir -p "$TMP_DIR/captions-and-sidecars/project"
printf '%s\n' '{"project":{"sequences":[{"tracks":[{"clips":[' \
  '{"id":"generated-speaker","authorship":{"logical_path":["captions-and-sidecars","main","captions","speaker"],"events":[]},"source":{"speaker":"讲述者","style":{"background":{"color":{"alpha":170,"blue":0,"green":0,"red":0},"padding_pixels":12},"outline":null}},"visual":{"placement":{"anchor":"bottom","inset":{"x":0,"y":48},"type":"anchor"}}},' \
  '{"id":"generated-timing","authorship":{"logical_path":["captions-and-sidecars","main","captions","timing"],"events":[]},"source":{"style":{"background":{"color":{"alpha":170,"blue":0,"green":0,"red":0},"padding_pixels":12},"outline":null}},"visual":{"placement":{"anchor":"bottom","inset":{"x":0,"y":48},"type":"anchor"}}},' \
  '{"id":"generated-sidecar","authorship":{"logical_path":["captions-and-sidecars","main","captions","sidecar"],"events":[]},"source":{"style":{"background":{"color":{"alpha":170,"blue":0,"green":0,"red":0},"padding_pixels":12},"outline":null}},"visual":{"placement":{"anchor":"bottom","inset":{"x":0,"y":48},"type":"anchor"}}}' \
  ']}]}]}}' > "$TMP_DIR/captions-and-sidecars/project/project.veac.json"
make_video "$TMP_DIR/captions-and-sidecars/rendered/preview.mp4" 6 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x111827:t=fill,drawbox=x=80:y=205:w=320:h=45:color=black:t=fill,drawbox=x=120:y=220:w=240:h=20:color=white:t=fill:enable='lt(t,2)',drawbox=x=95:y=220:w=290:h=20:color=white:t=fill:enable='between(t,2,3.999)',drawbox=x=105:y=220:w=270:h=20:color=white:t=fill:enable='gte(t,4)'"
printf '%s\n' 'WEBVTT' '' '00:00:00.000 --> 00:00:02.000' '<v 讲述者>字幕既能烧录，也能独立交付</v>' '' \
  '00:00:02.000 --> 00:00:04.000' '每条字幕都有明确的半开时间范围' '' \
  '00:00:04.000 --> 00:00:06.000' '同一字幕轨输出 WebVTT 边车文件' '' > \
  "$TMP_DIR/captions-and-sidecars/rendered/captions.vtt"

make_example agentsmesh-intro-15s
make_video "$TMP_DIR/agentsmesh-intro-15s/rendered/preview.mp4" 15 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x0f766e:t=fill,drawbox=x=180:y=95:w=120:h=24:color=white:t=fill:enable='between(t,0,0.6)',drawbox=x=110:y=90:w=260:h=30:color=white:t=fill:enable='between(t,0.6,5)',drawbox=x=70:y=118:w=340:h=24:color=0xd8f3ef:t=fill:enable='gte(t,5)',drawbox=x=145:y=220:w=190:h=20:color=white:t=fill:enable='between(t,2,12)'"

make_example hello-world
make_video "$TMP_DIR/hello-world/rendered/preview.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=135:color=0x143642:t=fill,drawbox=x=0:y=135:w=480:h=135:color=0x0f1a20:t=fill,drawbox=x=195:y=129:w=90:h=12:color=white:t=fill"

for name in text-layout text-overlay captions-and-sidecars agentsmesh-intro-15s hello-world; do
  check_text_render_contract "$name" "$TMP_DIR/$name" "$TMP_DIR/$name/rendered/preview.mp4"
done

make_video "$TMP_DIR/black-text-layout.mp4" 6 480x270 "null"
if (check_text_render_contract text-layout "$TMP_DIR/text-layout" "$TMP_DIR/black-text-layout.mp4"); then
  fail "text-layout without rendered scripts should fail"
fi

cp "$TMP_DIR/captions-and-sidecars/rendered/captions.vtt" "$TMP_DIR/captions-and-sidecars/rendered/captions.good.vtt"
printf 'WEBVTT\n\n00:00:00.000 --> 00:00:06.000\nWrong cue.\n' > "$TMP_DIR/captions-and-sidecars/rendered/captions.vtt"
if (check_text_render_contract captions-and-sidecars "$TMP_DIR/captions-and-sidecars" "$TMP_DIR/captions-and-sidecars/rendered/preview.mp4"); then
  fail "wrong VTT should fail the caption contract"
fi
mv "$TMP_DIR/captions-and-sidecars/rendered/captions.good.vtt" "$TMP_DIR/captions-and-sidecars/rendered/captions.vtt"

cp "$TMP_DIR/captions-and-sidecars/project/project.veac.json" "$TMP_DIR/captions-and-sidecars/project/project.good.json"
sed 's/"bottom"/"center"/' "$TMP_DIR/captions-and-sidecars/project/project.good.json" > \
  "$TMP_DIR/captions-and-sidecars/project/project.veac.json"
if (check_text_render_contract captions-and-sidecars "$TMP_DIR/captions-and-sidecars" "$TMP_DIR/captions-and-sidecars/rendered/preview.mp4"); then
  fail "centered caption placement should fail the caption contract"
fi
mv "$TMP_DIR/captions-and-sidecars/project/project.good.json" "$TMP_DIR/captions-and-sidecars/project/project.veac.json"

cp "$TMP_DIR/captions-and-sidecars/project/project.veac.json" "$TMP_DIR/captions-and-sidecars/project/project.good.json"
sed 's/"alpha":170/"alpha":204/g' "$TMP_DIR/captions-and-sidecars/project/project.good.json" > "$TMP_DIR/captions-and-sidecars/project/project.veac.json"
if (check_text_render_contract captions-and-sidecars "$TMP_DIR/captions-and-sidecars" "$TMP_DIR/captions-and-sidecars/rendered/preview.mp4"); then
  fail "caption style drift should fail the typed contract"
fi
mv "$TMP_DIR/captions-and-sidecars/project/project.good.json" "$TMP_DIR/captions-and-sidecars/project/project.veac.json"

make_video "$TMP_DIR/no-caption-panel.mp4" 6 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x111827:t=fill,drawbox=x=120:y=220:w=240:h=20:color=white:t=fill:enable='lt(t,2)',drawbox=x=95:y=220:w=290:h=20:color=white:t=fill:enable='between(t,2,3.999)',drawbox=x=105:y=220:w=270:h=20:color=white:t=fill:enable='gte(t,4)'"
if (check_text_render_contract captions-and-sidecars "$TMP_DIR/captions-and-sidecars" "$TMP_DIR/no-caption-panel.mp4"); then
  fail "caption text without its declared panel should fail"
fi

make_video "$TMP_DIR/no-shadow-overlay.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x1b263b:t=fill,drawbox=x=20:y=110:w=380:h=52:color=black@0.6:t=fill,drawbox=x=80:y=125:w=260:h=12:color=cyan:t=fill,drawbox=x=82:y=127:w=256:h=8:color=white:t=fill"
expect_overlay_failure "$TMP_DIR/no-shadow-overlay.mp4" "overlay-without-shadow"

make_video "$TMP_DIR/no-down-shadow-overlay.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x1b263b:t=fill,drawbox=x=20:y=110:w=380:h=52:color=black@0.6:t=fill,drawbox=x=95:y=120:w=260:h=12:color=0xff477e@0.8:t=fill,drawbox=x=80:y=125:w=260:h=12:color=cyan:t=fill,drawbox=x=82:y=127:w=256:h=8:color=white:t=fill"
expect_overlay_failure "$TMP_DIR/no-down-shadow-overlay.mp4" "overlay-without-down-shadow"

make_video "$TMP_DIR/no-right-shadow-overlay.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x1b263b:t=fill,drawbox=x=20:y=110:w=380:h=52:color=black@0.6:t=fill,drawbox=x=80:y=144:w=260:h=12:color=0xff477e@0.8:t=fill,drawbox=x=80:y=125:w=260:h=12:color=cyan:t=fill,drawbox=x=82:y=127:w=256:h=8:color=white:t=fill"
expect_overlay_failure "$TMP_DIR/no-right-shadow-overlay.mp4" "overlay-without-right-shadow"

make_video "$TMP_DIR/short-hello.mp4" 3 480x270 \
  "drawbox=x=0:y=0:w=480:h=135:color=0x143642:t=fill,drawbox=x=0:y=135:w=480:h=135:color=0x0f1a20:t=fill,drawbox=x=155:y=120:w=170:h=25:color=white:t=fill"
if (check_text_render_contract hello-world "$TMP_DIR/hello-world" "$TMP_DIR/short-hello.mp4"); then
  fail "wrong hello-world duration should fail the media contract"
fi

echo "text render evidence contract tests passed"
