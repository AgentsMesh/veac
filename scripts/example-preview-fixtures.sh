#!/usr/bin/env bash

# shellcheck source=example-preview-luts.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-luts.sh"

preview_die() {
  echo "examples preview: $*" >&2
  return 1
}

find_preview_font() {
  if [[ -n "${VEAC_PREVIEW_FONT:-}" ]]; then
    [[ -f "$VEAC_PREVIEW_FONT" ]] || preview_die "font not found: $VEAC_PREVIEW_FONT"
    printf '%s\n' "$VEAC_PREVIEW_FONT"
    return
  fi

  local candidate
  for candidate in \
    /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf \
    /usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf \
    /opt/homebrew/share/fonts/DejaVuSans.ttf \
    /System/Library/Fonts/Supplemental/Arial.ttf; do
    if [[ -f "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  done

  if command -v fc-match >/dev/null 2>&1; then
    candidate=$(fc-match -f '%{file}\n' 'sans-serif' | head -n 1)
    if [[ -f "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  fi
  preview_die "set VEAC_PREVIEW_FONT to a readable TTF or OTF file"
}

make_fixture_video() {
  local visual=$1
  local frequency=$2
  local output=$3
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i "$visual" \
    -f lavfi -i "sine=frequency=${frequency}:sample_rate=48000:duration=65" \
    -map 0:v:0 -map 1:a:0 -c:v libx264 -preset ultrafast -crf 34 \
    -pix_fmt yuv420p -c:a aac -b:a 64k -shortest "$output"
}

prepare_preview_fixtures() {
  local directory=$1
  local font
  mkdir -p "$directory"
  font=$(find_preview_font)
  cp "$font" "$directory/font.ttf"

  make_fixture_video \
    'testsrc2=size=320x180:rate=12:duration=65' 330 "$directory/motion.mp4"
  make_fixture_video \
    'color=c=0xC84630:size=320x180:rate=12:duration=65' 440 "$directory/red.mp4"
  make_fixture_video \
    'color=c=0x168AAD:size=320x180:rate=12:duration=65' 550 "$directory/green.mp4"
  make_fixture_video \
    'color=c=0x5A4FCF:size=320x180:rate=12:duration=65' 660 "$directory/blue.mp4"

  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i 'sine=frequency=220:sample_rate=48000:duration=65' \
    -c:a aac -b:a 64k "$directory/audio.m4a"
  make_fixture_image 0x20262E 0x4ADE80 320x180 "$directory/background.png"
  make_fixture_image 0xF4C95D 0x20262E 180x180 "$directory/logo.png"
  make_fixture_image 0xF7F7F2 0xC84630 640x160 "$directory/caption.png"
  make_fixture_image 0x168AAD 0xF7F7F2 320x320 "$directory/widget.png"
  write_preview_luts "$directory"
}

make_fixture_image() {
  local background=$1
  local grid=$2
  local size=$3
  local output=$4
  ffmpeg -nostdin -hide_banner -loglevel error -y \
    -f lavfi -i "color=c=${background}:size=${size}:duration=1" \
    -vf "drawgrid=width=32:height=32:color=${grid}@0.8:thickness=2" \
    -frames:v 1 -update 1 "$output"
}

fixture_for_material() {
  local kind=$1
  local uri=$2
  local fixtures=$3
  local name
  name=$(basename "$uri")
  case "$kind:$name" in
    video:*clip-a*) printf '%s/red.mp4\n' "$fixtures" ;;
    video:*clip-b*) printf '%s/green.mp4\n' "$fixtures" ;;
    video:*clip-c*) printf '%s/blue.mp4\n' "$fixtures" ;;
    video:*) printf '%s/motion.mp4\n' "$fixtures" ;;
    audio:*) printf '%s/audio.m4a\n' "$fixtures" ;;
    image:*logo*) printf '%s/logo.png\n' "$fixtures" ;;
    image:*caption*) printf '%s/caption.png\n' "$fixtures" ;;
    image:*widget*) printf '%s/widget.png\n' "$fixtures" ;;
    image:*) printf '%s/background.png\n' "$fixtures" ;;
    font:*) printf '%s/font.ttf\n' "$fixtures" ;;
    lut1d:*|lut_1d:*|lut-1d:*) printf '%s/lut1d.cube\n' "$fixtures" ;;
    lut3d:*|lut_3d:*|lut-3d:*) printf '%s/lut3d.cube\n' "$fixtures" ;;
    *) preview_die "no fixture for material kind '$kind' at '$uri'" ;;
  esac
}

materialize_preview_assets() {
  local project=$1
  local canonical=$2
  local fixtures=$3
  local kind uri target fixture
  while IFS=$'\t' read -r kind uri; do
    case "$uri" in
      ""|/*|../*|*/../*|*/..) preview_die "unsafe material URI: $uri" ;;
    esac
    target="$project/$uri"
    if [[ -L "$target" ]]; then
      preview_die "material must not be a symlink: $target"
    fi
    [[ -f "$target" ]] && continue
    fixture=$(fixture_for_material "$kind" "$uri" "$fixtures")
    mkdir -p "$(dirname "$target")"
    cp "$fixture" "$target"
  done < <(jq -r '.project.materials[] | select(.source.type == "file") | [.kind, .source.uri] | @tsv' "$canonical")
}

pad_preview_audio() {
  local project=$1
  local canonical=$2
  local minimum=${3:-65}
  local uri target duration temporary
  while IFS= read -r uri; do
    target="$project/$uri"
    duration=$(ffprobe -v error -show_entries format=duration \
      -of default=noprint_wrappers=1:nokey=1 "$target")
    if awk -v actual="$duration" -v required="$minimum" \
      'BEGIN { exit !(actual < required) }'; then
      temporary="${target}.preview-audio"
      ffmpeg -nostdin -hide_banner -loglevel error -y -i "$target" \
        -af apad -t "$minimum" -c:a aac -b:a 96k -f ipod "$temporary"
      mv "$temporary" "$target"
    fi
  done < <(jq -r '.project.materials[] | select(.kind == "audio" and .source.type == "file") | .source.uri' "$canonical")
}
