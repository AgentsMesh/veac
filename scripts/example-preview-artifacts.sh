#!/usr/bin/env bash

# shellcheck source=example-preview-package.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-package.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-source-edit-evidence.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-delivery.sh"
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-workflow-evidence.sh"

preview_artifact_error() {
  echo "examples preview: $*" >&2
  return 1
}

verify_preview_json() {
  local file=$1
  local label=$2
  if [[ ! -s "$file" || -L "$file" ]]; then
    preview_artifact_error "missing $label: $file"
    return 1
  fi
  if ! jq -e 'type == "object"' "$file" >/dev/null; then
    preview_artifact_error "invalid $label JSON: $file"
    return 1
  fi
}

verify_preview_file() {
  local rendered=$1
  local file=$2
  local kind=$3
  local regex candidate matched=false
  case "$file" in
    ""|*/*|*..*) preview_artifact_error "unsafe deliverable name: $file"; return 1 ;;
  esac
  if [[ "$kind" == image_sequence ]]; then
    regex=$(printf '%s' "$file" |
      sed -E 's/[][\\.^$*+?(){}|]/\\&/g; s/%d/[0-9]+/; s/%0([1-9][0-9]*)d/[0-9]{\1,}/')
    while IFS= read -r -d '' candidate; do
      if [[ $(basename "$candidate") =~ ^$regex$ && -s "$candidate" ]]; then
        matched=true
        break
      fi
    done < <(find "$rendered" -maxdepth 1 -type f -print0)
    if ! $matched; then
      preview_artifact_error "missing image sequence: $rendered/$file"
      return 1
    fi
    return
  fi
  if [[ ! -s "$rendered/$file" || -L "$rendered/$file" ]]; then
    preview_artifact_error "missing deliverable: $rendered/$file"
    return 1
  fi
  if [[ "$kind" == video || "$kind" == audio_stem ]]; then
    ffprobe -v error -show_entries format=duration \
      -of default=noprint_wrappers=1 "$rendered/$file" >/dev/null ||
      { preview_artifact_error "unreadable media deliverable: $rendered/$file"; return 1; }
  fi
}

verify_all_preview_deliverables() {
  local canonical=$1
  local rendered=$2
  local target_type file deliverable_kind package_kind deliverables count=0
  if ! deliverables=$(jq -r '
    def leaf($value):
      if ($value | type) == "string" and ($value | length) > 0 and
         $value != "." and $value != ".." and
         ($value | contains("..") | not) and
         ($value | explode | all(. >= 32 and . != 47 and . != 92 and . != 127))
      then $value else error("unsafe deliverable target name") end;
    def token($value):
      if ($value | type) == "string" and ($value | test("^[a-z_]+$"))
      then $value else error("invalid deliverable type") end;
    .project.render_configs[].deliverables[] |
    .target.type as $target |
    (if $target == "file" then .target.name
     elif $target == "image_sequence" then .target.pattern
     elif $target == "package" then .target.name
     else error("unsupported deliverable target") end) as $path |
    (if $target == "package" then
       if .kind.type == "adaptive_package" and .kind.settings.type == "hls"
       then "hls" else error("unsupported package deliverable kind") end
     else "" end) as $package |
    [$target, leaf($path), token(.kind.type), $package] | @tsv
  ' "$canonical"); then
    preview_artifact_error "invalid canonical deliverable declaration"
    return 1
  fi
  if [[ -z "$deliverables" ]]; then
    preview_artifact_error "canonical project declares no deliverables"
    return 1
  fi
  while IFS=$'\t' read -r target_type file deliverable_kind package_kind; do
    case "$target_type" in
      file|image_sequence)
        verify_preview_file "$rendered" "$file" "$deliverable_kind" || return 1 ;;
      package)
        verify_preview_package "$rendered" "$file" "$deliverable_kind" \
          "$package_kind" || return 1 ;;
      *) preview_artifact_error "unsupported deliverable target: $target_type"; return 1 ;;
    esac
    count=$((count + 1))
  done <<<"$deliverables"
  if [[ $count -eq 0 ]]; then
    preview_artifact_error "canonical project declares no deliverables"
    return 1
  fi
}

verify_expected_preview_artifacts() {
  local target=$1
  local authoring=$2
  local preview=$3
  local plan=$4
  local rendered=$5
  local artifact kind logical_key config expected actual entry count=0
  entry=$(dirname "$(dirname "$authoring")")
  while IFS= read -r artifact; do
    kind=$(jq -r '.kind' <<<"$artifact")
    case "$kind" in
      canonical_project) verify_preview_json "$authoring" "authoring canonical project" || return 1 ;;
      preview_canonical_project) verify_preview_json "$preview" "preview canonical project" || return 1 ;;
      preview_resolved_plan) verify_preview_json "$plan" "preview resolved plan" || return 1 ;;
      source_revision|source_index|source_edit_batch|source_edit_outcome)
        verify_example_source_edit_evidence "$entry" || return 1 ;;
      edit_batch|edit_outcome|edit_replay_outcome|probe_snapshot) ;;
      delivery)
        logical_key=$(jq -r '.logical_key' <<<"$artifact")
        if ! jq -e '
          .artifacts | type == "array" and length > 0 and
          all(.[];
            keys == ["kind", "target", "target_type"] and
            (.kind | type == "string") and (.target_type | type == "string") and
            (.target | type == "string" and length > 0))
        ' <<<"$artifact" >/dev/null; then
          preview_artifact_error "invalid artifact descriptors for delivery '$logical_key'"
          return 1
        fi
        config=$(delivery_config_id "$authoring" "$logical_key") || return 1
        expected=$(jq -c '.artifacts | sort_by(.kind, .target_type, .target)' <<<"$artifact")
        actual=$(canonical_delivery_artifacts "$authoring" "$config") || return 1
        if [[ "$actual" != "$expected" ]]; then
          preview_artifact_error "artifacts do not match delivery '$logical_key'"
          return 1
        fi
        [[ $(delivery_config_id "$preview" "$logical_key") == "$config" ]] ||
          { preview_artifact_error "preview delivery provenance drifted: $logical_key"; return 1; }
        actual=$(canonical_delivery_artifacts "$preview" "$config") || return 1
        if [[ "$actual" != "$expected" ]]; then
          preview_artifact_error "preview artifacts do not match delivery '$logical_key'"
          return 1
        fi
        ;;
      *) preview_artifact_error "non-buildable expected artifact kind: $kind"; return 1 ;;
    esac
    count=$((count + 1))
  done < <(jq -c '.expected_artifacts[]' <<<"$target")
  if [[ $count -eq 0 ]]; then
    preview_artifact_error "target declares no expected artifacts"
    return 1
  fi
  verify_example_workflow_evidence "$target" "$entry" "$authoring" "$plan" || return 1
  verify_all_preview_deliverables "$preview" "$rendered" || return 1
}
