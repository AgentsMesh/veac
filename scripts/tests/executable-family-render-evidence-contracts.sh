#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
SCRIPT_DIR="$ROOT/scripts"
for module in common pixels showcase-pixels audio-metrics executable-family; do
  # shellcheck source=/dev/null
  source "$SCRIPT_DIR/example-render-evidence/$module.sh"
done
for fixture in executable-family-fixture-common executable-family-basic-fixtures \
  executable-family-visual-fixtures executable-family-media-fixtures; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/tests/$fixture.sh"
done
tmp=$(mktemp -d "${TMPDIR:-/tmp}/veac-executable-family-contracts.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

expect_failure() {
  local label=$1 root=$2 expected=$3 log
  log="$tmp/$label.log"
  if (PREVIEW_ROOT=$root check_executable_family_evidence >"$log" 2>&1); then
    fail "$label negative executable-family contract passed"
  fi
  rg -q --fixed-strings "$expected" "$log" || {
    cat "$log" >&2
    fail "$label failed for the wrong mechanism"
  }
}

case_root() {
  local label=$1 id=$2 root
  root="$tmp/$label"
  copy_fixture_tree "$valid" "$root" "$id"
  printf '%s\n' "$root"
}

mutate_json() {
  local file=$1 filter=$2
  jq "$filter" "$file" >"$file.tmp"
  mv "$file.tmp" "$file"
}

valid="$tmp/valid"
write_executable_family_projects "$valid"
write_executable_family_media "$valid"
PREVIEW_ROOT=$valid check_executable_family_evidence

root=$(case_root programming-preview-drift programming-language)
mutate_json "$root/programming-language/project/project.preview.veac.json" \
  '(.project.sequences[].tracks[].clips[]|select(.id=="itm_second_title").source.text)="漂移"'
expect_failure programming_preview_drift "$root" 'programming-language project contract failed'

root=$(case_root programming-plural-attachment programming-language)
mutate_json "$root/programming-language/project/project.veac.json" \
  '(.project.sequences[0].authorship.tracks[0].entity.events[1].operation)=8233'
expect_failure programming_plural_attachment "$root" 'programming-language project contract failed'

root=$(case_root programming-plan-owner programming-language)
mutate_json "$root/programming-language/plans/preview/out_preview.json" \
  '.temporal.bindings[0].clocks[0].owner.item_id="itm_wrong"'
expect_failure programming_plan_owner "$root" 'programming-language resolved program contract failed'

root=$(case_root programming-media-color programming-language)
ffmpeg -v error -y -i "$root/programming-language/rendered/preview.mp4" -vf \
  "drawbox=x=0:y=0:w=iw:h=ih:color=0x2b6574:t=fill:enable='gte(t,3)'" \
  -c:v libx264 -pix_fmt yuv420p "$root/programming-language/rendered/bad.mp4"
mv "$root/programming-language/rendered/bad.mp4" "$root/programming-language/rendered/preview.mp4"
expect_failure programming_media_color "$root" 'programming-language enum color'

root=$(case_root audio-preview-speaker executable-audio-caption)
mutate_json "$root/executable-audio-caption/project/project.preview.veac.json" \
  '(.project.sequences[].tracks[].clips[]|select(.id=="itm_intro").source.speaker)=null'
expect_failure audio_preview_speaker "$root" 'executable-audio-caption project contract failed'

root=$(case_root audio-plan-fade executable-audio-caption)
mutate_json "$root/executable-audio-caption/plans/preview/out_preview.json" \
  '(.sequences[].tracks[].clips[]|select(.id=="itm_tone").audio.crossfade.curve)="linear"'
expect_failure audio_plan_fade "$root" 'executable-audio-caption plan contract failed'

root=$(case_root audio-media-silent executable-audio-caption)
ffmpeg -v error -y -i "$root/executable-audio-caption/rendered/preview.mp4" \
  -f lavfi -i 'anullsrc=r=48000:cl=stereo:d=4' -map 0:v:0 -map 1:a:0 \
  -c:v copy -c:a aac -shortest "$root/executable-audio-caption/rendered/bad.mp4"
mv "$root/executable-audio-caption/rendered/bad.mp4" \
  "$root/executable-audio-caption/rendered/preview.mp4"
expect_failure audio_media_silent "$root" 'audio-caption declared tone'

root=$(case_root dissolve-preview-alignment executable-centered-dissolve)
mutate_json "$root/executable-centered-dissolve/project/project.preview.veac.json" \
  '.project.relations[0].kind.transition.alignment="start"'
expect_failure dissolve_preview_alignment "$root" 'executable-centered-dissolve project contract failed'

root=$(case_root dissolve-plan-window executable-centered-dissolve)
mutate_json "$root/executable-centered-dissolve/plans/preview/out_preview.json" \
  '.sequences[0].tracks[0].transitions[0].record_window.duration.value=120'
expect_failure dissolve_plan_window "$root" 'executable-centered-dissolve plan contract failed'

root=$(case_root dissolve-media-cut executable-centered-dissolve)
ffmpeg -v error -y -i "$root/executable-centered-dissolve/rendered/preview.mp4" -vf \
  "drawbox=x=0:y=0:w=iw:h=ih:color=0xef4444:t=fill:enable='lt(t,2)',
   drawbox=x=0:y=0:w=iw:h=ih:color=0x2563eb:t=fill:enable='gte(t,2)'" \
  -c:v libx264 -pix_fmt yuv420p "$root/executable-centered-dissolve/rendered/bad.mp4"
mv "$root/executable-centered-dissolve/rendered/bad.mp4" \
  "$root/executable-centered-dissolve/rendered/preview.mp4"
expect_failure dissolve_media_cut "$root" 'centered dissolve is not monotonic'

root=$(case_root image-preview-flip executable-local-image)
mutate_json "$root/executable-local-image/project/project.preview.veac.json" \
  '(.project.sequences[].tracks[].clips[]|select(.id=="itm_poster").visual.transform.flip_horizontal)=false'
expect_failure image_preview_flip "$root" 'executable-local-image project contract failed'

root=$(case_root image-plan-probe executable-local-image)
mutate_json "$root/executable-local-image/plans/preview/out_preview.json" \
  '.inputs[0].video.info.width=63'
expect_failure image_plan_probe "$root" 'executable-local-image plan contract failed'

root=$(case_root image-media-unflipped executable-local-image)
ffmpeg -v error -y -f lavfi -i 'color=c=0x18384f:s=64x36:r=12:d=3' -vf \
  'drawbox=x=24:y=4:w=16:h=18:color=white:t=fill,
   drawbox=x=40:y=4:w=16:h=18:color=black:t=fill' \
  -c:v libx264 -pix_fmt yuv420p "$root/executable-local-image/rendered/preview.mp4"
expect_failure image_media_unflipped "$root" 'local image horizontal flip is visible'

root=$(case_root text-preview-granularity executable-text-family)
mutate_json "$root/executable-text-family/project/project.preview.veac.json" \
  '(.project.sequences[].tracks[].clips[]|select(.id=="itm_line").source.style.animation.granularity)="word"'
expect_failure text_preview_granularity "$root" 'executable-text-family project contract failed'

root=$(case_root text-author-fallback executable-text-family)
mutate_json "$root/executable-text-family/project/project.veac.json" \
  '(.project.sequences[].tracks[].clips[]|select(.id=="itm_word").source.style.fallback_fonts)|=reverse'
expect_failure text_author_fallback "$root" 'executable-text-family authoring font stack failed'

root=$(case_root text-plan-writing executable-text-family)
mutate_json "$root/executable-text-family/plans/preview/out_preview.json" \
  '(.sequences[].tracks[].clips[]|select(.id=="itm_grapheme").source.content.presentation.style.layout.writing_mode)="horizontal-tb"'
expect_failure text_plan_writing "$root" 'executable-text-family plan contract failed'

root=$(case_root text-media-missing executable-text-family)
ffmpeg -v error -y -i "$root/executable-text-family/rendered/preview.mp4" -vf \
  "drawbox=x=0:y=0:w=iw:h=ih:color=0x101827:t=fill:enable='between(t,4,5.999)'" \
  -c:v libx264 -pix_fmt yuv420p "$root/executable-text-family/rendered/bad.mp4"
mv "$root/executable-text-family/rendered/bad.mp4" \
  "$root/executable-text-family/rendered/preview.mp4"
expect_failure text_media_missing "$root" 'text-family line phase is not vertical'

printf 'executable family render evidence contract tests passed\n'
