#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/veac-codec-matrix-contracts.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
for module in common audio-metrics delivery-codec-matrix; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/delivery-codec-render-fixtures.sh"

profile_field_variant() {
  local stub_value=$1
  shift
  stream_field() { printf '%s\n' "$stub_value"; }
  assert_stream_field_one_of ignored v:0 profile fixture "$@"
}
for actual in 'Main 10' 'Main 10 Intra' 2; do
  (profile_field_variant "$actual" 'Main 10' 'Main 10 Intra' 2)
done
for actual in 'Profile 0' 0; do
  (profile_field_variant "$actual" 'Profile 0' 0)
done
if (profile_field_variant Main 'Main 10' 'Main 10 Intra' 2) >/dev/null 2>&1; then
  echo 'unexpected profile variant passed' >&2
  exit 1
fi

clone_fixture() { mkdir -p "$1"; cp -R "$VALID/delivery-codec-matrix" "$1/delivery-codec-matrix"; }
expect_failure() {
  local label=$1 root=$2 expected=$3 log="$TMP/$1.log"
  if (PREVIEW_ROOT=$root check_delivery_codec_matrix_evidence) >"$log" 2>&1; then
    echo "$label unexpectedly passed" >&2; exit 1
  fi
  rg -qF "$expected" "$log" || { cat "$log" >&2; echo "$label failed incorrectly" >&2; exit 1; }
}
json_variant() {
  local label=$1 relative=$2 filter=$3 expected=$4 root="$TMP/$1"
  clone_fixture "$root"; jq "$filter" "$root/delivery-codec-matrix/$relative" >"$root/value.json"
  mv "$root/value.json" "$root/delivery-codec-matrix/$relative"; expect_failure "$label" "$root" "$expected"
}
media_variant() {
  local label=$1 artifact=$2 mode=$3 expected=$4 root="$TMP/$1" file
  clone_fixture "$root"; file="$root/delivery-codec-matrix/rendered/$artifact"
  if [[ $artifact == hevc.mov ]]; then make_hevc_artifact "$file" "$mode"; else make_vp9_artifact "$file" "$mode"; fi
  expect_failure "$label" "$root" "$expected"
}

VALID="$TMP/valid"
make_codec_fixture "$VALID"
PREVIEW_ROOT=$VALID check_delivery_codec_matrix_evidence

json_variant wrong_author_profile project/project.veac.json \
  '(.. | objects | select(.id? == "dlv_hevc-main10") | .kind.settings.video.profile) = "h265_main"' \
  'authoring recipe contract failed'
json_variant missing_plan_deliverable plans/preview/out_codec-matrix.json \
  'del(.output.deliverables[] | select(.id == "dlv_vp9-opus"))' 'preview plan contract failed'

media_variant hevc_eight_bit hevc.mov eight-bit 'expected profile=Main 10'
media_variant hevc_wrong_color hevc.mov wrong-color 'expected color_primaries=bt2020'
media_variant hevc_low_levels hevc.mov low-levels 'distinct 10-bit luma levels'
media_variant hevc_wrong_size hevc.mov wrong-size 'expected width=480'
media_variant hevc_wrong_duration hevc.mov wrong-duration 'expected 1s'

media_variant vp9_wrong_codec vp9.webm wrong-codec 'expected codec_name=vp9'
media_variant vp9_wrong_container vp9.webm wrong-container 'artifact is not WebM'
media_variant opus_mono vp9.webm mono 'expected channels=2'
media_variant opus_silence vp9.webm silence 'Opus track'
media_variant vp9_wrong_size vp9.webm wrong-size 'expected width=480'
media_variant vp9_wrong_duration vp9.webm wrong-duration 'expected 1s'

root="$TMP/corrupt-hevc"; clone_fixture "$root"
corrupt_codec_artifact "$root/delivery-codec-matrix/rendered/hevc.mov"
expect_failure corrupt_hevc "$root" 'does not fully decode'

echo 'delivery codec matrix render evidence contract tests passed'
