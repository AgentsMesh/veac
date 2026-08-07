#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout.sh"
PROBE_EVIDENCE_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PROBE_EVIDENCE_CHECKER="$PROBE_EVIDENCE_DIR/check-example-probe-evidence.jq"

probe_evidence_error() {
  echo "examples preview: $*" >&2
  return 1
}

probe_evidence_requested() {
  jq -e 'any(.expected_artifacts[]; .kind == "probe_snapshot")' <<<"$1" >/dev/null
}

verify_probe_evidence_file() {
  local entry=$1 snapshot
  snapshot=$(example_probe_snapshot "$entry")
  require_preview_regular_file "$snapshot" "probe snapshot" || return 1
  jq -e 'type == "object"' "$snapshot" >/dev/null ||
    probe_evidence_error "invalid probe snapshot JSON: $snapshot"
}

verify_example_probe_evidence() {
  local entry=$1 authoring=$2 plan=$3 snapshot
  snapshot=$(example_probe_snapshot "$entry")
  verify_probe_evidence_file "$entry" || return 1
  jq -e --slurpfile authoring "$authoring" --slurpfile plan "$plan" \
    -f "$PROBE_EVIDENCE_CHECKER" "$snapshot" >/dev/null ||
    probe_evidence_error "typed probe evidence contract failed: $entry"
}

build_example_probe_evidence() {
  local entry=$1 authoring=$2 veac=$3 material snapshot
  snapshot=$(example_probe_snapshot "$entry")
  material=$(jq -er '
    [.project.materials[] |
      select(.authorship.logical_path[-1] == "source" and .kind == "video")]
    | if length == 1 then .[0].id else error("expected one source material") end
  ' "$authoring") || return 1
  echo "[probe-stream-selection] canonical material -> typed probe snapshot"
  "$veac" probe "$authoring" --material "$material" >"$snapshot" || return 1
  verify_probe_evidence_file "$entry"
}
