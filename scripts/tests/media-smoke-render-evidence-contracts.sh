#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
source "$ROOT/scripts/example-render-evidence/common.sh"
source "$ROOT/scripts/example-render-evidence/media-smoke-path.sh"
source "$ROOT/scripts/example-render-evidence/media-smoke-roster.sh"
source "$ROOT/scripts/example-render-evidence/media-smoke-container.sh"
source "$ROOT/scripts/example-render-evidence/media-smoke-timing.sh"
source "$ROOT/scripts/example-render-evidence/media-smoke-probe.sh"
source "$ROOT/scripts/example-render-evidence/media-smoke.sh"
source "$ROOT/scripts/tests/media-smoke-fixtures.sh"
source "$ROOT/scripts/tests/media-smoke-boundary-contracts.sh"

TEMP_DIR=$(mktemp -d)
TEMP_DIR=$(cd "$TEMP_DIR" && pwd -P)
trap 'rm -rf "$TEMP_DIR"' EXIT
AUDIO='{"codec":"aac","sample_rate":48000,"channels":2}'

expect_failure() {
  local label=$1 dir=$2
  if (check_example_media_smoke "$dir") >/dev/null 2>&1; then
    echo "expected media smoke failure: $label" >&2
    exit 1
  fi
}

valid="$TEMP_DIR/valid"
write_smoke_project "$valid" null
make_smoke_video "$valid/rendered/preview.mp4" '#203040'
check_example_media_smoke "$valid"

valid_webm="$TEMP_DIR/valid-webm"
write_smoke_project "$valid_webm" "$AUDIO"
jq '(.project.render_configs[0].deliverables[0]) |=
  (.target.name="preview.webm" | .kind.settings.container="webm" |
   .kind.settings.video.codec="vp9" | .kind.settings.audio.codec="opus")' \
  "$valid_webm/project/project.preview.veac.json" >"$valid_webm/project.tmp"
mv "$valid_webm/project.tmp" "$valid_webm/project/project.preview.veac.json"
ffmpeg -v error -y -f lavfi -i 'color=c=#203040:s=96x54:r=12:d=1' \
  -f lavfi -i 'sine=frequency=440:sample_rate=48000:duration=1' \
  -c:v libvpx-vp9 -pix_fmt yuv420p -c:a libopus -ac 2 "$valid_webm/rendered/preview.webm"
check_example_media_smoke "$valid_webm"

for stage in opening middle ending; do
  dir="$TEMP_DIR/black-$stage"
  write_smoke_project "$dir" null
  case $stage in
    opening) filter="drawbox=color=#203040:t=fill:enable='gte(t,.25)'" ;;
    middle) filter="drawbox=color=#203040:t=fill:enable='lt(t,.35)+gte(t,.65)'" ;;
    ending) filter="drawbox=color=#203040:t=fill:enable='lt(t,.75)'" ;;
  esac
  make_smoke_visibility_video "$dir/rendered/preview.mp4" "$filter"
  expect_failure "black-$stage" "$dir"
done

unexpected_audio="$TEMP_DIR/unexpected-audio"
write_smoke_project "$unexpected_audio" null
make_smoke_video "$unexpected_audio/rendered/preview.mp4" '#203040' aac
expect_failure unexpected-audio "$unexpected_audio"

missing_audio="$TEMP_DIR/missing-audio"
write_smoke_project "$missing_audio" "$AUDIO"
make_smoke_video "$missing_audio/rendered/preview.mp4" '#203040'
expect_failure missing-audio "$missing_audio"

for field in codec sample_rate channels; do
  dir="$TEMP_DIR/wrong-$field"
  write_smoke_project "$dir" "$AUDIO"
  make_smoke_video "$dir/rendered/preview.mp4" '#203040' aac
  case $field in
    codec) jq '.project.render_configs[0].deliverables[0].kind.settings.audio.codec="opus"' ;;
    sample_rate) jq '.project.render_configs[0].deliverables[0].kind.settings.audio.sample_rate=44100' ;;
    channels) jq '.project.render_configs[0].deliverables[0].kind.settings.audio.channels=1' ;;
  esac < "$dir/project/project.preview.veac.json" > "$dir/project.tmp"
  mv "$dir/project.tmp" "$dir/project/project.preview.veac.json"
  expect_failure "wrong-$field" "$dir"
done

short_audio="$TEMP_DIR/short-audio"
write_smoke_project "$short_audio" "$AUDIO"
make_smoke_video "$short_audio/rendered/preview.mp4" '#203040' short-aac
expect_failure short-audio "$short_audio"

short_video="$TEMP_DIR/short-video"
write_smoke_project "$short_video" "$AUDIO"
make_smoke_video "$short_video/rendered/preview.mp4" '#203040' short-video
expect_failure short-video "$short_video"

