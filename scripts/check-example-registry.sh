#!/usr/bin/env bash
set -euo pipefail

ROOT="${VEAC_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
MECHANISMS="${1:?combined mechanism JSON is required}"
COMPOSITION="$ROOT/examples/catalog/mechanisms/transitions-composition.json"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

fail() {
  echo "example registry check failed: $*" >&2
  exit 1
}

compare() {
  local name="$1"
  local actual="$2"
  local catalog="$3"
  if ! diff -u "$actual" "$catalog" > "$tmp/$name.diff"; then
    cat "$tmp/$name.diff" >&2
    fail "$name catalog differs from its Rust registry"
  fi
}

extract_enum() {
  local file="$1"
  local enum_name="$2"
  awk -v enum_name="$enum_name" '
    $0 ~ "pub enum " enum_name { inside = 1; next }
    inside && /^}/ { exit }
    inside {
      value = $0
      sub(/^[[:space:]]+/, "", value)
      sub(/[({,].*$/, "", value)
      sub(/[[:space:]]+$/, "", value)
      if (value !~ /^[A-Z][A-Za-z0-9]*$/) next
      snake = ""
      for (i = 1; i <= length(value); i++) {
        char = substr(value, i, 1)
        if (char ~ /[A-Z]/) {
          if (i > 1) snake = snake "_"
          snake = snake tolower(char)
        } else snake = snake char
      }
      print snake
    }
  ' "$file" | sort
}

{
  sed -n '/pub const fn type_name/,/^    }/p' \
    "$ROOT/crates/veac-ir/src/model/effect/catalog.rs" |
    sed -n 's/.*"\([^"]*\)".*/\1/p'
  sed -n 's/^pub const [A-Z0-9_]*_EFFECT_TYPE: &str = "\([^"]*\)";.*/\1/p' \
    "$ROOT/crates/veac-ir/src/plugin_registry.rs"
} | sort > "$tmp/effects-rust"
jq -r '.[] | select(.registry_key != null) | .registry_key' "$MECHANISMS" |
  sort > "$tmp/effects-catalog"
compare effect-registry "$tmp/effects-rust" "$tmp/effects-catalog"

extract_enum "$ROOT/crates/veac-ir/src/model/transition.rs" TransitionKind \
  > "$tmp/transitions-rust"
jq -r '.transition_enum_keys[]' "$COMPOSITION" | sort > "$tmp/transitions-catalog"
compare transition-enum "$tmp/transitions-rust" "$tmp/transitions-catalog"

extract_enum "$ROOT/crates/veac-ir/src/model/properties/composition.rs" BlendMode \
  > "$tmp/blends-rust"
jq -r '.blend_enum_keys[]' "$COMPOSITION" | sort > "$tmp/blends-catalog"
compare blend-enum "$tmp/blends-rust" "$tmp/blends-catalog"

enum_hyphen_keys() {
  extract_enum "$1" "$2" | tr '_' '-'
}

catalog_suffixes() {
  local prefix="$1"
  jq -r --arg prefix "$prefix" '
    .[].id | select(startswith($prefix)) | ltrimstr($prefix)
  ' "$MECHANISMS" | sort
}

ANIMATION="$ROOT/crates/veac-ir/src/model/properties/animation.rs"
MASK="$ROOT/crates/veac-ir/src/model/properties/mask.rs"
AUDIO="$ROOT/crates/veac-ir/src/model/properties/audio_processing.rs"
SOURCE_TIME="$ROOT/crates/veac-ir/src/model/source_time.rs"
enum_hyphen_keys "$ANIMATION" Interpolation > "$tmp/interpolation-rust"
catalog_suffixes animation.interpolation- > "$tmp/interpolation-catalog"
compare interpolation-enum "$tmp/interpolation-rust" "$tmp/interpolation-catalog"

enum_hyphen_keys "$MASK" MaskShape > "$tmp/masks-rust"
catalog_suffixes mask. > "$tmp/masks-catalog"
compare mask-enum "$tmp/masks-rust" "$tmp/masks-catalog"
enum_hyphen_keys "$MASK" TrackMatteMode > "$tmp/mattes-rust"
catalog_suffixes matte. | rg -v '^invert$' > "$tmp/mattes-catalog"
compare track-matte-enum "$tmp/mattes-rust" "$tmp/mattes-catalog"

enum_hyphen_keys "$AUDIO" AudioFadeCurve > "$tmp/fades-rust"
catalog_suffixes audio.fade-curve- > "$tmp/fades-catalog"
compare audio-fade-enum "$tmp/fades-rust" "$tmp/fades-catalog"
enum_hyphen_keys "$AUDIO" AudioProcessorKind |
  sed -e 's/^parametric-eq$/equalizer/' -e 's/^gate$/noise-gate/' \
    -e 's/^loudness$/loudness-target/' | sort > "$tmp/processors-rust"
jq -r '.[].id | select(. == "audio.equalizer" or . == "audio.high-pass" or
  . == "audio.low-pass" or . == "audio.compressor" or . == "audio.limiter" or
  . == "audio.noise-gate" or . == "audio.loudness-target") | ltrimstr("audio.")' \
  "$MECHANISMS" | sort > "$tmp/processors-catalog"
compare audio-processor-enum "$tmp/processors-rust" "$tmp/processors-catalog"

enum_hyphen_keys "$SOURCE_TIME" SourceOutOfRangePolicy |
  sed 's/^strict$/reject/' > "$tmp/out-of-range-rust"
catalog_suffixes source-time.out-of-range. > "$tmp/out-of-range-catalog"
compare source-out-of-range-enum "$tmp/out-of-range-rust" "$tmp/out-of-range-catalog"

