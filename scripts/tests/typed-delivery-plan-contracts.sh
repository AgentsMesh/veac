#!/usr/bin/env bash

run_delivery_plan_contracts() {
  local root dir file

  root="$TMP_DIR/plan-config-mismatch"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats"
  jq '.output.render_config_id = "out_wrong"' "$dir/plans/preview/out_master.json" \
    >"$dir/plan.tmp"
  mv "$dir/plan.tmp" "$dir/plans/preview/out_master.json"
  expect_typed_failure plan_config_mismatch "$root" check_delivery_plan_contracts \
    "canonical and resolved plan outputs differ"

  root="$TMP_DIR/plan-output-mismatch"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats"
  for file in "$dir/plans/preview/out_master.json"; do
    jq 'del(.output.deliverables[] | select(.id == "dlv_podcast"))' "$file" >"$file.tmp"
    mv "$file.tmp" "$file"
  done
  expect_typed_failure plan_output_mismatch "$root" check_delivery_plan_contracts \
    "canonical and resolved plan outputs differ"

  root="$TMP_DIR/not-nine-deliverables"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'if has("project") then .project.render_configs[].deliverables |= map(select(.id != "dlv_podcast"))
     else .output.deliverables |= map(select(.id != "dlv_podcast")) end'
  expect_typed_failure not_nine_deliverables "$root" check_delivery_plan_contracts \
    "exact 9-deliverable set"

  root="$TMP_DIR/extra-rendered-member"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats"
  cp "$dir/rendered/captions.vtt" "$dir/rendered/undeclared.txt"
  expect_typed_failure extra_rendered_member "$root" check_delivery_root_inventory \
    "rendered root inventory is not exact"

  root="$TMP_DIR/non-fast-start"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats"
  ffmpeg -nostdin -v error -y -i "$dir/rendered/master.mp4" -map 0 -c copy \
    "$dir/rendered/master.not-fast.mp4"
  mv "$dir/rendered/master.not-fast.mp4" "$dir/rendered/master.mp4"
  expect_typed_failure non_fast_start "$root" check_delivery_master "is not fast-start"

  root="$TMP_DIR/wrong-master-crf"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'walk(if type == "object" and .id? == "dlv_master"
      then .kind.settings.video.rate_control.value = 24 else . end)'
  expect_typed_failure wrong_master_crf "$root" check_delivery_master \
    "MP4 H.264/AAC CRF/pass/streaming canonical contract"

  root="$TMP_DIR/missing-master-aac-declaration"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'walk(if type == "object" and .id? == "dlv_master"
      then .kind.settings.audio = null else . end)'
  expect_typed_failure missing_master_aac "$root" check_delivery_master \
    "MP4 H.264/AAC CRF/pass/streaming canonical contract"

  root="$TMP_DIR/wrong-frame-format-declaration"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'walk(if type == "object" and .id? == "dlv_frames"
      then .kind.settings.format = "jpeg" else . end)'
  expect_typed_failure wrong_frame_format "$root" check_delivery_frames \
    "PNG sequence numbering canonical contract"

  root="$TMP_DIR/dynamic-preview-raster"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'if has("project") then .project.render_configs[].raster.width = 320
     else .output.raster.width = 320 end'
  expect_typed_failure dynamic_preview_raster "$root" check_delivery_master "expected width=320"

  root="$TMP_DIR/dynamic-preview-fps"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'if has("project") then .project.render_configs[].raster.frame_rate.numerator = 24
     else .output.raster.frame_rate.numerator = 24 end'
  local _width _height rate _duration count start _cover_time _cover before after
  read -r _width _height rate _duration count start _cover_time _cover < <(
    delivery_metadata "$root/delivery-formats")
  read -r before after < <(delivery_boundary_frame_numbers "$start" "$count")
  [[ $rate == 24/1 && $count == 72 && $before == 36 && $after == 37 ]] || {
    echo "24fps delivery boundary metadata is not exact" >&2
    exit 1
  }
  read -r before after < <(delivery_boundary_frame_numbers 1 45)
  [[ $before == 23 && $after == 24 ]] || {
    echo "15fps odd delivery boundary metadata is not exact" >&2
    exit 1
  }
  read -r before after < <(delivery_boundary_frame_numbers 1 36)
  [[ $before == 18 && $after == 19 ]] || {
    echo "12fps delivery boundary frame numbers are not exact" >&2
    exit 1
  }
  local before_time after_time
  read -r before_time after_time < <(delivery_boundary_times 36 12/1)
  awk -v before="$before_time" -v after="$after_time" \
    'BEGIN { exit !((before*12-17)^2 < 1e-12 && (after*12-18)^2 < 1e-12) }' || {
    echo "12fps delivery boundary sample times are not exact" >&2
    exit 1
  }
  read -r before_time after_time < <(delivery_boundary_times 45 15/1)
  awk -v before="$before_time" -v after="$after_time" \
    'BEGIN { exit !((before*15-22)^2 < 1e-12 && (after*15-23)^2 < 1e-12) }' || {
    echo "15fps delivery boundary sample times are not exact" >&2
    exit 1
  }
  expect_typed_failure dynamic_preview_fps "$root" check_delivery_master \
    "expected r_frame_rate=24/1"

  root="$TMP_DIR/dynamic-frame-count"
  clone_delivery_fixture "$root"
  dir="$root/delivery-formats"
  for file in "$dir/plans/preview/out_master.json"; do
    jq '(.sequences[] | select(.id == "seq_main") | .duration.value) = 2000' \
      "$file" >"$file.tmp"
    mv "$file.tmp" "$file"
  done
  expect_typed_failure dynamic_frame_count "$root" check_delivery_frames \
    "expected 24 frames, got 36"

  root="$TMP_DIR/dynamic-cover-frame"
  clone_delivery_fixture "$root"
  rewrite_delivery_json "$root/delivery-formats" \
    'walk(if type == "object" and .id? == "dlv_cover"
      then .kind.settings.frame.at.value = 1000 else . end)'
  expect_typed_failure dynamic_cover_frame "$root" check_delivery_cover \
    "does not visually match frame 13"
}