for field in container video-codec pixel-format alpha; do
  dir="$TEMP_DIR/wrong-$field"
  write_smoke_project "$dir" null
  make_smoke_video "$dir/rendered/preview.mp4" '#203040'
  case $field in
    container) filter='.project.render_configs[0].deliverables[0].kind.settings.container="mkv"' ;;
    video-codec) filter='.project.render_configs[0].deliverables[0].kind.settings.video.codec="vp9"' ;;
    pixel-format) filter='.project.render_configs[0].deliverables[0].kind.settings.video.pixel_format="yuv422p"' ;;
    alpha) filter='.project.render_configs[0].deliverables[0].kind.settings.video.alpha="straight"' ;;
  esac
  jq "$filter" "$dir/project/project.preview.veac.json" > "$dir/project.tmp"
  mv "$dir/project.tmp" "$dir/project/project.preview.veac.json"
  expect_failure "wrong-$field" "$dir"
done

wrong_raster="$TEMP_DIR/wrong-raster"
write_smoke_project "$wrong_raster" null 128
make_smoke_video "$wrong_raster/rendered/preview.mp4" '#203040'
expect_failure wrong-raster "$wrong_raster"

wrong_rate="$TEMP_DIR/wrong-rate"
write_smoke_project "$wrong_rate" null 96 24
make_smoke_video "$wrong_rate/rendered/preview.mp4" '#203040'
expect_failure wrong-rate "$wrong_rate"

unsafe_config="$TEMP_DIR/unsafe-config"
write_smoke_project "$unsafe_config" null
make_smoke_video "$unsafe_config/rendered/preview.mp4" '#203040'
jq '.project.render_configs[0].id="../outside"' \
  "$unsafe_config/project/project.preview.veac.json" > "$unsafe_config/project.tmp"
mv "$unsafe_config/project.tmp" "$unsafe_config/project/project.preview.veac.json"
expect_failure unsafe-config "$unsafe_config"

string_raster="$TEMP_DIR/string-raster"
write_smoke_project "$string_raster" null
make_smoke_video "$string_raster/rendered/preview.mp4" '#203040'
jq '.project.render_configs[0].raster.width="96"' \
  "$string_raster/project/project.preview.veac.json" > "$string_raster/project.tmp"
mv "$string_raster/project.tmp" "$string_raster/project/project.preview.veac.json"
expect_failure string-raster "$string_raster"

truncated="$TEMP_DIR/truncated"
write_smoke_project "$truncated" null 96 12 2000
make_smoke_video "$truncated/rendered/preview.mp4" '#203040'
expect_failure truncated "$truncated"

corrupt="$TEMP_DIR/corrupt"
write_smoke_project "$corrupt" null
make_smoke_video "$corrupt/rendered/preview.mp4" '#203040'
truncate -s 512 "$corrupt/rendered/preview.mp4"
expect_failure corrupt "$corrupt"

linked="$TEMP_DIR/linked-canonical"
write_smoke_project "$linked" null
make_smoke_video "$linked/rendered/preview.mp4" '#203040'
mv "$linked/project/project.preview.veac.json" "$TEMP_DIR/project.json"
ln -s "$TEMP_DIR/project.json" "$linked/project/project.preview.veac.json"
expect_failure linked-canonical "$linked"

for ancestor in project rendered; do
  dir="$TEMP_DIR/linked-$ancestor"
  write_smoke_project "$dir" null
  make_smoke_video "$dir/rendered/preview.mp4" '#203040'
  mv "$dir/$ancestor" "$TEMP_DIR/$ancestor-tree"
  ln -s "$TEMP_DIR/$ancestor-tree" "$dir/$ancestor"
  expect_failure "linked-$ancestor" "$dir"
done

no_video="$TEMP_DIR/no-video"
mkdir -p "$no_video/project" "$no_video/rendered"
printf '%s\n' '{"project":{"render_configs":[]}}' > "$no_video/project/project.preview.veac.json"
expect_failure no-video "$no_video"

roster="$TEMP_DIR/roster"
mkdir -p "$roster/minimal"
VEAC_EXAMPLES=minimal assert_smoke_roster "$roster" "$ROOT/examples/catalog/gallery.json"
for selector in '' all; do
  if (VEAC_EXAMPLES=$selector \
      assert_smoke_roster "$roster" "$ROOT/examples/catalog/gallery.json") >/dev/null 2>&1; then
    fail "incomplete roster passed selector '${selector:-<empty>}'"
  fi
done
if (VEAC_EXAMPLES='minimal,hello-world' \
    assert_smoke_roster "$roster" "$ROOT/examples/catalog/gallery.json") >/dev/null 2>&1; then
  fail "missing selected example passed the roster contract"
fi
ln -s minimal "$roster/hello-world"
if (VEAC_EXAMPLES='minimal,hello-world' \
    assert_smoke_roster "$roster" "$ROOT/examples/catalog/gallery.json") >/dev/null 2>&1; then
  fail "symlink example directory passed the roster contract"
fi
rm "$roster/hello-world"
ln -s "$roster" "$TEMP_DIR/roster-link"
if (VEAC_EXAMPLES=minimal \
    assert_smoke_roster "$TEMP_DIR/roster-link/" "$ROOT/examples/catalog/gallery.json") \
    >/dev/null 2>&1; then
  fail "symlink preview root passed the roster contract"
fi

run_media_smoke_boundary_contracts "$TEMP_DIR" "$ROOT/examples/catalog/gallery.json"

echo "media smoke render evidence contract tests passed"
