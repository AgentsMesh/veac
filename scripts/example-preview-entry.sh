#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-provenance.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-source-graph.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-source-edit-evidence.sh"

build_example() {
  local source_dir=$1 output=$2 fixtures=$3 veac=$4
  local name entry project source authoring preview rendered log
  local target window primary_delivery primary_config config config_plan authoring_oid plan_dir
  local source_edit_oid=""
  name=$(basename "$source_dir")
  [[ $name =~ ^[a-z0-9][a-z0-9-]*$ ]] || fail "unsafe example name: $name"
  target=$(jq -ce --arg id "$name" '.targets[] | select(.id == $id)' "$GALLERY")
  [[ -n $target ]] || fail "gallery target missing for example: $name"
  window=$(jq -c '.preview_window' <<<"$target")
  primary_delivery=$(jq -er '
    [.expected_artifacts[] | select(.kind == "authoring_delivery") | .id]
    | if length == 1 then .[0] else error("expected one authoring delivery") end
  ' <<<"$target") || fail "gallery must select one authoring delivery: $name"
  entry="$output/$name"; project="$entry/project"; rendered="$entry/rendered"
  source=$(example_authoring_source "$entry")
  authoring=$(example_authoring_canonical "$entry"); preview=$(example_preview_canonical "$entry")
  log="$entry/build.log"
  plan_dir=$(example_preview_plan_dir "$entry")
  mkdir -p "$project" "$rendered" "$plan_dir"
  stage_example_source_graph "$source_dir" "$project" ||
    fail "example source graph staging failed: $name"
  verify_example_source_graph "$source_dir" "$project" ||
    fail "authoring source graph copy drifted: $name"
  echo "[$name] source graph compile -> preview IR derive -> plan -> render -> probe"
  if ! (
    build_example_source_edit_evidence "$target" "$source_dir" "$entry" "$source" "$veac" || exit 1
    if source_edit_evidence_requested "$target"; then
      source_edit_oid=$(source_edit_evidence_oid "$entry") || exit 1
    fi
    "$veac" compile --emit-ir "$authoring" "$source" || exit 1
    authoring_oid=$(git hash-object --no-filters "$authoring") || exit 1
    jq --argjson edge "$PREVIEW_EDGE" --argjson fps "$PREVIEW_FPS" \
      --argjson window "$window" -f "$PREVIEW_FILTER" "$authoring" >"$preview" || exit 1
    verify_preview_derivation "$authoring" "$preview" "$PREVIEW_FILTER" \
      "$PREVIEW_EDGE" "$PREVIEW_FPS" "$window" || exit 1
    verify_canonical_roles "$authoring" "$preview" || exit 1
    materialize_preview_assets "$project" "$preview" "$fixtures" || exit 1
    pad_preview_audio "$project" "$preview" 65 || exit 1
    primary_config="out_$primary_delivery"
    jq -e --arg id "$primary_config" '
      [.project.render_configs[] | select(.id == $id)] | length == 1
    ' "$preview" >/dev/null || exit 1
    while IFS= read -r config; do
      config_plan=$(example_preview_plan "$entry" "$config") || exit 1
      (cd "$project" && "$veac" plan --config "$config" "$preview") >"$config_plan" || exit 1
      run_example_preview_process "$project" "$veac" render \
        --config "$config" --destination "$rendered" "$preview" || exit 1
    done < <(jq -r '.project.render_configs[].id' "$preview")
    rm -f "$rendered/.veac-render.lock"
    verify_preview_derivation "$authoring" "$preview" "$PREVIEW_FILTER" \
      "$PREVIEW_EDGE" "$PREVIEW_FPS" "$window" || exit 1
    verify_example_source_graph "$source_dir" "$project" || exit 1
    if source_edit_evidence_requested "$target"; then
      verify_declared_source_edit_evidence "$source_dir" "$entry" || exit 1
      [[ $(source_edit_evidence_oid "$entry") == "$source_edit_oid" ]] || exit 1
    fi
    [[ $(git hash-object --no-filters "$authoring") == "$authoring_oid" ]] || exit 1
    verify_example_preview_layout "$entry" || exit 1
    config_plan=$(example_preview_plan "$entry" "$primary_config") || exit 1
    verify_expected_preview_artifacts "$target" "$authoring" "$preview" \
      "$config_plan" "$rendered" || exit 1
  ) >"$log" 2>&1; then
    cat "$log" >&2
    return 1
  fi
}
