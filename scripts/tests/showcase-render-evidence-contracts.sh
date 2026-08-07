#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
source "$ROOT_DIR/scripts/tests/example-preview-layout-fixtures.sh"
TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/veac-showcase-contracts.XXXXXX")
trap 'rm -rf "$TMP_DIR"' EXIT

for module in common pixels showcase-pixels showcase-layout; do
  # shellcheck source=/dev/null
  source "$ROOT_DIR/scripts/example-render-evidence/$module.sh"
done

make_video() {
  local path=$1 duration=$2 filter=$3
  ffmpeg -v error -y -f lavfi -i "color=c=0x20262e:s=480x270:r=12:d=$duration" \
    -vf "$filter" -c:v libx264 -pix_fmt yuv420p "$path"
}

expect_failure() {
  local label=$1 function=$2 root=$3
  PREVIEW_ROOT=$root
  if ("$function" >/dev/null 2>&1); then
    fail "$label negative contract unexpectedly passed"
  fi
}

make_card() {
  local root=$1 dir="$1/card-overlay"
  prepare_preview_fixture_dirs "$dir"
  make_video "$dir/rendered/preview.mp4" 4 \
    "drawbox=x=0:y=0:w=iw:h=ih:color=0x10182a:t=fill,drawbox=x=95:y=207:w=290:h=18:color=black@0.35:t=fill,drawbox=x=105:y=64:w=270:h=143:color=0x5bc0be:t=fill,drawbox=x=107:y=66:w=266:h=139:color=0xe5e6e5:t=fill,drawbox=x=105:y=64:w=9:h=9:color=0x10182a:t=fill,drawbox=x=366:y=64:w=9:h=9:color=0x10182a:t=fill,drawbox=x=105:y=198:w=9:h=9:color=0x10182a:t=fill,drawbox=x=366:y=198:w=9:h=9:color=0x10182a:t=fill"
  cat >"$dir/project/project.veac.json" <<'JSON'
{"project":{"sequences":[{"tracks":[{"clips":[{"id":"generated-card","authorship":{"logical_path":["card-overlay","main","card","panel"],"events":[]},"source":{"generator":{"shape":{"geometry":{"type":"rectangle"},"stroke":{"width_pixels":14}}}},"visual":{"card":{"corner_radius_pixels":36,"shadow":{"blur_pixels":28,"color":{"alpha":255,"blue":0,"green":0,"red":0},"offset":{"x":0,"y":16},"opacity":0.38}},"frame":{"fit":"fill","height":{"unit":"pixels","value":380},"width":{"unit":"pixels","value":720}},"placement":{"anchor":"center","inset":{"x":0,"y":0},"type":"anchor"},"opacity":{"type":"constant","value":0.92}}}]}]}]}}
JSON
  mirror_fixture_preview_canonical "$dir"
}

make_template() {
  local root=$1 dir="$1/template-fill"
  prepare_preview_fixture_dirs "$dir"
  ffmpeg -v error -y -f lavfi -i 'testsrc2=size=480x270:rate=12:duration=4' \
    -vf 'drawbox=x=150:y=115:w=180:h=40:color=white:t=fill' \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
  cat >"$dir/project/project.veac.json" <<'JSON'
{"project":{"sequences":[{"tracks":[{"clips":[{"id":"generated-hero","authorship":{"logical_path":["template-fill","main","media-slots","hero"],"events":[]},"replaceable":{"fill":"fit_duration","kind":"video","label":"主视觉媒体","min_source_duration":{"timescale":1000,"value":2000}}},{"id":"generated-title","authorship":{"logical_path":["template-fill","main","text-slots","title"],"events":[]},"replaceable":{"fill":"fit_duration","kind":"text","label":"主标题文本","min_source_duration":null},"template_editable_text":true,"source":{"text":"可替换标题"}}]}]}]}}
JSON
  mirror_fixture_preview_canonical "$dir"
}

