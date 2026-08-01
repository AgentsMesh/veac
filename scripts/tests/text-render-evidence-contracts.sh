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

mutate_video() {
  local source=$1 path=$2 filter=$3
  ffmpeg -v error -y -i "$source" -vf "$filter" \
    -c:v libx264 -pix_fmt yuv420p -an "$path"
}

expect_animation_failure() {
  local video=$1 expected=$2 label=$3 log="$TMP_DIR/$3.log"
  if (check_text_render_contract text-animation "$TMP_DIR/text-animation" "$video") \
      >"$log" 2>&1; then
    fail "$label should fail"
  fi
  rg -qF "$expected" "$log" || {
    cat "$log" >&2
    fail "$label failed for the wrong reason"
  }
}

make_example() {
  local name=$1
  mkdir -p "$TMP_DIR/$name/rendered"
}

make_example text-layout
make_text_layout_project "$TMP_DIR/text-layout"
make_video "$TMP_DIR/text-layout/rendered/preview.mp4" 6 480x480 \
  "drawbox=x=25:y=30:w=120:h=70:color=cyan:t=fill:enable='lt(t,2)',drawbox=x=170:y=30:w=100:h=70:color=0xffd166:t=fill:enable='lt(t,2)',drawbox=x=300:y=30:w=100:h=70:color=white:t=fill:enable='lt(t,2)',drawbox=x=40:y=30:w=35:h=180:color=orange:t=fill:enable='between(t,2,3.999)',drawbox=x=375:y=30:w=35:h=180:color=cyan:t=fill:enable='between(t,2,3.999)',drawbox=x=40:y=300:w=380:h=28:color=0xffd166:t=fill:enable='gte(t,4)',drawbox=x=210:y=220:w=40:h=120:color=0xffd166:t=fill:enable='gte(t,4)'"

make_example text-animation
make_video "$TMP_DIR/text-animation/rendered/preview.mp4" 12 480x270 \
  "drawbox=x=115:y=120:w=80:h=22:color=0x808080:t=fill:enable='between(t,.15,.349)',drawbox=x=115:y=116:w=120:h=28:color=0xd0d0d0:t=fill:enable='between(t,.35,.75)',drawbox=x=115:y=115:w=160:h=30:color=white:t=fill:enable='between(t,.75,1.499)',drawbox=x=160:y=115:w=160:h=30:color=white:t=fill:enable='between(t,1.5,2.399)',drawbox=x=140:y=110:w=200:h=40:color=white:t=fill:enable='between(t,2.4,2.999)',drawbox=x=140:y=110:w=40:h=40:color=0xffd166:t=fill:enable='between(t,3,3.899)',drawbox=x=140:y=110:w=110:h=40:color=0xffd166:t=fill:enable='between(t,3.9,4.499)',drawbox=x=140:y=110:w=180:h=40:color=0xffd166:t=fill:enable='between(t,4.5,4.999)',drawbox=x=150:y=105:w=180:h=25:color=cyan:t=fill:enable='between(t,5,6.999)',drawbox=x=150:y=145:w=180:h=25:color=cyan:t=fill:enable='between(t,5.4,6.999)',drawbox=x=150:y=116:w=30:h=25:color=orange:t=fill:enable='between(t,7,7.299)',drawbox=x=150:y=116:w=75:h=25:color=orange:t=fill:enable='between(t,7.3,7.599)',drawbox=x=150:y=116:w=125:h=25:color=orange:t=fill:enable='between(t,7.6,7.899)',drawbox=x=150:y=116:w=180:h=25:color=orange:t=fill:enable='between(t,7.9,8.999)',drawbox=x=150:y=116:w=25:h=25:color=white:t=fill:enable='between(t,9,9.399)',drawbox=x=150:y=116:w=45:h=25:color=white:t=fill:enable='between(t,9.4,9.649)',drawbox=x=150:y=116:w=70:h=25:color=white:t=fill:enable='between(t,9.65,9.899)',drawbox=x=150:y=116:w=100:h=25:color=white:t=fill:enable='between(t,9.9,10.149)',drawbox=x=150:y=116:w=140:h=25:color=white:t=fill:enable='between(t,10.15,10.399)',drawbox=x=150:y=116:w=180:h=25:color=white:t=fill:enable='gte(t,10.4)'"

