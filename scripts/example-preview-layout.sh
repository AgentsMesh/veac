#!/usr/bin/env bash

example_authoring_source() {
  printf '%s/project/main.veac\n' "$1"
}

example_preview_source() {
  printf '%s/project/main.preview.veac\n' "$1"
}

example_authoring_canonical() {
  printf '%s/project/project.veac.json\n' "$1"
}

example_preview_canonical() {
  printf '%s/project/project.preview.veac.json\n' "$1"
}

example_preview_plan_dir() {
  printf '%s/plans/preview\n' "$1"
}

example_preview_plan() {
  local entry=$1 config=$2
  [[ $config =~ ^[a-z][a-z0-9_-]*$ ]] || return 1
  printf '%s/plans/preview/%s.json\n' "$entry" "$config"
}

require_preview_regular_file() {
  local file=$1 label=$2
  [[ -f $file && ! -L $file && -s $file ]] || {
    echo "examples preview: missing regular $label: $file" >&2
    return 1
  }
}

reject_legacy_preview_layout() {
  local entry=$1 legacy
  for legacy in "$entry/project/project.raw.json" \
      "$entry/project/.project.preview-input.veac.json" "$entry/plan.json"; do
    [[ ! -e $legacy && ! -L $legacy ]] || {
      echo "examples preview: forbidden preview artifact: $legacy" >&2
      return 1
    }
  done
  if find "$entry" -maxdepth 1 \( -name 'plan.out_*.json' -o -name 'plan.*.json' \) \
      -print -quit | grep -q .; then
    echo "examples preview: legacy root-level preview plan is forbidden: $entry" >&2
    return 1
  fi
}
