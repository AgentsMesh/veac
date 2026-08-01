#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/veac-all-features-contracts.XXXXXX")
trap 'rm -rf "$TMP_DIR"' EXIT

for module in common pixels showcase-pixels audio-metrics showcase-all-features audio-all-features; do
  # shellcheck source=/dev/null
  source "$ROOT_DIR/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT_DIR/scripts/tests/all-features-render-fixtures.sh"

clone_fixture() {
  local root=$1
  mkdir -p "$root"
  cp -R "$VALID/all-features" "$root/all-features"
}

rewrite_json() {
  local file=$1 filter=$2
  jq "$filter" "$file" >"$file.tmp"
  mv "$file.tmp" "$file"
}

expect_failure() {
  local label=$1 checker=$2 root=$3 expected=$4
  local log="$TMP_DIR/$label.log"
  PREVIEW_ROOT=$root
  if ("$checker") >"$log" 2>&1; then
    echo "$label unexpectedly passed" >&2
    exit 1
  fi
  rg -qF "$expected" "$log" || { cat "$log" >&2; echo "$label failed incorrectly" >&2; exit 1; }
}

visual_variant() {
  local label=$1 mode=$2 expected=$3 root="$TMP_DIR/$1"
  clone_fixture "$root"
  make_all_features_video "$root/all-features" "$mode"
  expect_failure "$label" check_all_features_visual_evidence "$root" "$expected"
}

audio_variant() {
  local label=$1 mode=$2 expected=$3 root="$TMP_DIR/$1"
  clone_fixture "$root"
  make_all_features_audio_variant "$root/all-features" "$mode"
  expect_failure "$label" check_all_features_audio_evidence "$root" "$expected"
}

json_variant() {
  local label=$1 relative=$2 filter=$3 checker=$4 expected=$5 root="$TMP_DIR/$1"
  clone_fixture "$root"
  rewrite_json "$root/all-features/$relative" "$filter"
  expect_failure "$label" "$checker" "$root" "$expected"
}

VALID="$TMP_DIR/valid"
make_all_features_fixture "$VALID"
export PREVIEW_ROOT=$VALID
check_all_features_visual_evidence
check_all_features_audio_evidence

json_variant author_truncated project/project.veac.json \
  '(.. | objects | select(.id? == "itm_shot") | .record_range.duration.value) = 4000' \
  check_all_features_visual_evidence 'authoring visual contract failed'
json_variant preview_truncated project/project.preview.veac.json \
  '(.. | objects | select(.id? == "itm_shot") | .record_range.duration.value) = 4000' \
  check_all_features_visual_evidence 'preview visual contract failed'
json_variant plan_truncated plans/preview/out_master.json \
  '(.sequences[] | select(.id == "seq_main") | .duration.value) = 4000' \
  check_all_features_visual_evidence 'preview plan visual contract failed'
json_variant missing_group project/project.veac.json \
  'del(.project.relations[] | select(.id == "rel_edit-unit"))' \
  check_all_features_visual_evidence 'authoring visual contract failed'
json_variant missing_av_link project/project.veac.json \
  'del(.project.relations[] | select(.id == "rel_linked-av"))' \
  check_all_features_visual_evidence 'authoring visual contract failed'
json_variant missing_caption_background project/project.veac.json \
  'del(.. | objects | select(.id? == "itm_cue") | .source.style.background)' \
  check_all_features_visual_evidence 'authoring visual contract failed'
json_variant missing_preview_caption_background project/project.preview.veac.json \
  'del(.. | objects | select(.id? == "itm_cue") | .source.style.background)' \
  check_all_features_visual_evidence 'preview visual contract failed'
json_variant missing_plan_caption_background plans/preview/out_master.json \
  'del(.. | objects | select(.id? == "itm_cue") | .source.content.presentation.style.background)' \
  check_all_features_visual_evidence 'preview plan visual contract failed'
json_variant canonical_caption_shape_in_plan plans/preview/out_master.json \
  '(.. | objects | select(.id? == "itm_cue") | .source) |=
    {type:"caption",text:.content.text,style:.content.presentation.style}' \
  check_all_features_visual_evidence 'preview plan visual contract failed'

visual_variant wrong_source_mapping wrong-mapping 'mapping at 0.5s'
visual_variant neutral_rendered_grade neutral-grade 'rendered grade delta is invalid'
visual_variant panel_always_visible always-panel 'lower-third lifecycle failed'
visual_variant panel_ends_early early-panel 'lower-third lifecycle failed'
visual_variant missing_burned_caption no-caption 'caption missing'
visual_variant discontinuous_caption_background late-background 'caption contrast is not continuous'
visual_variant four_second_media truncated 'invalid video contract'

json_variant missing_normalize project/project.veac.json \
  '(.. | objects | select(.id? == "itm_music-item") | .audio.normalize) = false' \
  check_all_features_audio_evidence 'authoring audio-chain contract failed'
json_variant missing_limiter project/project.veac.json \
  '(.. | objects | select(.id? == "itm_music-item") | .audio.processors) = []' \
  check_all_features_audio_evidence 'authoring audio-chain contract failed'
json_variant wrong_plan_fade plans/preview/out_master.json \
  '(.. | objects | select(.id? == "itm_music-item") | .audio.crossfade.fade_out.value) = 0' \
  check_all_features_audio_evidence 'plan audio-chain contract failed'

audio_variant missing_fade_in no-fade-in 'fade-in is missing'
audio_variant missing_fade_out no-fade-out 'fade-out is missing'
audio_variant silent_mix silence 'normalized music'
audio_variant wrong_audio_format wrong-format 'expected sample_rate=48000'
audio_variant unnormalized_mix unnormalized 'normalization and limiter'

echo "all-features render evidence contract tests passed"
