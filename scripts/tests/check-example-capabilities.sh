#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT/scripts/check-example-capabilities.sh"
REGISTRY_CHECKER="$ROOT/scripts/check-example-registry.sh"
CATALOG="$ROOT/examples/capabilities.json"
tmp="$(mktemp -d)"
gallery_test="$ROOT/examples/catalog/.gallery-contract-test.$$.json"
trap 'rm -rf "$tmp"; rm -f "$gallery_test"' EXIT

expect_failure() {
  local name="$1"
  local catalog="$2"
  if "$CHECKER" "$catalog" >"$tmp/$name.out" 2>&1; then
    echo "expected checker failure: $name" >&2
    exit 1
  fi
}

expect_gallery_failure() {
  local name=$1
  local filter=$2
  local catalog="$tmp/$name-catalog.json"
  jq "$filter" "$ROOT/examples/catalog/gallery.json" > "$gallery_test"
  jq --arg gallery "examples/catalog/$(basename "$gallery_test")" \
    '.gallery_catalog = $gallery' "$CATALOG" > "$catalog"
  expect_failure "$name" "$catalog"
}

"$CHECKER" "$CATALOG" >/dev/null

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
expect_gallery_failure legacy-artifacts \
  '.targets[0] += {"expected_deliverables":["preview"]}'
expect_gallery_failure unknown-artifact \
  '.targets[0].expected_artifacts[0].kind = "unknown"'
expect_gallery_failure legacy-deliverable-artifact \
  '.targets[0].expected_artifacts[2].kind = "deliverable"'
expect_gallery_failure legacy-render-config-artifact \
  '.targets[0].expected_artifacts[2].kind = "render_config"'
expect_gallery_failure untyped-authoring-output \
  'del(.targets[0].expected_artifacts[2].id)'
expect_gallery_failure artifact-extra-key \
  '.targets[0].expected_artifacts[0].label = "legacy"'
expect_gallery_failure nonzero-window \
  '.targets[0].preview_window.start_seconds = 1'
expect_gallery_failure missing-plan \
  '.targets[0].expected_artifacts |= map(select(.kind != "resolved_plan"))'
expect_gallery_failure workflow-artifact-on-source \
  '.targets[0].expected_artifacts[2] = {"kind":"probe_snapshot"}'

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

"$ROOT/scripts/tests/example-preview-contracts.sh" >/dev/null

echo "Example capability checker tests passed."