make_example text-overlay
make_video "$TMP_DIR/text-overlay/rendered/preview.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x1b263b:t=fill,drawbox=x=150:y=121:w=180:h=28:color=black@0.6:t=fill,drawbox=x=158:y=128:w=180:h=28:color=0xff477e@0.8:t=fill,drawbox=x=153:y=124:w=174:h=22:color=cyan:t=fill,drawbox=x=155:y=126:w=170:h=18:color=white:t=fill"

make_example captions-and-sidecars
mkdir -p "$TMP_DIR/captions-and-sidecars/project"
printf '%s\n' '{"project":{"sequences":[{"tracks":[{"clips":[' \
  '{"id":"itm_opening","source":{"style":{"background":{"color":{"alpha":170,"blue":0,"green":0,"red":0},"padding_pixels":12}}},"visual":{"placement":{"anchor":"bottom","inset":{"x":0,"y":48},"type":"anchor"}}},' \
  '{"id":"itm_closing","source":{"style":{"background":null}},"visual":{"placement":{"anchor":"bottom","inset":{"x":0,"y":48},"type":"anchor"}}}' \
  ']}]}]}}' > "$TMP_DIR/captions-and-sidecars/project/project.veac.json"
make_video "$TMP_DIR/captions-and-sidecars/rendered/preview.mp4" 6 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x111827:t=fill,drawbox=x=125:y=219:w=230:h=34:color=black@0.75:t=fill:enable='lt(t,3)',drawbox=x=135:y=225:w=210:h=22:color=white:t=fill:enable='lt(t,3)',drawbox=x=95:y=225:w=290:h=22:color=white:t=fill:enable='gte(t,3)'"
printf '%s\n' 'WEBVTT' '' '00:00:00.000 --> 00:00:03.000' '<v 旁白>字幕是时间线中的类型化源。</v>' '' \
  '00:00:03.000 --> 00:00:06.000' '<v 旁白>伴随文件从类型化字幕图层中选择内容。</v>' '' > \
  "$TMP_DIR/captions-and-sidecars/rendered/captions.vtt"

make_example agentsmesh-intro-15s
make_video "$TMP_DIR/agentsmesh-intro-15s/rendered/preview.mp4" 15 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x0f766e:t=fill,drawbox=x=180:y=95:w=120:h=24:color=white:t=fill:enable='between(t,0,0.6)',drawbox=x=110:y=90:w=260:h=30:color=white:t=fill:enable='between(t,0.6,5)',drawbox=x=70:y=118:w=340:h=24:color=0xd8f3ef:t=fill:enable='gte(t,5)',drawbox=x=145:y=220:w=190:h=20:color=white:t=fill:enable='between(t,2,12)'"

make_example hello-world
make_video "$TMP_DIR/hello-world/rendered/preview.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=135:color=0x143642:t=fill,drawbox=x=0:y=135:w=480:h=135:color=0x0f1a20:t=fill,drawbox=x=155:y=120:w=170:h=25:color=white:t=fill"

for name in text-layout text-animation text-overlay captions-and-sidecars agentsmesh-intro-15s hello-world; do
  check_text_render_contract "$name" "$TMP_DIR/$name" "$TMP_DIR/$name/rendered/preview.mp4"
done

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

make_video "$TMP_DIR/no-caption-panel.mp4" 6 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x111827:t=fill,drawbox=x=135:y=225:w=210:h=22:color=white:t=fill:enable='lt(t,3)',drawbox=x=95:y=225:w=290:h=22:color=white:t=fill:enable='gte(t,3)'"
if (check_text_render_contract captions-and-sidecars "$TMP_DIR/captions-and-sidecars" "$TMP_DIR/no-caption-panel.mp4"); then
  fail "caption text without its declared panel should fail"
fi

make_video "$TMP_DIR/black-animation.mp4" 12 480x270 "null"
if (check_text_render_contract text-animation "$TMP_DIR/text-animation" "$TMP_DIR/black-animation.mp4"); then
  fail "black animation should fail the text animation contract"
fi

