#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/veac-workflow-media-contracts.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
for module in common pixels showcase-pixels audio-metrics video-stabilization \
    workflow-showcase-core workflow-showcase-media; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/workflow-showcase-media-fixtures.sh"

expect_failure() {
  local label=$1 expected=$2 function=$3 video=$4 log="$TMP/$1.log"
  if ("$function" "$video") >"$log" 2>&1; then
    echo "$label unexpectedly passed" >&2; exit 1
  fi
  rg -qF "$expected" "$log" || {
    cat "$log" >&2; echo "$label failed for the wrong reason" >&2; exit 1;
  }
}

make_workflow_keying_video "$TMP/keying.mp4"
make_workflow_routing_video "$TMP/routing.mp4"
make_workflow_edit_video "$TMP/edit.mp4"
make_workflow_probe_video "$TMP/probe.mp4"
workflow_assert_keying_media "$TMP/keying.mp4"
workflow_assert_routing_media "$TMP/routing.mp4"
workflow_assert_edit_media "$TMP/edit.mp4"
workflow_assert_probe_media "$TMP/probe.mp4"

make_workflow_keying_video "$TMP/wrong-chroma.mp4" wrong-chroma
expect_failure wrong_chroma 'chroma-key pixels are wrong' \
  workflow_assert_keying_media "$TMP/wrong-chroma.mp4"
make_workflow_keying_video "$TMP/frozen.mp4" frozen
expect_failure frozen_stabilization 'output is frozen' \
  workflow_assert_keying_media "$TMP/frozen.mp4"
make_workflow_routing_video "$TMP/balanced.mp4" balanced
expect_failure missing_pan 'sidechain/pan balance is wrong' \
  workflow_assert_routing_media "$TMP/balanced.mp4"
make_workflow_edit_video "$TMP/wrong-second.mp4" wrong-second
expect_failure wrong_second_scene 'edit-operations second scene' \
  workflow_assert_edit_media "$TMP/wrong-second.mp4"
make_workflow_probe_video "$TMP/missing-marker.mp4" missing-marker
expect_failure missing_scene_marker 'SCENE 1 stream marker is missing' \
  workflow_assert_probe_media "$TMP/missing-marker.mp4"

echo 'workflow showcase media contracts passed'
