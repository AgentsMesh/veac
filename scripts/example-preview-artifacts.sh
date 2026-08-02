#!/usr/bin/env bash

# shellcheck source=example-preview-package.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/example-preview-package.sh"

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
  local artifact kind id canonical_id matches expected actual count=0
  while IFS= read -r artifact; do
    kind=$(jq -r '.kind' <<<"$artifact")
    case "$kind" in
      canonical_project) verify_preview_json "$authoring" "authoring canonical project" || return 1 ;;
      preview_canonical_project) verify_preview_json "$preview" "preview canonical project" || return 1 ;;
      preview_resolved_plan) verify_preview_json "$plan" "preview resolved plan" || return 1 ;;
      authoring_delivery)
        id=$(jq -r '.id' <<<"$artifact")
        if [[ ! "$id" =~ ^[a-z][a-z0-9-]*$ ]]; then
          preview_artifact_error "unsafe authoring delivery ID: $id"
          return 1
        fi
        if ! jq -e '
          .artifact_ids | type == "array" and length > 0 and
          all(.[]; type == "string" and test("^[a-z][a-z0-9-]*$")) and
          length == (unique | length)
        ' <<<"$artifact" >/dev/null; then
          preview_artifact_error "invalid artifact IDs for authoring delivery '$id'"
          return 1
        fi
        canonical_id="out_$id"
        matches=$(jq -r --arg id "$canonical_id" '
          [.project.render_configs[] | select(.id == $id)] | length
        ' "$authoring")
        if [[ "$matches" != 1 ]]; then
          preview_artifact_error "expected one canonical config for authoring delivery '$id'"
          return 1
        fi
        expected=$(jq -c '.artifact_ids | map("dlv_" + .) | sort' <<<"$artifact")
        actual=$(jq -c --arg id "$canonical_id" '
          [.project.render_configs[] | select(.id == $id) | .deliverables[].id] | sort
        ' "$authoring")
        if [[ "$actual" != "$expected" ]]; then
          preview_artifact_error "artifact IDs do not match authoring delivery '$id'"
          return 1
        fi
        actual=$(jq -c --arg id "$canonical_id" '
          [.project.render_configs[] | select(.id == $id) | .deliverables[].id] | sort
        ' "$preview")
        if [[ "$actual" != "$expected" ]]; then
          preview_artifact_error "preview artifact IDs do not match authoring delivery '$id'"
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
  verify_all_preview_deliverables "$preview" "$rendered" || return 1
}
