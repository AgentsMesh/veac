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
