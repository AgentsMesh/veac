#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
for module in common pixels showcase-pixels showcase-effects composition-image-overlay; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
tmp=$(mktemp -d "${TMPDIR:-/tmp}/veac-visual-contracts.XXXXXX")
trap 'rm -rf "$tmp"' EXIT

make_effects() {
  local root=$1 sharpen=${2:-yes} grain=${3:-yes} spill=${4:-yes} key=${5:-yes}
  local dir="$root/video-effects" filter
  mkdir -p "$dir/rendered"
  filter="drawgrid=w=48:h=45:color=0x4ade80:t=3,gblur=sigma=10:enable='lt(t,.5)'"
  [[ $sharpen != yes ]] || filter+=",cas=strength=.22:enable='between(t,1,1.999)'"
  filter+=",eq=brightness=.04:contrast=1.25:saturation=.35:enable='between(t,1,1.999)'"
  filter+=",vignette=PI/3:enable='between(t,1,1.999)'"
  [[ $grain != yes ]] || filter+=",noise=alls=28:allf=t+u:enable='between(t,1,1.999)'"
  if [[ $key == yes ]]; then
    filter+=",drawbox=x=0:y=0:w=iw:h=ih:color=0xd1495b:t=fill:enable='between(t,2,3.999)'"
  else
    filter+=",drawbox=x=0:y=0:w=iw:h=ih:color=0xd1495b:t=fill:enable='gte(t,3)'"
  fi
  [[ $spill == yes ]] || filter+=",drawbox=x=225:y=0:w=10:h=ih:color=0x4ade80:t=fill:enable='between(t,2,2.999)'"
  filter+=",drawbox=x=210:y=0:w=60:h=ih:color=0x4ade80:t=fill:enable='gte(t,3)'"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0x20262e:s=480x270:r=12:d=4' -vf "$filter" \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
}

make_overlay() {
  local root=$1 color=$2
  local dir="$root/image-overlay"
  mkdir -p "$dir/rendered" "$dir/project/assets"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0xf4c95d:s=180x180' -frames:v 1 "$dir/project/assets/logo.png"
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'color=c=0x203040:s=480x270:r=12:d=4' \
    -vf "drawbox=x=372:y=162:w=90:h=90:color=$color:t=fill" \
    -c:v libx264 -pix_fmt yuv420p "$dir/rendered/preview.mp4"
}

expect_effect_failure() {
  local label=$1 root=$2 expected=$3
  local log="$tmp/$label.log"
  if (PREVIEW_ROOT=$root check_effects_evidence >"$log" 2>&1); then
    fail "$label negative effects contract unexpectedly passed"
  fi
  rg -q --fixed-strings "$expected" "$log" || {
    cat "$log" >&2
    fail "$label failed for the wrong effects mechanism"
  }
}

expect_overlay_failure() {
  local label=$1 root=$2
  local log="$tmp/$label.log"
  if (PREVIEW_ROOT=$root check_image_overlay_evidence >"$log" 2>&1); then
    fail "$label negative image-overlay contract unexpectedly passed"
  fi
  rg -q --fixed-strings '90% opacity with screen blending' "$log" || {
    cat "$log" >&2
    fail "$label failed for the wrong image-overlay mechanism"
  }
}

valid_effects="$tmp/valid-effects"
make_effects "$valid_effects"
PREVIEW_ROOT=$valid_effects check_effects_evidence

no_sharpen="$tmp/no-sharpen"
make_effects "$no_sharpen" no yes yes yes
expect_effect_failure missing_sharpen "$no_sharpen" 'sharpen does not produce an edge halo'

no_grain="$tmp/no-grain"
make_effects "$no_grain" yes no yes yes
expect_effect_failure missing_grain "$no_grain" 'grain has no temporal texture'

no_spill="$tmp/no-spill"
make_effects "$no_spill" yes yes no yes
expect_effect_failure missing_spill_suppression "$no_spill" \
  'spill suppression leaves a green fringe'

no_key="$tmp/no-key"
make_effects "$no_key" yes yes yes no
expect_effect_failure missing_chroma_key "$no_key" 'chroma key evidence failed'

valid_overlay="$tmp/valid-overlay"
make_overlay "$valid_overlay" 0xe0c37f
PREVIEW_ROOT=$valid_overlay check_image_overlay_evidence

normal_overlay="$tmp/normal-overlay"
make_overlay "$normal_overlay" 0xdfba5a
expect_overlay_failure normal_blend "$normal_overlay"

opaque_overlay="$tmp/opaque-overlay"
make_overlay "$opaque_overlay" 0xf5d386
expect_overlay_failure wrong_opacity "$opaque_overlay"

printf 'visual mechanism render evidence contract tests passed\n'