make_all_features() {
  local root=$1 full=${2:-false} dir="$1/all-features" panel
  prepare_preview_fixture_dirs "$dir"
  if [[ $full == true ]]; then panel='x=0:y=0:w=iw:h=ih'; else panel='x=60:y=175:w=360:h=60'; fi
  ffmpeg -v error -y -f lavfi -i 'testsrc2=size=480x270:rate=12:duration=4' \
    -vf "drawbox=$panel:color=0x03045e:t=fill:enable='gte(t,1)',drawbox=x=130:y=205:w=220:h=15:color=white:t=fill" \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/all-features.mp4"
  cat >"$dir/project/project.veac.json" <<'JSON'
{"project":{"authorship":{"entity":{"logical_path":["all-features"],"events":[]},"multicam_groups":[],"annotations":[],"deliveries":[{"render_config_id":"generated-master","entity":{"logical_path":["all-features","master"],"events":[]}}]},"sequences":[{"tracks":[{"clips":[{"id":"generated-cue","authorship":{"logical_path":["all-features","main","subtitles","cue"],"events":[]},"source":{"text":"一种语言，一份类型化中间表示，一套渲染计划。"}},{"id":"generated-lower-third","authorship":{"logical_path":["all-features","main","graphics","lower-third"],"events":[]},"visual":{"frame":{"height":{"unit":"pixels","value":180}}}}]}]}],"relations":[{"kind":{"type":"group"}},{"kind":{"type":"av_link"}}],"render_configs":[{"id":"generated-master","raster":{"width":480,"height":270,"frame_rate":{"numerator":12,"denominator":1},"captions":"burn_in"},"deliverables":[{"id":"generated-video","target":{"type":"file","name":"all-features.mp4"},"kind":{"type":"video"}}]}]}}
JSON
  mirror_fixture_preview_canonical "$dir"
}

VALID="$TMP_DIR/valid"
make_card "$VALID"
make_template "$VALID"
make_all_features "$VALID"
export PREVIEW_ROOT=$VALID
check_card_overlay_evidence
check_template_fill_evidence
check_all_features_evidence

BAD_CARD="$TMP_DIR/bad-card"
mkdir -p "$BAD_CARD"
cp -R "$VALID/card-overlay" "$BAD_CARD/card-overlay"
make_video "$BAD_CARD/card-overlay/rendered/preview.mp4" 4 'null'
expect_failure clipped_card check_card_overlay_evidence "$BAD_CARD"

OPAQUE_CARD="$TMP_DIR/opaque-card"
mkdir -p "$OPAQUE_CARD"
cp -R "$VALID/card-overlay" "$OPAQUE_CARD/card-overlay"
make_video "$OPAQUE_CARD/card-overlay/rendered/preview.mp4" 4 \
  'drawbox=x=0:y=0:w=iw:h=ih:color=0x10182a:t=fill,drawbox=x=95:y=207:w=290:h=18:color=black@0.35:t=fill,drawbox=x=105:y=64:w=270:h=143:color=0x5bc0be:t=fill,drawbox=x=107:y=66:w=266:h=139:color=0xf7f7f2:t=fill,drawbox=x=105:y=64:w=9:h=9:color=0x10182a:t=fill,drawbox=x=366:y=64:w=9:h=9:color=0x10182a:t=fill,drawbox=x=105:y=198:w=9:h=9:color=0x10182a:t=fill,drawbox=x=366:y=198:w=9:h=9:color=0x10182a:t=fill'
expect_failure opaque_card check_card_overlay_evidence "$OPAQUE_CARD"

SQUARE_CARD="$TMP_DIR/square-card"
mkdir -p "$SQUARE_CARD"
cp -R "$VALID/card-overlay" "$SQUARE_CARD/card-overlay"
make_video "$SQUARE_CARD/card-overlay/rendered/preview.mp4" 4 \
  'drawbox=x=0:y=0:w=iw:h=ih:color=0x10182a:t=fill,drawbox=x=95:y=207:w=290:h=18:color=black@0.35:t=fill,drawbox=x=105:y=64:w=270:h=143:color=0x5bc0be:t=fill,drawbox=x=107:y=66:w=266:h=139:color=0xe5e6e5:t=fill'
