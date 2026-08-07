#!/usr/bin/env bash

# shellcheck source=example-preview-luts.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-luts.sh"
# shellcheck source=example-preview-media-fixtures.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-media-fixtures.sh"

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
    /usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc \
    /usr/share/fonts/truetype/wqy/wqy-zenhei.ttc \
    '/System/Library/Fonts/STHeiti Medium.ttc' \
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

find_preview_arabic_font() {
  local candidate
  for candidate in \
    "${VEAC_PREVIEW_ARABIC_FONT:-}" \
    /System/Library/Fonts/GeezaPro.ttc \
    /System/Library/Fonts/Supplemental/GeezaPro.ttc \
    /usr/share/fonts/opentype/noto/NotoSansArabic-Regular.ttf \
    /usr/share/fonts/opentype/noto/NotoNaskhArabic-Regular.ttf \
    /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf \
    /usr/share/fonts/truetype/freefont/FreeSans.ttf; do
    [[ -z "$candidate" || ! -f "$candidate" ]] && continue
    printf '%s\n' "$candidate"
    return
  done
  preview_die "set VEAC_PREVIEW_ARABIC_FONT to an Arabic-capable font"
}

prepare_preview_fixtures() {
  local directory=$1
  local font arabic_font
  mkdir -p "$directory"
  font=$(find_preview_font)
  cp "$font" "$directory/font.ttf"
  arabic_font=$(find_preview_arabic_font)
  cp "$arabic_font" "$directory/arabic-font.ttf"

  write_preview_media_fixtures "$directory"
  write_preview_luts "$directory"
}

fixture_for_material() {
  local kind=$1
  local uri=$2
  local fixtures=$3
  local name
  name=$(basename "$uri")
  case "$kind:$name" in
    video:shaky.mp4) printf '%s/shaky.mp4\n' "$fixtures" ;;
    video:*clip-a*) printf '%s/multicam-host.mp4\n' "$fixtures" ;;
    video:*clip-b*) printf '%s/multicam-guest.mp4\n' "$fixtures" ;;
    video:*clip-c*) printf '%s/blue.mp4\n' "$fixtures" ;;
    video:*) printf '%s/motion.mp4\n' "$fixtures" ;;
    audio:codec-tone.m4a) printf '%s/codec-tone.m4a\n' "$fixtures" ;;
    audio:*voice*) printf '%s/voice.wav\n' "$fixtures" ;;
    audio:*music*) printf '%s/music.wav\n' "$fixtures" ;;
    audio:*key*) printf '%s/key.wav\n' "$fixtures" ;;
    audio:*) printf '%s/audio.m4a\n' "$fixtures" ;;
    image:effects-plate.png) printf '%s/effects-plate.png\n' "$fixtures" ;;
    image:*logo*) printf '%s/logo.png\n' "$fixtures" ;;
    image:*caption*) printf '%s/caption.png\n' "$fixtures" ;;
    image:*widget*) printf '%s/widget.png\n' "$fixtures" ;;
    image:*) printf '%s/background.png\n' "$fixtures" ;;
    font:*preview-arabic-font*) printf '%s/arabic-font.ttf\n' "$fixtures" ;;
    font:*) printf '%s/font.ttf\n' "$fixtures" ;;
    lut1d:tone-curve.cube|lut_1d:tone-curve.cube|lut-1d:tone-curve.cube) \
      printf '%s/tone-curve.cube\n' "$fixtures" ;;
    lut3d:cinematic.cube|lut_3d:cinematic.cube|lut-3d:cinematic.cube) \
      printf '%s/cinematic.cube\n' "$fixtures" ;;
    lut1d:*|lut_1d:*|lut-1d:*) printf '%s/lut1d.cube\n' "$fixtures" ;;
    lut3d:*|lut_3d:*|lut-3d:*) printf '%s/lut3d.cube\n' "$fixtures" ;;
    *) preview_die "no fixture for material kind '$kind' at '$uri'" ;;
  esac
}

materialize_preview_assets() {
  local project=$1
  local canonical=$2
  local fixtures=$3
  local source_root=${4:-}
  local kind uri pinned target fixture authored
  jq -e '.project.materials | type == "array"' "$canonical" >/dev/null || {
    preview_die "canonical materials must be an array: $canonical"
    return 1
  }
  while IFS=$'\t' read -r kind uri pinned; do
    case "$uri" in
      ""|/*|../*|*/../*|*/..)
        preview_die "unsafe material URI: $uri"
        return 1 ;;
    esac
    target="$project/$uri"
    if [[ -L "$target" ]]; then
      preview_die "material must not be a symlink: $target"
      return 1
    fi
    [[ -f "$target" ]] && continue
    if [[ $pinned == true ]]; then
      [[ -n $source_root && -d $source_root && ! -L $source_root ]] || {
        preview_die "pinned material requires a regular source root: $uri"
        return 1
      }
      if find "$source_root" -type l -print -quit | grep -q .; then
        preview_die "pinned material source graph contains a symlink: $uri"
        return 1
      fi
      authored="$source_root/$uri"
      [[ -f $authored && ! -L $authored ]] || {
        preview_die "pinned material is missing from the example: $uri"
        return 1
      }
      mkdir -p "$(dirname "$target")" || return 1
      cp "$authored" "$target" || return 1
      continue
    fi
    fixture=$(fixture_for_material "$kind" "$uri" "$fixtures") || return 1
    [[ -f "$fixture" && ! -L "$fixture" ]] || {
      preview_die "fixture must be a regular non-symlink file: $fixture"
      return 1
    }
    mkdir -p "$(dirname "$target")" || return 1
    cp "$fixture" "$target" || return 1
  done < <(jq -r '.project.materials[] | select(.source.type == "file") |
    [.kind, .source.uri, (.identity != null)] | @tsv' "$canonical")
}

pad_preview_audio() {
  local project=$1
  local canonical=$2
  local minimum=${3:-65}
  local uri target duration temporary
  while IFS= read -r uri; do
    target="$project/$uri"
    duration=$(ffprobe -v error -show_entries format=duration \
      -of default=noprint_wrappers=1:nokey=1 "$target") || return 1
    if awk -v actual="$duration" -v required="$minimum" \
      'BEGIN { exit !(actual < required) }'; then
      temporary="${target}.preview-audio"
      ffmpeg -nostdin -hide_banner -loglevel error -y -i "$target" \
        -af apad -t "$minimum" -c:a aac -b:a 96k -f ipod "$temporary" || return 1
      mv "$temporary" "$target" || return 1
    fi
  done < <(jq -r '.project.materials[] | select(
    .kind == "audio" and .source.type == "file" and .identity == null
  ) | .source.uri' "$canonical")
}
