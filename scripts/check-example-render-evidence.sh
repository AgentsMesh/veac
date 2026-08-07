#!/usr/bin/env bash
set -euo pipefail

PREVIEW_ROOT=${1:-examples-preview}
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
CATALOG="$SCRIPT_DIR/../examples/catalog/gallery.json"
source "$SCRIPT_DIR/example-preview-provenance.sh"

for module in \
  common pixels showcase-pixels showcase-effects showcase-layout composition \
  showcase-all-features audio-metrics audio audio-all-features fixtures core-media \
  delivery-typed-common delivery-contracts \
  delivery-audio-file delivery-images delivery-waveform delivery-hls-playlist delivery-hls \
  delivery-master-audio delivery-master delivery media-smoke-path media-smoke-roster \
  media-smoke-container media-smoke-timing media-smoke-probe media-smoke \
  timing text text-showcase text-agentsmesh-intro mechanism-pixels \
  visual-canonical-contracts visual-mechanisms \
  generated-graphics advanced-color-stages mask-shape-gallery video-stabilization \
  executable-family \
  workflow-showcase-core workflow-keying workflow-routing workflow-operations \
  workflow-showcase-media workflow-showcases \
  delivery-codec-matrix; do
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/example-render-evidence/$module.sh"
done

validate_preview_directories() {
  [[ -d $PREVIEW_ROOT ]] || fail "preview root does not exist: $PREVIEW_ROOT"
  local directories
  shopt -s nullglob dotglob
  directories=("$PREVIEW_ROOT"/*/)
  shopt -u nullglob dotglob
  ((${#directories[@]} > 0)) || fail "preview root has no example directories: $PREVIEW_ROOT"

  local directory id
  for directory in "${directories[@]}"; do
    id=${directory%/}
    id=${id##*/}
    [[ $id == .fixtures ]] && continue
    jq -e --arg id "$id" 'any(.targets[]; .id == $id)' "$CATALOG" >/dev/null \
      || fail "unknown example directory: $id"
    verify_example_preview_layout "${directory%/}" ||
      fail "invalid authoring/preview layout: $id"
  done
}

validate_preview_directories
check_effects_evidence
check_card_overlay_evidence
check_image_overlay_evidence
check_positioned_evidence
check_audio_evidence
check_mask_evidence
check_fixture_coverage
check_minimal_evidence
check_transitions_evidence
check_delivery_evidence
check_timing_evidence "$PREVIEW_ROOT"
check_template_fill_evidence
check_all_features_visual_evidence
check_advanced_color_evidence
check_color_grade_evidence
check_blend_modes_evidence
check_generated_graphics_evidence
check_apply_scopes_evidence
for example in text-layout text-animation text-overlay captions-and-sidecars hello-world; do
  [[ ! -d $PREVIEW_ROOT/$example ]] || check_text_render_contract \
    "$example" "$PREVIEW_ROOT/$example" "$PREVIEW_ROOT/$example/rendered/preview.mp4"
done
check_agentsmesh_intro_evidence
check_advanced_color_stage_evidence
check_mask_shape_gallery_evidence
check_video_stabilization_evidence
check_executable_family_evidence
check_workflow_showcase_evidence
check_delivery_codec_matrix_evidence
check_all_example_media_smoke "$CATALOG"

echo "rendered example evidence is valid"
