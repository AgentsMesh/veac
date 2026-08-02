#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout.sh"

preview_provenance_error() {
  echo "examples preview: $*" >&2
  return 1
}

verify_preview_json_object() {
  local file=$1 label=$2
  require_preview_regular_file "$file" "$label" || return 1
  jq -e 'type == "object" and (.project | type) == "object"' "$file" >/dev/null ||
    preview_provenance_error "invalid $label: $file"
}

verify_preview_derivation() {
  local input=$1 preview=$2 filter=$3 edge=$4 fps=$5 window=$6
  verify_preview_json_object "$input" "preview input canonical" || return 1
  verify_preview_json_object "$preview" "preview canonical" || return 1
  cmp -s "$preview" <(
    jq --argjson edge "$edge" --argjson fps "$fps" --argjson window "$window" \
      -f "$filter" "$input"
  ) || preview_provenance_error "preview canonical is not the declared derivative"
}

verify_canonical_roles() {
  local authoring=$1 preview=$2
  verify_preview_json_object "$authoring" "authoring canonical" || return 1
  verify_preview_json_object "$preview" "preview canonical" || return 1
  jq -e --slurpfile preview "$preview" '
    def outputs:
      [.project.render_configs[] | {id, sequence_id,
        deliverables: [.deliverables[] | {id, target}]}] | sort_by(.id);
    (.project.id == $preview[0].project.id) and
    (outputs == ($preview[0] | outputs))
  ' "$authoring" >/dev/null ||
    preview_provenance_error "authoring and preview output identities differ"
}

verify_preview_plan_set() {
  local entry=$1 preview=$2 dir expected actual config plan
  dir=$(example_preview_plan_dir "$entry")
  [[ -d $dir && ! -L $dir ]] ||
    preview_provenance_error "missing preview plan directory: $dir" || return 1
  expected=$(jq -r '.project.render_configs[].id' "$preview" | sort)
  [[ -n $expected ]] || preview_provenance_error "preview canonical has no render configs" || return 1
  actual=$(find "$dir" -mindepth 1 -maxdepth 1 -type f -name '*.json' \
    -exec basename {} .json \; | sort)
  [[ $actual == "$expected" ]] ||
    preview_provenance_error "preview plan inventory differs from render configs" || return 1
  while IFS= read -r config; do
    plan=$(example_preview_plan "$entry" "$config") ||
      preview_provenance_error "unsafe preview config ID: $config" || return 1
    require_preview_regular_file "$plan" "preview plan" || return 1
    jq -e --arg id "$config" '
      type == "object" and .output.render_config_id == $id
    ' "$plan" >/dev/null || preview_provenance_error "preview plan config mismatch: $plan" || return 1
  done <<<"$expected"
  [[ $(find "$dir" -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ') -eq \
     $(wc -l <<<"$expected" | tr -d ' ') ]] ||
    preview_provenance_error "unexpected preview plan directory entry"
}

verify_example_preview_layout() {
  local entry=$1 authoring preview source
  source=$(example_authoring_source "$entry")
  authoring=$(example_authoring_canonical "$entry")
  preview=$(example_preview_canonical "$entry")
  reject_legacy_preview_layout "$entry" || return 1
  require_preview_regular_file "$source" "authoring source" || return 1
  verify_canonical_roles "$authoring" "$preview" || return 1
  verify_preview_plan_set "$entry" "$preview"
}
