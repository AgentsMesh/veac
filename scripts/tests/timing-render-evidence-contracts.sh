#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
SCRIPT_DIR="$ROOT/scripts"
source "$ROOT/scripts/tests/example-preview-layout-fixtures.sh"
# shellcheck source=../example-render-evidence/common.sh
source "$SCRIPT_DIR/example-render-evidence/common.sh"
# shellcheck source=../example-render-evidence/timing.sh
source "$SCRIPT_DIR/example-render-evidence/timing.sh"

tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/veac-timing-contracts.XXXXXX")
trap 'rm -rf "$tmp_dir"' EXIT
video="$tmp_dir/regions.mkv"
source_video="$tmp_dir/source.mkv"
empty_preview="$tmp_dir/empty-preview"

ffmpeg -nostdin -hide_banner -loglevel error \
  -f lavfi -i 'color=c=red:s=80x90:r=10:d=2' \
  -f lavfi -i 'color=c=blue:s=80x90:r=10:d=2' \
  -filter_complex '[0:v][1:v]hstack=inputs=2' -c:v ffv1 "$video"
ffmpeg -nostdin -hide_banner -loglevel error \
  -f lavfi -i 'color=c=red:s=160x90:r=10:d=1' \
  -f lavfi -i 'color=c=blue:s=160x90:r=10:d=1' \
  -filter_complex '[0:v][1:v]concat=n=2:v=1:a=0' -c:v ffv1 "$source_video"
mkdir -p "$empty_preview"

mapped_example="$tmp_dir/mapped-example"
prepare_preview_fixture_dirs "$mapped_example"
cp "$video" "$mapped_example/rendered/actual-main.mkv"
cat >"$mapped_example/project/project.veac.json" <<'JSON'
{"project":{"render_configs":[{"id":"out_main","deliverables":[{"id":"dlv_main","target":{"type":"file","name":"actual-main.mkv"},"kind":{"type":"video"}},{"id":"dlv_captions","target":{"type":"file","name":"captions.vtt"},"kind":{"type":"caption_sidecar"}}]}]}}
JSON
mirror_fixture_preview_canonical "$mapped_example"
resolved_video=$(timing_video "$mapped_example" main main)
[[ $resolved_video == "$mapped_example/rendered/actual-main.mkv" ]] ||
  fail 'authoring output did not resolve its canonical video file name'

unsafe_example="$tmp_dir/unsafe-example"
cp -R "$mapped_example" "$unsafe_example"
jq '(.project.render_configs[] | select(.id == "out_main") |
  .deliverables[] | select(.id == "dlv_main") | .target.name) = "../escape.mkv"' \
  "$unsafe_example/project/project.preview.veac.json" >"$unsafe_example/project.tmp"
mv "$unsafe_example/project.tmp" "$unsafe_example/project/project.preview.veac.json"
if (timing_video "$unsafe_example" main main >/dev/null 2>&1); then
  fail 'unsafe video deliverable basename was accepted'
fi

ambiguous_example="$tmp_dir/ambiguous-example"
cp -R "$mapped_example" "$ambiguous_example"
jq '(.project.render_configs[] | select(.id == "out_main") | .deliverables) +=
  [.project.render_configs[] | select(.id == "out_main") |
    .deliverables[] | select(.id == "dlv_main")]' \
  "$ambiguous_example/project/project.preview.veac.json" >"$ambiguous_example/project.tmp"
mv "$ambiguous_example/project.tmp" "$ambiguous_example/project/project.preview.veac.json"
if (timing_video "$ambiguous_example" main main >/dev/null 2>&1); then
  fail 'ambiguous video deliverables were accepted'
fi

assert_rgb_near_at 'red half is sampled at record time' "$video" 0.25 \
  'iw/2:ih:0:0' '255 0 0' 2
assert_rgb_near_at 'blue half is sampled at record time' "$video" 0.25 \
  'iw/2:ih:iw/2:0' '0 0 255' 2
assert_channel_gap_at 'red channel separates the two regions' "$video" 0.25 \
  'iw/2:ih:0:0' r 'iw/2:ih:iw/2:0' r 200
assert_rgb_distance_at_least 'different regions have a measurable RGB distance' \
  "$video" 0.25 'iw/2:ih:0:0' 'iw/2:ih:iw/2:0' 480
assert_rgb_distance_at_most 'the same region has zero RGB distance' \
  "$video" 0.25 'iw/2:ih:0:0' 'iw/2:ih:0:0' 0

assert_psnr_preferred 'early record time maps to early source time' \
  "$source_video" 0.5 "$source_video" 0.5 1.5 10
assert_psnr_preferred 'later record time maps to later source time' \
  "$source_video" 1.5 "$source_video" 1.5 0.5 10
assert_frame_psnr_at_least 'equal source-time frames remain equivalent' \
  "$source_video" 1.5 "$source_video" 1.5 90

first_score=$(timing_frame_psnr "$source_video" 1.5 "$source_video" 1.5)
second_score=$(timing_frame_psnr "$source_video" 1.5 "$source_video" 1.5)
[[ $first_score == "$second_score" ]] || fail 'PSNR sampling must be deterministic'

VEAC_EXAMPLES=
assert_selected_timing_directories "$empty_preview"
if timing_selected speed-demo; then
  fail 'an empty selector must not select every timing example'
fi
VEAC_EXAMPLES=all
timing_selected speed-demo || fail 'all must select every timing example'
VEAC_EXAMPLES='speed-demo, transition-gallery'
timing_selected speed-demo || fail 'comma-separated selector missed speed-demo'
timing_selected transition-gallery || fail 'selector missed transition-gallery'
if timing_selected speed; then
  fail 'selector membership must be exact'
fi
VEAC_EXAMPLES=speed-demo
export VEAC_EXAMPLES
if (assert_selected_timing_directories "$empty_preview" >/dev/null 2>&1); then
  fail 'an explicitly selected missing timing example must fail'
fi

if (assert_rgb_near_at 'wrong color must fail' "$video" 0.25 \
  'iw/2:ih:0:0' '0 0 255' 2 >/dev/null 2>&1); then
  fail 'RGB assertion accepted an incorrect color'
fi
if (assert_psnr_preferred 'wrong source-time mapping must fail' \
  "$source_video" 0.5 "$source_video" 1.5 0.5 5 >/dev/null 2>&1); then
  fail 'PSNR assertion accepted an incorrect source-time mapping'
fi
if (assert_rgb_distance_at_most 'different regions must fail similarity' \
  "$video" 0.25 'iw/2:ih:0:0' 'iw/2:ih:iw/2:0' 10 >/dev/null 2>&1); then
  fail 'RGB distance assertion accepted visibly different regions'
fi

printf 'timing render evidence contract tests passed\n'
bash "$ROOT/scripts/tests/transform-animation-render-evidence-contracts.sh"
