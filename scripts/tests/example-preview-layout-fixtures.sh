#!/usr/bin/env bash

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/example-preview-layout.sh"

prepare_preview_fixture_dirs() {
  local entry=$1 source prepared
  mkdir -p "$entry/project" "$entry/rendered" "$(example_preview_plan_dir "$entry")"
  source=$(example_authoring_source "$entry")
  prepared=$(example_preview_source "$entry")
  if [[ ! -f $source ]]; then
    printf 'project fixture {}\n' >"$source"
  fi
  if [[ ! -f $prepared ]]; then
    cp "$source" "$prepared"
  fi
}

mirror_fixture_preview_canonical() {
  local entry=$1 authoring preview
  authoring=$(example_authoring_canonical "$entry")
  preview=$(example_preview_canonical "$entry")
  [[ -f $authoring ]] || return 1
  cp "$authoring" "$preview"
}
