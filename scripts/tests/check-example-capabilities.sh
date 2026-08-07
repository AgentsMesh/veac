#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT/scripts/check-example-capabilities.sh"
REGISTRY_CHECKER="$ROOT/scripts/check-example-registry.sh"
PRESENTATION_CHECKER="$ROOT/scripts/check-example-presentation.sh"
CATALOG="$ROOT/examples/capabilities.json"
tmp="$(mktemp -d)"
gallery_test="$ROOT/examples/catalog/.gallery-contract-test.$$.json"
mechanism_test="$ROOT/examples/catalog/.mechanism-contract-test.$$.json"
trap 'rm -rf "$tmp"; rm -f "$gallery_test" "$mechanism_test"' EXIT

expect_failure() {
  local name="$1"
  local catalog="$2"
  if "$CHECKER" "$catalog" >"$tmp/$name.out" 2>&1; then
    echo "expected checker failure: $name" >&2
    exit 1
  fi
}

jq '.capabilities[0].not_applicable = "English explanation"' "$CATALOG" \
  > "$tmp/English-capability-explanation-catalog.json"
expect_failure English-capability-explanation \
  "$tmp/English-capability-explanation-catalog.json"

expect_gallery_failure() {
  local name=$1
  local filter=$2
  local catalog="$tmp/$name-catalog.json"
  jq "$filter" "$ROOT/examples/catalog/gallery.json" > "$gallery_test"
  jq --arg gallery "examples/catalog/$(basename "$gallery_test")" \
    '.gallery_catalog = $gallery' "$CATALOG" > "$catalog"
  expect_failure "$name" "$catalog"
}

expect_mechanism_failure() {
  local name=$1
  local filter=$2
  local catalog="$tmp/$name-catalog.json"
  jq "$filter" "$ROOT/examples/catalog/mechanisms/delivery-workflows.json" \
    > "$mechanism_test"
  jq --arg fragment "examples/catalog/$(basename "$mechanism_test")" \
    '.mechanism_catalogs[0] = $fragment' "$CATALOG" > "$catalog"
  expect_failure "$name" "$catalog"
}

"$CHECKER" "$CATALOG" >/dev/null

jq '.examples[0].summary =
  "交付 MOV、HEVC Main10、HDR PQ、VP9 Opus、WebM、BT.2020 与 H.265。"' \
  "$ROOT/examples/catalog/gallery.json" >"$gallery_test"
jq --arg gallery "examples/catalog/$(basename "$gallery_test")" \
  '.gallery_catalog = $gallery' "$CATALOG" >"$tmp/standard-token-catalog.json"
"$CHECKER" "$tmp/standard-token-catalog.json" >/dev/null

jq '.capabilities |= .[1:]' "$CATALOG" > "$tmp/missing-id.json"
expect_failure missing-id "$tmp/missing-id.json"

jq '.capabilities += [.capabilities[0]]' "$CATALOG" > "$tmp/duplicate-id.json"
expect_failure duplicate-id "$tmp/duplicate-id.json"

jq '(.capabilities[] | select(.id == "P1-02")) |=
  (.example = null | del(.not_applicable))' "$CATALOG" > "$tmp/no-coverage.json"
expect_failure no-coverage "$tmp/no-coverage.json"

jq '(.capabilities[] | select(.id == "P1-02") | .example) =
  "examples/missing/main.veac"' "$CATALOG" > "$tmp/missing-example.json"
expect_failure missing-example "$tmp/missing-example.json"

jq '.gallery_catalog = "examples/catalog/missing-gallery.json"' \
  "$CATALOG" > "$tmp/missing-gallery.json"
expect_failure missing-gallery "$tmp/missing-gallery.json"

jq '.mechanism_catalogs[0] = "examples/catalog/missing-mechanisms.json"' \
  "$CATALOG" > "$tmp/missing-mechanisms.json"
expect_failure missing-mechanisms "$tmp/missing-mechanisms.json"

expect_gallery_failure legacy-schema '.schema_version = 1'
expect_gallery_failure missing-source-frontend 'del(.targets[0].frontend)'
expect_gallery_failure unknown-source-frontend '.targets[0].frontend = "dynamic"'
expect_gallery_failure missing-presentation-language 'del(.presentation_language)'
expect_gallery_failure English-presentation-title '.examples[0].title = "English title"'
expect_gallery_failure English-presentation-summary '.examples[0].summary = "English summary"'
expect_gallery_failure English-presentation-cue '.examples[0].checks[0].cue = "Timeline"'
expect_gallery_failure mixed-English-presentation-cue '.examples[0].checks[0].cue = "时间 - Timeline"'
expect_gallery_failure English-presentation-expect '.examples[0].checks[0].expect = "Visible result"'
expect_gallery_failure English-after-standard-tokens \
  '.examples[0].summary = "交付 MOV、HEVC Main10、HDR PQ、VP9 Opus、WebM、BT.2020 与 H.265 codec sentence。"'
expect_gallery_failure missing-delivery-capability \
  '(.targets[] | select(.id == "delivery-formats") | .capability_ids) = ["P1-30"]'
expect_gallery_failure missing-delivery-artifact \
  '(.targets[] | select(.id == "delivery-formats") |
    .expected_artifacts[] | select(.kind == "delivery") |
    .artifacts) |= map(select(.target != "stream"))'
expect_gallery_failure missing-delivery-presentation \
  '(.examples[] | select(.id == "delivery-formats") | .checks) |= .[:-1]'
expect_gallery_failure legacy-artifacts \
  '.targets[0] += {"expected_deliverables":["preview"]}'
