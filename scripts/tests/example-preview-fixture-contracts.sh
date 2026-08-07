#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
source "$ROOT/scripts/example-preview-fixtures.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
FIXTURES="$TMP/fixtures"
PROJECT="$TMP/project"
mkdir -p "$FIXTURES" "$PROJECT"

fail() {
  echo "preview fixture contract failed: $*" >&2
  exit 1
}

ffmpeg() {
  local output=${!#}
  printf 'generated:%s\n' "$(basename "$output")" >"$output"
}

write_preview_media_fixtures "$FIXTURES"
write_preview_luts "$FIXTURES"
for file in effects-plate.png shaky.mp4 codec-tone.m4a cinematic.cube tone-curve.cube; do
  [[ -s $FIXTURES/$file && ! -L $FIXTURES/$file ]] ||
    fail "dedicated fixture was not generated: $file"
done

cp "$FIXTURES/cinematic.cube" "$TMP/cinematic.expected"
printf 'stale\n' >"$FIXTURES/cinematic.cube"
write_preview_luts "$FIXTURES"
cmp -s "$TMP/cinematic.expected" "$FIXTURES/cinematic.cube" ||
  fail "LUT generation retained stale content"

cat >"$TMP/canonical.json" <<'JSON'
{"project":{"materials":[
  {"kind":"lut3d","source":{"type":"file","uri":"assets/cinematic.cube"}},
  {"kind":"lut1d","source":{"type":"file","uri":"assets/tone-curve.cube"}},
  {"kind":"image","source":{"type":"file","uri":"assets/effects-plate.png"}},
  {"kind":"video","source":{"type":"file","uri":"assets/shaky.mp4"}},
  {"kind":"audio","source":{"type":"file","uri":"assets/codec-tone.m4a"}}
]}}
JSON
materialize_preview_assets "$PROJECT" "$TMP/canonical.json" "$FIXTURES"
for file in cinematic.cube tone-curve.cube effects-plate.png shaky.mp4 codec-tone.m4a; do
  cmp -s "$FIXTURES/$file" "$PROJECT/assets/$file" ||
    fail "dedicated fixture mapping drifted: $file"
done

SOURCE="$TMP/source"
PINNED_PROJECT="$TMP/pinned-project"
mkdir -p "$SOURCE/assets" "$PINNED_PROJECT"
printf 'authored font\n' >"$SOURCE/assets/font.ttf"
cat >"$TMP/pinned.json" <<'JSON'
{"project":{"materials":[{"kind":"font","identity":{"algorithm":"sha256","digest":"00"},
"source":{"type":"file","uri":"assets/font.ttf"}}]}}
JSON
materialize_preview_assets "$PINNED_PROJECT" "$TMP/pinned.json" "$FIXTURES" "$SOURCE"
cmp -s "$SOURCE/assets/font.ttf" "$PINNED_PROJECT/assets/font.ttf" ||
  fail "pinned material did not retain the authored bytes"
if materialize_preview_assets "$TMP/missing-root" "$TMP/pinned.json" "$FIXTURES" \
    >/dev/null 2>&1; then
  fail "pinned material without a source root was accepted"
fi
ln -s "$TMP/cinematic.expected" "$SOURCE/assets/link.ttf"
jq '.project.materials[0].source.uri = "assets/link.ttf"' \
  "$TMP/pinned.json" >"$TMP/pinned-link.json"
if materialize_preview_assets "$TMP/pinned-link" "$TMP/pinned-link.json" \
    "$FIXTURES" "$SOURCE" >/dev/null 2>&1; then
  fail "symlinked authored material was accepted"
fi
rm "$SOURCE/assets/link.ttf"
mkdir -p "$TMP/outside"
printf 'outside font\n' >"$TMP/outside/font.ttf"
ln -s "$TMP/outside" "$SOURCE/assets/link-dir"
jq '.project.materials[0].source.uri = "assets/link-dir/font.ttf"' \
  "$TMP/pinned.json" >"$TMP/pinned-parent-link.json"
if materialize_preview_assets "$TMP/pinned-parent-link" \
    "$TMP/pinned-parent-link.json" "$FIXTURES" "$SOURCE" >/dev/null 2>&1; then
  fail "authored material beneath a symlinked directory was accepted"
fi
rm "$SOURCE/assets/link-dir"

ffprobe() { printf '1.0\n'; }
printf 'unpinned\n' >"$PROJECT/assets/unpinned.wav"
printf 'pinned\n' >"$PROJECT/assets/pinned.wav"
cat >"$TMP/audio-padding.json" <<'JSON'
{"project":{"materials":[
  {"kind":"audio","identity":null,"source":{"type":"file","uri":"assets/unpinned.wav"}},
  {"kind":"audio","identity":{"algorithm":"sha256","digest":"00"},"source":{"type":"file","uri":"assets/pinned.wav"}}
]}}
JSON
pad_preview_audio "$PROJECT" "$TMP/audio-padding.json" 65
rg -q '^generated:unpinned.wav.preview-audio$' "$PROJECT/assets/unpinned.wav" ||
  fail "unpinned preview audio was not padded"
rg -q '^pinned$' "$PROJECT/assets/pinned.wav" ||
  fail "source-authored audio identity was mutated"

rm "$FIXTURES/shaky.mp4" "$PROJECT/assets/shaky.mp4"
if materialize_preview_assets "$PROJECT" "$TMP/canonical.json" "$FIXTURES" \
    >/dev/null 2>&1; then
  fail "missing dedicated fixture was accepted"
fi
printf 'source\n' >"$TMP/shaky-source"
ln -s "$TMP/shaky-source" "$FIXTURES/shaky.mp4"
if materialize_preview_assets "$PROJECT" "$TMP/canonical.json" "$FIXTURES" \
    >/dev/null 2>&1; then
  fail "symlinked dedicated fixture was accepted"
fi

jq '.project.materials = [{"kind":"video","source":{"type":"file","uri":"../escape.mp4"}}]' \
  "$TMP/canonical.json" >"$TMP/unsafe.json"
if materialize_preview_assets "$PROJECT" "$TMP/unsafe.json" "$FIXTURES" \
    >/dev/null 2>&1; then
  fail "unsafe material URI was accepted"
fi
jq '.project.materials = [{"kind":"archive","source":{"type":"file","uri":"assets/a.zip"}}]' \
  "$TMP/canonical.json" >"$TMP/unknown.json"
if materialize_preview_assets "$PROJECT" "$TMP/unknown.json" "$FIXTURES" \
    >/dev/null 2>&1; then
  fail "unsupported material kind was accepted"
fi

echo "example preview fixture contracts passed"
