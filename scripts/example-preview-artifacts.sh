#!/usr/bin/env bash

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
  local file deliverable_kind count=0
  while IFS=$'\t' read -r file deliverable_kind; do
    verify_preview_file "$rendered" "$file" "$deliverable_kind" || return 1
    count=$((count + 1))
  done < <(jq -r '
    .project.render_configs[].deliverables[] | [.file_name, .kind.type] | @tsv
  ' "$canonical")
  if [[ $count -eq 0 ]]; then
    preview_artifact_error "canonical project declares no deliverables"
    return 1
  fi
}

verify_expected_preview_artifacts() {
  local target=$1
  local canonical=$2
  local plan=$3
  local rendered=$4
  local artifact kind id canonical_id matches deliverables count=0
  while IFS= read -r artifact; do
    kind=$(jq -r '.kind' <<<"$artifact")
    case "$kind" in
      canonical_project) verify_preview_json "$canonical" "canonical project" || return 1 ;;
      resolved_plan) verify_preview_json "$plan" "resolved plan" || return 1 ;;
      authoring_output)
        id=$(jq -r '.id' <<<"$artifact")
        if [[ ! "$id" =~ ^[a-z][a-z0-9-]*$ ]]; then
          preview_artifact_error "unsafe authoring output ID: $id"
          return 1
        fi
        canonical_id="out_$id"
        matches=$(jq -r --arg id "$canonical_id" '
          [.project.render_configs[] | select(.id == $id)] | length
        ' "$canonical")
        if [[ "$matches" != 1 ]]; then
          preview_artifact_error "expected one canonical config for authoring output '$id'"
          return 1
        fi
        deliverables=$(jq -r --arg id "$canonical_id" '
          .project.render_configs[] | select(.id == $id) | .deliverables | length
        ' "$canonical")
        if [[ "$deliverables" -eq 0 ]]; then
          preview_artifact_error "authoring output '$id' declares no deliverables"
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
  verify_all_preview_deliverables "$canonical" "$rendered" || return 1
}
