#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/veac-agentsmesh-contracts.XXXXXX")
trap 'rm -rf "$TMP_DIR"' EXIT

for module in common pixels showcase-pixels text text-agentsmesh-intro; do
  # shellcheck source=/dev/null
  source "$ROOT_DIR/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/tests/agentsmesh-intro-render-fixtures.sh"

clone_fixture() {
  local root=$1
  mkdir -p "$root"
  cp -R "$VALID/agentsmesh-intro-15s" "$root/agentsmesh-intro-15s"
}

rewrite_json() {
  local file=$1 filter=$2
  jq "$filter" "$file" >"$file.tmp"
  mv "$file.tmp" "$file"
}

expect_failure() {
  local label=$1 root=$2 expected=$3
  local log="$TMP_DIR/$label.log"
  PREVIEW_ROOT=$root
  if (check_agentsmesh_intro_evidence) >"$log" 2>&1; then
    echo "$label unexpectedly passed" >&2
    exit 1
  fi
  rg -qF "$expected" "$log" || { cat "$log" >&2; echo "$label failed incorrectly" >&2; exit 1; }
}

video_variant() {
  local label=$1 mode=$2 expected=$3 root="$TMP_DIR/$1"
  clone_fixture "$root"
  make_agentsmesh_video "$root/agentsmesh-intro-15s" "$mode"
  expect_failure "$label" "$root" "$expected"
}

json_variant() {
  local label=$1 relative=$2 filter=$3 expected=$4 root="$TMP_DIR/$1"
  clone_fixture "$root"
  rewrite_json "$root/agentsmesh-intro-15s/$relative" "$filter"
  expect_failure "$label" "$root" "$expected"
}

VALID="$TMP_DIR/valid"
make_agentsmesh_fixture "$VALID"
export PREVIEW_ROOT=$VALID
check_agentsmesh_intro_evidence

json_variant wrong_stagger project/project.veac.json \
  '(.. | objects | select(.id? == "itm_title") | .source.style.animation.stagger.value) = 90' \
  'authoring contract failed'
json_variant wrong_title_range project/project.preview.veac.json \
  '(.. | objects | select(.id? == "itm_title") | .record_range.duration.value) = 6000' \
  'preview contract failed'
json_variant wrong_plan_duration plans/preview/out_preview.json \
  '(.sequences[] | select(.id == "seq_main") | .duration.value) = 12000' \
  'preview plan contract failed'
json_variant wrong_plan_granularity plans/preview/out_preview.json \
  '(.. | objects | select(.id? == "itm_title") | .source.content.presentation.style.animation.granularity) = "word"' \
  'preview plan contract failed'
json_variant opaque_panel project/project.veac.json \
  '(.. | objects | select(.id? == "itm_caption") | .source.style.background.color.alpha) = 255' \
  'authoring contract failed'
json_variant missing_padding project/project.veac.json \
  '(.. | objects | select(.id? == "itm_caption") | .source.style.background.padding_pixels) = 0' \
  'authoring contract failed'

video_variant static_full_title static 'grapheme reveal does not progress'
video_variant reveal_without_scale no-scale 'title scale is not visible'
video_variant transform_without_rise no-rise 'does not rise into position'
video_variant incomplete_at_1170ms incomplete 'not complete and stable by 1.17s'
video_variant title_promise_overlap overlap 'switch is not exclusive at 5s'
video_variant missing_promise no-promise 'switch is not exclusive at 5s'
video_variant off_center_promise off-center-promise 'promise is not centered at 5s'
video_variant missing_caption no-caption 'bottom caption is missing'
video_variant off_center_caption off-center-caption 'bottom caption is not centered'
video_variant missing_panel no-panel 'panel opacity or padding is not visible'
video_variant caption_after_12s late-caption 'does not honor the 2-12s range'
video_variant solid_background solid 'diagonal blue-green gradient is missing'
video_variant wrong_duration short 'invalid video contract'

echo "agentsmesh intro render evidence contract tests passed"