expect_failure square_card check_card_overlay_evidence "$SQUARE_CARD"

NO_SHADOW="$TMP_DIR/no-shadow"
mkdir -p "$NO_SHADOW"
cp -R "$VALID/card-overlay" "$NO_SHADOW/card-overlay"
make_video "$NO_SHADOW/card-overlay/rendered/preview.mp4" 4 \
  'drawbox=x=0:y=0:w=iw:h=ih:color=0x10182a:t=fill,drawbox=x=105:y=64:w=270:h=143:color=0x5bc0be:t=fill,drawbox=x=107:y=66:w=266:h=139:color=0xe5e6e5:t=fill,drawbox=x=105:y=64:w=9:h=9:color=0x10182a:t=fill,drawbox=x=366:y=64:w=9:h=9:color=0x10182a:t=fill,drawbox=x=105:y=198:w=9:h=9:color=0x10182a:t=fill,drawbox=x=366:y=198:w=9:h=9:color=0x10182a:t=fill'
expect_failure missing_card_shadow check_card_overlay_evidence "$NO_SHADOW"

BAD_TEMPLATE="$TMP_DIR/bad-template"
mkdir -p "$BAD_TEMPLATE"
cp -R "$VALID/template-fill" "$BAD_TEMPLATE/template-fill"
make_video "$BAD_TEMPLATE/template-fill/rendered/preview.mp4" 4 \
  'drawbox=x=150:y=115:w=180:h=40:color=white:t=fill'
expect_failure static_template check_template_fill_evidence "$BAD_TEMPLATE"

BAD_TEMPLATE_DURATION="$TMP_DIR/bad-template-duration"
mkdir -p "$BAD_TEMPLATE_DURATION"
cp -R "$VALID/template-fill" "$BAD_TEMPLATE_DURATION/template-fill"
jq '(.project.sequences[].tracks[].clips[] |
  select(.authorship.logical_path[-1] == "hero").replaceable.min_source_duration.value) = 1000' \
  "$BAD_TEMPLATE_DURATION/template-fill/project/project.veac.json" >"$BAD_TEMPLATE_DURATION/edit.json"
mv "$BAD_TEMPLATE_DURATION/edit.json" \
  "$BAD_TEMPLATE_DURATION/template-fill/project/project.veac.json"
expect_failure wrong_template_duration check_template_fill_evidence "$BAD_TEMPLATE_DURATION"

UNTITLED_TEMPLATE="$TMP_DIR/untitled-template"
mkdir -p "$UNTITLED_TEMPLATE"
cp -R "$VALID/template-fill" "$UNTITLED_TEMPLATE/template-fill"
ffmpeg -v error -y -f lavfi -i 'testsrc2=size=480x270:rate=12:duration=4' \
  -c:v libx264 -pix_fmt yuv420p "$UNTITLED_TEMPLATE/template-fill/rendered/preview.mp4"
expect_failure missing_template_title check_template_fill_evidence "$UNTITLED_TEMPLATE"

BAD_ALL="$TMP_DIR/bad-all"
make_all_features "$BAD_ALL" true
expect_failure fullscreen_lower_third check_all_features_evidence "$BAD_ALL"

BAD_CAPTION="$TMP_DIR/bad-caption"
make_all_features "$BAD_CAPTION"
ffmpeg -v error -y -f lavfi -i 'testsrc2=size=480x270:rate=12:duration=4' \
  -vf "drawbox=x=60:y=175:w=360:h=60:color=0x03045e:t=fill:enable='gte(t,1)'" \
  -c:v libx264 -pix_fmt yuv420p \
  "$BAD_CAPTION/all-features/rendered/all-features.mp4"
expect_failure missing_burned_caption check_all_features_evidence "$BAD_CAPTION"

bash "$ROOT_DIR/scripts/tests/visual-mechanism-render-evidence-contracts.sh"
echo "showcase render evidence contract tests passed"
