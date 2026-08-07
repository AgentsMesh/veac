#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-layout.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-provenance.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-delivery.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-source-graph.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-source-edit-evidence.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-workflow-evidence.sh"

build_example() {
  local source_dir=$1 output=$2 fixtures=$3 veac=$4
  local name entry project source authoring preview rendered log
  local target window primary_delivery primary_config config config_plan
  local authoring_oid plan_dir input_manifest=""
  local source_edit_oid=""
  name=$(basename "$source_dir")
  [[ $name =~ ^[a-z0-9][a-z0-9-]*$ ]] || fail "unsafe example name: $name"
  target=$(jq -ce --arg id "$name" '.targets[] | select(.id == $id)' "$GALLERY")
  [[ -n $target ]] || fail "gallery target missing for example: $name"
  jq -e '.frontend == "executable"' <<<"$target" >/dev/null ||
    fail "gallery frontend must be executable: $name"
  window=$(jq -c '.preview_window' <<<"$target")
  primary_delivery=$(jq -er '
    [.expected_artifacts[] | select(.kind == "delivery") | .logical_key]
    | if length == 1 then .[0] else error("expected one delivery") end
  ' <<<"$target") || fail "gallery must select one delivery: $name"
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
  if [[ -f $source_dir/build-inputs.json && ! -L $source_dir/build-inputs.json ]]; then
    input_manifest="$project/build-inputs.json"
    cp "$source_dir/build-inputs.json" "$input_manifest" ||
      fail "Build input manifest staging failed: $name"
  fi
  echo "[$name] executable source graph -> preview IR derive -> plan -> render -> probe"
  if ! (
    build_example_source_edit_evidence \
      "$target" "$source_dir" "$entry" "$source" "$veac" "$input_manifest" || exit 1
    if source_edit_evidence_requested "$target"; then
      source_edit_oid=$(source_edit_evidence_oid "$entry") || exit 1
    fi
    if [[ -n $input_manifest ]]; then
      "$veac" build --inputs "$input_manifest" --emit-ir "$authoring" "$source" || exit 1
    else
      "$veac" build --emit-ir "$authoring" "$source" || exit 1
    fi
    authoring_oid=$(git hash-object --no-filters "$authoring") || exit 1
    jq --argjson edge "$PREVIEW_EDGE" --argjson fps "$PREVIEW_FPS" \
      --argjson window "$window" -f "$PREVIEW_FILTER" "$authoring" >"$preview" || exit 1
    verify_preview_derivation "$authoring" "$preview" "$PREVIEW_FILTER" \
      "$PREVIEW_EDGE" "$PREVIEW_FPS" "$window" || exit 1
    verify_canonical_roles "$authoring" "$preview" || exit 1
    materialize_preview_assets "$project" "$preview" "$fixtures" "$source_dir" || exit 1
    pad_preview_audio "$project" "$preview" 65 || exit 1
    build_example_workflow_evidence "$target" "$entry" "$authoring" "$veac" || exit 1
    primary_config=$(delivery_config_id "$preview" "$primary_delivery") || exit 1
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
    if [[ -n $input_manifest ]]; then
      cmp -s "$source_dir/build-inputs.json" "$input_manifest" || exit 1
    fi
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
