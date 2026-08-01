#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
source "$ROOT/scripts/example-preview-artifacts.sh"
source "$ROOT/scripts/example-preview-publish.sh"
FILTER="$ROOT/scripts/example-preview.jq"
tmp=$(mktemp -d)
output="$ROOT/examples-preview/.contract-test.$$"
trap 'rm -rf "$tmp" "$output" "$output.staging.$$" "$output.previous.$$"' EXIT

fail() {
  echo "example preview contract test failed: $*" >&2
  exit 1
}

cat > "$tmp/project.json" <<'JSON'
{"project":{"sequences":[{"settings":{"width":1280,"height":720,"frame_rate":{"numerator":30,"denominator":1}},"tracks":[{"kind":"visual","clips":[{"id":"keep","record_range":{"start":{"value":0,"timescale":1},"duration":{"value":4,"timescale":1}},"source":{"type":"generated"}},{"id":"drop","record_range":{"start":{"value":3,"timescale":1},"duration":{"value":1,"timescale":1}},"source":{"type":"generated"}}]}],"applies":[{"id":"keep-apply","record_range":{"start":{"value":0,"timescale":1},"duration":{"value":4,"timescale":1}},"target":{"type":"item_set","item_ids":["keep","drop"]}},{"id":"drop-apply","record_range":{"start":{"value":3,"timescale":1},"duration":{"value":1,"timescale":1}},"target":{"type":"item_set","item_ids":["drop"]}}]}],"relations":[{"kind":{"type":"group","members":[{"type":"item","item_id":"drop"}]}},{"kind":{"type":"group","members":[{"type":"item","item_id":"keep"}]}}],"annotations":[{"target":{"type":"clip","clip_id":"drop"}},{"target":{"type":"clip","clip_id":"keep"}}],"render_configs":[{"id":"out_preview","raster":{"width":1280,"height":720,"frame_rate":{"numerator":30,"denominator":1},"captions":"discard"},"deliverables":[]}],"materials":[]}}
JSON

jq --argjson edge 240 --argjson fps 12 \
  --argjson window '{"start_seconds":0,"duration_seconds":2}' \
  -f "$FILTER" "$tmp/project.json" > "$tmp/preview.json"
jq -e '
  .project.sequences[0].tracks[0].clips | length == 1 and
  .[0].id == "keep" and .[0].record_range.duration.value == 2
' "$tmp/preview.json" >/dev/null || fail "catalog window did not trim clips"
jq -e '
  .project.sequences[0].settings == {
    "width": 1280, "height": 720,
    "frame_rate": {"numerator": 12, "denominator": 1}
  } and
  (.project.render_configs[] | select(.id == "out_preview") | .raster |
    .width == 240 and .height == 134 and
    .frame_rate == {"numerator": 12, "denominator": 1})
' "$tmp/preview.json" >/dev/null || fail "preview changed spatial authoring dimensions"
jq -e '
  (.project.sequences[0].applies | length) == 1 and
  .project.sequences[0].applies[0].target.item_ids == ["keep"] and
  (.project.relations | length) == 1 and
  (.project.annotations | length) == 1
' "$tmp/preview.json" >/dev/null || fail "window left dangling references"
if jq --argjson edge 240 --argjson fps 12 \
  --argjson window '{"start_seconds":1,"duration_seconds":2}' \
  -f "$FILTER" "$tmp/project.json" >/dev/null 2>&1; then
  fail "preview filter accepted a nonzero window"
fi

cat > "$tmp/source-curve.json" <<'JSON'
{"project":{"sequences":[{"settings":{"width":1280,"height":720,"frame_rate":{"numerator":30,"denominator":1}},"tracks":[{"kind":"visual","clips":[{"id":"curve","record_range":{"start":{"value":0,"timescale":1},"duration":{"value":9,"timescale":1}},"source_mapping":{"time_map":{"type":"curve","segments":[{"record_duration":{"value":3,"timescale":1},"source_start":{"value":0,"timescale":1},"source_end":{"value":3,"timescale":1},"interpolation":"linear"},{"record_duration":{"value":6,"timescale":1},"source_start":{"value":3,"timescale":1},"source_end":{"value":15,"timescale":1},"interpolation":"linear"}]}},"source":{"type":"generated"}}]}],"applies":[]}],"relations":[],"annotations":[],"render_configs":[{"id":"out_preview","raster":{"width":1280,"height":720,"frame_rate":{"numerator":30,"denominator":1},"captions":"discard"},"deliverables":[]}],"materials":[]}}
JSON
jq --argjson edge 240 --argjson fps 12 \
  --argjson window '{"start_seconds":0,"duration_seconds":5}' \
  -f "$FILTER" "$tmp/source-curve.json" > "$tmp/source-curve-preview.json"
jq -e '
  .project.sequences[0].tracks[0].clips[0] as $clip |
  $clip.record_range.duration.value == 5 and
  ($clip.source_mapping.time_map.segments | map(.record_duration.value)) == [3, 2] and
  $clip.source_mapping.time_map.segments[1].source_end.value == 7
' "$tmp/source-curve-preview.json" >/dev/null || fail "window did not trim source curve"

cat > "$tmp/font-source.veac" <<'VEAC'
project font-preview {
  font family "Inter";
  fallback-font family "Arial";
  sequence main {}
  entry sequence main;
}
VEAC
cp "$tmp/font-source.veac" "$tmp/font-source.original.veac"
bash "$ROOT/scripts/prepare-example-source.sh" "$tmp/font-source.veac" "$tmp/font-preview.veac"
cmp -s "$tmp/font-source.original.veac" "$tmp/font-source.veac" ||
  fail "source preparation mutated the authoring source"