expect_gallery_failure unknown-artifact \
  '.targets[0].expected_artifacts[0].kind = "unknown"'
expect_gallery_failure legacy-deliverable-artifact \
  '.targets[0].expected_artifacts[3].kind = "deliverable"'
expect_gallery_failure legacy-render-config-artifact \
  '.targets[0].expected_artifacts[3].kind = "render_config"'
expect_gallery_failure untyped-delivery \
  'del(.targets[0].expected_artifacts[3].logical_key)'
expect_gallery_failure missing-artifact-descriptors \
  'del(.targets[0].expected_artifacts[3].artifacts)'
expect_gallery_failure duplicate-artifact-descriptors \
  '.targets[0].expected_artifacts[3].artifacts +=
    [.targets[0].expected_artifacts[3].artifacts[0]]'
expect_gallery_failure artifact-extra-key \
  '.targets[0].expected_artifacts[0].label = "legacy"'
expect_gallery_failure nonzero-window \
  '.targets[0].preview_window.start_seconds = 1'
expect_gallery_failure cue-outside-preview \
  '.examples[0].checks[0].cue = "99-100 秒 - 线性裁剪"'
expect_gallery_failure missing-preview-canonical \
  '.targets[0].expected_artifacts |= map(select(.kind != "preview_canonical_project"))'
expect_gallery_failure missing-preview-plan \
  '.targets[0].expected_artifacts |= map(select(.kind != "preview_resolved_plan"))'
expect_gallery_failure legacy-source-plan \
  '.targets[0].expected_artifacts[2] = {"kind":"resolved_plan"}'
expect_gallery_failure workflow-artifact-on-source \
  '.targets[0].expected_artifacts[2] = {"kind":"probe_snapshot"}'
expect_gallery_failure partial-source-edit-evidence \
  '(.targets[] | select(.id == "programming-language") | .expected_artifacts) |= map(select(.kind != "source_index"))'
expect_gallery_failure missing-presentation 'del(.examples[0])'
expect_gallery_failure empty-presentation-checks '.examples[0].checks = []'
expect_gallery_failure blank-presentation-summary '.examples[0].summary = "   "'
expect_gallery_failure blank-presentation-cue '.examples[0].checks[0].cue = " "'
expect_gallery_failure extra-presentation-check-key \
  '.examples[0].checks[0].note = "not canonical"'
expect_mechanism_failure English-mechanism-title \
  '.mechanisms[0].title = "English title"'
expect_mechanism_failure English-mechanism-family \
  '.mechanisms[0].family = "English family"'
expect_mechanism_failure stale-delivery-example-evidence \
  '(.mechanisms[] | select(.id == "delivery.gif") | .evidence) =
    "crates/veac-codegen/src/unit_tests/emitter_tests/delivery_aux.rs"'

jq '(.capabilities[] | select(.id == "P1-32")) |=
  (.example = null | .not_applicable = "专业交付只通过运行时夹具验证。")' \
  "$CATALOG" > "$tmp/missing-delivery-example.json"
expect_failure missing-delivery-example "$tmp/missing-delivery-example.json"

presentation_root="$tmp/presentation"
mkdir -p "$presentation_root/examples/demo"
printf '%s\n' '{"examples":[{"source":"examples/demo/main.veac"}]}' \
  > "$presentation_root/gallery.json"
write_presentation_example() {
  local content=$1
  printf '%s\n' \
    'import "./visible.veac" as visible;' 'fn main(context: Context) -> Project {' \
    '  project(identifier("demo"), project_settings(600))' '}' \
    > "$presentation_root/examples/demo/main.veac"
  printf '%s\n' "fn visible(style: TextStyle) -> Source { source_text(\"$content\", style) }" \
    > "$presentation_root/examples/demo/visible.veac"
}
write_presentation_example '中文排版 · مرحبا · VEAC'
LC_ALL=C "$PRESENTATION_CHECKER" "$presentation_root" \
  "$presentation_root/gallery.json" >/dev/null
write_presentation_example 'English explanation'
if LC_ALL=C "$PRESENTATION_CHECKER" "$presentation_root" \
  "$presentation_root/gallery.json" >/dev/null 2>&1; then
  echo "expected English-only visible explanation failure" >&2
  exit 1
fi

jq -s '[.[].mechanisms[]]' "$ROOT"/examples/catalog/mechanisms/*.json \
  > "$tmp/mechanisms.json"
"$REGISTRY_CHECKER" "$tmp/mechanisms.json" >/dev/null

jq 'map(if .registry_key == "video.blur" then del(.registry_key) else . end)' \
  "$tmp/mechanisms.json" > "$tmp/missing-registry-key.json"
if "$REGISTRY_CHECKER" "$tmp/missing-registry-key.json" >/dev/null 2>&1; then
  echo "expected missing registry key failure" >&2
  exit 1
fi

jq '.[0].registry_key = "video.not_registered"' "$tmp/mechanisms.json" \
  > "$tmp/extra-registry-key.json"
if "$REGISTRY_CHECKER" "$tmp/extra-registry-key.json" >/dev/null 2>&1; then
  echo "expected extra registry key failure" >&2
  exit 1
fi

for id in mask.star animation.interpolation-spring audio.high-pass generator.shape.path \
  delivery.video-codec.av1; do
  jq --arg id "$id" 'map(select(.id != $id))' "$tmp/mechanisms.json" > "$tmp/missing-family.json"
  if "$REGISTRY_CHECKER" "$tmp/missing-family.json" >/dev/null 2>&1; then
    echo "expected closed registry family failure: $id" >&2
    exit 1
  fi
done

echo "Example capability checker tests passed."