make_video "$TMP_DIR/static-animation.mp4" 12 480x270 \
  "drawbox=x=140:y=110:w=200:h=40:color=white:t=fill:enable='lt(t,3)',drawbox=x=140:y=110:w=200:h=40:color=0xffd166:t=fill:enable='between(t,3,4.999)',drawbox=x=150:y=105:w=180:h=65:color=cyan:t=fill:enable='between(t,5,6.999)',drawbox=x=150:y=116:w=180:h=25:color=orange:t=fill:enable='between(t,7,8.999)',drawbox=x=150:y=116:w=180:h=25:color=white:t=fill:enable='gte(t,9)'"
if (check_text_render_contract text-animation "$TMP_DIR/text-animation" "$TMP_DIR/static-animation.mp4"); then
  fail "static text stages should fail the progression contract"
fi

make_video "$TMP_DIR/no-slide-animation.mp4" 12 480x270 \
  "drawbox=x=160:y=120:w=80:h=22:color=0x808080:t=fill:enable='between(t,.15,.349)',drawbox=x=160:y=116:w=120:h=28:color=0xd0d0d0:t=fill:enable='between(t,.35,.75)',drawbox=x=160:y=115:w=160:h=30:color=white:t=fill:enable='between(t,.75,2.399)',drawbox=x=140:y=110:w=200:h=40:color=white:t=fill:enable='between(t,2.4,2.999)',drawbox=x=140:y=110:w=200:h=40:color=0xffd166:t=fill:enable='between(t,3,4.999)',drawbox=x=150:y=105:w=180:h=25:color=cyan:t=fill:enable='between(t,5,6.999)',drawbox=x=150:y=145:w=180:h=25:color=cyan:t=fill:enable='between(t,5.5,6.999)',drawbox=x=150:y=116:w=180:h=25:color=orange:t=fill:enable='between(t,7,8.999)',drawbox=x=150:y=116:w=180:h=25:color=white:t=fill:enable='gte(t,9)'"
if (check_text_render_contract text-animation "$TMP_DIR/text-animation" "$TMP_DIR/no-slide-animation.mp4"); then
  fail "text animation without slide should fail"
fi

make_video "$TMP_DIR/no-scale-animation.mp4" 12 480x270 \
  "drawbox=x=115:y=120:w=80:h=22:color=0x808080:t=fill:enable='between(t,.15,.349)',drawbox=x=115:y=116:w=120:h=28:color=0xd0d0d0:t=fill:enable='between(t,.35,.75)',drawbox=x=115:y=115:w=160:h=30:color=white:t=fill:enable='between(t,.75,1.499)',drawbox=x=160:y=115:w=160:h=30:color=white:t=fill:enable='between(t,1.5,2.999)',drawbox=x=140:y=110:w=200:h=40:color=0xffd166:t=fill:enable='between(t,3,4.999)',drawbox=x=150:y=105:w=180:h=25:color=cyan:t=fill:enable='between(t,5,6.999)',drawbox=x=150:y=145:w=180:h=25:color=cyan:t=fill:enable='between(t,5.5,6.999)',drawbox=x=150:y=116:w=180:h=25:color=orange:t=fill:enable='between(t,7,8.999)',drawbox=x=150:y=116:w=180:h=25:color=white:t=fill:enable='gte(t,9)'"
if (check_text_render_contract text-animation "$TMP_DIR/text-animation" "$TMP_DIR/no-scale-animation.mp4"); then
  fail "text animation without scale should fail"
fi

make_video "$TMP_DIR/no-highlight-animation.mp4" 12 480x270 \
  "drawbox=x=115:y=120:w=80:h=22:color=0x808080:t=fill:enable='between(t,.15,.349)',drawbox=x=115:y=116:w=120:h=28:color=0xd0d0d0:t=fill:enable='between(t,.35,.75)',drawbox=x=115:y=115:w=160:h=30:color=white:t=fill:enable='between(t,.75,1.499)',drawbox=x=160:y=115:w=160:h=30:color=white:t=fill:enable='between(t,1.5,2.399)',drawbox=x=140:y=110:w=200:h=40:color=white:t=fill:enable='between(t,2.4,4.999)',drawbox=x=150:y=105:w=180:h=25:color=cyan:t=fill:enable='between(t,5,6.999)',drawbox=x=150:y=145:w=180:h=25:color=cyan:t=fill:enable='between(t,5.5,6.999)',drawbox=x=150:y=116:w=180:h=25:color=orange:t=fill:enable='between(t,7,8.999)',drawbox=x=150:y=116:w=180:h=25:color=white:t=fill:enable='gte(t,9)'"