GENERATOR_MODEL="$ROOT/crates/veac-ir/src/model/generator.rs"
enum_hyphen_keys "$GENERATOR_MODEL" Gradient > "$tmp/gradients-rust"
catalog_suffixes generator.gradient. | rg -v '^(custom-geometry|multi-stop)$' \
  > "$tmp/gradients-catalog"
compare gradient-enum "$tmp/gradients-rust" "$tmp/gradients-catalog"
enum_hyphen_keys "$GENERATOR_MODEL" VectorGeometry > "$tmp/shapes-rust"
catalog_suffixes generator.shape. > "$tmp/shapes-catalog"
compare vector-geometry-enum "$tmp/shapes-rust" "$tmp/shapes-catalog"

enum_hyphen_keys "$ROOT/crates/veac-ir/src/model/output.rs" OutputFormat \
  > "$tmp/containers-rust"
catalog_suffixes delivery.container. > "$tmp/containers-catalog"
compare output-format-enum "$tmp/containers-rust" "$tmp/containers-catalog"
enum_hyphen_keys "$ROOT/crates/veac-ir/src/model/output_video.rs" VideoCodec |
  sed -e 's/^pro-res$/prores/' -e 's/^dnx-hr$/dnxhr/' > "$tmp/video-codecs-rust"
catalog_suffixes delivery.video-codec. > "$tmp/video-codecs-catalog"
compare video-codec-enum "$tmp/video-codecs-rust" "$tmp/video-codecs-catalog"

enum_hyphen_keys "$ROOT/crates/veac-ir/src/model/output.rs" AudioCodec |
  sed -E 's/(pcm-s[0-9]+)-le/\1le/' > "$tmp/audio-codecs-rust"
enum_hyphen_keys "$ROOT/crates/veac-ir/src/model/output_delivery.rs" AudioFileEncoding \
  >> "$tmp/audio-codecs-rust"
sort -o "$tmp/audio-codecs-rust" "$tmp/audio-codecs-rust"
catalog_suffixes delivery.audio-codec. > "$tmp/audio-codecs-catalog"
compare audio-codec-enum "$tmp/audio-codecs-rust" "$tmp/audio-codecs-catalog"

AUX="$ROOT/crates/veac-ir/src/model/output_aux.rs"
enum_hyphen_keys "$AUX" ImageFormat > "$tmp/image-formats-rust"
catalog_suffixes delivery.image-format. > "$tmp/image-formats-catalog"
compare image-format-enum "$tmp/image-formats-rust" "$tmp/image-formats-catalog"
enum_hyphen_keys "$AUX" CaptionSidecarFormat > "$tmp/caption-formats-rust"
catalog_suffixes delivery.caption-format. > "$tmp/caption-formats-catalog"
compare caption-format-enum "$tmp/caption-formats-rust" "$tmp/caption-formats-catalog"
enum_hyphen_keys "$AUX" AudioStemFormat > "$tmp/stem-formats-rust"
catalog_suffixes delivery.audio-stem-format. > "$tmp/stem-formats-catalog"
compare audio-stem-format-enum "$tmp/stem-formats-rust" "$tmp/stem-formats-catalog"
enum_hyphen_keys "$AUX" VideoScope > "$tmp/scopes-rust"
catalog_suffixes delivery.scope. > "$tmp/scopes-catalog"
compare video-scope-enum "$tmp/scopes-rust" "$tmp/scopes-catalog"

TEXT="$ROOT/crates/veac-ir/src/model/properties/text_layout.rs"
enum_hyphen_keys "$TEXT" TextWritingMode > "$tmp/writing-rust"
catalog_suffixes text.layout.writing- | rg -v '^(horizontal|vertical)$' \
  > "$tmp/writing-catalog"
compare text-writing-enum "$tmp/writing-rust" "$tmp/writing-catalog"
enum_hyphen_keys "$TEXT" TextWrap > "$tmp/wrap-rust"
catalog_suffixes text.layout.wrap- > "$tmp/wrap-catalog"
compare text-wrap-enum "$tmp/wrap-rust" "$tmp/wrap-catalog"
enum_hyphen_keys "$TEXT" TextOverflow > "$tmp/overflow-rust"
catalog_suffixes text.layout.overflow- > "$tmp/overflow-catalog"
compare text-overflow-enum "$tmp/overflow-rust" "$tmp/overflow-catalog"
enum_hyphen_keys "$TEXT" HorizontalTextAlignment > "$tmp/horizontal-align-rust"
catalog_suffixes text.layout.align-horizontal- > "$tmp/horizontal-align-catalog"
compare horizontal-text-alignment-enum "$tmp/horizontal-align-rust" "$tmp/horizontal-align-catalog"
enum_hyphen_keys "$TEXT" VerticalTextAlignment > "$tmp/vertical-align-rust"
catalog_suffixes text.layout.align-vertical- > "$tmp/vertical-align-catalog"
compare vertical-text-alignment-enum "$tmp/vertical-align-rust" "$tmp/vertical-align-catalog"
enum_hyphen_keys "$TEXT" TextOrientation > "$tmp/orientation-rust"
catalog_suffixes text.layout.orientation- > "$tmp/orientation-catalog"
compare text-orientation-enum "$tmp/orientation-rust" "$tmp/orientation-catalog"
enum_hyphen_keys "$ROOT/crates/veac-ir/src/model/properties/color.rs" ColorStage \
  > "$tmp/color-stage-rust"
catalog_suffixes color.stage. > "$tmp/color-stage-catalog"
compare color-stage-enum "$tmp/color-stage-rust" "$tmp/color-stage-catalog"

echo "Example registry mappings are exact."