grep -q 'font resource preview-font;' "$tmp/font-preview.veac" || fail "primary family was not normalized"
grep -q 'fallback-font resource preview-arabic-font;' "$tmp/font-preview.veac" ||
  fail "fallback family lost its Arabic-capable resource"
grep -q 'resource font preview-font' "$tmp/font-preview.veac" || fail "font fixture resource is missing"
grep -q 'resource font preview-arabic-font' "$tmp/font-preview.veac" ||
  fail "Arabic font fixture resource is missing"
if grep -Eq '^[[:space:]]*delivery[[:space:]]' "$tmp/font-preview.veac"; then
  fail "source preparation injected delivery semantics"
fi
if grep -q 'font family' "$tmp/font-preview.veac"; then
  fail "family font escaped deterministic preview normalization"
fi
bash "$ROOT/scripts/prepare-example-source.sh" \
  "$tmp/font-preview.veac" "$tmp/font-preview.twice.veac"
cmp -s "$tmp/font-preview.veac" "$tmp/font-preview.twice.veac" ||
  fail "source preparation is not idempotent"
[[ $(grep -c 'resource font preview-font' "$tmp/font-preview.veac") == 1 ]] ||
  fail "preview font resource was injected more than once"
[[ $(grep -c 'resource font preview-arabic-font' "$tmp/font-preview.veac") == 1 ]] ||
  fail "Arabic preview font resource was injected more than once"
if bash "$ROOT/scripts/prepare-example-source.sh" \
    "$tmp/font-source.veac" "$tmp/font-source.veac" >/dev/null 2>&1; then
  fail "source preparation accepted in-place output"
fi
bash "$ROOT/scripts/prepare-example-source.sh" \
  "$ROOT/examples/delivery-codec-matrix/main.veac" "$tmp/codec-matrix.preview.veac"
grep -q 'font resource preview-font;' "$tmp/codec-matrix.preview.veac" ||
  fail "codec-matrix caption has no deterministic preview font"

mkdir -p "$tmp/rendered"
printf '{}\n' > "$tmp/canonical.json"
printf '{}\n' > "$tmp/preview.json"
printf '{"output":{"render_config_id":"out_captions"}}\n' > "$tmp/plan.json"
printf 'WEBVTT\n' > "$tmp/rendered/captions.vtt"
jq '.project = {"render_configs":[{"id":"out_captions","deliverables":[
  {"id":"dlv_captions","target":{"type":"file","name":"captions.vtt"},"kind":{"type":"caption_sidecar"}}
]}]}' "$tmp/canonical.json" > "$tmp/canonical.next"
mv "$tmp/canonical.next" "$tmp/canonical.json"
cp "$tmp/canonical.json" "$tmp/preview.json"
target='{"expected_artifacts":[{"kind":"canonical_project"},{"kind":"preview_canonical_project"},{"kind":"preview_resolved_plan"},{"kind":"authoring_delivery","id":"captions","artifact_ids":["captions"]}]}'
verify_expected_preview_artifacts "$target" "$tmp/canonical.json" \
  "$tmp/preview.json" "$tmp/plan.json" "$tmp/rendered" || fail "valid artifacts were rejected"
missing_config='{"expected_artifacts":[{"kind":"canonical_project"},{"kind":"preview_canonical_project"},{"kind":"preview_resolved_plan"},{"kind":"authoring_delivery","id":"missing","artifact_ids":["captions"]}]}'
if verify_expected_preview_artifacts "$missing_config" "$tmp/canonical.json" \
  "$tmp/preview.json" "$tmp/plan.json" "$tmp/rendered" >/dev/null 2>&1; then
  fail "missing authoring delivery was accepted"
fi
missing_artifact='{"expected_artifacts":[{"kind":"canonical_project"},{"kind":"preview_canonical_project"},{"kind":"preview_resolved_plan"},{"kind":"authoring_delivery","id":"captions","artifact_ids":["missing"]}]}'
if verify_expected_preview_artifacts "$missing_artifact" "$tmp/canonical.json" \
  "$tmp/preview.json" "$tmp/plan.json" "$tmp/rendered" >/dev/null 2>&1; then
  fail "missing delivery artifact was accepted"
fi
rm "$tmp/rendered/captions.vtt"
if verify_expected_preview_artifacts "$target" "$tmp/canonical.json" \
  "$tmp/preview.json" "$tmp/plan.json" "$tmp/rendered" >/dev/null 2>&1; then
  fail "missing expected deliverable was accepted"
fi

mkdir -p "$output"
printf 'old\n' > "$output/index.html"
create_preview_staging "$ROOT" "$output"
printf 'new\n' > "$PREVIEW_STAGING/index.html"
publish_preview_staging "$ROOT" "$output"
[[ $(< "$output/index.html") == new ]] || fail "staged output was not published"

create_preview_staging "$ROOT" "$output"
rm -rf "$PREVIEW_STAGING"
printf 'not-a-directory\n' > "$PREVIEW_STAGING"
failed_staging=$PREVIEW_STAGING
if publish_preview_staging "$ROOT" "$output" >/dev/null 2>&1; then
  fail "invalid staging content was published"
fi
[[ $(< "$output/index.html") == new ]] || fail "failed publish replaced prior output"
cleanup_preview_transaction "$ROOT" "$output"
[[ ! -e "$failed_staging" ]] || fail "failed staging was not cleaned"

printf 'outside\n' > "$tmp/outside"
if remove_preview_scratch "$ROOT" "$output" "$tmp/outside" >/dev/null 2>&1; then
  fail "cross-root scratch removal was accepted"
fi
[[ -f "$tmp/outside" ]] || fail "cross-root path was removed"

bash "$ROOT/scripts/tests/example-preview-package-contracts.sh"

echo "Example preview contract tests passed."
