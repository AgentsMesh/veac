#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/veac-visual-mechanisms.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
for module in common pixels mechanism-pixels visual-canonical-contracts visual-mechanisms; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/visual-canonical-fixtures.sh"
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/visual-mechanism-video-fixtures.sh"

delivery_video_path() { printf '%s/rendered/preview.mp4\n' "$1"; }

check_fixture() {
  case $1 in
    color-grade) check_color_grade_evidence ;;
    blend-modes) check_blend_modes_evidence ;;
    apply-scopes) check_apply_scopes_evidence ;;
  esac
}

make_fixture() {
  local root=$1 name=$2 dir="$1/$2"
  mkdir -p "$dir/project" "$dir/rendered"
  case $name in
    color-grade)
      write_color_grade_fixture "$dir/project/project.veac.json"
      make_color_grade_video "$dir/rendered/preview.mp4" ;;
    blend-modes)
      write_blend_modes_fixture "$dir/project/project.veac.json"
      make_blend_modes_video "$dir/rendered/preview.mp4" ;;
    apply-scopes)
      write_apply_scopes_fixture "$dir/project/project.veac.json"
      make_apply_scopes_video "$dir/rendered/preview.mp4" ;;
  esac
}

expect_failure() {
  local root=$1 name=$2 expected=$3 log="$TMP/$name.log"
  if (PREVIEW_ROOT=$root check_fixture "$name") >"$log" 2>&1; then
    fail "$name negative visual mechanism contract passed"
  fi
  rg -qF "$expected" "$log" || { cat "$log" >&2; fail "$name failed incorrectly"; }
}

VALID="$TMP/valid"
for name in color-grade blend-modes apply-scopes; do
  make_fixture "$VALID" "$name"
  PREVIEW_ROOT=$VALID check_fixture "$name"
done

canonical_variant() {
  local name=$1 filter=$2 root="$TMP/$1-canonical"
  mkdir -p "$root"
  cp -R "$VALID/$name" "$root/$name"
  jq "$filter" "$root/$name/project/project.veac.json" >"$root/value.json"
  mv "$root/value.json" "$root/$name/project/project.veac.json"
  expect_failure "$root" "$name" "canonical mechanism identity contract failed"
}

canonical_variant color-grade \
  '(.project.sequences[].tracks[].clips[]|select(.authorship.logical_path[-1]=="balanced")|.visual.color_pipeline.stages[0].adjustment.exposure_stops)=0'
canonical_variant blend-modes \
  '(.project.sequences[].tracks[].clips[]|select(.authorship.logical_path[-1]=="screen")|.visual.compositing.blend_mode)="normal"'
canonical_variant apply-scopes \
  '(.project.sequences[].applies[1].target.type)="item_set"'
canonical_variant color-grade \
  '(.project.sequences[].tracks[].clips[]|select(.authorship.logical_path[-1]=="balanced")|.visual.placement)={type:"absolute",position:{x:{unit:"pixels",value:0},y:{unit:"pixels",value:0}}}'
canonical_variant blend-modes \
  '(.project.sequences[].tracks[].clips[]|select(.authorship.logical_path[-1]=="screen")|.visual.placement)={type:"absolute",position:{x:{unit:"pixels",value:0},y:{unit:"pixels",value:0}}}'
canonical_variant apply-scopes \
  '(.project.sequences[].tracks[].clips[]|select(.authorship.logical_path[-1]=="accent")|.visual.placement)={type:"absolute",position:{x:{unit:"pixels",value:0},y:{unit:"pixels",value:0}}}'

hard_edge="$TMP/apply-scopes-hard-edge"
mkdir -p "$hard_edge"
cp -R "$VALID/apply-scopes" "$hard_edge/apply-scopes"
make_apply_scopes_hard_edge_video \
  "$hard_edge/apply-scopes/rendered/preview.mp4"
expect_failure "$hard_edge" apply-scopes "圆形模糊边界"

for name in color-grade blend-modes; do
  root="$TMP/$name-quarter-footprint"
  mkdir -p "$root"
  cp -R "$VALID/$name" "$root/$name"
  video="$root/$name/rendered/preview.mp4"
  make_quarter_footprint_video "$video" "$root/quarter.mp4"
  mv "$root/quarter.mp4" "$video"
  expect_failure "$root" "$name" "覆盖右侧无标签 ROI"
done

misplaced="$TMP/apply-scopes-misplaced"
mkdir -p "$misplaced"
cp -R "$VALID/apply-scopes" "$misplaced/apply-scopes"
make_apply_scopes_misplaced_video "$misplaced/apply-scopes/rendered/preview.mp4"
expect_failure "$misplaced" apply-scopes "画面中心而非左上角"

for name in color-grade blend-modes apply-scopes; do
  root="$TMP/$name-label-only"
  mkdir -p "$root"
  cp -R "$VALID/$name" "$root/$name"
  case $name in
    color-grade)
      make_label_only_video "$root/$name/rendered/preview.mp4" 2 2
      assert_unique_frames "$root/$name/rendered/preview.mp4" 2 1 3
      expected="无标签 ROI" ;;
    blend-modes)
      make_label_only_video "$root/$name/rendered/preview.mp4" 12 1
      assert_unique_frames "$root/$name/rendered/preview.mp4" 12 \
        0.5 1.5 2.5 3.5 4.5 5.5 6.5 7.5 8.5 9.5 10.5 11.5
      expected=label-free ;;
    apply-scopes)
      make_label_only_video "$root/$name/rendered/preview.mp4" 3 2
      assert_unique_frames "$root/$name/rendered/preview.mp4" 3 1 3 5
      expected=label-free ;;
  esac
  expect_failure "$root" "$name" "$expected"
done

printf 'visual mechanism render evidence contract tests passed\n'
