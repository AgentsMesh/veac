#!/usr/bin/env bash

check_delivery_evidence() {
  local dir="$PREVIEW_ROOT/delivery-formats"
  [[ -d $dir ]] || return 0
  local rendered="$dir/rendered"
  [[ -d $rendered ]] || fail "delivery-formats rendered directory is missing"
  check_delivery_plan_contracts "$dir"
  check_delivery_preview_policy "$(example_preview_canonical "$dir")"
  check_delivery_root_inventory "$dir"
  check_delivery_master "$dir"
  check_delivery_burn_in "$dir"
  check_delivery_frames "$dir"
  check_delivery_captions "$rendered"
  check_delivery_audio "$dir"
  assert_delivery_audio_match "$dir"
  check_delivery_waveform "$dir"
  check_delivery_mp3 "$dir"
  check_delivery_images "$dir"
  check_delivery_hls "$dir"
}
