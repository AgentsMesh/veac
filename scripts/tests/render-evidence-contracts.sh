#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
CHECK="$ROOT_DIR/scripts/check-example-render-evidence.sh"
TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/veac-render-contracts.XXXXXX")
TMP_DIR=$(cd "$TMP_DIR" && pwd -P)
trap 'rm -rf "$TMP_DIR"' EXIT
# shellcheck source=scripts/tests/typed-delivery-fixtures.sh
source "$ROOT_DIR/scripts/tests/typed-delivery-fixtures.sh"
# shellcheck source=scripts/tests/media-smoke-fixtures.sh
source "$ROOT_DIR/scripts/tests/media-smoke-fixtures.sh"
# shellcheck source=scripts/tests/all-features-render-fixtures.sh
source "$ROOT_DIR/scripts/tests/all-features-render-fixtures.sh"

expect_pass() {
  local label=$1
  local root=$2
  bash "$CHECK" "$root" >"$TMP_DIR/$label.log" 2>&1 \
    || { cat "$TMP_DIR/$label.log" >&2; echo "expected pass: $label" >&2; exit 1; }
}

expect_fail() {
  local label=$1
  local root=$2
  if bash "$CHECK" "$root" >"$TMP_DIR/$label.log" 2>&1; then
    echo "expected failure: $label" >&2
    exit 1
  fi
}

make_minimal() {
  local root=$1
  prepare_preview_fixture_dirs "$root/minimal"
  ffmpeg -v error -f lavfi -i 'color=c=#1b3a57:s=480x270:r=12:d=3' \
    -c:v libx264 -pix_fmt yuv420p "$root/minimal/rendered/preview.mp4"
  cat >"$root/minimal/project/project.veac.json" <<'JSON'
{"project":{"render_configs":[{"id":"out_preview","raster":{"width":480,"height":270,"frame_rate":{"numerator":12,"denominator":1}},"deliverables":[{"target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video","settings":{"audio":null}}}]}]}}
JSON
  complete_video_settings "$root/minimal/project/project.veac.json"
  mirror_fixture_preview_canonical "$root/minimal"
  write_smoke_plan "$root/minimal/plans/preview/out_preview.json" out_preview 3000
}

make_all_features() {
  local root=$1
  local project="$root/all-features/project"

  make_all_features_fixture "$root"
  complete_video_settings "$project/project.veac.json"
  write_all_features_preview \
    "$project/project.veac.json" \
    "$project/project.preview.veac.json"
  write_all_features_plan \
    "$project/project.preview.veac.json" \
    "$root/all-features/plans/preview/out_master.json"
}

make_transitions() {
  local root=$1
  local dir="$root/transitions"
  prepare_preview_fixture_dirs "$dir"
  ffmpeg -v error -f lavfi -i 'color=c=black:s=480x270:r=12:d=4' \
    -vf "format=gbrp,geq=r='if(lt(T,1.8),230,if(gt(T,2.2),69,230+(69-230)*(T-1.8)/.4))':g='if(lt(T,1.8),57,if(gt(T,2.2),123,57+(123-57)*(T-1.8)/.4))':b='if(lt(T,1.8),70,if(gt(T,2.2),157,70+(157-70)*(T-1.8)/.4))'" \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
  cat >"$dir/plans/preview/out_preview.json" <<'JSON'
{
  "output": {"render_config_id":"out_preview","sequence_id":"seq_main"},
  "sequences": [{"id":"seq_main","duration":{"timescale":1000,"value":4000},
    "tracks": [{
      "transitions": [{
        "record_window": {
          "start": { "value": 1800, "timescale": 1000 },
          "duration": { "value": 400, "timescale": 1000 }
        },
        "kind": { "type": "dissolve" }
      }]
    }]
  }]
}
JSON
  cat >"$dir/project/project.veac.json" <<'JSON'
{"project":{"render_configs":[{"id":"out_preview","raster":{"width":480,"height":270,"frame_rate":{"numerator":12,"denominator":1}},"deliverables":[{"target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video","settings":{"audio":null}}}]}]}}
JSON
  complete_video_settings "$dir/project/project.veac.json"
  mirror_fixture_preview_canonical "$dir"
}

VALID="$TMP_DIR/valid"
export VEAC_EXAMPLES='minimal,all-features,transitions,delivery-formats'
mkdir -p "$VALID"
make_minimal "$VALID"
make_all_features "$VALID"
make_transitions "$VALID"
make_delivery "$VALID"
expect_pass valid_contracts "$VALID"
for selector in '' all; do
  if VEAC_EXAMPLES=$selector bash "$CHECK" "$VALID" >/dev/null 2>&1; then
    echo "incomplete preview roster passed selector '${selector:-<empty>}'" >&2
    exit 1
  fi
done

UNKNOWN="$TMP_DIR/unknown"
mkdir -p "$UNKNOWN/not-a-gallery-example"
expect_fail unknown_directory "$UNKNOWN"
grep -q 'unknown example directory' "$TMP_DIR/unknown_directory.log"

