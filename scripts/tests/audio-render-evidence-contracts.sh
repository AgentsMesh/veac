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

audio_plan_variant() {
  local name=$1 filter=$2 root="$TMP_DIR/$1" plan
  mkdir -p "$root"
  cp -R "$VALID/audio-processing" "$root/audio-processing"
  plan="$root/audio-processing/plans/preview/out_preview.json"
  jq "$filter" "$plan" >"$root/plan.tmp"
  mv "$root/plan.tmp" "$plan"
  expect_fail "$name" "$root"
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

audio_plan_variant wrong-duration '.sequences[0].duration.value=2399'
audio_plan_variant wrong-output-audio '.output.deliverables[0].kind.settings.audio.channels=1'
audio_plan_variant wrong-record-range '(.sequences[].tracks[].clips[]|select(.id=="itm_voice")|.record_range.duration.value)=1199'
audio_plan_variant wrong-source-input '(.sequences[].tracks[].clips[]|select(.id=="itm_key")|.source.input_id)="pin_wrong"'
audio_plan_variant wrong-source-rate '(.sequences[].tracks[].clips[]|select(.id=="itm_music")|.source_mapping.time_map.rate.numerator)=2'
audio_plan_variant wrong-music-bus '(.sequences[].tracks[]|select(.id=="trk_music")|.routing.audio.bus_id)="bus_wrong"'
audio_plan_variant wrong-gain-value '(.sequences[].tracks[].clips[]|select(.id=="itm_voice")|.audio.gain.keyframes[1].value)=0.8'
audio_plan_variant wrong-pan-interpolation '(.sequences[].tracks[].clips[]|select(.id=="itm_voice")|.audio.pan.keyframes[1].interpolation.type)="linear"'
audio_plan_variant wrong-processor-order '(.sequences[].tracks[].clips[]|select(.id=="itm_voice")|.audio.processors)|=([.[1],.[0]]+.[2:])'
audio_plan_variant wrong-processor-id '(.sequences[].tracks[].clips[]|select(.id=="itm_voice")|.audio.processors[0].id)="aud_wrong"'
audio_plan_variant wrong-eq-band '(.sequences[].tracks[].clips[]|select(.id=="itm_voice")|.audio.processors[0].kind.bands[0].id)="eqb_wrong"'
audio_plan_variant wrong-compressor '(.sequences[].tracks[].clips[]|select(.id=="itm_voice")|.audio.processors[3].kind.ratio)=2'
audio_plan_variant wrong-music-fade '(.sequences[].tracks[].clips[]|select(.id=="itm_music")|.audio.crossfade.curve)="equal_power"'
audio_plan_variant wrong-key-fade '(.sequences[].tracks[].clips[]|select(.id=="itm_key")|.audio.crossfade.curve)="linear"'
audio_plan_variant wrong-pitch-policy '(.sequences[].tracks[].clips[]|select(.id=="itm_music")|.audio.pitch_policy)="preserve"'
audio_plan_variant wrong-sidechain-source '(.sequences[].tracks[].clips[]|select(.id=="itm_music")|.audio.sidechain.source.track_id)="trk_voice"'
audio_plan_variant wrong-sidechain-ratio '(.sequences[].tracks[].clips[]|select(.id=="itm_music")|.audio.sidechain.ratio)=8'
audio_plan_variant wrong-normalize-target '(.sequences[].tracks[].clips[]|select(.id=="itm_music")|.effects[0].effect.target_lufs)=-18'
audio_plan_variant wrong-normalize-range '(.sequences[].tracks[].clips[]|select(.id=="itm_music")|.effects[0].active_range.start.value)=600'

echo "audio render evidence contract tests passed"
