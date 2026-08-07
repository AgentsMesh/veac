#!/usr/bin/env bash

for module in common pixels audio-metrics delivery-typed-common delivery-contracts \
  delivery-audio-file delivery-images delivery-waveform delivery-hls-playlist delivery-hls \
  delivery-master-audio delivery-master delivery; do
  # shellcheck disable=SC1090
  source "$ROOT_DIR/scripts/example-render-evidence/$module.sh"
done

for module in typed-delivery-plan-contracts typed-delivery-audio-contracts \
  typed-delivery-image-contracts typed-delivery-waveform-contracts \
  typed-delivery-hls-contracts typed-delivery-hls-playlist-contracts; do
  # shellcheck disable=SC1090
  source "$ROOT_DIR/scripts/tests/$module.sh"
done

clone_delivery_fixture() {
  local root=$1
  mkdir -p "$root"
  cp -R "$VALID/delivery-formats" "$root/delivery-formats"
}

expect_typed_failure() {
  local label=$1 root=$2 checker=$3 expected=$4
  if ("$checker" "$root/delivery-formats") >"$TMP_DIR/$label.log" 2>&1; then
    echo "expected typed delivery failure: $label" >&2
    exit 1
  fi
  grep -Fq "$expected" "$TMP_DIR/$label.log" || {
    cat "$TMP_DIR/$label.log" >&2
    echo "typed delivery failure did not report '$expected': $label" >&2
    exit 1
  }
}

rewrite_delivery_json() {
  local dir=$1 filter=$2 file
  for file in "$dir/project/project.veac.json" \
      "$dir/project/project.preview.veac.json" \
      "$dir/plans/preview/out_master.json"; do
    jq "$filter" "$file" >"$file.tmp"
    mv "$file.tmp" "$file"
  done
}

run_typed_delivery_contracts() {
  run_delivery_plan_contracts
  run_delivery_audio_contracts
  run_delivery_image_contracts
  run_delivery_waveform_contracts
  run_delivery_hls_contracts
  run_delivery_hls_playlist_contracts
}