if (check_text_render_contract text-animation "$TMP_DIR/text-animation" "$TMP_DIR/no-highlight-animation.mp4"); then
  fail "text animation without highlight progress should fail"
fi

mutate_video "$TMP_DIR/text-animation/rendered/preview.mp4" \
  "$TMP_DIR/no-line-stagger.mp4" \
  "drawbox=x=150:y=105:w=180:h=65:color=cyan:t=fill:enable='between(t,5,5.35)'"
expect_animation_failure "$TMP_DIR/no-line-stagger.mp4" \
  "line stagger is not visible" "text animation without line stagger"

mutate_video "$TMP_DIR/text-animation/rendered/preview.mp4" \
  "$TMP_DIR/no-word-stagger.mp4" \
  "drawbox=x=150:y=116:w=180:h=25:color=orange:t=fill:enable='between(t,7,8.2)'"
expect_animation_failure "$TMP_DIR/no-word-stagger.mp4" \
  "four-word stagger does not progress" "text animation without word stagger"

mutate_video "$TMP_DIR/text-animation/rendered/preview.mp4" \
  "$TMP_DIR/no-grapheme-reveal.mp4" \
  "drawbox=x=150:y=116:w=180:h=25:color=white:t=fill:enable='between(t,9,10.7)'"
expect_animation_failure "$TMP_DIR/no-grapheme-reveal.mp4" \
  "six-grapheme reveal does not progress" "text animation without grapheme reveal"

make_video "$TMP_DIR/no-shadow-overlay.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x1b263b:t=fill,drawbox=x=150:y=121:w=180:h=28:color=black@0.6:t=fill,drawbox=x=153:y=124:w=174:h=22:color=cyan:t=fill,drawbox=x=155:y=126:w=170:h=18:color=white:t=fill"
if (check_text_render_contract text-overlay "$TMP_DIR/text-overlay" "$TMP_DIR/no-shadow-overlay.mp4"); then
  fail "text overlay without a right-down shadow should fail"
fi

make_video "$TMP_DIR/no-down-shadow-overlay.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x1b263b:t=fill,drawbox=x=150:y=121:w=180:h=28:color=black@0.6:t=fill,drawbox=x=158:y=126:w=180:h=18:color=0xff477e@0.8:t=fill,drawbox=x=153:y=124:w=174:h=22:color=cyan:t=fill,drawbox=x=155:y=126:w=170:h=18:color=white:t=fill"
if (check_text_render_contract text-overlay "$TMP_DIR/text-overlay" "$TMP_DIR/no-down-shadow-overlay.mp4"); then
  fail "text overlay without a downward shadow should fail"
fi

make_video "$TMP_DIR/no-right-shadow-overlay.mp4" 4 480x270 \
  "drawbox=x=0:y=0:w=480:h=270:color=0x1b263b:t=fill,drawbox=x=150:y=121:w=180:h=28:color=black@0.6:t=fill,drawbox=x=150:y=128:w=180:h=28:color=0xff477e@0.8:t=fill,drawbox=x=153:y=124:w=174:h=22:color=cyan:t=fill,drawbox=x=155:y=126:w=170:h=18:color=white:t=fill"
if (check_text_render_contract text-overlay "$TMP_DIR/text-overlay" "$TMP_DIR/no-right-shadow-overlay.mp4"); then
  fail "text overlay without a rightward shadow should fail"
fi

make_video "$TMP_DIR/short-hello.mp4" 3 480x270 \
  "drawbox=x=0:y=0:w=480:h=135:color=0x143642:t=fill,drawbox=x=0:y=135:w=480:h=135:color=0x0f1a20:t=fill,drawbox=x=155:y=120:w=170:h=25:color=white:t=fill"
if (check_text_render_contract hello-world "$TMP_DIR/hello-world" "$TMP_DIR/short-hello.mp4"); then
  fail "wrong hello-world duration should fail the media contract"
fi

echo "text render evidence contract tests passed"
