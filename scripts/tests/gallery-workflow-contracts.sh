#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
SCHEMA="$ROOT/scripts/check-gallery-catalog.jq"
GALLERY="$ROOT/examples/catalog/gallery.json"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

expect_failure() {
  local label=$1 filter=$2 candidate
  candidate="$TMP/$label.json"
  jq "$filter" "$GALLERY" >"$candidate"
  if jq -e -f "$SCHEMA" "$candidate" >/dev/null 2>&1; then
    echo "gallery workflow contract accepted $label" >&2
    exit 1
  fi
}

jq -e -f "$SCHEMA" "$GALLERY" >/dev/null
expect_failure incomplete-edit \
  '(.targets[] | select(.id == "edit-operations") | .expected_artifacts) |=
    map(select(.kind != "edit_replay_outcome"))'
expect_failure edit-without-group-capability \
  '(.targets[] | select(.id == "edit-operations") | .capability_ids) = ["P1-04"]'
expect_failure edit-artifact-on-render \
  '(.targets[] | select(.id == "minimal") | .expected_artifacts) +=
    [{"kind":"edit_batch"},{"kind":"edit_outcome"},{"kind":"edit_replay_outcome"}]'
expect_failure probe-without-capability \
  '(.targets[] | select(.id == "probe-stream-selection") | .capability_ids) = ["P1-04"]'
expect_failure probe-on-render \
  '(.targets[] | select(.id == "minimal") | .expected_artifacts) +=
    [{"kind":"probe_snapshot"}]'

echo "gallery workflow contracts passed"