HIDDEN_UNKNOWN="$TMP_DIR/hidden-unknown"
mkdir -p "$HIDDEN_UNKNOWN/.not-fixtures"
expect_fail hidden_unknown_directory "$HIDDEN_UNKNOWN"
grep -q 'unknown example directory' "$TMP_DIR/hidden_unknown_directory.log"

BAD_TRANSITION="$TMP_DIR/bad-transition"
mkdir -p "$BAD_TRANSITION"
cp -R "$VALID/transitions" "$BAD_TRANSITION/transitions"
jq '.sequences[0].tracks[0].transitions[0].record_window.duration.value = 500' \
  "$BAD_TRANSITION/transitions/plans/preview/out_preview.json" >"$BAD_TRANSITION/plan.tmp"
mv "$BAD_TRANSITION/plan.tmp" "$BAD_TRANSITION/transitions/plans/preview/out_preview.json"
expect_fail bad_transition_duration "$BAD_TRANSITION"

BAD_DELIVERY="$TMP_DIR/bad-delivery"
mkdir -p "$BAD_DELIVERY"
cp -R "$VALID/delivery-formats" "$BAD_DELIVERY/delivery-formats"
rm "$BAD_DELIVERY/delivery-formats/rendered/frame-0036.png"
expect_fail missing_delivery_frame "$BAD_DELIVERY"

STATIC_DELIVERY="$TMP_DIR/static-delivery"
mkdir -p "$STATIC_DELIVERY"
cp -R "$VALID/delivery-formats" "$STATIC_DELIVERY/delivery-formats"
cp "$STATIC_DELIVERY/delivery-formats/rendered/frame-0001.png" \
  "$STATIC_DELIVERY/delivery-formats/rendered/frame-0036.png"
expect_fail static_delivery_frames "$STATIC_DELIVERY"

BAD_SCOPE="$TMP_DIR/bad-scope"
mkdir -p "$BAD_SCOPE"
cp -R "$VALID/delivery-formats" "$BAD_SCOPE/delivery-formats"
jq '(.project.render_configs[] | select(.id == "out_master") |
  .deliverables[] | select(.id == "dlv_video-waveform") |
  .kind.settings.scope) = "vectorscope"' \
  "$BAD_SCOPE/delivery-formats/project/project.veac.json" >"$BAD_SCOPE/project.tmp"
mv "$BAD_SCOPE/project.tmp" "$BAD_SCOPE/delivery-formats/project/project.veac.json"
expect_fail wrong_video_waveform_scope "$BAD_SCOPE"

SILENT_DELIVERY="$TMP_DIR/silent-delivery"
mkdir -p "$SILENT_DELIVERY"
cp -R "$VALID/delivery-formats" "$SILENT_DELIVERY/delivery-formats"
ffmpeg -v error -y -f lavfi -i 'color=c=white:s=480x270:r=12:d=3' \
  -f lavfi -i 'anullsrc=r=48000:cl=stereo' -t 3 \
  -c:v libx264 -pix_fmt yuv420p -c:a aac "$SILENT_DELIVERY/delivery-formats/rendered/master.mp4"
expect_fail silent_delivery_master "$SILENT_DELIVERY"

MISMATCHED_STEM="$TMP_DIR/mismatched-stem"
mkdir -p "$MISMATCHED_STEM"
cp -R "$VALID/delivery-formats" "$MISMATCHED_STEM/delivery-formats"
ffmpeg -v error -y -f lavfi -i 'sine=frequency=880:sample_rate=48000:duration=3' \
  -ac 2 -c:a pcm_s24le "$MISMATCHED_STEM/delivery-formats/rendered/master.wav"
expect_fail mismatched_delivery_stem "$MISMATCHED_STEM"

# shellcheck source=scripts/tests/typed-delivery-render-evidence-contracts.sh
source "$ROOT_DIR/scripts/tests/typed-delivery-render-evidence-contracts.sh"
assert_delivery_caption_geometry_contract() {
  local checker=$1 metrics
  (
    frame_bright_bbox() {
      [[ ${4:-} == 500 ]] || exit 1
      printf '40 68 9\n'
    }
    "$checker" "$VALID/delivery-formats"
  )
  for metrics in '39 68 9' '40 67 9' '40 68 8'; do
    if (
      frame_bright_bbox() { printf '%s\n' "$metrics"; }
      "$checker" "$VALID/delivery-formats"
    ) >/dev/null 2>&1; then
      echo "invalid delivery caption metrics passed: $checker $metrics" >&2
      exit 1
    fi
  done
}
assert_delivery_caption_geometry_contract check_delivery_burn_in
assert_delivery_caption_geometry_contract check_delivery_cover
run_typed_delivery_contracts

bash "$ROOT_DIR/scripts/tests/showcase-render-evidence-contracts.sh"
bash "$ROOT_DIR/scripts/tests/audio-render-evidence-contracts.sh"

echo "render evidence contract tests passed"
