#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
CHECK="$ROOT_DIR/scripts/check-example-audio-evidence.sh"
TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/veac-audio-contracts.XXXXXX")
trap 'rm -rf "$TMP_DIR"' EXIT

# shellcheck disable=SC1091
source "$ROOT_DIR/scripts/tests/audio-render-fixtures.sh"

expect_pass() {
  bash "$CHECK" "$2" >"$TMP_DIR/$1.log" 2>&1 \
    || { cat "$TMP_DIR/$1.log" >&2; echo "expected pass: $1" >&2; exit 1; }
}

expect_fail() {
  if bash "$CHECK" "$2" >"$TMP_DIR/$1.log" 2>&1; then
    echo "expected failure: $1" >&2
    exit 1
  fi
}

VALID="$TMP_DIR/valid"
make_audio_processing_fixture "$VALID"
make_generated_audio_fixture "$VALID"
make_all_features_audio_fixture "$VALID"
make_executable_audio_fixture "$VALID"
make_nested_multicam_audio_fixture "$VALID"
expect_pass valid_audio_evidence "$VALID"

SILENT_PHASE="$TMP_DIR/silent-phase"
make_audio_processing_fixture "$SILENT_PHASE" silence
expect_fail silent_final_processing_phase "$SILENT_PHASE"

AUDIBLE_GENERATOR="$TMP_DIR/audible-generator"
make_generated_audio_fixture "$AUDIBLE_GENERATOR" tone
expect_fail audible_generated_silence "$AUDIBLE_GENERATOR"

SILENT_INTEGRATION="$TMP_DIR/silent-integration"
make_all_features_audio_fixture "$SILENT_INTEGRATION" silence
expect_fail silent_all_features_mix "$SILENT_INTEGRATION"

AUDIBLE_EXECUTABLE="$TMP_DIR/audible-executable"
make_executable_audio_fixture "$AUDIBLE_EXECUTABLE" tone
expect_fail audible_executable_silence "$AUDIBLE_EXECUTABLE"

AUDIBLE_MULTICAM="$TMP_DIR/audible-multicam"
make_nested_multicam_audio_fixture "$AUDIBLE_MULTICAM" tone
expect_fail unexpected_multicam_delivery_audio "$AUDIBLE_MULTICAM"

BAD_MULTICAM_SYNC="$TMP_DIR/bad-multicam-sync"
make_nested_multicam_audio_fixture "$BAD_MULTICAM_SYNC" none timecode
expect_fail wrong_multicam_sync_basis "$BAD_MULTICAM_SYNC"

MISROUTED_SIDECHAIN="$TMP_DIR/misrouted-sidechain"
mkdir -p "$MISROUTED_SIDECHAIN"
cp -R "$VALID/audio-processing" "$MISROUTED_SIDECHAIN/audio-processing"
jq '(.sequences[].tracks[].clips[] | select(.id == "itm_music-bed") |
  .audio.sidechain.source.bus_id) = "bus_wrong"' \
  "$MISROUTED_SIDECHAIN/audio-processing/plans/preview/out_preview.json" \
  >"$MISROUTED_SIDECHAIN/plan.tmp"
mv "$MISROUTED_SIDECHAIN/plan.tmp" \
  "$MISROUTED_SIDECHAIN/audio-processing/plans/preview/out_preview.json"
expect_fail misrouted_audio_sidechain "$MISROUTED_SIDECHAIN"

echo "audio render evidence contract tests passed"
