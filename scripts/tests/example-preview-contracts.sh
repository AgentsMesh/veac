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
jq '.project.render_configs[0].raster.width = 320 |
  .project.render_configs[0].raster.height = 180' "$tmp/project.json" >"$tmp/small.json"
jq --argjson edge 480 --argjson fps 12 --argjson window null \
  -f "$FILTER" "$tmp/small.json" >"$tmp/small-preview.json"
jq -e '.project.render_configs[0].raster | .width == 320 and .height == 180 and
  .frame_rate == {"numerator":12,"denominator":1}' "$tmp/small-preview.json" \
  >/dev/null || fail "preview policy upscaled a smaller delivery"
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

jq '.project.annotations = [{style: {
  font: {type:"family", family:"Inter"},
  fallback_fonts: [{type:"family", family:"Noto Sans Arabic"}],
  spans: [{font:{type:"family", family:"Arial"}},
    {font:{type:"family", family:"Noto Sans Arabic"}}]
}}]' "$tmp/project.json" >"$tmp/font-authoring.json"
jq --argjson edge 240 --argjson fps 12 --argjson window null \
  -f "$FILTER" "$tmp/font-authoring.json" >"$tmp/font-preview.json"
jq -e '
  .project.annotations[0].style.font ==
    {type:"material", material_id:"med_preview-font"} and
  .project.annotations[0].style.spans[0].font ==
    {type:"material", material_id:"med_preview-font"} and
  .project.annotations[0].style.spans[1].font ==
    {type:"material", material_id:"med_preview-arabic-font"} and
  .project.annotations[0].style.fallback_fonts == [
    {type:"material", material_id:"med_preview-arabic-font"}] and
  ([.project.materials[] | {id, uri:.source.uri}] | sort_by(.id)) == [
    {id:"med_preview-arabic-font", uri:"assets/preview-arabic-font.ttf"},
    {id:"med_preview-font", uri:"assets/preview-font.ttf"}] and
  ([.project.materials[] | select(.id | startswith("med_preview-"))] |
    all(.authorship == null and (has("metadata") | not)))
' "$tmp/font-preview.json" >/dev/null || fail "canonical family fonts were not adapted"
jq -e '[.. | objects | select(.type? == "family" and has("family"))] | length == 4' \
  "$tmp/font-authoring.json" >/dev/null || fail "preview adaptation mutated authoring IR"
jq '.project.materials = [{id:"med_preview-font"}]' "$tmp/font-authoring.json" \
  >"$tmp/font-collision.json"
if jq --argjson edge 240 --argjson fps 12 --argjson window null \
    -f "$FILTER" "$tmp/font-collision.json" >/dev/null 2>&1; then
  fail "reserved preview font collision was accepted"
fi

mkdir -p "$tmp/rendered"
printf '{}\n' > "$tmp/canonical.json"
printf '{}\n' > "$tmp/preview.json"
config=out_4a4f4ce03f87
printf '{"output":{"render_config_id":"%s"}}\n' "$config" > "$tmp/plan.json"
printf 'WEBVTT\n' > "$tmp/rendered/captions.vtt"
jq --arg config "$config" '.project = {
  "authorship":{"entity":{"logical_path":["demo"],"events":[]},
    "multicam_groups":[],"annotations":[],"deliveries":[{"render_config_id":$config,
    "entity":{"logical_path":["demo","captions"],"events":[]}}]},
  "render_configs":[{"id":$config,"deliverables":[
    {"id":"dlv_92f3","target":{"type":"file","name":"captions.vtt"},
     "kind":{"type":"caption_sidecar"}}
  ]}]}' "$tmp/canonical.json" > "$tmp/canonical.next"
mv "$tmp/canonical.next" "$tmp/canonical.json"
cp "$tmp/canonical.json" "$tmp/preview.json"
target='{"expected_artifacts":[{"kind":"canonical_project"},{"kind":"preview_canonical_project"},{"kind":"preview_resolved_plan"},{"kind":"delivery","logical_key":"captions","artifacts":[{"kind":"caption_sidecar","target_type":"file","target":"captions.vtt"}]}]}'
verify_expected_preview_artifacts "$target" "$tmp/canonical.json" \
  "$tmp/preview.json" "$tmp/plan.json" "$tmp/rendered" || fail "valid artifacts were rejected"
missing_config='{"expected_artifacts":[{"kind":"canonical_project"},{"kind":"preview_canonical_project"},{"kind":"preview_resolved_plan"},{"kind":"delivery","logical_key":"missing","artifacts":[{"kind":"caption_sidecar","target_type":"file","target":"captions.vtt"}]}]}'
if verify_expected_preview_artifacts "$missing_config" "$tmp/canonical.json" \
  "$tmp/preview.json" "$tmp/plan.json" "$tmp/rendered" >/dev/null 2>&1; then
  fail "missing delivery was accepted"
fi
missing_artifact='{"expected_artifacts":[{"kind":"canonical_project"},{"kind":"preview_canonical_project"},{"kind":"preview_resolved_plan"},{"kind":"delivery","logical_key":"captions","artifacts":[{"kind":"caption_sidecar","target_type":"file","target":"missing.vtt"}]}]}'
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
